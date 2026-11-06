<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-TOOL-MANAGER banner" width="100%">
</p>

# <img src="images/brand/png/icon-64.png" alt="" width="40" height="40" align="absmiddle"> EH3D-TOOL-MANAGER

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | 🇯🇵 <b>日本語</b> | <a href="README_por.md">🇵🇹 Português</a></p>

### EH3D Fabrication Suite の工具庫：工具、摩耗、送りと回転数

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Version-0.0.6-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**正直さの確認 - 現在実際に動くもの：** Rust ライブラリとコマンドライン プログラム（`eh3d-tools`）で、コンピューター上で実行される 115 件のテストがあります：工具の形状と摩耗を管理し、JSON または CSV で保存し、材料別の初期送りと回転数を計算します。ウィンドウ（`eh3d-tools-gui`、`gui` 機能でビルド）は書かれ、そのロジックはテスト済みですが、ビルドも画面での確認もしていません。機械では何も試しておらず、実際のファイルの集まりでも確認していません。

---

> **安全とセキュリティの状況：** このプログラムは機械を制御せず、安全システムでもありません。出力は、機械で何かを実行する前に人が確認する必要があります。ここにあるものは実機で動かしたことがありません。


## 📋 ひと目で

| | |
|---|---|
| バージョン | `0.0.6` |
| 成熟度 | 足場 |
| テスト | 115 |
| 技術スタック | Rust |
| 実機での動作 | 未試験：ここにあるものは機械で動かしたことがありません |

**移動先：** [概要](#-概要) · [アーキテクチャ](#-アーキテクチャ) · [クイックスタート](#-クイックスタート) · [テスト](#-テスト) · [構成](#-リポジトリの構成) · [関連](#-関連プロジェクト)

## 🚦 プロジェクトの状況

各行はこのリポジトリの一部分で、信号は今日どこまで進んでいるかを示します。計画ではなく、リポジトリが示せるもの（テスト、ビルド、実行）に基づいて書かれています。

🟢 完了・検証済み · 🟡 動作する（既知の制限あり） · 🟠 一部のみ、または未検証 · 🔴 未実装または動作しない

| 領域 | 状態 | ここでの意味 |
|---|---|---|
| コア | 🟡 | 工具の形状と摩耗を管理し、JSON または CSV で保存し、材料別の初期送りと回転数を計算します。コンピューター上で通常入力と破損入力でテスト済み。実際のファイルの集まりでは未確認。 |
| コマンドライン | 🟡 | `eh3d-tools` はライブラリと同じ関数を実行して結果を表示します。エラーはメッセージと 0 以外の終了コードで返ります。 |
| テスト | 🟡 | 115 件のテストが通ります（`cargo test`）。通常ケース、境界ケース、破損入力を含みます。 |
| セキュリティ | 🟡 | 安全な Rust のみ（`forbid(unsafe_code)`）。受け取ったファイルでパニックせず、不正な値は理由付きのエラーになります。独立したレビューはありません。 |
| ウィンドウ | 🟠 | 作成済み（egui、`--features gui`、プログラム `eh3d-tools-gui`）：コマンドラインと同じ検証で工具を追加・変更・削除できる表、フィルターと検索、ライブラリのエラーを返す CSV の読み込みと書き出し、計算式と条件を示す送り・回転数の計算機。ロジックはテスト済みですが、まだビルドも画面での確認もしていません。 |
| 実機 | 🔴 | 未試験：ここにあるものは機械で実行したことがありません。 |
| ドキュメント | 🟡 | 8 言語の README、変更履歴、`docs/` から描いた 2 枚の図。 |

## 🎯 概要

* **工具庫**（`src/model.rs`）：ドリル、エンドミル、V ビット、エングレーバー、レーザー、ノズルを直径・刃数・長さ・先端角・シャンクとともに管理し、各値を検査して重複名を拒否し、各工具に番号を付けます。摩耗カウンターや刃数が整数でない場合は、フィールド名付きで拒否されます（ファイルでもコマンドラインでも）。
* **摩耗**（`src/model.rs`）：ジョブごとに時間・距離・穴数を加算し、時間または穴数の上限を設定すると、良好・まもなく交換（90 % 以上）・交換に分かれ、`wear` は悪い順に一覧します。
* **保存と送り・回転数**（`src/store.rs`、`src/csv.rs`、`src/calc.rs`）：工具庫を JSON 文書または表計算用 CSV として入出力し、エラーごとに行を示します。FR4、アルミ、アクリル、木材、真鍮の切削速度と刃当たり送りを、主軸の上限内で計算します。
* **ウィンドウ**（`src/window.rs`、`src/gui.rs`；`--features gui`、プログラム `eh3d-tools-gui`）：工具の表と、コマンドラインと同じ検証で追加・変更・削除するフォーム（削除した工具の番号は再利用されません）、フィルターと検索、工具の使用量をカウンターに加算する機能、ライブラリのエラーを返す CSV の読み込みと書き出し、各数値の式と条件を示す計算機。計算に使う数値（切削速度、刃当たり送り、係数）は `tools.cfg` にあり、ウィンドウで変更できます。状態、操作、設定、文言はテスト付きの単純なコードで、描画は薄い層です。作成済みですが、まだ画面では確認していません。

## 🧩 EH3D での役割

EH3D-TOOL-MANAGER は EH3D Fabrication Suite のプログラムの一つです。ライブラリ、コマンドライン、そして `gui` 機能でビルドするとスイートの他のプログラムと共通の外観のウィンドウ（`eh3d-tools-gui`）として動作します。単純なファイルを読み書きするため、単独でも他のプログラムの間でも使えます。

## 🏛️ アーキテクチャ

<p align="center">
  <img src="images/architecture.svg" alt="EH3D-TOOL-MANAGER のアーキテクチャ：誰が使い、何を持ち、何に依存するか" width="100%">
</p>


このリポジトリの各部分の関係：誰が使い、何を持ち、何に依存するか。図は `docs/architecture.json` から描かれ、コードと照合されます。

## ✨ 機能

* **ライブラリとコマンドライン：** `eh3d-tools` の各コマンドはライブラリの関数なので、スイートの他のプログラムがプロセスを起動せずに呼び出せます。
* **破損入力のテスト：** 空のファイル、ゴミデータ、巨大な数値、途中で切れたファイルも与え、どれもパニックしてはいけません。
* **推測しない：** 意味のない値は、どの値かと理由を示すエラーになり、見かけだけ正しい結果にはなりません。
* **unsafe コードなし：** ライブラリとプログラムに `#![forbid(unsafe_code)]`。
* **しっかりした工具庫：** 取り外した工具の番号は再び割り当てられず（工具庫が `next_id` を保持）、整数でない id はそのとおりに知らせ、CSV は閉じ引用符の後ろの文字を拒否し、数式のように始まる文字（`=`、`+`、`-`、`@`）は表計算ソフトが実行しないよう文字として書き出し、材質には上限があり、`--holes` は整数で、未知のオプションはエラーになり、`calc` は V ビットや彫刻刃の結果が出発点にすぎないことを示します。

## 🔒 セキュリティと安全

| 脅威 | 対策 |
|---|---|
| 破損・途中切れ・空のファイル | パニックなし：テストは有効な入力をあらゆる位置で切り、ゴミデータも与えます |
| 意味のない数値（負、無限、極端に大きい） | 値を示すエラーで拒否され、もっともらしい値に丸められることはありません |
| 安全でないメモリ使用 | `forbid(unsafe_code)` の安全な Rust のみ |

**防御していないこと：** エラーなく作られた結果は、検証された結果ではありません。ここには機械の限界を知るものはなく、プログラムは独立したレビューを受けていません。脆弱性は非公開で報告してください（[SECURITY.md](SECURITY.md) 参照）。

## 🔌 インターフェース

ライブラリは `eh3d_tool_manager`（`use eh3d_tool_manager::...`）、コマンドラインは `eh3d-tools` です。コマンドは次のとおりで、結果は標準出力に表示され、エラーは標準エラー出力に 0 以外の終了コードで報告されます。ネットワークアクセスはありません。

```text
EH3D-TOOL-MANAGER/
├── src/
│   ├── calc.rs                                  ロジックの一部
│   ├── cli_args.rs                              コマンドライン引数（厳密に解釈）
│   ├── csv.rs                                   ロジックの一部
│   ├── gui.rs                                   ウィンドウ（egui、gui 機能）：状態を描き、操作を伝える
│   ├── gui_main.rs                              独立したウィンドウ用プログラム（Windows ではコンソールなし）
│   ├── lang.rs                                  ウィンドウの文言（8言語）
│   ├── lib.rs                                   ライブラリ：各コマンドが使うもの
│   ├── main.rs                                  コマンドライン
│   ├── model.rs                                 ロジックの一部
│   ├── settings.rs                              tools.cfg の設定：言語、ウィンドウサイズ、工具庫ファイル、計算に使う数値
│   ├── store.rs                                 ロジックの一部
│   └── window.rs                                画面なしのウィンドウの状態：工具庫、フォーム、フィルター、計算機
├── tests/
│   └── crib.rs                                  ライブラリとコマンドラインのテスト
├── docs/
│   ├── architecture.json                        アーキテクチャ図のデータ
│   └── structure.json                           構成図のデータ
├── tools/                                       CI と README のチェック、図のツール
├── images/                                      バナーとアーキテクチャ・リポジトリ構成の図
├── .github/                                     CI とリリースのワークフロー、Issue と PR のテンプレート
├── eh3d.project.json                            マニフェスト：バージョン、役割、成熟度、ビルドコマンド
├── Cargo.toml Cargo.lock                        ワークスペースと固定された依存関係
├── rust-toolchain.toml                          固定されたRustツールチェーン
├── CHANGELOG.md                                 各バージョンの変更点
├── SECURITY.md SUPPORT.md                       脆弱性の報告方法と問い合わせ先
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   貢献方法、行動規範、ライセンス
└── tools.cfg.example                            tools.cfg のすべての設定と既定値
```

## ⚙️ 設定

設定ファイルはプログラムのフォルダ（環境変数 `EH3D_TOOLS_DIR`、Windows では `%APPDATA%\eh3d-tools`、それ以外では `~/.config/eh3d-tools`）にある `tools.cfg` で、ウィンドウと `calc` コマンドが読みます。ウィンドウの言語と大きさ、工具庫のファイル、計算機が使い始める主軸の回転数、そして送りと回転数の計算に使うすべての数値（材料ごとの切削速度と刃当たり送り、突っ込みと 1 パスの深さの係数）は `キー = 値` の行で、上限が検査され、不正な行はエラーで名前が示されます。`tools.cfg.example` にすべてのキーと既定値を載せています。それ以外は、選択は（インターフェースを参照）コマンドラインの引数で渡し、各既定値はコマンドのヘルプ（`eh3d-tools help`）に書かれています。値の上限はコード内の名前付き定数で、テストで確認されています。

## 🚀 クイックスタート

安定版の Rust ツールチェーン（1.95 以降）が必要です。

```bash
cargo build --release
cargo test
cargo run --release -- calc drill 0.8 fr4 24000
# ウィンドウ（gui 機能が必要）
cargo build --release --features gui
cargo run --release --features gui --bin eh3d-tools-gui
```

## 🧪 テスト

テストは 115 件あります（`cargo test`、CI では Windows と Linux で実行）。通常ケース、境界ケース、破損入力を確認し、空のファイル、ゴミデータ、途中で切れたファイルでもパニックしません。CI は `cargo fmt --check`、警告をエラーにした `cargo clippy` （`--features gui` でも）、`python -I tools/ci_validate.py` も実行します。

## 🏗️ ビルドとリリース

バージョンは `Cargo.toml` と `eh3d.project.json` にあり、各バージョンは [CHANGELOG.md](CHANGELOG.md) に項目があります。`build.bat` / `build.sh` でビルド、`test.bat` / `test.sh` でテスト、`run.bat` / `run.sh` でサンプル実行します。リリースは GitHub で公開されます。

### ダウンロード

各リリースには、Windows（64 ビットと 32 ビット）、Linux（64 ビット）、macOS（Apple silicon と Intel）向けのコマンドライン プログラムとウィンドウが、`<名前>-<バージョン>-<システム>.zip`（Windows）または `.tar.gz` として含まれ、2 つのプログラム、ライセンス、readme、アイコン、`tools.cfg.example` が入っています。ファイルは GitHub Actions でビルドされ、`SHA256SUMS` に記載されます。ダウンロードを確認するには、隣の `.sha256` ファイルとハッシュを比較してください（`sha256sum -c`）。

## 📂 リポジトリの構成

<p align="center">
  <img src="images/repository-structure.svg" alt="EH3D-TOOL-MANAGER のリポジトリ構成" width="100%">
</p>

<details>
<summary>ツリーのテキスト版</summary>

```text
EH3D-TOOL-MANAGER/
├── src/
│   ├── calc.rs                                  ロジックの一部
│   ├── cli_args.rs                              コマンドライン引数（厳密に解釈）
│   ├── csv.rs                                   ロジックの一部
│   ├── gui.rs                                   ウィンドウ（egui、gui 機能）：状態を描き、操作を伝える
│   ├── gui_main.rs                              独立したウィンドウ用プログラム（Windows ではコンソールなし）
│   ├── lang.rs                                  ウィンドウの文言（8言語）
│   ├── lib.rs                                   ライブラリ：各コマンドが使うもの
│   ├── main.rs                                  コマンドライン
│   ├── model.rs                                 ロジックの一部
│   ├── settings.rs                              tools.cfg の設定：言語、ウィンドウサイズ、工具庫ファイル、計算に使う数値
│   ├── store.rs                                 ロジックの一部
│   └── window.rs                                画面なしのウィンドウの状態：工具庫、フォーム、フィルター、計算機
├── tests/
│   └── crib.rs                                  ライブラリとコマンドラインのテスト
├── docs/
│   ├── architecture.json                        アーキテクチャ図のデータ
│   └── structure.json                           構成図のデータ
├── tools/                                       CI と README のチェック、図のツール
├── images/                                      バナーとアーキテクチャ・リポジトリ構成の図
├── .github/                                     CI とリリースのワークフロー、Issue と PR のテンプレート
├── eh3d.project.json                            マニフェスト：バージョン、役割、成熟度、ビルドコマンド
├── Cargo.toml Cargo.lock                        ワークスペースと固定された依存関係
├── rust-toolchain.toml                          固定されたRustツールチェーン
├── CHANGELOG.md                                 各バージョンの変更点
├── SECURITY.md SUPPORT.md                       脆弱性の報告方法と問い合わせ先
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   貢献方法、行動規範、ライセンス
└── tools.cfg.example                            tools.cfg のすべての設定と既定値
```

</details>

## 🗺️ ロードマップと既知の制限

**現時点で分かっている制限**

* ウィンドウは書かれロジックはテスト済みですが、まだビルドも画面での確認もしていません。
* ウィンドウは工具庫ファイルと CSV 表をパスの入力で指定します（ファイルダイアログはまだありません）。
* 機械でも実際のファイルの集まりでも試しておらず、独立したレビューも受けていません。
* 工具庫は JSON ファイルでデータベースではなく、送りと回転数は一般的な値から取った出発点で機械では確認しておらず、スイートのプログラムはまだ工具庫を読みません。

**次のステップ**

* DRILLPRO、ROUTEPRO、CAM プログラムが独自の工具表ではなく工具庫を読むようにし、各工具の 2D プロファイルを追加する。
* 各システムの画面でウィンドウを確認し、見つかった点を調整する。
* 実際のファイルとスイートの他のプログラムで試す。

## 🌍 言語

README とテキストは、英語、スペイン語、フランス語、イタリア語、ドイツ語、中国語、日本語、ヨーロッパポルトガル語で提供されています。

## 🔗 関連プロジェクト

* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): プロジェクト コンテナーとスイート共通の型
* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): 穴あけプログラム
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): 輪郭ミリング プログラム
* [EH3D-ISOCAM](https://github.com/JuanenRac/EH3D-ISOCAM): 銅箔の絶縁ミリング
* [EH3D-PHOENIX-LASER](https://github.com/JuanenRac/EH3D-PHOENIX-LASER): レーザー彫刻と切断
* [EH3D-PHOENIX-PNP](https://github.com/JuanenRac/EH3D-PHOENIX-PNP): ピック＆プレース実装
* [EH3D-SUITE-LAUNCHER](https://github.com/JuanenRac/EH3D-SUITE-LAUNCHER): スイートのプログラムを見つけて起動するハブ

## 📚 ドキュメント

* [変更履歴](CHANGELOG.md)
* [アーキテクチャと構成のデータ](docs/architecture.json)：2 枚の図を描くためのデータ。

## 🤝 コントリビュートとコミュニティ

* プルリクエストの前に `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`、`python -I tools/ci_validate.py` を実行し、すべて通してください。[CONTRIBUTING.md](CONTRIBUTING.md) と[行動規範](CODE_OF_CONDUCT.md)を参照。
* README は 8 言語あります。文章の変更は 8 ファイルすべてで行い、2 枚の図は `tools/make_architecture.py` と `tools/make_structure.py` が `docs/architecture.json` と `docs/structure.json` から描きます。
* 脆弱性は非公開で報告してください：[SECURITY.md](SECURITY.md)。質問・提案・報告：electrohobby3d@gmail.com（[SUPPORT.md](SUPPORT.md)）。

## 👤 作者

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 ライセンス

ソフトウェア：GPL-3.0-or-later。[LICENSE](LICENSE) を参照。スイートのハードウェア設計には CERN OHL-S を使います。
