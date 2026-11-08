# RepoHub

Gestore di repository GitHub personali, **solo interfaccia** (dati di esempio, nessuna chiamata
all'API di GitHub), scritto in Rust con [Rune](https://github.com/anomalyco/rune) e `rune-kit`.

- `web/` — build per il browser (WASM + WebGPU), servibile così com'è da GitHub Pages: `/repohub/web/`.
  Serve un browser con WebGPU (Chrome/Edge recenti, Safari 26+).
- `src/` — sorgente: `lib.rs` (stato, shell, sidebar, toolbar), `pages.rs` (panoramica, repository,
  dettaglio), `lists.rs` (PR, issue, Actions, impostazioni), `ui.rs` (componenti), `icons.rs`, `data.rs`.

## Interazioni
Sidebar ridimensionabile e richiudibile (⇧⌘B), sezione "Fissati" comprimibile, ricerca e filtro
dei repository, apertura del dettaglio, stella on/off, schede del repository, filtro PR,
tema scuro e interruttori delle impostazioni.

## Build
Rune è pre-release ed è fissato a una revisione di `dev` (vedi `Cargo.toml`).

```sh
cargo run --release                       # finestra desktop
target/release/repo-hub --snap out.png --theme dark --size 1180x760 -- page repos   # headless
```

Per il web: dal checkout di Rune, con il crate aggiunto al workspace,
`scripts/web-build.sh repo-hub --out web` (richiede `wasm32-unknown-unknown` e
`wasm-bindgen-cli 0.2.129`).
