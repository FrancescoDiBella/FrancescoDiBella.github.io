
export async function gh_fetch(method, url, accept) {
    try {
        const r = await fetch(url, { method, headers: { Accept: accept }, credentials: 'same-origin' });
        const body = await r.text();
        return [r.status, body, r.headers.get('x-ratelimit-remaining') || '', r.headers.get('x-ratelimit-reset') || ''];
    } catch (e) {
        return [0, String(e), '', ''];
    }
}
export function now_secs() { return Math.floor(Date.now() / 1000); }
