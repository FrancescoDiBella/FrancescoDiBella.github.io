// createBlueBunnyModel.js
// Procedural, code-only Three.js reconstruction of the "Blue Bunny" reference
// (reference.jpg), authored following the img2threejs pipeline
// (https://github.com/img2threejs/img2threejs): no mesh files, no downloaded
// art — every part is built from primitives, generated geometry and canvas
// textures. Units: 1 unit = 1/100 of the reference image height (displayed at
// 2000 px), feet on y = 0, character faces +Z, character's own left is +X.
//
// The returned THREE.Group carries a runtime hierarchy (pivots) so the model
// is animation-ready, plus `userData.parts` (clickable meshes) and
// `userData.rig` (named pivots) for interaction.

import * as THREE from 'three';

// ---------------------------------------------------------------------------
// Reconstruction data (kept separate from renderer objects)
// ---------------------------------------------------------------------------
export const BUNNY_SPEC = {
  palette: {
    fur: '#86cdd2',
    furDark: '#3f8c94',
    furTip: '#a6dde0',
    brow: '#2d6870',
    claw: '#2b5c63',
    innerEar: '#f2ab9c',
    nose: '#e79688',
    eye: '#0b0b0d',
    poncho: '#c4483f',
    ponchoInner: '#7d2622',
    mouth: '#2a5a60',
  },
  // measured landmarks (units)
  hipY: 6.4,
  chestY: 8.6,
  neckY: 10.25,
  shoulderX: 0.95,
  shoulderY: 9.66,
  handReachX: 5.96,
  legX: 0.76,
  furLength: 0.06,
  furShells: 10,
  seed: 1337,
};

// deterministic PRNG (mulberry32)
function rng(seed) {
  let a = seed >>> 0;
  return () => {
    a |= 0; a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

// ---------------------------------------------------------------------------
// Materials
// ---------------------------------------------------------------------------
function furNoiseTexture(seed) {
  const c = document.createElement('canvas');
  c.width = c.height = 256;
  const g = c.getContext('2d');
  const r = rng(seed);
  g.fillStyle = '#f2f2f2';
  g.fillRect(0, 0, 256, 256);
  for (let i = 0; i < 9000; i++) {
    const x = r() * 256, y = r() * 256, l = 3 + r() * 7, a = (r() - 0.5) * 0.6 + Math.PI / 2;
    const v = 205 + Math.floor(r() * 50);
    g.strokeStyle = `rgba(${v},${v},${v},0.55)`;
    g.lineWidth = 0.8;
    g.beginPath();
    g.moveTo(x, y);
    g.lineTo(x + Math.cos(a) * l, y + Math.sin(a) * l);
    g.stroke();
  }
  const t = new THREE.CanvasTexture(c);
  t.wrapS = t.wrapT = THREE.RepeatWrapping;
  t.repeat.set(4, 4);
  t.colorSpace = THREE.SRGBColorSpace;
  return t;
}

// Shell-texturing fur: N transparent-discard shells pushed along the normal.
// Shares the base geometry, so it follows every pivot / animation for free.
function makeFurShellMaterial(layer, colorUniform, tipUniform, length, density) {
  return new THREE.ShaderMaterial({
    uniforms: {
      uLayer: { value: layer },
      uLen: { value: length },
      uDensity: { value: density },
      uColor: colorUniform,
      uTip: tipUniform,
      uLightDir: { value: new THREE.Vector3(0.4, 0.8, 0.6).normalize() },
      uTime: { value: 0 },
    },
    vertexShader: /* glsl */`
      uniform float uLayer; uniform float uLen; uniform float uTime;
      varying vec3 vObj; varying vec3 vN;
      void main(){
        vec3 p = position + normal * uLen * uLayer;
        p.y -= uLayer * uLayer * uLen * 0.35;           // gravity droop
        vObj = position;
        vN = normalize(normalMatrix * normal);
        gl_Position = projectionMatrix * modelViewMatrix * vec4(p,1.0);
      }`,
    fragmentShader: /* glsl */`
      uniform float uLayer; uniform float uDensity; uniform vec3 uColor; uniform vec3 uTip;
      uniform vec3 uLightDir;
      varying vec3 vObj; varying vec3 vN;
      float h(vec3 p){ return fract(sin(dot(p, vec3(127.1,311.7,74.7)))*43758.5453); }
      void main(){
        vec3 q = vObj * uDensity;
        vec3 cell = floor(q);
        vec3 f = fract(q) - 0.5;
        float n = h(cell);
        float d = length(f);
        float radius = 0.75 * (1.0 - 0.75 * uLayer);
        if (n < uLayer * 0.8 || d > radius) discard;
        vec3 N = normalize(vN);
        float lam = 0.5 + 0.62 * max(dot(N, normalize((viewMatrix * vec4(uLightDir,0.0)).xyz)), 0.0);
        vec3 col = mix(uColor * 0.9, mix(uColor, uTip, 0.6), uLayer) * lam;
        gl_FragColor = vec4(col, 1.0);
        #include <colorspace_fragment>
      }`,
  });
}

function buildMaterials(spec) {
  const P = spec.palette;
  const furTex = furNoiseTexture(spec.seed);
  const fur = new THREE.MeshStandardMaterial({
    color: P.fur, roughness: 0.92, metalness: 0, map: furTex, bumpMap: furTex, bumpScale: 0.05,
  });
  const furDark = fur.clone(); furDark.color = new THREE.Color(P.furDark);
  return {
    fur,
    furDark,
    brow: new THREE.MeshStandardMaterial({ color: P.brow, roughness: 0.95, map: furTex }),
    claw: new THREE.MeshStandardMaterial({ color: P.claw, roughness: 0.45 }),
    innerEar: new THREE.MeshStandardMaterial({ color: P.innerEar, roughness: 0.75, map: furTex, side: THREE.DoubleSide }),
    nose: new THREE.MeshPhysicalMaterial({ color: P.nose, roughness: 0.45, clearcoat: 0.3 }),
    eye: new THREE.MeshPhysicalMaterial({ color: P.eye, roughness: 0.08, clearcoat: 1, clearcoatRoughness: 0.03 }),
    catchlight: new THREE.MeshBasicMaterial({ color: '#ffffff' }),
    mouth: new THREE.MeshStandardMaterial({ color: P.mouth, roughness: 0.9 }),
    furColorU: { value: new THREE.Color(P.fur) },
    furTipU: { value: new THREE.Color(P.furTip) },
    furDarkColorU: { value: new THREE.Color(P.furDark) },
    furDarkTipU: { value: new THREE.Color(P.fur) },
  };
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------
// ellipsoid with optional per-vertex deform callback (baked, mesh scale stays 1)
function blob(rx, ry, rz, deform, ws = 40, hs = 28) {
  const g = new THREE.SphereGeometry(1, ws, hs);
  const p = g.attributes.position;
  const v = new THREE.Vector3();
  for (let i = 0; i < p.count; i++) {
    v.fromBufferAttribute(p, i);
    if (deform) deform(v);
    p.setXYZ(i, v.x * rx, v.y * ry, v.z * rz);
  }
  g.computeVertexNormals();
  return g;
}

// tapered limb along +X (or any axis via rotation by caller), length L
function limb(L, r0, r1, radial = 20) {
  const pts = [];
  const steps = 14;
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    const r = THREE.MathUtils.lerp(r0, r1, t);
    // rounded caps
    const cap = Math.sin(Math.PI * Math.min(t, 1 - t) * 4 > Math.PI / 2 ? Math.PI / 2 : Math.PI * Math.min(t, 1 - t) * 4);
    pts.push(new THREE.Vector2(Math.max(0.001, r * (0.82 + 0.18 * cap)), t * L));
  }
  pts.unshift(new THREE.Vector2(0.001, -r0 * 0.25));
  pts.push(new THREE.Vector2(0.001, L + r1 * 0.25));
  const g = new THREE.LatheGeometry(pts, radial);
  g.computeVertexNormals();
  return g; // along +Y from 0..L
}

// capsule-ish lathe along +Y with explicit profile of [t, r]
function lathe(profile, L, radial = 24) {
  const pts = profile.map(([t, r]) => new THREE.Vector2(Math.max(0.0005, r), t * L));
  const g = new THREE.LatheGeometry(pts, radial);
  g.computeVertexNormals();
  return g;
}

// thin bent spike (fur tuft lock) along +X, curving in its local XY plane
function lockGeometry(len, rad, bend) {
  const g = new THREE.ConeGeometry(rad, len, 6, 6, false);
  g.rotateZ(-Math.PI / 2); // tip to +X
  g.translate(len / 2, 0, 0);
  const p = g.attributes.position;
  for (let i = 0; i < p.count; i++) {
    const x = p.getX(i);
    const t = x / len;
    p.setY(i, p.getY(i) + bend * t * t * len);
  }
  g.computeVertexNormals();
  return g;
}

function surfacePoint(mesh, x, y, fromZ = 10) {
  const rc = new THREE.Raycaster(new THREE.Vector3(x, y, fromZ), new THREE.Vector3(0, 0, -1));
  const hit = rc.intersectObject(mesh, false)[0];
  return hit ? { point: hit.point.clone(), normal: hit.face.normal.clone() } : null;
}

// ---------------------------------------------------------------------------
// Poncho (torn cloth): deformed grid + generated alpha/weave canvas
// ---------------------------------------------------------------------------
function ponchoTexture(seed) {
  const W = 512, H = 512;
  const c = document.createElement('canvas');
  c.width = W; c.height = H;
  const g = c.getContext('2d');
  const r = rng(seed);
  // outline: half-width (0..1 of panel half width) as function of v (0 top .. 1 bottom)
  const prof = [[0, 1], [0.12, 0.97], [0.3, 0.88], [0.45, 0.74], [0.6, 0.56], [0.72, 0.4], [0.84, 0.22], [0.93, 0.1], [1.0, 0.02]];
  const hw = (v) => {
    for (let i = 1; i < prof.length; i++) if (v <= prof[i][0]) {
      const [v0, w0] = prof[i - 1], [v1, w1] = prof[i];
      return w0 + (w1 - w0) * (v - v0) / (v1 - v0);
    }
    return 0;
  };
  const N = 46;
  const jagAt = [];
  for (let i = 0; i <= N; i++) {
    const deep = r() < 0.16 ? -(0.12 + r() * 0.2) : 0;
    jagAt.push((r() - 0.55) * 0.16 + deep);
  }
  const side = (sgn) => {
    const out = [];
    for (let k = 0; k <= N; k++) {
      const i = sgn > 0 ? k : N - k;
      const v = 0.02 + (i / N) * 0.98;
      const j = jagAt[i] * (v > 0.1 ? 1 : 0.15) * (sgn > 0 ? 1 : 0.9);
      // long torn strips: drop the vertical position of some points
      const dv = (i % 5 === 2 && v > 0.25 && v < 0.9) ? 0.05 + r() * 0.06 : 0;
      out.push([W / 2 + sgn * Math.max(0.01, hw(v) + j) * W / 2 * 0.98, Math.min(1, v + dv) * H]);
    }
    return out;
  };
  const pts = [...side(1), ...side(-1)];
  g.clearRect(0, 0, W, H);
  g.beginPath();
  g.moveTo(pts[0][0], 0);
  pts.forEach(([x, y]) => g.lineTo(x, y));
  g.lineTo(pts[pts.length - 1][0], 0);
  g.closePath();
  g.fillStyle = '#ffffff';
  g.fill();
  // weave
  g.globalCompositeOperation = 'source-atop';
  for (let y = 0; y < H; y += 3) {
    g.fillStyle = `rgba(0,0,0,${0.06 + r() * 0.05})`;
    g.fillRect(0, y, W, 1);
  }
  for (let x = 0; x < W; x += 3) {
    g.fillStyle = `rgba(0,0,0,${0.04 + r() * 0.04})`;
    g.fillRect(x, 0, 1, H);
  }
  for (let i = 0; i < 400; i++) { // fold shading streaks
    g.fillStyle = `rgba(0,0,0,${r() * 0.05})`;
    g.fillRect(r() * W, r() * H * 0.2 + r() * H * 0.8, 2 + r() * 10, 30 + r() * 120);
  }
  // torn holes
  g.globalCompositeOperation = 'destination-out';
  const holes = [[0.5, 0.78, 0.035, 0.12], [0.2, 0.3, 0.06, 0.1], [0.8, 0.28, 0.06, 0.12], [0.36, 0.6, 0.03, 0.05], [0.66, 0.55, 0.025, 0.06], [0.12, 0.22, 0.03, 0.03]];
  holes.forEach(([hx, hy, rx, ry]) => {
    g.beginPath();
    const k = 14;
    for (let i = 0; i < k; i++) {
      const a = (i / k) * Math.PI * 2;
      const j = 0.6 + r() * 0.7;
      const x = (hx + Math.cos(a) * rx * j) * W, y = (hy + Math.sin(a) * ry * j) * H;
      i ? g.lineTo(x, y) : g.moveTo(x, y);
    }
    g.closePath();
    g.fill();
  });
  // neck opening
  g.beginPath();
  g.ellipse(W / 2, 0, W * 0.09, H * 0.045, 0, 0, Math.PI * 2);
  g.fill();
  const t = new THREE.CanvasTexture(c);
  t.colorSpace = THREE.SRGBColorSpace;
  t.anisotropy = 4;
  return t;
}

function ponchoPanelGeometry(front, spec) {
  const halfW = 3.6, H = 7.2, topY = 10.1;
  const g = new THREE.PlaneGeometry(1, 1, 48, 48);
  const p = g.attributes.position;
  const sign = front ? 1 : -1;
  const base = new Float32Array(p.count * 3);
  for (let i = 0; i < p.count; i++) {
    const u = p.getX(i) * 2;          // -1..1
    const v = 0.5 - p.getY(i);         // 0 top .. 1 bottom
    const x = u * halfW;
    const ax = Math.abs(x);
    // depth off the body: hugs torso near centre, drapes off arms on the sides
    const bodyZ = 1.02 * Math.sqrt(Math.max(0, 1 - Math.pow(ax / 1.75, 2))) + 0.22;
    const drape = THREE.MathUtils.smoothstep(v, 0, 0.16);
    let z = THREE.MathUtils.lerp(ax < 1.2 ? bodyZ * 0.8 : 0.02, bodyZ, drape);
    z += 0.18 * v * (1 - ax / halfW);                 // belly bulge forward lower down
    z += 0.09 * Math.sin(x * 3.1 + v * 5) + 0.05 * Math.sin(x * 7.3 - v * 3); // soft folds
    // top edge sits on the shoulders / arms, rises toward the neck
    let y = topY + 0.35 * Math.exp(-ax * ax) - v * H;
    y -= 0.25 * THREE.MathUtils.smoothstep(ax, 1.2, halfW) * (1 - v * 0.3); // slight sag over arms
    const xx = x * (1 - 0.06 * v);
    p.setXYZ(i, xx, y, z * sign);
    base[i * 3] = xx; base[i * 3 + 1] = y; base[i * 3 + 2] = z * sign;
  }
  if (!front) { // mirror UVs so outline matches
    const uv = g.attributes.uv;
    for (let i = 0; i < uv.count; i++) uv.setX(i, 1 - uv.getX(i));
    g.index.array.reverse();
  }
  g.computeVertexNormals();
  g.userData.base = base;
  return g;
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------
export function createBlueBunnyModel(spec = BUNNY_SPEC, options = {}) {
  const S = { ...BUNNY_SPEC, ...spec, palette: { ...BUNNY_SPEC.palette, ...(spec.palette || {}) } };
  const M = buildMaterials(S);
  const R = rng(S.seed);
  const root = new THREE.Group();
  root.name = 'BlueBunny';
  const parts = [];      // clickable meshes
  const furTargets = []; // meshes that receive fur shells
  const rig = {};
  const explode = [];    // {obj, dir}

  const pivot = (name, parent, x = 0, y = 0, z = 0) => {
    const o = new THREE.Group();
    o.name = name; o.position.set(x, y, z);
    parent.add(o); rig[name] = o;
    return o;
  };
  const mesh = (name, geo, mat, parent, opts = {}) => {
    const m = new THREE.Mesh(geo, mat);
    m.name = name; m.castShadow = true; m.receiveShadow = true;
    m.userData.label = opts.label || name;
    m.userData.part = opts.part || name;
    parent.add(m);
    if (opts.clickable !== false) parts.push(m);
    if (opts.fur) furTargets.push({ mesh: m, dark: opts.fur === 'dark', len: opts.furLen || S.furLength });
    if (opts.explode) explode.push({ obj: m, dir: new THREE.Vector3(...opts.explode), base: m.position.clone() });
    return m;
  };

  // ---------------- hierarchy -----------------
  const hips = pivot('hips', root, 0, S.hipY, 0);
  const spine = pivot('spine', hips, 0, 0.3, 0);
  const chest = pivot('chest', spine, 0, S.chestY - S.hipY - 0.3, 0);
  const neck = pivot('neck', chest, 0, S.neckY - S.chestY, 0);
  const head = pivot('head', neck, 0, 0.25, 0);

  // ---------------- torso -----------------
  const torsoGeo = lathe([[0, 0.02], [0.04, 0.7], [0.15, 0.95], [0.35, 1.0], [0.55, 0.92], [0.75, 0.82], [0.9, 0.62], [0.97, 0.5], [1, 0.02]], 4.3, 28);
  torsoGeo.scale(1, 1, 0.82);
  torsoGeo.translate(0, -0.45, 0);
  mesh('torso', torsoGeo, M.fur, hips, { label: 'Busto', fur: true, explode: [0, 0, 0] });
  // tail (inferred — not visible in the reference)
  mesh('tail', blob(0.55, 0.55, 0.5, null, 20, 14), M.fur, hips, { label: 'Coda (dedotta)', fur: true, furLen: 0.18, explode: [0, 0, -1.5] })
    .position.set(0, 0.2, -0.85);

  // ---------------- head -----------------
  const headGeo = blob(1.56, 1.85, 1.5, (v) => {
    // widen lower half (jowls), slight flat top, muzzle bump
    const low = Math.max(0, -v.y);
    v.x *= 1 + 0.12 * low;
    v.z *= 1 + 0.06 * low;
    const mz = Math.exp(-((v.x / 0.45) ** 2 + ((v.y - 0.02) / 0.3) ** 2));
    if (v.z > 0) v.z += 0.2 * mz;
    const br = Math.exp(-((Math.abs(v.x) - 0.38) ** 2 / 0.05 + ((v.y - 0.6) ** 2) / 0.02));
    if (v.z > 0) v.z += 0.05 * br; // brow ridge
  });
  headGeo.translate(0, 1.85, 0);
  const headMesh = mesh('head', headGeo, M.fur, head, { label: 'Testa', fur: true, explode: [0, 1.2, 0] });
  // raycast the face in head-local space (pivot temporarily at the origin)
  const headPos = head.position.clone();
  head.position.set(0, 0, 0);
  head.updateMatrixWorld(true);

  const hc = 1.85; // head centre (local y)
  // eyes (glossy black + catchlight)
  [1, -1].forEach((s) => {
    const sp = surfacePoint(headMesh, 0.6 * s, hc + 0.72);
    const e = mesh(s > 0 ? 'eye-l' : 'eye-r', blob(0.21, 0.29, 0.15, null, 20, 14), M.eye, head, { label: 'Occhio', explode: [0.3 * s, 0.3, 1.2] });
    e.position.copy(sp.point).addScaledVector(sp.normal, 0.05);
    e.lookAt(e.position.clone().add(sp.normal));
    const cl = new THREE.Mesh(new THREE.SphereGeometry(0.035, 8, 6), M.catchlight);
    cl.position.set(0.05, 0.07, 0.11);
    e.add(cl);
    // brow: thick bent capsule angled down toward the centre (angry)
    const bp = surfacePoint(headMesh, 0.58 * s, hc + 1.1);
    const browGeo = new THREE.CapsuleGeometry(0.09, 0.56, 4, 10);
    browGeo.rotateZ(Math.PI / 2);
    const bpos = browGeo.attributes.position;
    for (let i = 0; i < bpos.count; i++) { const x = bpos.getX(i); bpos.setY(i, bpos.getY(i) - 0.55 * x * x); }
    browGeo.computeVertexNormals();
    const b = mesh(s > 0 ? 'brow-l' : 'brow-r', browGeo, M.brow, head, { label: 'Sopracciglio', explode: [0.3 * s, 0.6, 1.2] });
    b.position.copy(bp.point).addScaledVector(bp.normal, 0.06);
    b.lookAt(b.position.clone().add(bp.normal));
    b.rotateZ(0.38 * s);
    rig[s > 0 ? 'browL' : 'browR'] = b;
  });
  // nose: rounded inverted triangle
  {
    const sp = surfacePoint(headMesh, 0, hc + 0.3);
    const shape = new THREE.Shape();
    shape.moveTo(-0.17, 0.08); shape.quadraticCurveTo(0, 0.14, 0.17, 0.08);
    shape.quadraticCurveTo(0.12, -0.06, 0, -0.12); shape.quadraticCurveTo(-0.12, -0.06, -0.17, 0.08);
    const ng = new THREE.ExtrudeGeometry(shape, { depth: 0.08, bevelEnabled: true, bevelThickness: 0.05, bevelSize: 0.04, bevelSegments: 4 });
    const n = mesh('nose', ng, M.nose, head, { label: 'Naso', explode: [0, 0, 1.6] });
    n.position.copy(sp.point).add(new THREE.Vector3(0, 0, -0.06));
  }
  // mouth: "w" line + chin crease, projected on the face
  {
    const ptsXY = [[-0.36, -0.3], [-0.2, -0.4], [-0.07, -0.37], [0, -0.26], [0.07, -0.37], [0.2, -0.4], [0.36, -0.3]];
    const pts = ptsXY.map(([x, y]) => surfacePoint(headMesh, x, hc + y).point.add(new THREE.Vector3(0, 0, 0.04)));
    const tube = new THREE.TubeGeometry(new THREE.CatmullRomCurve3(pts), 30, 0.022, 6, false);
    mesh('mouth', tube, M.mouth, head, { label: 'Bocca', explode: [0, -0.3, 1.4] });
    const philtrum = [surfacePoint(headMesh, 0, hc + 0.2).point, surfacePoint(headMesh, 0, hc - 0.26).point];
    mesh('philtrum', new THREE.TubeGeometry(new THREE.LineCurve3(philtrum[0], philtrum[1]), 2, 0.015, 5, false), M.mouth, head, { clickable: false });
  }
  head.position.copy(headPos);
  head.updateMatrixWorld(true);

  // cheek fur tufts: instanced bent locks fanning sideways (thin, numerous, mixed teal)
  const tuftMat = new THREE.MeshStandardMaterial({ color: '#ffffff', roughness: 0.9, map: M.fur.map });
  M.tuft = tuftMat;
  [1, -1].forEach((s) => {
    const group = pivot(s > 0 ? 'cheekTuftL' : 'cheekTuftR', head, 1.2 * s, hc - 0.7, 0.25);
    const variants = [lockGeometry(1.0, 0.17, -0.3), lockGeometry(1.0, 0.13, 0.25), lockGeometry(1.0, 0.19, -0.45), lockGeometry(1.0, 0.11, 0.4)];
    const counts = [110, 100, 70, 80];
    const X = new THREE.Vector3(1, 0, 0);
    variants.forEach((geo, vi) => {
      const im = new THREE.InstancedMesh(geo, tuftMat, counts[vi]);
      im.name = 'cheek-tuft'; im.userData.label = 'Ciuffi delle guance'; im.userData.part = 'cheek-tuft';
      im.castShadow = true;
      const m4 = new THREE.Matrix4(), q = new THREE.Quaternion(), tw = new THREE.Quaternion(), sc = new THREE.Vector3(), pos = new THREE.Vector3(), d = new THREE.Vector3();
      const col = new THREE.Color();
      for (let i = 0; i < counts[vi]; i++) {
        // fan: mostly sideways, upper locks flick up, lower locks droop (as in the reference)
        const a = (R() * 2 - 1) * 0.68 - 0.2;
        const len = (1.2 + R() * 1.4) * (1 - Math.abs(a) * 0.35);
        pos.set(s * (R() * 0.45 - 0.1), (R() - 0.5) * 1.2 + a * 0.4, (R() - 0.35) * 0.9);
        d.set(s * Math.cos(a), Math.sin(a), (R() - 0.4) * 0.7).normalize();
        q.setFromUnitVectors(X, d);
        tw.setFromAxisAngle(X, (R() - 0.5) * 1.2);
        q.multiply(tw);
        sc.set(len, 0.7 + R() * 0.7, 0.7 + R() * 0.7);
        m4.compose(pos, q, sc);
        im.setMatrixAt(i, m4);
        col.setHSL(0.505 + R() * 0.02, 0.45 + R() * 0.15, 0.2 + R() * 0.15);
        im.setColorAt(i, col);
      }
      group.add(im);
      parts.push(im);
    });
    // base mass so the tuft root reads as continuous fur
    const baseMass = mesh(s > 0 ? 'cheek-l' : 'cheek-r', blob(0.8, 0.75, 0.7, null, 24, 16), M.furDark, group, { label: 'Ciuffi delle guance', fur: 'dark', furLen: 0.12 });
    baseMass.position.set(0.1 * s, 0, 0);
    explode.push({ obj: group, dir: new THREE.Vector3(1.5 * s, 0, 0.3), base: group.position.clone() });
  });

  // ears
  [1, -1].forEach((s) => {
    const earPivot = pivot(s > 0 ? 'earL' : 'earR', head, 0.74 * s, hc + 1.35, -0.1);
    earPivot.rotation.z = -0.1 * s;
    earPivot.rotation.x = -0.06;
    const L = 5.7;
    const outer = lathe([[0, 0.36], [0.08, 0.54], [0.3, 0.66], [0.55, 0.64], [0.78, 0.52], [0.93, 0.3], [1, 0.02]], L, 24);
    outer.scale(1, 1, 0.32);
    // cup the ear: push the front face back
    const op = outer.attributes.position;
    for (let i = 0; i < op.count; i++) { const x = op.getX(i); op.setZ(i, op.getZ(i) - 0.1 * (x * x) / 0.3); }
    outer.computeVertexNormals();
    mesh(s > 0 ? 'ear-l' : 'ear-r', outer, M.fur, earPivot, { label: 'Orecchio', fur: true, furLen: 0.07, explode: [0.6 * s, 1.5, 0] });
    const inner = lathe([[0, 0.12], [0.1, 0.32], [0.3, 0.44], [0.55, 0.43], [0.78, 0.33], [0.92, 0.16], [1, 0.01]], L * 0.94, 20);
    inner.scale(1, 1, 0.08);
    const ip = inner.attributes.position;
    for (let i = 0; i < ip.count; i++) { const x = ip.getX(i); ip.setZ(i, ip.getZ(i) - 0.1 * (x * x) / 0.3); }
    inner.computeVertexNormals();
    const inn = mesh(s > 0 ? 'inner-ear-l' : 'inner-ear-r', inner, M.innerEar, earPivot, { label: 'Interno orecchio', explode: [0.6 * s, 1.5, 0.3] });
    inn.position.set(0, 0.2, 0.25);
  });

  // ---------------- arms -----------------
  [1, -1].forEach((s) => {
    const side = s > 0 ? 'L' : 'R';
    const shoulder = pivot('shoulder' + side, chest, S.shoulderX * s, S.shoulderY - S.chestY, 0);
    const upper = limb(2.35, 0.36, 0.27);
    upper.rotateZ(-Math.PI / 2 * s); // +Y -> ±X
    mesh('upper-arm-' + side.toLowerCase(), upper, M.fur, shoulder, { label: 'Braccio', fur: true, explode: [0.8 * s, 0, 0] });
    const elbow = pivot('elbow' + side, shoulder, 2.2 * s, 0, 0);
    const fore = limb(2.1, 0.27, 0.3);
    fore.rotateZ(-Math.PI / 2 * s);
    mesh('forearm-' + side.toLowerCase(), fore, M.fur, elbow, { label: 'Avambraccio', fur: true, explode: [1.4 * s, 0, 0] });
    const wrist = pivot('wrist' + side, elbow, 2.0 * s, -0.03, 0);
    wrist.rotation.z = -0.12 * s; // paws droop slightly
    const palm = blob(0.5, 0.2, 0.42, null, 20, 14);
    palm.translate(0.35 * s, 0, 0);
    mesh('paw-' + side.toLowerCase(), palm, M.fur, wrist, { label: 'Zampa', fur: true, furLen: 0.05, explode: [2.0 * s, 0, 0] });
    // four fingers + claws, spread like the reference
    const spread = [-0.42, -0.14, 0.14, 0.42];
    spread.forEach((a, i) => {
      const f = pivot(`finger${side}${i}`, wrist, 0.62 * s, -0.02, a * 0.45);
      f.rotation.y = -a * s * 0.9;
      f.rotation.z = -0.15 * s;
      const len = i === 0 ? 0.42 : 0.55;
      const fg = new THREE.CapsuleGeometry(0.085, len, 4, 8);
      fg.rotateZ(-Math.PI / 2 * s);
      fg.translate((len / 2 + 0.05) * s, 0, 0);
      mesh(`finger-${side.toLowerCase()}${i}`, fg, M.fur, f, { label: 'Dito', explode: [2.4 * s, 0, a] });
      const claw = new THREE.ConeGeometry(0.05, 0.22, 8);
      claw.rotateZ(-Math.PI / 2 * s);
      claw.translate((len + 0.22) * s, -0.02, 0);
      mesh(`claw-${side.toLowerCase()}${i}`, claw, M.claw, f, { label: 'Artiglio', explode: [2.6 * s, 0, a] });
    });
  });

  // ---------------- legs -----------------
  [1, -1].forEach((s) => {
    const side = s > 0 ? 'L' : 'R';
    const hip = pivot('hip' + side, hips, S.legX * s, -0.2, 0);
    const thigh = lathe([[0, 0.5], [0.15, 0.5], [0.6, 0.44], [1, 0.4]], 2.7, 20);
    thigh.rotateX(Math.PI); // grow downward
    mesh('thigh-' + side.toLowerCase(), thigh, M.fur, hip, { label: 'Coscia', fur: true, explode: [0.5 * s, -0.4, 0] });
    const knee = pivot('knee' + side, hip, 0, -2.6, 0);
    const shin = lathe([[0, 0.4], [0.5, 0.39], [0.85, 0.42], [1, 0.48]], 2.75, 20);
    shin.rotateX(Math.PI);
    mesh('shin-' + side.toLowerCase(), shin, M.fur, knee, { label: 'Gamba', fur: true, explode: [0.7 * s, -0.8, 0] });
    const ankle = pivot('ankle' + side, knee, 0, -2.85, 0);
    ankle.rotation.y = 0.18 * s; // toes point slightly outward
    // big oval foot + three toe pads in front
    const foot = blob(1.6, 0.76, 1.65, (v) => { if (v.y < 0) v.y *= 0.55; }, 32, 20);
    foot.translate(0.8 * s, 0.05, 0.55);
    const fm = mesh('foot-' + side.toLowerCase(), foot, M.fur, ankle, { label: 'Piede', fur: true, explode: [1.0 * s, -1.2, 0.4] });
    [-0.62, 0, 0.62].forEach((dx, i) => {
      const toe = blob(0.48, 0.52, 0.46, (v) => { if (v.y < 0) v.y *= 0.5; }, 16, 12);
      toe.translate(0.8 * s + dx * 1.25, -0.08, 1.8 - Math.abs(dx) * 0.3);
      mesh(`toe-${side.toLowerCase()}${i}`, toe, M.fur, ankle, { label: 'Dita del piede', fur: true, furLen: 0.05, explode: [1.0 * s, -1.2, 1.0] });
    });
    rig['foot' + side] = fm;
  });

  // ---------------- poncho -----------------
  const ptex = ponchoTexture(S.seed + 7);
  const ponchoMatOut = new THREE.MeshStandardMaterial({ color: S.palette.poncho, map: ptex, alphaTest: 0.5, roughness: 0.95, side: THREE.FrontSide, bumpMap: ptex, bumpScale: 0.4 });
  const ponchoMatIn = new THREE.MeshStandardMaterial({ color: S.palette.ponchoInner, map: ptex, alphaTest: 0.5, roughness: 1, side: THREE.BackSide });
  const ponchoPivot = pivot('poncho', chest, 0, -S.chestY, 0);
  const ponchoMeshes = [];
  [true, false].forEach((front) => {
    const geo = ponchoPanelGeometry(front, S);
    const out = mesh(front ? 'poncho-front' : 'poncho-back', geo, ponchoMatOut, ponchoPivot, { label: 'Poncho strappato', part: 'poncho', explode: [0, 0, front ? 2 : -2] });
    const inn = new THREE.Mesh(geo, ponchoMatIn);
    inn.name = 'poncho-inner'; inn.userData.label = 'Poncho strappato'; inn.castShadow = true;
    out.add(inn); parts.push(inn);
    ponchoMeshes.push(out);
  });

  // ---------------- fur shells -----------------
  const furShellGroup = [];
  const shellMats = [];
  for (let l = 1; l <= S.furShells; l++) {
    const t = l / S.furShells;
    shellMats.push({
      light: makeFurShellMaterial(t, M.furColorU, M.furTipU, 1, 70),
      dark: makeFurShellMaterial(t, M.furDarkColorU, M.furDarkTipU, 1, 45),
    });
  }
  furTargets.forEach(({ mesh: m, dark, len }) => {
    shellMats.forEach((sm) => {
      const mat = dark ? sm.dark : sm.light;
      const s = new THREE.Mesh(m.geometry, mat);
      s.onBeforeRender = () => { mat.uniforms.uLen.value = len; };
      s.raycast = () => {};
      s.name = m.name + '-fur';
      m.add(s);
      furShellGroup.push(s);
    });
  });

  // ---------------- API -----------------
  root.userData = {
    spec: S,
    parts,
    rig,
    materials: M,
    poncho: ponchoMeshes,
    ponchoMaterials: [ponchoMatOut, ponchoMatIn],
    setFurVisible(v) { furShellGroup.forEach((s) => (s.visible = v)); },
    setFurColor(hex) {
      const c = new THREE.Color(hex);
      M.fur.color.copy(c);
      M.furColorU.value.copy(c);
      M.furTipU.value.copy(c).lerp(new THREE.Color('#ffffff'), 0.3);
      const d = c.clone().multiplyScalar(0.62);
      M.furDark.color.copy(d); M.furDarkColorU.value.copy(d); M.furDarkTipU.value.copy(c);
      const b0 = new THREE.Color(S.palette.fur);
      M.tuft.color.setRGB(c.r / b0.r, c.g / b0.g, c.b / b0.b);
    },
    setPonchoColor(hex) {
      const c = new THREE.Color(hex);
      ponchoMatOut.color.copy(c); ponchoMatIn.color.copy(c).multiplyScalar(0.6);
    },
    setExplode(k) { explode.forEach(({ obj, dir, base }) => obj.position.copy(base).addScaledVector(dir, k)); },
    // cloth flutter: cheap per-vertex wave, stronger toward the hem
    updateCloth(time, energy = 1) {
      ponchoMeshes.forEach((m) => {
        const g = m.geometry, p = g.attributes.position, base = g.userData.base;
        for (let i = 0; i < p.count; i++) {
          const y = base[i * 3 + 1];
          const w = THREE.MathUtils.clamp((10 - y) / 7.2, 0, 1);
          const k = w * w * energy;
          p.setXYZ(i,
            base[i * 3] + Math.sin(time * 2.1 + y * 1.3) * 0.04 * k,
            base[i * 3 + 1] + Math.sin(time * 3 + base[i * 3] * 1.7) * 0.03 * k,
            base[i * 3 + 2] + Math.sin(time * 2.6 + base[i * 3] * 2.3 + y) * 0.12 * k);
        }
        p.needsUpdate = true;
        g.computeVertexNormals();
      });
    },
  };
  return root;
}

// ---------------------------------------------------------------------------
// Procedural animation clips (applied to the rig every frame)
// ---------------------------------------------------------------------------
export const ANIMATIONS = ['Idle', 'Saluto', 'Salto', 'Ballo', 'Orecchie', 'T-pose'];

export function animateBunny(model, clip, t) {
  const r = model.userData.rig;
  // reset to bind pose
  Object.values(r).forEach((o) => { if (o.userData.bind === undefined) o.userData.bind = { p: o.position.clone(), q: o.quaternion.clone() }; });
  Object.values(r).forEach((o) => { o.position.copy(o.userData.bind.p); o.quaternion.copy(o.userData.bind.q); });
  const add = (o, x = 0, y = 0, z = 0) => { o.rotateX(x); o.rotateY(y); o.rotateZ(z); };
  let cloth = 0.4;
  const breathe = Math.sin(t * 1.6);
  if (clip !== 'T-pose') {
    r.chest.scale.setScalar(1 + breathe * 0.008);
    add(r.earL, 0, 0, Math.sin(t * 1.3) * 0.03);
    add(r.earR, 0, 0, Math.sin(t * 1.3 + 1) * 0.03);
  }
  switch (clip) {
    case 'Idle': {
      add(r.shoulderL, 0, 0, -1.2 + Math.sin(t * 1.6) * 0.03);
      add(r.shoulderR, 0, 0, 1.2 - Math.sin(t * 1.6) * 0.03);
      add(r.elbowL, 0, -0.25, 0); add(r.elbowR, 0, 0.25, 0);
      add(r.head, Math.sin(t * 0.7) * 0.04, Math.sin(t * 0.5) * 0.15, 0);
      if ((t % 4) < 0.25) { add(r.earL, -0.3, 0, 0); }
      break;
    }
    case 'Saluto': {
      add(r.shoulderR, 0, 0, 1.2);
      add(r.elbowR, 0, 0.25, 0);
      add(r.shoulderL, 0, 0, 0.55);
      add(r.elbowL, 0, 0, 1.25 + Math.sin(t * 7) * 0.35);
      add(r.wristL, 0, 0, Math.sin(t * 7 + 0.6) * 0.3);
      add(r.head, 0, 0.15, 0.1 * Math.sin(t * 3.5));
      add(r.browL, 0, 0, 0.15); add(r.browR, 0, 0, -0.15);
      cloth = 0.6;
      break;
    }
    case 'Salto': {
      const ph = (t * 1.4) % 1;
      const h = Math.max(0, Math.sin(ph * Math.PI * 2)) * 2.4;
      const squash = ph > 0.5 ? Math.sin((ph - 0.5) * Math.PI * 2) : 0;
      r.hips.position.y += h - squash * 0.4;
      add(r.hipL, -squash * 0.5, 0, 0); add(r.kneeL, squash * 0.9, 0, 0);
      add(r.hipR, -squash * 0.5, 0, 0); add(r.kneeR, squash * 0.9, 0, 0);
      add(r.ankleL, -squash * 0.4, 0, 0); add(r.ankleR, -squash * 0.4, 0, 0);
      add(r.shoulderL, 0, 0, 0.4 - h * 0.25); add(r.shoulderR, 0, 0, -0.4 + h * 0.25);
      add(r.earL, h * 0.12, 0, 0); add(r.earR, h * 0.12, 0, 0);
      cloth = 1.5;
      break;
    }
    case 'Ballo': {
      const b = Math.sin(t * 6);
      r.hips.position.x += b * 0.35;
      r.hips.position.y += Math.abs(Math.cos(t * 6)) * 0.3;
      add(r.hips, 0, b * 0.3, b * 0.08);
      add(r.chest, 0, -b * 0.2, -b * 0.1);
      add(r.shoulderL, 0, 0, 0.3 + Math.sin(t * 6) * 0.6); add(r.elbowL, 0, 0, 0.6 + Math.cos(t * 6) * 0.5);
      add(r.shoulderR, 0, 0, -0.3 + Math.sin(t * 6) * 0.6); add(r.elbowR, 0, 0, -0.6 + Math.cos(t * 6) * 0.5);
      add(r.head, 0, 0, -b * 0.15);
      add(r.earL, 0, 0, -b * 0.3); add(r.earR, 0, 0, -b * 0.3);
      add(r.hipL, Math.max(0, b) * -0.5, 0, 0); add(r.kneeL, Math.max(0, b) * 0.9, 0, 0);
      add(r.hipR, Math.max(0, -b) * -0.5, 0, 0); add(r.kneeR, Math.max(0, -b) * 0.9, 0, 0);
      cloth = 1.2;
      break;
    }
    case 'Orecchie': {
      add(r.earL, -0.25 + Math.sin(t * 5) * 0.35, 0, -0.25 + Math.sin(t * 5) * 0.25);
      add(r.earR, -0.25 + Math.sin(t * 5 + Math.PI) * 0.35, 0, 0.25 - Math.sin(t * 5 + Math.PI) * 0.25);
      add(r.head, 0, Math.sin(t * 2) * 0.25, 0);
      add(r.shoulderL, 0, 0, -1.2); add(r.shoulderR, 0, 0, 1.2);
      break;
    }
    default: cloth = 0.15;
  }
  model.userData.updateCloth(t, cloth);
}
