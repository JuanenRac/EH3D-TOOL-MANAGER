<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-TOOL-MANAGER banner" width="100%">
</p>

# <img src="images/brand/png/icon-64.png" alt="" width="40" height="40" align="absmiddle"> EH3D-TOOL-MANAGER

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | 🇵🇹 <b>Português</b></p>

### O armazém de ferramentas da EH3D Fabrication Suite: ferramentas, desgaste, avanços e velocidades

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Version-0.0.6-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Verificação de honestidade - o que funciona hoje:** uma biblioteca Rust e um programa de linha de comandos (`eh3d-tools`) com 126 testes que correm num computador: guarda as ferramentas com a sua geometria e desgaste, armazena-as em JSON ou CSV e calcula avanços e velocidades de partida por material. Uma janela (`eh3d-tools-gui`, compilada com a funcionalidade `gui`) está escrita e a sua lógica está testada, mas não foi compilada nem vista num ecrã; nada foi experimentado numa máquina e nada foi verificado com uma coleção de ficheiros reais.

---

> **Estado de segurança:** este programa não controla nenhuma máquina e não é um sistema de segurança. O que produz deve ser verificado por uma pessoa antes de se executar algo numa máquina; nada disto correu em hardware real.


## 📋 Num relance

| | |
|---|---|
| Versão | `0.0.6` |
| Maturidade | Estrutura inicial |
| Testes | 126 |
| Tecnologias | Rust |
| Em hardware real | Não experimentado: nada disto correu numa máquina |

**Ir para:** [Visão geral](#-visão-geral) · [Arquitetura](#-arquitetura) · [Início rápido](#-início-rápido) · [Testes](#-testes) · [Estrutura](#-estrutura-do-repositório) · [Relacionados](#-projetos-relacionados)

## 🚦 Estado do projeto

Cada linha é uma parte deste repositório e a luz diz até onde chega hoje. É escrita com o que o repositório consegue demonstrar (testes, compilações, execuções), não com o que está previsto.

🟢 Feito e verificado · 🟡 Funciona, com limites conhecidos · 🟠 Feito a meio ou ainda por verificar · 🔴 Por fazer ou a não funcionar

| Área | Estado | O que significa aqui |
|---|---|---|
| Núcleo | 🟡 | guarda as ferramentas com a sua geometria e desgaste, armazena-as em JSON ou CSV e calcula avanços e velocidades de partida por material; testado num computador com entradas normais e danificadas, não com uma coleção de ficheiros reais. |
| Linha de comandos | 🟡 | `eh3d-tools` executa as mesmas funções da biblioteca e imprime o resultado; cada erro volta como mensagem e código de saída diferente de zero. |
| Testes | 🟡 | 126 testes passam (`cargo test`), com casos normais, casos-limite e entradas danificadas. |
| Segurança | 🟡 | Rust seguro e simples (`forbid(unsafe_code)`): nada entra em pânico com os ficheiros recebidos e um valor mau é um erro com o seu motivo; sem revisão independente. |
| Janela | 🟠 | Escrita (egui, `--features gui`, programa `eh3d-tools-gui`): uma tabela das ferramentas que se adicionam, alteram e removem com as verificações da linha de comandos, filtros e pesquisa, importação e exportação de CSV com os erros da biblioteca e uma calculadora de avanços e velocidades que mostra as suas fórmulas e condições; a sua lógica está testada, mas ainda não foi compilada nem vista num ecrã. |
| Hardware real | 🔴 | Não experimentado: nada disto foi executado numa máquina. |
| Documentação | 🟡 | README em oito idiomas, o registo de alterações e as duas imagens desenhadas a partir de `docs/`. |

## 🎯 Visão geral

* **O armazém** (`src/model.rs`): brocas, fresas, fresas em V, gravadores, lasers e bicos com diâmetro, lâminas, comprimento, ângulo de ponta e haste; cada valor é verificado e um nome repetido é recusado; cada ferramenta recebe o seu número. Um contador de desgaste ou um número de arestas que não seja inteiro é recusado com o nome do campo, nos ficheiros e na linha de comandos.
* **O desgaste** (`src/model.rs`): horas, metros e furos somam-se após cada trabalho; com um limite de horas ou furos, uma ferramenta está boa, para trocar em breve (a partir de 90 %) ou para trocar, e `wear` lista as piores primeiro.
* **Armazenamento e avanços e velocidades** (`src/store.rs`, `src/csv.rs`, `src/calc.rs`): o armazém como documento JSON ou tabela CSV para uma folha de cálculo, lido com cuidado e com a linha indicada em cada erro; velocidade de corte e avanço por dente para FR4, alumínio, acrílico, madeira e latão, limitados pelo fuso.
* **A janela** (`src/window.rs`, `src/gui.rs`; `--features gui`, programa `eh3d-tools-gui`): uma tabela das ferramentas com um formulário para as adicionar, alterar e remover com as mesmas verificações da linha de comandos (o número de uma ferramenta removida não volta a ser dado), filtros e pesquisa, a utilização de uma ferramenta somada aos seus contadores, importação e exportação de CSV com os erros da biblioteca e uma calculadora que mostra a fórmula e as condições de cada número; os números com que trabalha (velocidades de corte, avanços por dente, fatores) estão em `tools.cfg` e podem ser alterados na janela. O estado, as ações, as definições e as palavras são código simples com testes; o desenho é fino. Escrita, ainda não vista num ecrã.

## 🧩 Papel no EH3D

EH3D-TOOL-MANAGER é um programa da EH3D Fabrication Suite. Funciona como biblioteca, como linha de comandos e, compilado com a funcionalidade `gui`, como janela (`eh3d-tools-gui`) com o aspeto que os outros programas da suíte partilham. Lê e escreve ficheiros simples, por isso pode ser usado sozinho ou entre os outros programas.

## 🏛️ Arquitetura

<p align="center">
  <img src="images/architecture.svg" alt="Arquitetura de EH3D-TOOL-MANAGER: quem o usa, o que contém e de que depende" width="100%">
</p>


Como as partes deste repositório se encaixam: quem o usa, o que contém e de que depende. A imagem é desenhada a partir de `docs/architecture.json` e verificada com o código.

## ✨ Funcionalidades

* **Biblioteca e linha de comandos:** cada comando de `eh3d-tools` é uma função da biblioteca, por isso outro programa da suíte pode chamá-la sem lançar um processo.
* **Testes com entradas danificadas:** os testes também dão ficheiros vazios, lixo, números enormes e ficheiros cortados, e nenhum pode entrar em pânico.
* **Sem adivinhar:** um valor sem sentido é um erro que diz qual e porquê, nunca um resultado que apenas parece certo.
* **Sem código unsafe:** `#![forbid(unsafe_code)]` na biblioteca e no programa.
* **Um armazém que aguenta:** o número de uma ferramenta retirada nunca volta a ser dado (o armazém guarda `next_id`), um id que não é um número inteiro diz-se como tal, o CSV recusa texto depois de umas aspas de fecho, um texto que começa como uma fórmula (`=`, `+`, `-`, `@`) escreve-se como texto para que a folha de cálculo não o execute, o material tem um limite, `--holes` é um número inteiro, uma opção desconhecida é um erro e `calc` diz que o resultado de uma fresa em V ou de um gravador é só um ponto de partida.
* **A largura do corte de uma fresa em V:** `eh3d-tools vbit --depth MM [--angle GRAUS] [--flat MM]` calcula `2 x profundidade x tan(ângulo / 2) + ponta plana` (uma fresa de 60 graus a 0,5 mm dá 0,5774 mm mais a ponta plana de 0,1 mm = 0,6774 mm); o ângulo vai de 10 a 170 graus, a profundidade de 0 a 100 mm e a ponta plana de 0 a 5 mm, e o que não for indicado é o valor de `tools.cfg`. `calc --depth MM` usa essa largura como o diâmetro com que uma fresa em V ou um gravador trabalha nos avanços e velocidades, e a calculadora da janela tem um campo para a profundidade, mostra a largura do corte e a fórmula com os seus números; sem profundidade os números são os de sempre.

## 🔒 Segurança e proteção

| Ameaça | Defesa |
|---|---|
| Um ficheiro danificado, cortado ou vazio | sem pânico: os testes cortam uma entrada válida em cada posição e dão lixo |
| Um número sem sentido (negativo, não finito, absurdamente grande) | recusado com um erro que nomeia o valor; nada é arredondado para algo plausível |
| Uso inseguro de memória | Rust seguro e simples com `forbid(unsafe_code)` |

**O que não é defendido:** um resultado produzido sem erros não é um resultado verificado. Nada aqui conhece os limites de uma máquina e o programa não teve revisão independente. Comunique uma vulnerabilidade em privado (ver [SECURITY.md](SECURITY.md)).

## 🔌 Interfaces

A biblioteca é `eh3d_tool_manager` (`use eh3d_tool_manager::...`); a linha de comandos é `eh3d-tools`. Os comandos são estes; cada um imprime o resultado na saída padrão e comunica um erro na saída de erros com um código de saída diferente de zero. Não há acesso à rede.

```text
EH3D-TOOL-MANAGER/
├── src/
│   ├── calc.rs                                  uma parte da lógica
│   ├── cli_args.rs                              os argumentos da linha de comandos, lidos com rigor
│   ├── csv.rs                                   uma parte da lógica
│   ├── gui.rs                                   a janela (egui, com a funcionalidade gui): desenha o estado e transmite o que a pessoa faz
│   ├── gui_main.rs                              a janela como programa à parte, sem consola no Windows
│   ├── lang.rs                                  as palavras da janela nos oito idiomas
│   ├── lib.rs                                   a biblioteca: o que cada comando usa
│   ├── main.rs                                  a linha de comandos
│   ├── model.rs                                 uma parte da lógica
│   ├── settings.rs                              as definições de tools.cfg: idioma, tamanho da janela, ficheiro do armazém e os números do cálculo
│   ├── store.rs                                 uma parte da lógica
│   └── window.rs                                o estado da janela sem ecrã: o armazém, o formulário, os filtros e a calculadora
├── tests/
│   ├── crib.rs                                  testes da biblioteca e da linha de comandos
│   └── vbit.rs                                  testes da largura do corte de uma fresa em V: o comando, a profundidade em calc e as definições
├── docs/
│   ├── architecture.json                        dados da imagem de Arquitetura
│   └── structure.json                           dados da imagem de Estrutura
├── tools/                                       verificação da CI e do README e ferramentas de imagens
├── images/                                      banner e as imagens de Arquitetura e Estrutura do repositório
├── .github/                                     fluxos de CI e de publicação, modelos de issues e pull requests
├── eh3d.project.json                            manifesto: versão, papel, maturidade e comando de compilação
├── Cargo.toml Cargo.lock                        o workspace e as suas dependências fixadas
├── rust-toolchain.toml                          a toolchain de Rust fixada
├── CHANGELOG.md                                 o que mudou em cada versão
├── SECURITY.md SUPPORT.md                       como comunicar uma vulnerabilidade e onde perguntar
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   como contribuir, o código de conduta e a licença
└── tools.cfg.example                            todas as definições de tools.cfg com o valor predefinido
```

## ⚙️ Configuração

O ficheiro de configuração é `tools.cfg`, na pasta do programa (a variável `EH3D_TOOLS_DIR`, ou `%APPDATA%\eh3d-tools` no Windows, ou `~/.config/eh3d-tools` nos outros sistemas), lido pela janela e pelo comando `calc`: o idioma e o tamanho da janela, o ficheiro do armazém, a velocidade do fuso com que a calculadora arranca e cada número com que se calculam os avanços e velocidades (a velocidade de corte e o avanço por dente de cada material, os fatores da penetração e da profundidade de passagem, e `v_bit_angle_deg` e `v_bit_flat_mm`, o ângulo (de 10 a 170 graus, 60 por omissão) e a largura da ponta plana (de 0 a 5 mm, 0,1 por omissão) com que se calcula uma fresa em V) são linhas `chave = valor`, com os limites verificados e uma linha má nomeada no erro. `tools.cfg.example` lista cada chave com o valor por omissão. Para além disso, o que a pessoa escolhe é dado como argumentos da linha de comandos (ver Interfaces) e cada valor por omissão está escrito na ajuda do comando (`eh3d-tools help`). Os limites de um valor são constantes com nome no código, cobertas por testes.

## 🚀 Início rápido

Precisa de uma cadeia de ferramentas Rust estável (1.95 ou mais recente).

```bash
cargo build --release
cargo test
cargo run --release -- calc drill 0.8 fr4 24000
# a janela (precisa da funcionalidade gui)
cargo build --release --features gui
cargo run --release --features gui --bin eh3d-tools-gui
```

## 🧪 Testes

O conjunto tem 126 testes (`cargo test`, executados em Windows e Linux no CI). Verificam os casos normais, os casos-limite e as entradas danificadas: ficheiros vazios, lixo e ficheiros cortados nunca provocam pânico. O CI executa ainda `cargo fmt --check`, `cargo clippy` (também com `--features gui`) com avisos como erros e `python -I tools/ci_validate.py`.

## 🏗️ Compilação e publicação

A versão está em `Cargo.toml` e em `eh3d.project.json`; cada versão tem a sua entrada em [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` compilam o programa, `test.bat` / `test.sh` executam os testes e `run.bat` / `run.sh` executam-no com um exemplo. As versões são publicadas no GitHub.

### Transferências

Cada versão traz o programa de linha de comandos e a janela para Windows (64 e 32 bits), Linux (64 bits) e macOS (Apple silicon e Intel), como `<nome>-<versão>-<sistema>.zip` (Windows) ou `.tar.gz` com os dois programas, a licença, o readme, o ícone e `tools.cfg.example`. Os ficheiros são construídos pelo GitHub Actions e listados em `SHA256SUMS`. Para verificar uma transferência, compare o seu hash com o ficheiro `.sha256` ao lado (`sha256sum -c`).

## 📂 Estrutura do repositório

<p align="center">
  <img src="images/repository-structure.svg" alt="Estrutura do repositório EH3D-TOOL-MANAGER" width="100%">
</p>

<details>
<summary>Versão em texto da árvore</summary>

```text
EH3D-TOOL-MANAGER/
├── src/
│   ├── calc.rs                                  uma parte da lógica
│   ├── cli_args.rs                              os argumentos da linha de comandos, lidos com rigor
│   ├── csv.rs                                   uma parte da lógica
│   ├── gui.rs                                   a janela (egui, com a funcionalidade gui): desenha o estado e transmite o que a pessoa faz
│   ├── gui_main.rs                              a janela como programa à parte, sem consola no Windows
│   ├── lang.rs                                  as palavras da janela nos oito idiomas
│   ├── lib.rs                                   a biblioteca: o que cada comando usa
│   ├── main.rs                                  a linha de comandos
│   ├── model.rs                                 uma parte da lógica
│   ├── settings.rs                              as definições de tools.cfg: idioma, tamanho da janela, ficheiro do armazém e os números do cálculo
│   ├── store.rs                                 uma parte da lógica
│   └── window.rs                                o estado da janela sem ecrã: o armazém, o formulário, os filtros e a calculadora
├── tests/
│   ├── crib.rs                                  testes da biblioteca e da linha de comandos
│   └── vbit.rs                                  testes da largura do corte de uma fresa em V: o comando, a profundidade em calc e as definições
├── docs/
│   ├── architecture.json                        dados da imagem de Arquitetura
│   └── structure.json                           dados da imagem de Estrutura
├── tools/                                       verificação da CI e do README e ferramentas de imagens
├── images/                                      banner e as imagens de Arquitetura e Estrutura do repositório
├── .github/                                     fluxos de CI e de publicação, modelos de issues e pull requests
├── eh3d.project.json                            manifesto: versão, papel, maturidade e comando de compilação
├── Cargo.toml Cargo.lock                        o workspace e as suas dependências fixadas
├── rust-toolchain.toml                          a toolchain de Rust fixada
├── CHANGELOG.md                                 o que mudou em cada versão
├── SECURITY.md SUPPORT.md                       como comunicar uma vulnerabilidade e onde perguntar
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   como contribuir, o código de conduta e a licença
└── tools.cfg.example                            todas as definições de tools.cfg com o valor predefinido
```

</details>

## 🗺️ Roteiro e limites conhecidos

**Limites conhecidos hoje**

* A janela está escrita e a sua lógica está testada, mas ainda não foi compilada nem vista num ecrã.
* A janela pede o ficheiro do armazém e a tabela CSV como caminhos escritos (ainda não há diálogo de ficheiros).
* Não foi experimentado numa máquina nem com uma coleção de ficheiros reais e não teve revisão independente.
* O armazém é um ficheiro JSON, não uma base de dados; os avanços e velocidades são pontos de partida de valores típicos e não foram verificados numa máquina; os programas da suíte ainda não leem o armazém.

**A seguir**

* Fazer o DRILLPRO, o ROUTEPRO e os programas CAM lerem o armazém em vez das suas tabelas próprias, e acrescentar o perfil 2D de cada ferramenta.
* Ver a janela num ecrã de cada sistema e ajustar o que se encontrar.
* Experimentá-lo com ficheiros reais e com os outros programas da suíte.

## 🌍 Idiomas

O README e os textos existem em inglês, espanhol, francês, italiano, alemão, chinês, japonês e português europeu.

## 🔗 Projetos relacionados

* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): o contentor de projeto e os tipos partilhados da suíte
* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): o programa de furação
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): o programa de fresagem de contorno
* [EH3D-ISOCAM](https://github.com/JuanenRac/EH3D-ISOCAM): a fresagem de isolamento do cobre
* [EH3D-PHOENIX-LASER](https://github.com/JuanenRac/EH3D-PHOENIX-LASER): a gravação e corte a laser
* [EH3D-PHOENIX-PNP](https://github.com/JuanenRac/EH3D-PHOENIX-PNP): a montagem pick and place
* [EH3D-SUITE-LAUNCHER](https://github.com/JuanenRac/EH3D-SUITE-LAUNCHER): o centro que encontra e arranca os programas da suíte

## 📚 Documentação

* [Registo de alterações](CHANGELOG.md)
* [Dados de arquitetura e estrutura](docs/architecture.json): os dados a partir dos quais as duas imagens são desenhadas.

## 🤝 Contribuir e comunidade

* Antes de um pull request execute `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` e `python -I tools/ci_validate.py`: tudo tem de passar. Veja [CONTRIBUTING.md](CONTRIBUTING.md) e o [código de conduta](CODE_OF_CONDUCT.md).
* O README existe em oito idiomas: uma alteração ao texto faz-se nos oito ficheiros, e as duas imagens são desenhadas a partir de `docs/architecture.json` e `docs/structure.json` por `tools/make_architecture.py` e `tools/make_structure.py`.
* Comunique uma vulnerabilidade em privado: [SECURITY.md](SECURITY.md). Perguntas, ideias e relatos: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTOR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENÇA

Software: GPL-3.0-or-later. Ver [LICENSE](LICENSE). Os projetos de hardware da suite usam a CERN OHL-S.
