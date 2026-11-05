<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-TOOL-MANAGER banner" width="100%">
</p>

# <img src="images/brand/png/icon-64.png" alt="" width="40" height="40" align="absmiddle"> EH3D-TOOL-MANAGER

<p align="center">🇺🇸 <b>English</b> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### The tool crib of the EH3D Fabrication Suite: tools, wear, feeds and speeds

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Version-0.0.5-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Honesty check - what actually runs today:** a Rust library and a command line program (`eh3d-tools`) with 115 tests that run on a computer: keeps the tools with their geometry and wear, stores them as JSON or CSV and works out starting feeds and speeds by material. A window (`eh3d-tools-gui`, built with the `gui` feature) is written and its logic is tested, but it has not been built or looked at on a screen; nothing has been tried on a machine and nothing has been checked against a collection of real files.

---

> **Safety and security status:** this program does not control any machine and is not a safety system. What it produces must be checked by a person before anything is run on a machine; nothing here has run on real hardware.


## 📋 At a glance

| | |
|---|---|
| Version | `0.0.5` |
| Maturity | Scaffolding |
| Tests | 115 |
| Stack | Rust |
| On real hardware | Not tried: nothing here has run on a machine |

**Jump to:** [Overview](#-overview) · [Architecture](#-architecture) · [Quick start](#-quick-start) · [Testing](#-testing) · [Structure](#-repository-structure) · [Related](#-related-projects)

## 🚦 Project status

Each line is one part of this repository and the light says how far it is today. It is written from what the repository can show (tests, builds, runs), not from what is planned.

🟢 Done and verified · 🟡 Works, with known limits · 🟠 Partly done or not verified yet · 🔴 Not done or not working

| Area | State | What it means here |
|---|---|---|
| Core | 🟡 | keeps the tools with their geometry and wear, stores them as JSON or CSV and works out starting feeds and speeds by material; tested on a computer with normal and damaged input, not with a collection of real files. |
| Command line | 🟡 | `eh3d-tools` runs the same functions as the library and prints the result; every error comes back as a message and a non-zero exit code. |
| Tests | 🟡 | 115 tests pass (`cargo test`), with normal cases, edge cases and damaged input. |
| Security | 🟡 | Plain safe Rust (`forbid(unsafe_code)`): nothing panics on the files it is given and a bad value is an error with its reason; no independent review. |
| Window | 🟠 | Written (egui, `--features gui`, program `eh3d-tools-gui`): a table of the tools that are added, changed and removed with the checks of the command line, filters and search, CSV import and export with the errors of the library, and a calculator of feeds and speeds that shows its formulas and conditions; its logic is tested, but it has not been built or looked at on a screen yet. |
| Real hardware | 🔴 | Not tried: nothing here has been run on a machine. |
| Documentation | 🟡 | README in eight languages, the changelog and the two pictures drawn from `docs/`. |

## 🎯 Overview

* **The crib** (`src/model.rs`): drills, end mills, V-bits, engravers, lasers and nozzles with diameter, flutes, length, tip angle and shank; every value is checked and a repeated name is refused; each tool gets its own number. A wear counter or a number of flutes that is not a whole number is refused with the name of the field, in the files and on the command line.
* **The wear** (`src/model.rs`): hours, metres and holes are added after each job; with a limit of hours or holes set, a tool is good, to replace soon (from 90 %) or to replace, and `wear` lists the worst first.
* **Storage and feeds and speeds** (`src/store.rs`, `src/csv.rs`, `src/calc.rs`): the crib as a JSON document or a CSV table for a spreadsheet, read with care and with the row named in every error; surface speed and chip load for FR4, aluminium, acrylic, wood and brass, limited by the spindle.
* **The window** (`src/window.rs`, `src/gui.rs`; `--features gui`, program `eh3d-tools-gui`): a table of the tools with a form to add, change and remove them with the same checks as the command line (the number of a removed tool is never given again), filters and search, the use of a tool added to its counters, CSV import and export with the errors of the library, and a calculator that shows the formula and the conditions of every number; the numbers it works with (surface speeds, chip loads, factors) are in `tools.cfg` and can be changed in the window. The state, the actions, the settings and the words are plain code with tests; the drawing is thin. Written, not seen on a screen yet.

## 🧩 Role in EH3D

EH3D-TOOL-MANAGER is one program of the EH3D Fabrication Suite. It works as a library, a command line and, built with the `gui` feature, a window (`eh3d-tools-gui`) in the look the other programs of the suite share. It reads and writes plain files, so it can be used on its own or between the other programs.

## 🏛️ Architecture

<p align="center">
  <img src="images/architecture.svg" alt="EH3D-TOOL-MANAGER architecture: who uses it, what it holds and what it depends on" width="100%">
</p>


How the parts of this repository fit together: who uses it, what it holds and what it depends on. The picture is drawn from `docs/architecture.json` and checked against the code.

## ✨ Features

* **A library and a command line:** every command of `eh3d-tools` is a function of the library, so another program of the suite can call it without starting a process.
* **Tests with damaged input:** the tests also feed empty files, garbage, huge numbers and cut files, and none of them may panic.
* **No guessing:** a value that does not make sense is an error that says which value and why, never a result that only looks right.
* **No unsafe code:** `#![forbid(unsafe_code)]` in the library and the program.
* **A crib that holds up:** the number of a removed tool is never given again (the crib keeps `next_id`), an id that is not a whole number is told as such, the CSV refuses text after a closing quote, a text that starts like a formula (`=`, `+`, `-`, `@`) is written as text so a spreadsheet does not run it, the material has a limit, `--holes` is a whole number, an unknown option is an error, and `calc` says that the result for a V-bit or an engraver is only a starting point.

## 🔒 Security & Safety

| Threat | Defense |
|---|---|
| A damaged, truncated or empty file | no panic: the tests cut valid input at every position and feed garbage |
| A number that makes no sense (negative, not finite, absurdly large) | refused with an error that names the value; nothing is rounded into something plausible |
| Unsafe memory use | plain safe Rust with `forbid(unsafe_code)` |

**What is not defended:** a result that was produced cleanly is not a verified result. Nothing here knows the limits of a machine, and the program has had no independent review. Report a vulnerability privately (see [SECURITY.md](SECURITY.md)).

## 🔌 Interfaces

The library is `eh3d_tool_manager` (`use eh3d_tool_manager::...`); the command line is `eh3d-tools`. The commands are these; each prints its result on the standard output and reports an error on the standard error with a non-zero exit code. There is no network access.

```text
EH3D-TOOL-MANAGER/
├── src/
│   ├── calc.rs                                  a part of the logic
│   ├── cli_args.rs                              the arguments of the command line, read strictly
│   ├── csv.rs                                   a part of the logic
│   ├── gui.rs                                   the window (egui, with the gui feature): draws the state and passes on what the person does
│   ├── gui_main.rs                              the window as a program of its own, without a console on Windows
│   ├── lang.rs                                  the words of the window in the eight languages
│   ├── lib.rs                                   the library: what every command uses
│   ├── main.rs                                  the command line
│   ├── model.rs                                 a part of the logic
│   ├── settings.rs                              the settings of tools.cfg: language, window size, crib file and the numbers of the calculation
│   ├── store.rs                                 a part of the logic
│   └── window.rs                                the state of the window, no screen needed: the crib, the form, the filters and the calculator
├── tests/
│   └── crib.rs                                  tests of the library and the command line
├── docs/
│   ├── architecture.json                        data of the Architecture picture
│   └── structure.json                           data of the Repository Structure picture
├── tools/                                       CI check, README check and the picture tools
├── images/                                      banner and the Architecture and Repository Structure pictures
├── .github/                                     CI and release workflows, issue and pull-request templates
├── eh3d.project.json                            manifest: version, role, maturity and build command
├── Cargo.toml Cargo.lock                        the workspace and its locked dependencies
├── rust-toolchain.toml                          the pinned Rust toolchain
├── CHANGELOG.md                                 what changed in each version
├── SECURITY.md SUPPORT.md                       how to report a vulnerability and where to ask
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   how to contribute, the code of conduct and the licence
└── tools.cfg.example                            every setting of tools.cfg with its default
```

## ⚙️ Configuration

The configuration file is `tools.cfg` in the folder of the program (the variable `EH3D_TOOLS_DIR`, or `%APPDATA%\eh3d-tools` on Windows, or `~/.config/eh3d-tools` elsewhere), read by the window and by the `calc` command: the language and the size of the window, the file of the crib, the spindle speed the calculator starts with and every number the feeds and speeds are worked out with (the surface speed and the chip load of each material and the factors of the plunge and of the depth of a pass) are lines `key = value`, with their limits checked and a bad line named in the error. `tools.cfg.example` lists every key with its default. Apart from it, what a person chooses is given as arguments of the command line (see Interfaces) and every default is written in the help of the command (`eh3d-tools help`). The limits of a value are named constants of the code and are covered by tests.

## 🚀 Quick start

You need a stable Rust toolchain (1.95 or newer).

```bash
cargo build --release
cargo test
cargo run --release -- calc drill 0.8 fr4 24000
# the window (needs the gui feature)
cargo build --release --features gui
cargo run --release --features gui --bin eh3d-tools-gui
```

## 🧪 Testing

The suite has 115 tests (`cargo test`, run on Windows and Linux in the CI). They check the normal cases, the edge cases and damaged input: empty files, garbage and cut files never panic. The CI also runs `cargo fmt --check`, `cargo clippy` (also with `--features gui`) with warnings as errors and `python -I tools/ci_validate.py`.

## 🏗️ Build & Release

The version lives in `Cargo.toml` and in `eh3d.project.json`; each version has its entry in [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` build the program, `test.bat` / `test.sh` run the tests and `run.bat` / `run.sh` run it with a sample. Releases are published on GitHub.

### Downloads

Each release has the command line program and the window for Windows (64 and 32 bits), Linux (64 bits) and macOS (Apple silicon and Intel), as `<name>-<version>-<system>.zip` (Windows) or `.tar.gz` with the two programs, the licence, the readme, the icon and `tools.cfg.example`. The files are built by GitHub Actions and listed in `SHA256SUMS`. To check a download, compare its hash with the `.sha256` file next to it (`sha256sum -c`).

## 📂 Repository Structure

<p align="center">
  <img src="images/repository-structure.svg" alt="Repository structure of EH3D-TOOL-MANAGER" width="100%">
</p>

<details>
<summary>Text version of the tree</summary>

```text
EH3D-TOOL-MANAGER/
├── src/
│   ├── calc.rs                                  a part of the logic
│   ├── cli_args.rs                              the arguments of the command line, read strictly
│   ├── csv.rs                                   a part of the logic
│   ├── gui.rs                                   the window (egui, with the gui feature): draws the state and passes on what the person does
│   ├── gui_main.rs                              the window as a program of its own, without a console on Windows
│   ├── lang.rs                                  the words of the window in the eight languages
│   ├── lib.rs                                   the library: what every command uses
│   ├── main.rs                                  the command line
│   ├── model.rs                                 a part of the logic
│   ├── settings.rs                              the settings of tools.cfg: language, window size, crib file and the numbers of the calculation
│   ├── store.rs                                 a part of the logic
│   └── window.rs                                the state of the window, no screen needed: the crib, the form, the filters and the calculator
├── tests/
│   └── crib.rs                                  tests of the library and the command line
├── docs/
│   ├── architecture.json                        data of the Architecture picture
│   └── structure.json                           data of the Repository Structure picture
├── tools/                                       CI check, README check and the picture tools
├── images/                                      banner and the Architecture and Repository Structure pictures
├── .github/                                     CI and release workflows, issue and pull-request templates
├── eh3d.project.json                            manifest: version, role, maturity and build command
├── Cargo.toml Cargo.lock                        the workspace and its locked dependencies
├── rust-toolchain.toml                          the pinned Rust toolchain
├── CHANGELOG.md                                 what changed in each version
├── SECURITY.md SUPPORT.md                       how to report a vulnerability and where to ask
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   how to contribute, the code of conduct and the licence
└── tools.cfg.example                            every setting of tools.cfg with its default
```

</details>

## 🗺️ Roadmap & Known limits

**Known limits today**

* The window is written and its logic is tested, but it has not been built or looked at on a screen yet.
* The window asks for the crib file and the CSV table as written paths (there is no file dialog yet).
* It has not been tried on a machine nor with a collection of real files, and it has had no independent review.
* The crib is a JSON file, not a database; the feeds and speeds are starting points taken from typical values and have not been checked on a machine; the programs of the suite do not read the crib yet.

**Next**

* Let DRILLPRO, ROUTEPRO and the CAM programs read the crib instead of their own tool tables, and add the 2D profile of each tool.
* Look at the window on a screen of each system and adjust what is found.
* Try it with real files and with the other programs of the suite.

## 🌍 Languages

README and texts exist in English, Spanish, French, Italian, German, Chinese, Japanese and European Portuguese.

## 🔗 Related Projects

* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): the project container and the shared types of the suite
* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): the drilling program
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): the contour milling program
* [EH3D-ISOCAM](https://github.com/JuanenRac/EH3D-ISOCAM): isolation milling of copper
* [EH3D-PHOENIX-LASER](https://github.com/JuanenRac/EH3D-PHOENIX-LASER): laser engraving and cutting
* [EH3D-PHOENIX-PNP](https://github.com/JuanenRac/EH3D-PHOENIX-PNP): pick and place assembly
* [EH3D-SUITE-LAUNCHER](https://github.com/JuanenRac/EH3D-SUITE-LAUNCHER): the hub that finds and starts the programs of the suite

## 📚 Documentation

* [Changelog](CHANGELOG.md)
* [Architecture and structure data](docs/architecture.json): the data the two pictures are drawn from.

## 🤝 Contributing & Community

* Before a pull request run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `python -I tools/ci_validate.py`: all must pass. See [CONTRIBUTING.md](CONTRIBUTING.md) and the [code of conduct](CODE_OF_CONDUCT.md).
* The README exists in eight languages: a change to the text is made in all eight files, and the two pictures are drawn from `docs/architecture.json` and `docs/structure.json` by `tools/make_architecture.py` and `tools/make_structure.py`.
* Report a vulnerability privately: [SECURITY.md](SECURITY.md). Questions, ideas and reports: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTHOR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENSE

Software: GPL-3.0-or-later. See [LICENSE](LICENSE). Hardware designs of the suite use CERN OHL-S.
