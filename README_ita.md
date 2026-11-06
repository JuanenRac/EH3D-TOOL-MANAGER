<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-TOOL-MANAGER banner" width="100%">
</p>

# <img src="images/brand/png/icon-64.png" alt="" width="40" height="40" align="absmiddle"> EH3D-TOOL-MANAGER

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | 🇮🇹 <b>Italiano</b> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### Il magazzino utensili della EH3D Fabrication Suite: utensili, usura, avanzamenti e velocità

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Version-0.0.7-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Verifica di onestà - cosa funziona oggi:** una libreria Rust e un programma a riga di comando (`eh3d-tools`) con 138 test eseguiti su un computer: conserva gli utensili con geometria e usura, li memorizza in JSON o CSV e calcola avanzamenti e velocità di partenza per materiale. Una finestra (`eh3d-tools-gui`, compilata con la funzione `gui`) è scritta e la sua logica è testata, ma non è stata compilata né guardata su uno schermo; nulla è stato provato su una macchina e nulla è stato verificato su una raccolta di file reali.

---

> **Stato di sicurezza:** questo programma non controlla alcuna macchina e non è un sistema di sicurezza. Ciò che produce deve essere controllato da una persona prima di eseguire qualsiasi cosa su una macchina; nulla di questo ha girato su hardware reale.


## 📋 In breve

| | |
|---|---|
| Versione | `0.0.7` |
| Maturità | Impalcatura |
| Test | 138 |
| Stack tecnologico | Rust |
| Su hardware reale | Non provato: nulla di questo ha girato su una macchina |

**Vai a:** [Panoramica](#-panoramica) · [Architettura](#-architettura) · [Avvio rapido](#-avvio-rapido) · [Test](#-test) · [Struttura](#-struttura-del-repository) · [Correlati](#-progetti-correlati)

## 🚦 Stato del progetto

Ogni riga è una parte di questo repository e il semaforo dice a che punto è oggi. È scritta in base a ciò che il repository può mostrare (test, compilazioni, esecuzioni), non a ciò che è previsto.

🟢 Fatto e verificato · 🟡 Funziona, con limiti noti · 🟠 Fatto a metà o non ancora verificato · 🔴 Non fatto o non funzionante

| Area | Stato | Cosa significa qui |
|---|---|---|
| Nucleo | 🟡 | conserva gli utensili con geometria e usura, li memorizza in JSON o CSV e calcola avanzamenti e velocità di partenza per materiale; provato su un computer con input normali e danneggiati, non con una raccolta di file reali. |
| Riga di comando | 🟡 | `eh3d-tools` esegue le stesse funzioni della libreria e stampa il risultato; ogni errore torna come messaggio e codice di uscita diverso da zero. |
| Test | 🟡 | 138 test passano (`cargo test`), con casi normali, casi limite e input danneggiati. |
| Sicurezza | 🟡 | Rust sicuro e semplice (`forbid(unsafe_code)`): nulla va in panico con i file ricevuti e un valore errato è un errore con il suo motivo; nessuna revisione indipendente. |
| Finestra | 🟠 | Scritta (egui, `--features gui`, programma `eh3d-tools-gui`): una tabella degli utensili che si aggiungono, modificano e rimuovono con i controlli della riga di comando, filtri e ricerca, importazione ed esportazione CSV con gli errori della libreria e un calcolatore di avanzamenti e velocità che mostra formule e condizioni; la sua logica è testata, ma non è ancora stata compilata né guardata su uno schermo. |
| Hardware reale | 🔴 | Non provato: nulla di questo è stato eseguito su una macchina. |
| Documentazione | 🟡 | README in otto lingue, il changelog e le due immagini disegnate da `docs/`. |

## 🎯 Panoramica

* **Il magazzino** (`src/model.rs`): punte, frese, frese a V, incisori, laser e ugelli con diametro, taglienti, lunghezza, angolo di punta e gambo; ogni valore è controllato e un nome ripetuto è rifiutato; ogni utensile riceve il suo numero. Un contatore di usura o un numero di taglienti che non sia intero viene rifiutato con il nome del campo, nei file e sulla riga di comando.
* **L'usura** (`src/model.rs`): ore, metri e fori si sommano dopo ogni lavoro; con un limite di ore o fori, un utensile è buono, da sostituire presto (dal 90 %) o da sostituire, e `wear` elenca i peggiori per primi.
* **Archiviazione e avanzamenti e velocità** (`src/store.rs`, `src/csv.rs`, `src/calc.rs`): il magazzino come documento JSON o tabella CSV per un foglio di calcolo, letto con cura e con la riga indicata in ogni errore; velocità di taglio e avanzamento per dente per FR4, alluminio, acrilico, legno e ottone, limitati dal mandrino.
* **La finestra** (`src/window.rs`, `src/gui.rs`; `--features gui`, programma `eh3d-tools-gui`): una tabella degli utensili con un modulo per aggiungerli, modificarli e rimuoverli con gli stessi controlli della riga di comando (il numero di un utensile rimosso non viene mai riassegnato), filtri e ricerca, l'uso di un utensile sommato ai suoi contatori, importazione ed esportazione CSV con gli errori della libreria e un calcolatore che mostra la formula e le condizioni di ogni numero; i numeri con cui lavora (velocità di taglio, avanzamenti per dente, fattori) sono in `tools.cfg` e si possono cambiare nella finestra. Stato, azioni, impostazioni e parole sono codice semplice con test; il disegno è sottile. Scritta, non ancora vista su uno schermo.

## 🧩 Ruolo in EH3D

EH3D-TOOL-MANAGER è un programma della EH3D Fabrication Suite. Funziona come libreria, come riga di comando e, compilato con la funzione `gui`, come finestra (`eh3d-tools-gui`) con l'aspetto comune agli altri programmi della suite. Legge e scrive file semplici, quindi può essere usato da solo o tra gli altri programmi.

## 🏛️ Architettura

<p align="center">
  <img src="images/architecture.svg" alt="Architettura di EH3D-TOOL-MANAGER: chi lo usa, che cosa contiene e da che cosa dipende" width="100%">
</p>


Come si incastrano le parti di questo repository: chi lo usa, cosa contiene e da cosa dipende. L'immagine è disegnata da `docs/architecture.json` e verificata con il codice.

## ✨ Funzionalità

* **Libreria e riga di comando:** ogni comando di `eh3d-tools` è una funzione della libreria, così un altro programma della suite può chiamarla senza avviare un processo.
* **Test con input danneggiati:** i test forniscono anche file vuoti, spazzatura, numeri enormi e file tagliati, e nessuno può andare in panico.
* **Nessuna ipotesi:** un valore privo di senso è un errore che dice quale e perché, mai un risultato che sembra solo giusto.
* **Nessun codice unsafe:** `#![forbid(unsafe_code)]` nella libreria e nel programma.
* **Un magazzino che regge:** il numero di un utensile rimosso non viene più riassegnato (il magazzino conserva `next_id`), un id che non è un numero intero viene indicato come tale, il CSV rifiuta testo dopo le virgolette di chiusura, un testo che inizia come una formula (`=`, `+`, `-`, `@`) viene scritto come testo perché il foglio di calcolo non lo esegua, il materiale ha un limite, `--holes` è un numero intero, un'opzione sconosciuta è un errore e `calc` dice che il risultato per una fresa a V o un incisore è solo un punto di partenza.
* **La larghezza del taglio di una fresa a V:** `eh3d-tools vbit --depth MM [--angle GRADI] [--flat MM]` calcola `2 x profondità x tan(angolo / 2) + punta piatta` (una fresa da 60 gradi a 0,5 mm dà 0,5774 mm più la punta piatta di 0,1 mm = 0,6774 mm); l'angolo va da 10 a 170 gradi, la profondità da 0 a 100 mm e la punta piatta da 0 a 5 mm, e ciò che non viene indicato è il valore di `tools.cfg`. `calc --depth MM` usa questa larghezza come diametro con cui lavora una fresa a V o un incisore negli avanzamenti e velocità, e il calcolatore della finestra ha un campo per la profondità, mostra la larghezza del taglio e la formula con i suoi numeri; senza profondità i numeri sono quelli di sempre.

## 🔒 Sicurezza e protezione

| Minaccia | Difesa |
|---|---|
| Un file danneggiato, tagliato o vuoto | nessun panico: i test tagliano un input valido in ogni posizione e forniscono spazzatura |
| Un numero privo di senso (negativo, non finito, assurdamente grande) | rifiutato con un errore che nomina il valore; nulla viene arrotondato in qualcosa di plausibile |
| Uso non sicuro della memoria | Rust sicuro e semplice con `forbid(unsafe_code)` |

**Cosa non è difeso:** un risultato prodotto senza errori non è un risultato verificato. Nulla qui conosce i limiti di una macchina e il programma non ha avuto revisione indipendente. Segnalate una vulnerabilità in privato (vedi [SECURITY.md](SECURITY.md)).

## 🔌 Interfacce

La libreria è `eh3d_tool_manager` (`use eh3d_tool_manager::...`); la riga di comando è `eh3d-tools`. I comandi sono questi; ciascuno stampa il risultato sull'output standard e segnala un errore sull'output di errore con un codice di uscita diverso da zero. Non c'è accesso alla rete.

```text
EH3D-TOOL-MANAGER/
├── src/
│   ├── calc.rs                                  una parte della logica
│   ├── cli_args.rs                              gli argomenti della riga di comando, letti con rigore
│   ├── csv.rs                                   una parte della logica
│   ├── gui.rs                                   la finestra (egui, con la funzione gui): disegna lo stato e inoltra ciò che fa la persona
│   ├── gui_main.rs                              la finestra come programma a sé, senza console su Windows
│   ├── lang.rs                                  le parole della finestra nelle otto lingue
│   ├── lib.rs                                   la libreria: ciò che usa ogni comando
│   ├── main.rs                                  la riga di comando
│   ├── model.rs                                 una parte della logica
│   ├── settings.rs                              le impostazioni di tools.cfg: lingua, dimensione della finestra, file del magazzino e numeri del calcolo
│   ├── shot.rs                                  la modalità immagine della finestra (--shot): i suoi argomenti, letti e verificati senza schermo
│   ├── store.rs                                 una parte della logica
│   └── window.rs                                lo stato della finestra senza schermo: il magazzino, il modulo, i filtri e il calcolatore
├── tests/
│   ├── crib.rs                                  test della libreria e della riga di comando
│   ├── shot_sample.rs                           test della cassetta di esempio e della finestra di un'immagine
│   └── vbit.rs                                  test della larghezza del taglio di una fresa a V: il comando, la profondità in calc e le impostazioni
├── examples/
│   └── crib.json                                un esempio per provare il programma
├── docs/
│   ├── architecture.json                        dati dell'immagine di Architettura
│   └── structure.json                           dati dell'immagine di Struttura
├── tools/                                       controllo della CI e del README e strumenti per le immagini
├── images/                                      banner e le immagini di Architettura e Struttura del repository
├── .github/                                     workflow di CI e di rilascio, modelli di issue e pull request
├── eh3d.project.json                            manifesto: versione, ruolo, maturità e comando di compilazione
├── Cargo.toml Cargo.lock                        il workspace e le sue dipendenze bloccate
├── rust-toolchain.toml                          la toolchain Rust fissata
├── CHANGELOG.md                                 cosa è cambiato in ogni versione
├── SECURITY.md SUPPORT.md                       come segnalare una vulnerabilità e dove chiedere
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   come contribuire, codice di condotta e licenza
└── tools.cfg.example                            tutte le impostazioni di tools.cfg con il valore predefinito
```

## ⚙️ Configurazione

Il file di configurazione è `tools.cfg` nella cartella del programma (la variabile `EH3D_TOOLS_DIR`, oppure `%APPDATA%\eh3d-tools` su Windows, oppure `~/.config/eh3d-tools` altrove), letto dalla finestra e dal comando `calc`: la lingua e la dimensione della finestra, il file del magazzino, la velocità del mandrino con cui parte il calcolatore e ogni numero con cui si calcolano avanzamenti e velocità (la velocità di taglio e l'avanzamento per dente di ogni materiale, i fattori dell'affondamento e della profondità di passata, e `v_bit_angle_deg` e `v_bit_flat_mm`, l'angolo (da 10 a 170 gradi, 60 per impostazione predefinita) e la larghezza della punta piatta (da 0 a 5 mm, 0,1 per impostazione predefinita) con cui si calcola una fresa a V) sono righe `chiave = valore`, con i limiti controllati e una riga sbagliata nominata nell'errore. `tools.cfg.example` elenca ogni chiave con il valore predefinito. A parte questo, ciò che la persona sceglie si passa come argomenti della riga di comando (vedi Interfacce) e ogni valore predefinito è scritto nella guida del comando (`eh3d-tools help`). I limiti di un valore sono costanti con nome nel codice, coperte da test.

## 🚀 Avvio rapido

Serve una toolchain Rust stabile (1.95 o più recente).

```bash
cargo build --release
cargo test
cargo run --release -- calc drill 0.8 fr4 24000
# la finestra (richiede la funzione gui)
cargo build --release --features gui
cargo run --release --features gui --bin eh3d-tools-gui
cargo run --release --features gui --bin eh3d-tools-gui -- --shot screen.png --open examples/crib.json   # a picture of the window
```

Per fare un'immagine della finestra senza aprirla a mano, eseguila con `--shot FILE.png` (anche `--size WxH`, `--theme forge|dark|light`, `--wait N`, `--lang CODE`, `--open FILE`, `--replace`): si apre con quella dimensione, salva il PNG, non conserva impostazioni e si chiude da sola (codice di uscita 0 se il file è stato scritto, 1 se no, 2 per argomenti errati).

## 🧪 Test

La suite ha 138 test (`cargo test`, eseguiti su Windows e Linux nella CI). Verificano i casi normali, i casi limite e gli input danneggiati: file vuoti, spazzatura e file tagliati non causano mai un panico. La CI esegue anche `cargo fmt --check`, `cargo clippy` (anche con `--features gui`) con gli avvisi come errori e `python -I tools/ci_validate.py`.

## 🏗️ Compilazione e rilascio

La versione sta in `Cargo.toml` e in `eh3d.project.json`; ogni versione ha la sua voce in [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` compilano il programma, `test.bat` / `test.sh` eseguono i test e `run.bat` / `run.sh` lo eseguono con un esempio. Le versioni sono pubblicate su GitHub.

### Download

Ogni versione contiene il programma a riga di comando e la finestra per Windows (64 e 32 bit), Linux (64 bit) e macOS (Apple silicon e Intel), come `<nome>-<versione>-<sistema>.zip` (Windows) o `.tar.gz` con i due programmi, la licenza, il readme, l'icona e `tools.cfg.example`. I file sono costruiti da GitHub Actions ed elencati in `SHA256SUMS`. Per verificare un download, confronta il suo hash con il file `.sha256` accanto (`sha256sum -c`).

## 📂 Struttura del repository

<p align="center">
  <img src="images/repository-structure.svg" alt="Struttura del repository EH3D-TOOL-MANAGER" width="100%">
</p>

<details>
<summary>Versione testuale dell'albero</summary>

```text
EH3D-TOOL-MANAGER/
├── src/
│   ├── calc.rs                                  una parte della logica
│   ├── cli_args.rs                              gli argomenti della riga di comando, letti con rigore
│   ├── csv.rs                                   una parte della logica
│   ├── gui.rs                                   la finestra (egui, con la funzione gui): disegna lo stato e inoltra ciò che fa la persona
│   ├── gui_main.rs                              la finestra come programma a sé, senza console su Windows
│   ├── lang.rs                                  le parole della finestra nelle otto lingue
│   ├── lib.rs                                   la libreria: ciò che usa ogni comando
│   ├── main.rs                                  la riga di comando
│   ├── model.rs                                 una parte della logica
│   ├── settings.rs                              le impostazioni di tools.cfg: lingua, dimensione della finestra, file del magazzino e numeri del calcolo
│   ├── shot.rs                                  la modalità immagine della finestra (--shot): i suoi argomenti, letti e verificati senza schermo
│   ├── store.rs                                 una parte della logica
│   └── window.rs                                lo stato della finestra senza schermo: il magazzino, il modulo, i filtri e il calcolatore
├── tests/
│   ├── crib.rs                                  test della libreria e della riga di comando
│   ├── shot_sample.rs                           test della cassetta di esempio e della finestra di un'immagine
│   └── vbit.rs                                  test della larghezza del taglio di una fresa a V: il comando, la profondità in calc e le impostazioni
├── examples/
│   └── crib.json                                un esempio per provare il programma
├── docs/
│   ├── architecture.json                        dati dell'immagine di Architettura
│   └── structure.json                           dati dell'immagine di Struttura
├── tools/                                       controllo della CI e del README e strumenti per le immagini
├── images/                                      banner e le immagini di Architettura e Struttura del repository
├── .github/                                     workflow di CI e di rilascio, modelli di issue e pull request
├── eh3d.project.json                            manifesto: versione, ruolo, maturità e comando di compilazione
├── Cargo.toml Cargo.lock                        il workspace e le sue dipendenze bloccate
├── rust-toolchain.toml                          la toolchain Rust fissata
├── CHANGELOG.md                                 cosa è cambiato in ogni versione
├── SECURITY.md SUPPORT.md                       come segnalare una vulnerabilità e dove chiedere
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   come contribuire, codice di condotta e licenza
└── tools.cfg.example                            tutte le impostazioni di tools.cfg con il valore predefinito
```

</details>

## 🗺️ Roadmap e limiti noti

**Limiti noti oggi**

* La finestra è scritta e la sua logica è testata, ma non è ancora stata compilata né guardata su uno schermo.
* La finestra chiede il file del magazzino e la tabella CSV come percorsi scritti (non c'è ancora una finestra di selezione dei file).
* Non è stato provato su una macchina né con una raccolta di file reali e non ha avuto revisione indipendente.
* Il magazzino è un file JSON, non un database; gli avanzamenti e le velocità sono punti di partenza da valori tipici e non sono stati verificati su una macchina; i programmi della suite non leggono ancora il magazzino.

**Prossimi passi**

* Far leggere il magazzino a DRILLPRO, ROUTEPRO e ai programmi CAM invece delle loro tabelle e aggiungere il profilo 2D di ogni utensile.
* Guardare la finestra su uno schermo di ogni sistema e correggere ciò che si trova.
* Provarlo con file reali e con gli altri programmi della suite.

## 🌍 Lingue

Il README e i testi esistono in inglese, spagnolo, francese, italiano, tedesco, cinese, giapponese e portoghese europeo.

## 🔗 Progetti correlati

* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): il contenitore di progetto e i tipi condivisi della suite
* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): il programma di foratura
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): il programma di fresatura del contorno
* [EH3D-ISOCAM](https://github.com/JuanenRac/EH3D-ISOCAM): la fresatura di isolamento del rame
* [EH3D-PHOENIX-LASER](https://github.com/JuanenRac/EH3D-PHOENIX-LASER): incisione e taglio laser
* [EH3D-PHOENIX-PNP](https://github.com/JuanenRac/EH3D-PHOENIX-PNP): l'assemblaggio pick and place
* [EH3D-SUITE-LAUNCHER](https://github.com/JuanenRac/EH3D-SUITE-LAUNCHER): il centro che trova e avvia i programmi della suite

## 📚 Documentazione

* [Changelog](CHANGELOG.md)
* [Dati di architettura e struttura](docs/architecture.json): i dati da cui si disegnano le due immagini.

## 🤝 Contribuire e comunità

* Prima di una pull request eseguite `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` e `python -I tools/ci_validate.py`: tutto deve passare. Vedi [CONTRIBUTING.md](CONTRIBUTING.md) e il [codice di condotta](CODE_OF_CONDUCT.md).
* Il README esiste in otto lingue: una modifica al testo si fa in tutti e otto i file, e le due immagini sono disegnate da `docs/architecture.json` e `docs/structure.json` con `tools/make_architecture.py` e `tools/make_structure.py`.
* Segnalate una vulnerabilità in privato: [SECURITY.md](SECURITY.md). Domande, idee e segnalazioni: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTORE

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENZA

Software: GPL-3.0-or-later. Vedi [LICENSE](LICENSE). I progetti hardware della suite usano la CERN OHL-S.
