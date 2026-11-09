# RepoHub

Gestore dei propri repository GitHub scritto in Rust con [Rune](https://github.com/anomalyco/rune)
e `rune-kit`. Legge i dati veri dalle API REST di GitHub (profilo, repository, file, README,
linguaggi, ultimo commit, pull request, issue, esecuzioni di Actions, commit settimanali) e
permette di mettere o togliere stelle.

Tre modi di usarla:

| | Cosa serve | Dati |
| --- | --- | --- |
| **Docker** (consigliato) | solo Docker e un browser con WebGPU | i tuoi, dal file `.env` |
| **App desktop** | Rust | i tuoi, dal file `.env` |
| **Pagina statica** `/repohub/web/` su GitHub Pages | un browser con WebGPU | solo di esempio |

## Docker

Non serve installare altro che [Docker](https://docs.docker.com/get-docker/) (Docker Desktop su
Windows e macOS).

```sh
git clone https://github.com/FrancescoDiBella/FrancescoDiBella.github.io
cd FrancescoDiBella.github.io/repohub
cp .env.example .env          # su Windows: copy .env.example .env — poi compila GITHUB_USER e/o GITHUB_TOKEN
docker compose up --build     # la prima volta compila tutto: 10-20 minuti
```

Poi apri **http://localhost:8080** con Chrome o Edge (serve WebGPU). `Ctrl+C` lo ferma,
`docker compose up -d` lo avvia in background, `docker compose down` lo spegne.

Dopo aver cambiato il `.env` basta `docker compose up -d --force-recreate`: non serve ricompilare.

### Come funziona e perché il token è al sicuro

Il container contiene due cose:
- la versione web di RepoHub (Rust compilato in WebAssembly);
- `repohub-server`, un piccolo server che serve la pagina e inoltra le chiamate a GitHub
  aggiungendo il token.

- Il token resta dentro il container: la pagina non lo riceve mai. `config.json` dice solo se un
  token c'è.
- La porta è pubblicata solo su questo computer (`127.0.0.1:8080`).
- Il server inoltra solo le chiamate che RepoHub usa: lettura, e scrittura solo per le stelle.
- Il server rifiuta le richieste che arrivano da altri siti o da nomi host estranei.
- Per aprirlo da un altro dispositivo della tua rete devi cambiare `ports` in `compose.yaml` e
  aggiungere il nome host a `REPOHUB_ALLOWED_HOSTS`. Fallo solo su una rete di cui ti fidi:
  chiunque raggiunga la porta usa il tuo token.

### Immagine già pronta (facoltativo)

La GitHub Action `.github/workflows/repohub-image.yml` pubblica l'immagine su
`ghcr.io/francescodibella/repohub` a ogni modifica di `repohub/` su `main`. Con l'immagine
pubblicata non serve compilare nulla:

```sh
docker run --rm --env-file .env -p 127.0.0.1:8080:8080 ghcr.io/francescodibella/repohub
```

Il primo pacchetto creato su GHCR è privato. Rendilo pubblico da GitHub → profilo → Packages →
repohub → Package settings, oppure esegui prima `docker login ghcr.io`.

## App desktop

Serve [Rust](https://rustup.rs). Su Linux servono anche `libxkbcommon-dev` e `libwayland-dev`.

```sh
cd repohub
cp .env.example .env      # poi apri .env e compila GITHUB_USER e/o GITHUB_TOKEN
cargo run --release       # la prima compilazione richiede qualche minuto
```

## Configurazione (`.env`)

Il file `.env` è escluso da git (`.gitignore`) e da Docker (`.dockerignore`): non va mai committato.

```sh
# Solo dati pubblici (60 richieste API l'ora, nessun repository privato)
GITHUB_USER=il-tuo-username

# Tutto, privati compresi (5000 richieste l'ora)
GITHUB_TOKEN=github_pat_...
```

Tutte le opzioni, con i permessi consigliati per il token, sono spiegate in
[`.env.example`](.env.example): titolo, tema, repository fissati, fork e archiviati, numero massimo
di repository, da quanti repository leggere Actions e attività, modalità demo, URL per GitHub
Enterprise.

- Le variabili d'ambiente già impostate hanno la precedenza sul file. Attenzione se hai già un
  `GITHUB_TOKEN` esportato nella shell (vale per l'app desktop).
- Solo per l'app desktop: `REPOHUB_ENV=/percorso/altro.env` usa un file diverso. Senza, il `.env`
  viene cercato nella cartella corrente e in quelle superiori.
- Solo per Docker: `REPOHUB_ALLOWED_HOSTS` aggiunge nomi host consentiti oltre a `localhost`.

## Cosa fa con i dati

| Dove | Dati e azioni | API usate |
| --- | --- | --- |
| Panoramica | numeri riassuntivi, commit delle ultime 26 settimane, repository recenti, PR aperte | `/user` o `/users/{u}`, `/user/repos` o `/users/{u}/repos`, `/search/issues`, `/repos/{r}/stats/participation` |
| Repository | ricerca per nome, descrizione e topic; filtri pubblici, privati e fork; stella | `/user/starred`, `PUT`/`DELETE /user/starred/{r}` |
| Dettaglio | file della radice, README, linguaggi, ultimo commit, copia dell'URL di clone, apri su GitHub | `/repos/{r}/contents`, `/readme`, `/languages`, `/commits` |
| Pull request | PR in cui sei coinvolto: aperte, bozze, unite, chiuse; clic per aprirle | `/search/issues?q=is:pr involves:{u}` |
| Issue | issue aperte nei tuoi repository; clic per aprirle | `/search/issues?q=is:issue is:open user:{u}` |
| Actions | ultime esecuzioni dei repository più recenti | `/repos/{r}/actions/runs` |
| Impostazioni | stato della connessione e configurazione letta (il token non viene mai mostrato) | — |

- **Aggiorna** (icona in alto, oppure ⌘R / Ctrl+R) ricarica tutto.
- Le richieste girano in background e la finestra resta reattiva.
- In caso di errore (token rifiutato, limite di richieste, utente inesistente) compare un avviso
  con il motivo e il pulsante **Riprova**.
- RepoHub non crea, modifica o elimina repository. "Nuovo repository" e "Impostazioni su GitHub"
  aprono le pagine di GitHub nel browser.

## Sviluppo

```sh
cargo test --release                                       # test di parser .env, date e helper
REPOHUB_BLOCKING=1 target/release/repo-hub --snap out.png -- page repos   # screenshot headless con dati veri
```

`src/`: `lib.rs` (stato, shell), `config.rs` (`.env` e `config.json`), `github.rs` (client
REST, comune a desktop e web), `net.rs` (trasporto: `ureq` sul desktop, `fetch` verso il proxy
sul web), `model.rs` (modello dati e dati di esempio), `pages.rs` e `lists.rs` (pagine), `ui.rs`,
`icons.rs`. `server/`: `repohub-server` (file statici, `config.json`, proxy `/gh/*`).

Provare la versione web con il server senza Docker:

```sh
cargo run --release --manifest-path server/Cargo.toml   # legge le variabili dall'ambiente
# WEB_DIR=web (default) · REPOHUB_ADDR=127.0.0.1:8080 (default)
```

Per ricompilare `web/` a mano servono il target `wasm32-unknown-unknown` e
`wasm-bindgen-cli 0.2.129`. I comandi sono quelli del `Dockerfile`:
`cargo build --release --lib --target wasm32-unknown-unknown`, poi
`wasm-bindgen --target web --out-dir web --out-name app target/wasm32-unknown-unknown/release/repo_hub.wasm`.
