# Blue Bunny — modello Three.js procedurale

Ricostruzione **solo codice** (nessun file mesh, nessun asset scaricato) del coniglio blu in
`reference.jpg`, fatta seguendo la pipeline [img2threejs](https://github.com/img2threejs/img2threejs)
(skill v2.0.1, profilo `character`).

Apri `bunny/index.html` (su GitHub Pages: `/bunny/`).

## Interazione
- **Trascina / rotella / pinch**: orbita e zoom
- **Animazioni** procedurali sul rig: Idle, Saluto, Salto, Ballo, Orecchie, T-pose
- **Clic su una parte**: la evidenzia e ne mostra il nome (clic sulla testa = salto)
- **Esplodi parti**, **colore pelo / poncho**, pelliccia shell on/off, poncho on/off, wireframe,
  rotazione automatica, immagine di riferimento, screenshot PNG
- Parametri URL: `?anim=Ballo`, `?fur=0`

## File
- `createBlueBunnyModel.js` — factory `createBlueBunnyModel(spec)` → `THREE.Group` con gerarchia di
  pivot (hips → spine → chest → neck → head → ears; shoulder → elbow → wrist → fingers; hip → knee → ankle),
  pelliccia a shell, poncho strappato (griglia deformata + texture canvas con alpha), ciuffi delle
  guance istanziati; `animateBunny(model, clip, t)` per le animazioni.
- `img2threejs/` — evidenze della pipeline: analisi immagine, ammissione del riferimento,
  pre-spec assessment, diagnostica Tier‑1, foglio di confronto e viste turntable.

## Limiti (onestamente)
- Una sola vista frontale: retro, coda e profilo sono **dedotti**.
- Stilizzato: Silhouette IoU Tier‑1 ≈ 0.72 (soglia della pipeline 0.85) — il gate non è superato;
  il confronto visivo è in `img2threejs/comparison-front.jpg`.
- Il poncho è rigido rispetto al busto: nelle pose con braccia abbassate le braccia passano sotto il tessuto.
