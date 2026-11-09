# RepoHub

Gestore dei propri repository GitHub scritto in Rust con [Rune](https://github.com/anomalyco/rune)
e `rune-kit`.

- **App desktop:** legge i dati veri dalle API REST di GitHub (profilo, repository, file, README,
  linguaggi, ultimo commit, pull request, issue, esecuzioni di Actions, commit settimanali) e
  permette di mettere o togliere stelle. Si configura con un file `.env`.
- **Versione web** (`web/`, pubblicata su `/repohub/web/`): mostra i dati di esempio. Un token
  dentro una pagina web pubblica sarebbe visibile a chiunque, quindi lì non c'è nessuna connessione.

## Provarla in locale con i tuoi dati

Serve [Rust](https://rustup.rs). Su Linux servono anche `libxkbcommon-dev` e `libwayland-dev`.

```sh
cd repohub
cp .env.example .env      # poi apri .env e compila GITHUB_USER e/o GITHUB_TOKEN
cargo run --release       # la prima compilazione richiede qualche minuto
```

Il file `.env` è escluso da git (`.gitignore`): non va mai committato.

Configurazioni minime:

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

- Le variabili d'ambiente già impostate hanno la precedenza sul file, come di consueto.
  Attenzione se hai già un `GITHUB_TOKEN` esportato nella shell.
- `REPOHUB_ENV=/percorso/altro.env` usa un file diverso. Senza, il `.env` viene cercato nella
  cartella corrente e in quelle superiori.

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

`src/`: `lib.rs` (stato, caricamento in background, shell), `config.rs` (`.env`),
`github.rs` (client REST), `model.rs` (modello dati e dati di esempio), `pages.rs` e `lists.rs`
(pagine), `ui.rs`, `icons.rs`.

Per ricompilare `web/` servono il target `wasm32-unknown-unknown` e `wasm-bindgen-cli 0.2.129`.
Si usa `scripts/web-build.sh` dal checkout di Rune, con il crate aggiunto al suo workspace.
