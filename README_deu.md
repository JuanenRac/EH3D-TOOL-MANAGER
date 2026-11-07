<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-TOOL-MANAGER banner" width="100%">
</p>

# <img src="images/brand/png/icon-64.png" alt="" width="40" height="40" align="absmiddle"> EH3D-TOOL-MANAGER

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | 🇩🇪 <b>Deutsch</b> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### Das Werkzeuglager der EH3D Fabrication Suite: Werkzeuge, Verschleiß, Vorschübe und Drehzahlen

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Version-0.0.7-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Ehrlichkeitscheck - was heute wirklich läuft:** eine Rust-Bibliothek und ein Kommandozeilenprogramm (`eh3d-tools`) mit 147 Tests, die auf einem Computer laufen: verwaltet die Werkzeuge mit Geometrie und Verschleiß, speichert sie als JSON oder CSV und berechnet Start-Vorschübe und Drehzahlen je Material. Ein Fenster (`eh3d-tools-gui`, mit dem Feature `gui` gebaut) ist geschrieben und seine Logik getestet, aber es wurde weder gebaut noch auf einem Bildschirm angesehen; nichts wurde an einer Maschine ausprobiert und nichts an einer Sammlung echter Dateien geprüft.

---

> **Sicherheitsstatus:** Dieses Programm steuert keine Maschine und ist kein Sicherheitssystem. Was es erzeugt, muss ein Mensch prüfen, bevor etwas an einer Maschine läuft; nichts davon lief auf echter Hardware.


## 📋 Auf einen Blick

| | |
|---|---|
| Version | `0.0.7` |
| Reifegrad | Gerüst |
| Tests | 147 |
| Technologie-Stack | Rust |
| Auf echter Hardware | Nicht erprobt: nichts davon lief an einer Maschine |

**Springe zu:** [Überblick](#-überblick) · [Architektur](#-architektur) · [Schnellstart](#-schnellstart) · [Tests](#-tests) · [Struktur](#-struktur-des-repositorys) · [Verwandte](#-verwandte-projekte)

## 🚦 Projektstatus

Jede Zeile ist ein Teil dieses Repositorys, und die Ampel sagt, wie weit er heute ist. Sie beruht auf dem, was das Repository zeigen kann (Tests, Builds, Läufe), nicht auf dem, was geplant ist.

🟢 Fertig und überprüft · 🟡 Funktioniert, mit bekannten Grenzen · 🟠 Teilweise fertig oder noch nicht überprüft · 🔴 Nicht gemacht oder funktioniert nicht

| Bereich | Stand | Was das hier bedeutet |
|---|---|---|
| Kern | 🟡 | verwaltet die Werkzeuge mit Geometrie und Verschleiß, speichert sie als JSON oder CSV und berechnet Start-Vorschübe und Drehzahlen je Material; am Computer mit normalen und beschädigten Eingaben getestet, nicht mit einer Sammlung echter Dateien. |
| Kommandozeile | 🟡 | `eh3d-tools` führt dieselben Funktionen wie die Bibliothek aus und gibt das Ergebnis aus; jeder Fehler kommt als Meldung mit einem Exit-Code ungleich null zurück. |
| Tests | 🟡 | 147 Tests bestehen (`cargo test`), mit Normalfällen, Grenzfällen und beschädigten Eingaben. |
| Sicherheit | 🟡 | Reines sicheres Rust (`forbid(unsafe_code)`): nichts gerät bei den übergebenen Dateien in Panik und ein schlechter Wert ist ein Fehler mit Begründung; keine unabhängige Prüfung. |
| Fenster | 🟠 | Geschrieben (egui, `--features gui`, Programm `eh3d-tools-gui`): eine Tabelle der Werkzeuge, die mit den Prüfungen der Kommandozeile hinzugefügt, geändert und entfernt werden, Filter und Suche, CSV-Import und -Export mit den Fehlern der Bibliothek und ein Rechner für Vorschübe und Drehzahlen, der seine Formeln und Bedingungen zeigt; seine Logik ist getestet, aber es wurde noch nicht gebaut oder auf einem Bildschirm angesehen. |
| Echte Hardware | 🔴 | Nicht erprobt: nichts davon wurde an einer Maschine ausgeführt. |
| Dokumentation | 🟡 | README in acht Sprachen, das Changelog und die zwei aus `docs/` gezeichneten Bilder. |

## 🎯 Überblick

* **Das Lager** (`src/model.rs`): Bohrer, Fräser, V-Fräser, Gravierstichel, Laser und Düsen mit Durchmesser, Schneiden, Länge, Spitzenwinkel und Schaft; jeder Wert wird geprüft, ein doppelter Name abgelehnt; jedes Werkzeug bekommt seine Nummer. Ein Verschleißzähler oder eine Schneidenzahl, die keine ganze Zahl ist, wird mit dem Namen des Feldes abgelehnt, in den Dateien und in der Befehlszeile.
* **Der Verschleiß** (`src/model.rs`): Stunden, Meter und Bohrungen werden nach jedem Auftrag addiert; mit einem Limit für Stunden oder Bohrungen ist ein Werkzeug gut, bald zu wechseln (ab 90 %) oder zu wechseln, und `wear` listet die schlechtesten zuerst.
* **Speicherung und Vorschübe und Drehzahlen** (`src/store.rs`, `src/csv.rs`, `src/calc.rs`): das Lager als JSON-Dokument oder CSV-Tabelle für eine Tabellenkalkulation, sorgfältig gelesen und mit der Zeile in jedem Fehler; Schnittgeschwindigkeit und Zahnvorschub für FR4, Aluminium, Acryl, Holz und Messing, begrenzt durch die Spindel.
* **Das Fenster** (`src/window.rs`, `src/gui.rs`; `--features gui`, Programm `eh3d-tools-gui`): eine Werkzeugtabelle mit einem Formular zum Hinzufügen, Ändern und Entfernen mit denselben Prüfungen wie die Kommandozeile (die Nummer eines entfernten Werkzeugs wird nie wieder vergeben), Filtern und Suche, dem Einsatz eines Werkzeugs, der zu seinen Zählern addiert wird, CSV-Import und -Export mit den Fehlern der Bibliothek und einem Rechner, der die Formel und die Bedingungen jeder Zahl zeigt; die Zahlen, mit denen er arbeitet (Schnittgeschwindigkeiten, Zahnvorschübe, Faktoren), stehen in `tools.cfg` und lassen sich im Fenster ändern. Zustand, Aktionen, Einstellungen und Wörter sind einfacher Code mit Tests; das Zeichnen ist dünn. Geschrieben, noch nicht auf einem Bildschirm gesehen.

## 🧩 Rolle in EH3D

EH3D-TOOL-MANAGER ist ein Programm der EH3D Fabrication Suite. Es arbeitet als Bibliothek, als Kommandozeile und, mit dem Feature `gui` gebaut, als Fenster (`eh3d-tools-gui`) im gemeinsamen Aussehen der Suite. Es liest und schreibt einfache Dateien und lässt sich allein oder zwischen den anderen Programmen nutzen.

## 🏛️ Architektur

<p align="center">
  <img src="images/architecture.svg" alt="Architektur von EH3D-TOOL-MANAGER: wer es nutzt, was es enthält und wovon es abhängt" width="100%">
</p>


Wie die Teile dieses Repositorys zusammenpassen: wer es nutzt, was es enthält und wovon es abhängt. Das Bild wird aus `docs/architecture.json` gezeichnet und gegen den Code geprüft.

## ✨ Funktionen

* **Bibliothek und Kommandozeile:** Jeder Befehl von `eh3d-tools` ist eine Funktion der Bibliothek, sodass ein anderes Programm der Suite ihn ohne Prozessstart aufrufen kann.
* **Tests mit beschädigten Eingaben:** Die Tests füttern auch leere Dateien, Datenmüll, riesige Zahlen und abgeschnittene Dateien; keiner darf in Panik geraten.
* **Kein Raten:** Ein unsinniger Wert ist ein Fehler, der sagt welcher und warum, nie ein Ergebnis, das nur richtig aussieht.
* **Größenlimit und atomares Schreiben der Dateien:** Die Werkzeugkartei und die CSV-Tabelle werden in der Kommandozeile und im Fenster nur gelesen, wenn sie nicht größer sind als `max_file_mib` in `tools.cfg` (standardmäßig 16 MiB, 1 bis 1024); eine größere Datei wird mit dem Code `file_too_large` und ihrem Namen abgelehnt, eine nicht lesbare mit `file_unreadable` und eine, die kein Text ist, mit `file_not_text`; es wird nichts geladen. Kartei und CSV-Tabelle werden in eine temporäre Datei neben der echten geschrieben und darüber umbenannt, sodass ein Abbruch oder eine volle Platte die alte Datei heil lassen (`file_unwritable` und die temporäre Datei wird entfernt, wenn es scheitert); der CSV-Export ersetzt weiterhin nie eine Datei, außer die Person sagt es.
* **Kein unsafe-Code:** `#![forbid(unsafe_code)]` in Bibliothek und Programm.
* **Ein Werkzeuglager, das hält:** die Nummer eines entfernten Werkzeugs wird nie wieder vergeben (das Lager merkt sich `next_id`), eine ID, die keine ganze Zahl ist, wird als solche gemeldet, das CSV lehnt Text nach einem schließenden Anführungszeichen ab, ein Text, der wie eine Formel beginnt (`=`, `+`, `-`, `@`), wird als Text geschrieben, damit die Tabellenkalkulation ihn nicht ausführt, das Material hat eine Grenze, `--holes` ist eine ganze Zahl, eine unbekannte Option ist ein Fehler, und `calc` sagt, dass das Ergebnis für einen V-Fräser oder Gravierstichel nur ein Ausgangspunkt ist.
* **Die Schnittbreite eines V-Fräsers:** `eh3d-tools vbit --depth MM [--angle GRAD] [--flat MM]` berechnet `2 x Tiefe x tan(Winkel / 2) + Spitzenfläche` (ein 60-Grad-Fräser bei 0,5 mm ergibt 0,5774 mm plus die flache Spitze von 0,1 mm = 0,6774 mm); der Winkel liegt zwischen 10 und 170 Grad, die Tiefe zwischen 0 und 100 mm und die flache Spitze zwischen 0 und 5 mm, und was nicht angegeben ist, ist der Wert aus `tools.cfg`. `calc --depth MM` nimmt diese Breite als Durchmesser, mit dem ein V-Fräser oder Gravierstichel in den Vorschüben und Drehzahlen arbeitet, und der Rechner des Fensters hat ein Feld für die Tiefe, zeigt die Schnittbreite und die Formel mit ihren Zahlen; ohne Tiefe sind die Zahlen die gewohnten.

## 🔒 Sicherheit und Schutz

| Bedrohung | Schutz |
|---|---|
| Eine beschädigte, abgeschnittene oder leere Datei | keine Panik: die Tests schneiden gültige Eingaben an jeder Stelle ab und füttern Datenmüll |
| Eine unsinnige Zahl (negativ, nicht endlich, absurd groß) | wird mit einem Fehler abgelehnt, der den Wert nennt; nichts wird zu etwas Plausiblem gerundet |
| Unsichere Speichernutzung | reines sicheres Rust mit `forbid(unsafe_code)` |

**Was nicht abgesichert ist:** Ein fehlerfrei erzeugtes Ergebnis ist kein geprüftes Ergebnis. Nichts hier kennt die Grenzen einer Maschine, und das Programm hatte keine unabhängige Prüfung. Eine Schwachstelle bitte privat melden (siehe [SECURITY.md](SECURITY.md)).

## 🔌 Schnittstellen

Die Bibliothek ist `eh3d_tool_manager` (`use eh3d_tool_manager::...`); die Kommandozeile ist `eh3d-tools`. Die Befehle sind diese; jeder gibt sein Ergebnis auf der Standardausgabe aus und meldet einen Fehler auf der Fehlerausgabe mit einem Exit-Code ungleich null. Es gibt keinen Netzwerkzugriff.

```text
EH3D-TOOL-MANAGER/
├── src/
│   ├── calc.rs                                  ein Teil der Logik
│   ├── cli_args.rs                              die Argumente der Befehlszeile, streng gelesen
│   ├── csv.rs                                   ein Teil der Logik
│   ├── files.rs                                 Dateien mit Größenlimit lesen, über eine temporäre schreiben
│   ├── gui.rs                                   das Fenster (egui, mit dem Feature gui): zeichnet den Zustand und gibt weiter, was die Person tut
│   ├── gui_main.rs                              das Fenster als eigenes Programm, unter Windows ohne Konsole
│   ├── lang.rs                                  die Wörter des Fensters in den acht Sprachen
│   ├── lib.rs                                   die Bibliothek: was jeder Befehl nutzt
│   ├── main.rs                                  die Kommandozeile
│   ├── model.rs                                 ein Teil der Logik
│   ├── settings.rs                              die Einstellungen aus tools.cfg: Sprache, Fenstergröße, Werkzeugdatei und die Zahlen der Berechnung
│   ├── shot.rs                                  der Bildmodus des Fensters (--shot): seine Argumente, ohne Bildschirm gelesen und geprüft
│   ├── store.rs                                 ein Teil der Logik
│   └── window.rs                                der Zustand des Fensters ohne Bildschirm: Bestand, Formular, Filter und Rechner
├── tests/
│   ├── crib.rs                                  Tests von Bibliothek und Kommandozeile
│   ├── files.rs                                 Tests des Größenlimits und des atomaren Schreibens der Dateien
│   ├── shot_sample.rs                           Tests der Beispiel-Werkzeugkiste und des Fensters eines Bildes
│   └── vbit.rs                                  Tests der Schnittbreite eines V-Fräsers: der Befehl, die Tiefe in calc und die Einstellungen
├── examples/
│   └── crib.json                                ein Beispiel zum Ausprobieren
├── docs/
│   ├── architecture.json                        Daten des Architektur-Bildes
│   └── structure.json                           Daten des Struktur-Bildes
├── tools/                                       CI-Prüfung, README-Prüfung und die Bildwerkzeuge
├── images/                                      Banner und die Bilder von Architektur und Repository-Struktur
├── .github/                                     CI- und Release-Workflows, Vorlagen für Issues und Pull Requests
├── eh3d.project.json                            Manifest: Version, Rolle, Reifegrad und Build-Befehl
├── Cargo.toml Cargo.lock                        der Workspace und seine festgelegten Abhängigkeiten
├── rust-toolchain.toml                          die festgelegte Rust-Toolchain
├── CHANGELOG.md                                 was sich in jeder Version geändert hat
├── SECURITY.md SUPPORT.md                       Schwachstellen melden und Fragen stellen
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   Mitwirken, Verhaltenskodex und Lizenz
└── tools.cfg.example                            alle Einstellungen von tools.cfg mit ihrem Standardwert
```

## ⚙️ Konfiguration

Die Konfigurationsdatei ist `tools.cfg` im Ordner des Programms (die Variable `EH3D_TOOLS_DIR`, unter Windows `%APPDATA%\eh3d-tools`, sonst `~/.config/eh3d-tools`), gelesen vom Fenster und vom Befehl `calc`: die Sprache und die Größe des Fensters, die Datei des Werkzeuglagers, die Spindeldrehzahl, mit der der Rechner startet, und jede Zahl, mit der Vorschübe und Drehzahlen berechnet werden (Schnittgeschwindigkeit und Zahnvorschub je Material, die Faktoren für das Eintauchen und die Zustelltiefe sowie `v_bit_angle_deg` und `v_bit_flat_mm`, der Winkel (10 bis 170 Grad, standardmäßig 60) und die Breite der flachen Spitze (0 bis 5 mm, standardmäßig 0,1), mit denen ein V-Fräser berechnet wird), sind Zeilen `Schlüssel = Wert`; ihre Grenzen werden geprüft und eine fehlerhafte Zeile wird im Fehler genannt. `tools.cfg.example` listet jeden Schlüssel mit Standardwert auf. Davon abgesehen: Was die Person wählt, wird als Argument der Kommandozeile angegeben (siehe Schnittstellen), und jeder Standardwert steht in der Hilfe des Befehls (`eh3d-tools help`). Die Grenzen eines Werts sind benannte Konstanten im Code und durch Tests abgedeckt.

## 🚀 Schnellstart

Sie brauchen eine stabile Rust-Toolchain (1.95 oder neuer).

```bash
cargo build --release
cargo test
cargo run --release -- calc drill 0.8 fr4 24000
# das Fenster (braucht das Feature gui)
cargo build --release --features gui
cargo run --release --features gui --bin eh3d-tools-gui
cargo run --release --features gui --bin eh3d-tools-gui -- --shot screen.png --open examples/crib.json   # a picture of the window
```

Um ein Bild des Fensters aufzunehmen, ohne es von Hand zu öffnen, starten Sie es mit `--shot DATEI.png` (auch `--size WxH`, `--theme forge|dark|light`, `--wait N`, `--lang CODE`, `--tab tools|calc`, `--open FILE`, `--replace`): Es öffnet sich in dieser Größe, speichert das PNG, behält keine Einstellungen und schließt sich selbst (Exit-Code 0, wenn die Datei geschrieben wurde, 1, wenn nicht, 2 bei falschen Argumenten).

## 🧪 Tests

Die Suite hat 147 Tests (`cargo test`, in der CI unter Windows und Linux). Sie prüfen Normalfälle, Grenzfälle und beschädigte Eingaben: leere Dateien, Datenmüll und abgeschnittene Dateien lösen nie eine Panik aus. Die CI führt außerdem `cargo fmt --check`, `cargo clippy` (auch mit `--features gui`) mit Warnungen als Fehlern und `python -I tools/ci_validate.py` aus.

## 🏗️ Build und Veröffentlichung

Die Version steht in `Cargo.toml` und in `eh3d.project.json`; jede Version hat ihren Eintrag in [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` bauen das Programm, `test.bat` / `test.sh` führen die Tests aus und `run.bat` / `run.sh` starten es mit einem Beispiel. Releases erscheinen auf GitHub.

### Downloads

Jedes Release enthält das Kommandozeilenprogramm und das Fenster für Windows (64 und 32 Bit), Linux (64 Bit) und macOS (Apple Silicon und Intel) als `<Name>-<Version>-<System>.zip` (Windows) oder `.tar.gz` mit den beiden Programmen, der Lizenz, der Readme, dem Symbol und `tools.cfg.example`. Die Dateien werden von GitHub Actions gebaut und stehen in `SHA256SUMS`. Um einen Download zu prüfen, vergleichen Sie seinen Hash mit der danebenliegenden `.sha256`-Datei (`sha256sum -c`).

## 📂 Struktur des Repositorys

<p align="center">
  <img src="images/repository-structure.svg" alt="Struktur des Repositorys EH3D-TOOL-MANAGER" width="100%">
</p>

<details>
<summary>Textversion des Baums</summary>

```text
EH3D-TOOL-MANAGER/
├── src/
│   ├── calc.rs                                  ein Teil der Logik
│   ├── cli_args.rs                              die Argumente der Befehlszeile, streng gelesen
│   ├── csv.rs                                   ein Teil der Logik
│   ├── gui.rs                                   das Fenster (egui, mit dem Feature gui): zeichnet den Zustand und gibt weiter, was die Person tut
│   ├── gui_main.rs                              das Fenster als eigenes Programm, unter Windows ohne Konsole
│   ├── lang.rs                                  die Wörter des Fensters in den acht Sprachen
│   ├── lib.rs                                   die Bibliothek: was jeder Befehl nutzt
│   ├── main.rs                                  die Kommandozeile
│   ├── model.rs                                 ein Teil der Logik
│   ├── settings.rs                              die Einstellungen aus tools.cfg: Sprache, Fenstergröße, Werkzeugdatei und die Zahlen der Berechnung
│   ├── shot.rs                                  der Bildmodus des Fensters (--shot): seine Argumente, ohne Bildschirm gelesen und geprüft
│   ├── store.rs                                 ein Teil der Logik
│   └── window.rs                                der Zustand des Fensters ohne Bildschirm: Bestand, Formular, Filter und Rechner
├── tests/
│   ├── crib.rs                                  Tests von Bibliothek und Kommandozeile
│   ├── shot_sample.rs                           Tests der Beispiel-Werkzeugkiste und des Fensters eines Bildes
│   └── vbit.rs                                  Tests der Schnittbreite eines V-Fräsers: der Befehl, die Tiefe in calc und die Einstellungen
├── examples/
│   └── crib.json                                ein Beispiel zum Ausprobieren
├── docs/
│   ├── architecture.json                        Daten des Architektur-Bildes
│   └── structure.json                           Daten des Struktur-Bildes
├── tools/                                       CI-Prüfung, README-Prüfung und die Bildwerkzeuge
├── images/                                      Banner und die Bilder von Architektur und Repository-Struktur
├── .github/                                     CI- und Release-Workflows, Vorlagen für Issues und Pull Requests
├── eh3d.project.json                            Manifest: Version, Rolle, Reifegrad und Build-Befehl
├── Cargo.toml Cargo.lock                        der Workspace und seine festgelegten Abhängigkeiten
├── rust-toolchain.toml                          die festgelegte Rust-Toolchain
├── CHANGELOG.md                                 was sich in jeder Version geändert hat
├── SECURITY.md SUPPORT.md                       Schwachstellen melden und Fragen stellen
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   Mitwirken, Verhaltenskodex und Lizenz
└── tools.cfg.example                            alle Einstellungen von tools.cfg mit ihrem Standardwert
```

</details>

## 🗺️ Roadmap und bekannte Grenzen

**Bekannte Grenzen heute**

* Das Fenster ist geschrieben und seine Logik getestet, wurde aber noch nicht gebaut oder auf einem Bildschirm angesehen.
* Das Fenster verlangt die Werkzeugdatei und die CSV-Tabelle als geschriebene Pfade (noch ohne Dateidialog).
* Es wurde weder an einer Maschine noch mit einer Sammlung echter Dateien erprobt und hatte keine unabhängige Prüfung.
* Das Lager ist eine JSON-Datei, keine Datenbank; die Vorschübe und Drehzahlen sind Startwerte aus typischen Werten und an keiner Maschine geprüft; die Programme der Suite lesen das Lager noch nicht.

**Als Nächstes**

* DRILLPRO, ROUTEPRO und die CAM-Programme das Lager lesen lassen statt eigener Werkzeugtabellen und das 2D-Profil jedes Werkzeugs ergänzen.
* Das Fenster auf einem Bildschirm jedes Systems ansehen und das Gefundene anpassen.
* Mit echten Dateien und den anderen Programmen der Suite ausprobieren.

## 🌍 Sprachen

README und Texte gibt es auf Englisch, Spanisch, Französisch, Italienisch, Deutsch, Chinesisch, Japanisch und europäischem Portugiesisch.

## 🔗 Verwandte Projekte

* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): der Projektcontainer und die gemeinsamen Typen der Suite
* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): das Bohrprogramm
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): das Konturfräsprogramm
* [EH3D-ISOCAM](https://github.com/JuanenRac/EH3D-ISOCAM): das Isolationsfräsen von Kupfer
* [EH3D-PHOENIX-LASER](https://github.com/JuanenRac/EH3D-PHOENIX-LASER): Lasergravur und Laserschneiden
* [EH3D-PHOENIX-PNP](https://github.com/JuanenRac/EH3D-PHOENIX-PNP): die Pick-and-Place-Bestückung
* [EH3D-SUITE-LAUNCHER](https://github.com/JuanenRac/EH3D-SUITE-LAUNCHER): die Zentrale, die die Programme der Suite findet und startet

## 📚 Dokumentation

* [Changelog](CHANGELOG.md)
* [Architektur- und Strukturdaten](docs/architecture.json): die Daten, aus denen die beiden Bilder gezeichnet werden.

## 🤝 Mitwirken und Community

* Vor einem Pull Request `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` und `python -I tools/ci_validate.py` ausführen: alles muss bestehen. Siehe [CONTRIBUTING.md](CONTRIBUTING.md) und den [Verhaltenskodex](CODE_OF_CONDUCT.md).
* Die README gibt es in acht Sprachen: Eine Textänderung erfolgt in allen acht Dateien, und die zwei Bilder werden aus `docs/architecture.json` und `docs/structure.json` mit `tools/make_architecture.py` und `tools/make_structure.py` gezeichnet.
* Eine Schwachstelle privat melden: [SECURITY.md](SECURITY.md). Fragen, Ideen und Berichte: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTOR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LIZENZ

Software: GPL-3.0-or-later. Siehe [LICENSE](LICENSE). Die Hardware-Entwürfe der Suite verwenden die CERN OHL-S.
