<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-TOOL-MANAGER banner" width="100%">
</p>

# <img src="images/brand/png/icon-64.png" alt="" width="40" height="40" align="absmiddle"> EH3D-TOOL-MANAGER

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | 🇫🇷 <b>Français</b> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### Le magasin d'outils de l'EH3D Fabrication Suite : outils, usure, avances et vitesses

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Version-0.0.6-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Vérification d'honnêteté - ce qui fonctionne aujourd'hui :** une bibliothèque Rust et un programme en ligne de commande (`eh3d-tools`) avec 138 tests exécutés sur un ordinateur : garde les outils avec leur géométrie et leur usure, les stocke en JSON ou CSV et calcule des avances et vitesses de départ par matériau. Une fenêtre (`eh3d-tools-gui`, compilée avec la fonctionnalité `gui`) est écrite et sa logique est testée, mais elle n'a été ni compilée ni regardée sur un écran ; rien n'a été essayé sur une machine et rien n'a été vérifié sur une collection de fichiers réels.

---

> **État de sécurité :** ce programme ne commande aucune machine et n'est pas un système de sécurité. Ce qu'il produit doit être vérifié par une personne avant tout essai sur une machine ; rien ici n'a fonctionné sur du matériel réel.


## 📋 En un coup d'œil

| | |
|---|---|
| Version | `0.0.6` |
| Maturité | Échafaudage |
| Tests | 138 |
| Pile technique | Rust |
| Sur du matériel réel | Non essayé : rien ici n'a fonctionné sur une machine |

**Aller à  :** [Présentation](#-présentation) · [Architecture](#-architecture) · [Démarrage rapide](#-démarrage-rapide) · [Tests](#-tests) · [Structure](#-structure-du-dépôt) · [Liés](#-projets-liés)

## 🚦 État du projet

Chaque ligne est une partie de ce dépôt et le feu dit où elle en est aujourd'hui. Elle est écrite d'après ce que le dépôt peut montrer (tests, compilations, exécutions), pas d'après ce qui est prévu.

🟢 Fait et vérifié · 🟡 Fonctionne, avec des limites connues · 🟠 À moitié fait ou pas encore vérifié · 🔴 Pas fait ou ne fonctionne pas

| Domaine | État | Ce que cela veut dire ici |
|---|---|---|
| Cœur | 🟡 | garde les outils avec leur géométrie et leur usure, les stocke en JSON ou CSV et calcule des avances et vitesses de départ par matériau ; testé sur un ordinateur avec des entrées normales et abîmées, pas avec une collection de fichiers réels. |
| Ligne de commande | 🟡 | `eh3d-tools` exécute les mêmes fonctions que la bibliothèque et affiche le résultat ; chaque erreur revient sous forme de message et de code de sortie non nul. |
| Tests | 🟡 | 138 tests passent (`cargo test`), avec des cas normaux, des cas limites et des entrées abîmées. |
| Sécurité | 🟡 | Rust sûr et simple (`forbid(unsafe_code)`) : rien ne panique sur les fichiers reçus et une mauvaise valeur est une erreur avec sa raison ; pas de revue indépendante. |
| Fenêtre | 🟠 | Écrite (egui, `--features gui`, programme `eh3d-tools-gui`) : un tableau des outils que l'on ajoute, modifie et retire avec les vérifications de la ligne de commande, des filtres et une recherche, l'import et l'export CSV avec les erreurs de la bibliothèque et un calculateur d'avances et de vitesses qui montre ses formules et ses conditions ; sa logique est testée, mais elle n'a pas encore été compilée ni regardée sur un écran. |
| Matériel réel | 🔴 | Non essayé : rien ici n'a été exécuté sur une machine. |
| Documentation | 🟡 | README en huit langues, le journal des modifications et les deux images dessinées depuis `docs/`. |

## 🎯 Présentation

* **Le magasin** (`src/model.rs`) : forets, fraises, fraises en V, gravures, lasers et buses avec diamètre, dents, longueur, angle de pointe et queue ; chaque valeur est vérifiée et un nom répété est refusé ; chaque outil reçoit son numéro. Un compteur d'usure ou un nombre de dents qui n'est pas entier est refusé avec le nom du champ, dans les fichiers et sur la ligne de commande.
* **L'usure** (`src/model.rs`) : heures, mètres et trous s'ajoutent après chaque travail ; avec une limite d'heures ou de trous, un outil est bon, à remplacer bientôt (dès 90 %) ou à remplacer, et `wear` liste les pires d'abord.
* **Stockage et avances et vitesses** (`src/store.rs`, `src/csv.rs`, `src/calc.rs`) : le magasin comme document JSON ou tableau CSV pour un tableur, lu avec soin et avec la ligne nommée dans chaque erreur ; vitesse de coupe et avance par dent pour FR4, aluminium, acrylique, bois et laiton, limitées par la broche.
* **La fenêtre** (`src/window.rs`, `src/gui.rs` ; `--features gui`, programme `eh3d-tools-gui`) : un tableau des outils avec un formulaire pour les ajouter, les modifier et les retirer avec les mêmes vérifications que la ligne de commande (le numéro d'un outil retiré n'est jamais redonné), des filtres et une recherche, l'utilisation d'un outil ajoutée à ses compteurs, l'import et l'export CSV avec les erreurs de la bibliothèque et un calculateur qui montre la formule et les conditions de chaque nombre ; les nombres avec lesquels il travaille (vitesses de coupe, avances par dent, facteurs) sont dans `tools.cfg` et peuvent être changés dans la fenêtre. L'état, les actions, les réglages et les mots sont du code simple avec des tests ; le dessin est mince. Écrite, pas encore vue sur un écran.

## 🧩 Rôle dans EH3D

EH3D-TOOL-MANAGER est un programme de l'EH3D Fabrication Suite. Il fonctionne comme bibliothèque, comme ligne de commande et, compilé avec la fonctionnalité `gui`, comme fenêtre (`eh3d-tools-gui`) avec l'aspect commun aux autres programmes de la suite. Il lit et écrit des fichiers simples et peut donc s'utiliser seul ou entre les autres programmes.

## 🏛️ Architecture

<p align="center">
  <img src="images/architecture.svg" alt="Architecture de EH3D-TOOL-MANAGER : qui l'utilise, ce qu'il contient et de quoi il dépend" width="100%">
</p>


Comment les parties de ce dépôt s'articulent : qui l'utilise, ce qu'il contient et de quoi il dépend. L'image est dessinée depuis `docs/architecture.json` et vérifiée avec le code.

## ✨ Fonctionnalités

* **Une bibliothèque et une ligne de commande :** chaque commande de `eh3d-tools` est une fonction de la bibliothèque ; un autre programme de la suite peut l'appeler sans lancer de processus.
* **Tests avec entrées abîmées :** les tests fournissent aussi des fichiers vides, du bruit, des nombres énormes et des fichiers coupés, et aucun ne doit paniquer.
* **Sans deviner :** une valeur absurde est une erreur qui dit laquelle et pourquoi, jamais un résultat qui n'en a que l'air.
* **Pas de code unsafe :** `#![forbid(unsafe_code)]` dans la bibliothèque et le programme.
* **Un magasin qui tient :** le numéro d'un outil retiré n'est jamais redonné (le magasin garde `next_id`), un id qui n'est pas un nombre entier est signalé comme tel, le CSV refuse du texte après un guillemet fermant, un texte qui commence comme une formule (`=`, `+`, `-`, `@`) est écrit comme texte pour que le tableur ne l'exécute pas, la matière a une limite, `--holes` est un nombre entier, une option inconnue est une erreur, et `calc` dit que le résultat pour une fraise en V ou un graveur n'est qu'un point de départ.
* **La largeur de coupe d'une fraise en V :** `eh3d-tools vbit --depth MM [--angle DEGRÉS] [--flat MM]` calcule `2 x profondeur x tan(angle / 2) + méplat` (une fraise de 60 degrés à 0,5 mm donne 0,5774 mm plus le méplat de 0,1 mm = 0,6774 mm) ; l'angle va de 10 à 170 degrés, la profondeur de 0 à 100 mm et le méplat de 0 à 5 mm, et ce qui n'est pas donné est la valeur de `tools.cfg`. `calc --depth MM` utilise cette largeur comme diamètre avec lequel une fraise en V ou un burin travaille dans les avances et vitesses, et le calculateur de la fenêtre a un champ pour la profondeur, montre la largeur de coupe et la formule avec ses nombres ; sans profondeur, les nombres sont ceux de toujours.

## 🔒 Sécurité et protection

| Menace | Défense |
|---|---|
| Un fichier abîmé, coupé ou vide | pas de panique : les tests coupent une entrée valide à chaque position et fournissent du bruit |
| Un nombre absurde (négatif, non fini, démesuré) | refusé avec une erreur qui nomme la valeur ; rien n'est arrondi en quelque chose de plausible |
| Usage non sûr de la mémoire | Rust sûr et simple avec `forbid(unsafe_code)` |

**Ce qui n'est pas défendu :** un résultat produit sans erreur n'est pas un résultat vérifié. Rien ici ne connaît les limites d'une machine et le programme n'a pas eu de revue indépendante. Signalez une vulnérabilité en privé (voir [SECURITY.md](SECURITY.md)).

## 🔌 Interfaces

La bibliothèque est `eh3d_tool_manager` (`use eh3d_tool_manager::...`) ; la ligne de commande est `eh3d-tools`. Les commandes sont celles-ci ; chacune affiche son résultat sur la sortie standard et signale une erreur sur la sortie d'erreur avec un code de sortie non nul. Il n'y a pas d'accès réseau.

```text
EH3D-TOOL-MANAGER/
├── src/
│   ├── calc.rs                                  une partie de la logique
│   ├── cli_args.rs                              les arguments de la ligne de commande, lus strictement
│   ├── csv.rs                                   une partie de la logique
│   ├── gui.rs                                   la fenêtre (egui, avec la fonctionnalité gui) : dessine l'état et transmet ce que fait la personne
│   ├── gui_main.rs                              la fenêtre comme programme à part, sans console sous Windows
│   ├── lang.rs                                  les mots de la fenêtre dans les huit langues
│   ├── lib.rs                                   la bibliothèque : ce que chaque commande utilise
│   ├── main.rs                                  la ligne de commande
│   ├── model.rs                                 une partie de la logique
│   ├── settings.rs                              les réglages de tools.cfg : langue, taille de la fenêtre, fichier de l'outillage et nombres du calcul
│   ├── shot.rs                                  le mode capture de la fenêtre (--shot) : ses arguments, lus et vérifiés sans écran
│   ├── store.rs                                 une partie de la logique
│   └── window.rs                                l'état de la fenêtre sans écran : l'outillage, le formulaire, les filtres et le calculateur
├── tests/
│   ├── crib.rs                                  tests de la bibliothèque et de la ligne de commande
│   ├── shot_sample.rs                           tests de la boîte à outils d'exemple et de la fenêtre d'une capture
│   └── vbit.rs                                  tests de la largeur de coupe d'une fraise en V : la commande, la profondeur dans calc et les réglages
├── examples/
│   └── crib.json                                un exemple pour essayer le programme
├── docs/
│   ├── architecture.json                        données de l'image d'Architecture
│   └── structure.json                           données de l'image de Structure
├── tools/                                       contrôle de la CI et du README et outils d'images
├── images/                                      bannière et images d'Architecture et de Structure du dépôt
├── .github/                                     workflows de CI et de publication, modèles d'issues et de pull requests
├── eh3d.project.json                            manifeste : version, rôle, maturité et commande de compilation
├── Cargo.toml Cargo.lock                        le workspace et ses dépendances verrouillées
├── rust-toolchain.toml                          la toolchain Rust épinglée
├── CHANGELOG.md                                 ce qui a changé dans chaque version
├── SECURITY.md SUPPORT.md                       signaler une vulnérabilité et poser des questions
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   contribuer, code de conduite et licence
└── tools.cfg.example                            tous les réglages de tools.cfg avec leur valeur par défaut
```

## ⚙️ Configuration

Le fichier de configuration est `tools.cfg` dans le dossier du programme (la variable `EH3D_TOOLS_DIR`, ou `%APPDATA%\eh3d-tools` sous Windows, ou `~/.config/eh3d-tools` ailleurs), lu par la fenêtre et par la commande `calc` : la langue et la taille de la fenêtre, le fichier de l'outillage, la vitesse de broche avec laquelle le calculateur démarre et chaque nombre avec lequel les avances et vitesses sont calculées (la vitesse de coupe et l'avance par dent de chaque matériau, les facteurs de la plongée et de la profondeur de passe, et `v_bit_angle_deg` et `v_bit_flat_mm`, l'angle (de 10 à 170 degrés, 60 par défaut) et la largeur du méplat de la pointe (de 0 à 5 mm, 0,1 par défaut) avec lesquels une fraise en V est calculée) sont des lignes `clé = valeur`, avec leurs limites vérifiées et une mauvaise ligne nommée dans l'erreur. `tools.cfg.example` liste chaque clé avec sa valeur par défaut. En dehors de cela, ce que la personne choisit est donné en arguments de la ligne de commande (voir Interfaces) et chaque valeur par défaut est écrite dans l'aide de la commande (`eh3d-tools help`). Les limites d'une valeur sont des constantes nommées dans le code, couvertes par des tests.

## 🚀 Démarrage rapide

Il faut une chaîne d'outils Rust stable (1.95 ou plus récente).

```bash
cargo build --release
cargo test
cargo run --release -- calc drill 0.8 fr4 24000
# la fenêtre (demande la fonctionnalité gui)
cargo build --release --features gui
cargo run --release --features gui --bin eh3d-tools-gui
cargo run --release --features gui --bin eh3d-tools-gui -- --shot screen.png --open examples/crib.json   # a picture of the window
```

Pour prendre une capture de la fenêtre sans l'ouvrir à la main, lancez-la avec `--shot FICHIER.png` (aussi `--size WxH`, `--theme forge|dark|light`, `--wait N`, `--lang CODE`, `--open FILE`, `--replace`) : elle s'ouvre à cette taille, enregistre le PNG, ne garde aucun réglage et se ferme toute seule (code de sortie 0 si le fichier a été écrit, 1 sinon, 2 si les arguments sont erronés).

## 🧪 Tests

La suite compte 138 tests (`cargo test`, exécutés sous Windows et Linux dans le CI). Ils vérifient les cas normaux, les cas limites et les entrées abîmées : fichiers vides, bruit et fichiers coupés ne font jamais paniquer. Le CI exécute aussi `cargo fmt --check`, `cargo clippy` (aussi avec `--features gui`) avec les avertissements comme erreurs et `python -I tools/ci_validate.py`.

## 🏗️ Compilation et publication

La version se trouve dans `Cargo.toml` et `eh3d.project.json` ; chaque version a son entrée dans [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` compilent le programme, `test.bat` / `test.sh` lancent les tests et `run.bat` / `run.sh` l'exécutent avec un exemple. Les versions sont publiées sur GitHub.

### Téléchargements

Chaque version contient le programme en ligne de commande et la fenêtre pour Windows (64 et 32 bits), Linux (64 bits) et macOS (Apple silicon et Intel), sous la forme `<nom>-<version>-<système>.zip` (Windows) ou `.tar.gz` avec les deux programmes, la licence, le readme, l'icône et `tools.cfg.example`. Les fichiers sont construits par GitHub Actions et listés dans `SHA256SUMS`. Pour vérifier un téléchargement, comparez son empreinte avec le fichier `.sha256` voisin (`sha256sum -c`).

## 📂 Structure du dépôt

<p align="center">
  <img src="images/repository-structure.svg" alt="Structure du dépôt EH3D-TOOL-MANAGER" width="100%">
</p>

<details>
<summary>Version texte de l'arborescence</summary>

```text
EH3D-TOOL-MANAGER/
├── src/
│   ├── calc.rs                                  une partie de la logique
│   ├── cli_args.rs                              les arguments de la ligne de commande, lus strictement
│   ├── csv.rs                                   une partie de la logique
│   ├── gui.rs                                   la fenêtre (egui, avec la fonctionnalité gui) : dessine l'état et transmet ce que fait la personne
│   ├── gui_main.rs                              la fenêtre comme programme à part, sans console sous Windows
│   ├── lang.rs                                  les mots de la fenêtre dans les huit langues
│   ├── lib.rs                                   la bibliothèque : ce que chaque commande utilise
│   ├── main.rs                                  la ligne de commande
│   ├── model.rs                                 une partie de la logique
│   ├── settings.rs                              les réglages de tools.cfg : langue, taille de la fenêtre, fichier de l'outillage et nombres du calcul
│   ├── shot.rs                                  le mode capture de la fenêtre (--shot) : ses arguments, lus et vérifiés sans écran
│   ├── store.rs                                 une partie de la logique
│   └── window.rs                                l'état de la fenêtre sans écran : l'outillage, le formulaire, les filtres et le calculateur
├── tests/
│   ├── crib.rs                                  tests de la bibliothèque et de la ligne de commande
│   ├── shot_sample.rs                           tests de la boîte à outils d'exemple et de la fenêtre d'une capture
│   └── vbit.rs                                  tests de la largeur de coupe d'une fraise en V : la commande, la profondeur dans calc et les réglages
├── examples/
│   └── crib.json                                un exemple pour essayer le programme
├── docs/
│   ├── architecture.json                        données de l'image d'Architecture
│   └── structure.json                           données de l'image de Structure
├── tools/                                       contrôle de la CI et du README et outils d'images
├── images/                                      bannière et images d'Architecture et de Structure du dépôt
├── .github/                                     workflows de CI et de publication, modèles d'issues et de pull requests
├── eh3d.project.json                            manifeste : version, rôle, maturité et commande de compilation
├── Cargo.toml Cargo.lock                        le workspace et ses dépendances verrouillées
├── rust-toolchain.toml                          la toolchain Rust épinglée
├── CHANGELOG.md                                 ce qui a changé dans chaque version
├── SECURITY.md SUPPORT.md                       signaler une vulnérabilité et poser des questions
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   contribuer, code de conduite et licence
└── tools.cfg.example                            tous les réglages de tools.cfg avec leur valeur par défaut
```

</details>

## 🗺️ Feuille de route et limites connues

**Limites connues aujourd'hui**

* La fenêtre est écrite et sa logique est testée, mais elle n'a pas encore été compilée ni regardée sur un écran.
* La fenêtre demande le fichier de l'outillage et le tableau CSV sous forme de chemins écrits (pas encore de boîte de dialogue de fichiers).
* Il n'a été essayé ni sur une machine ni avec une collection de fichiers réels, et n'a pas eu de revue indépendante.
* Le magasin est un fichier JSON, pas une base de données ; les avances et vitesses sont des points de départ tirés de valeurs typiques, non vérifiés sur une machine ; les programmes de la suite ne lisent pas encore le magasin.

**Ensuite**

* Faire lire le magasin par DRILLPRO, ROUTEPRO et les programmes CAM au lieu de leurs tables propres, et ajouter le profil 2D de chaque outil.
* Regarder la fenêtre sur un écran de chaque système et ajuster ce qui est trouvé.
* L'essayer avec des fichiers réels et avec les autres programmes de la suite.

## 🌍 Langues

Le README et les textes existent en anglais, espagnol, français, italien, allemand, chinois, japonais et portugais européen.

## 🔗 Projets liés

* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): le conteneur de projet et les types partagés de la suite
* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): le programme de perçage
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): le programme de fraisage de contour
* [EH3D-ISOCAM](https://github.com/JuanenRac/EH3D-ISOCAM): le fraisage d'isolation du cuivre
* [EH3D-PHOENIX-LASER](https://github.com/JuanenRac/EH3D-PHOENIX-LASER): la gravure et la découpe laser
* [EH3D-PHOENIX-PNP](https://github.com/JuanenRac/EH3D-PHOENIX-PNP): l'assemblage pick and place
* [EH3D-SUITE-LAUNCHER](https://github.com/JuanenRac/EH3D-SUITE-LAUNCHER): le centre qui trouve et lance les programmes de la suite

## 📚 Documentation

* [Journal des modifications](CHANGELOG.md)
* [Données d'architecture et de structure](docs/architecture.json) : les données qui servent à dessiner les deux images.

## 🤝 Contribuer et communauté

* Avant une pull request, lancez `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` et `python -I tools/ci_validate.py` : tout doit passer. Voir [CONTRIBUTING.md](CONTRIBUTING.md) et le [code de conduite](CODE_OF_CONDUCT.md).
* Le README existe en huit langues : un changement de texte se fait dans les huit fichiers, et les deux images sont dessinées depuis `docs/architecture.json` et `docs/structure.json` par `tools/make_architecture.py` et `tools/make_structure.py`.
* Signalez une vulnérabilité en privé : [SECURITY.md](SECURITY.md). Questions, idées et rapports : electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTEUR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENCE

Logiciel : GPL-3.0-or-later. Voir [LICENSE](LICENSE). Les conceptions matérielles de la suite utilisent la CERN OHL-S.
