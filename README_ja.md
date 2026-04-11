# openapi-php

[![CI](https://github.com/soipon05/openapi-php/actions/workflows/ci.yml/badge.svg)](https://github.com/soipon05/openapi-php/actions)
[![Crates.io](https://img.shields.io/crates/v/openapi-php.svg)](https://crates.io/crates/openapi-php)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**OpenAPI 3.x 仕様から PHP モデル・API クライアント・Laravel ボイラープレートをミリ秒で生成する Rust CLI ツール。**

```
openapi-php generate --input openapi.yaml --framework laravel
```

[English README](README.md)

---

## 目次

- [特徴](#特徴)
- [インストール](#インストール)
- [クイックスタート](#クイックスタート)
- [設定ファイル](#設定ファイル)
- [CLI リファレンス](#cli-リファレンス)
- [アーキテクチャ](#アーキテクチャ)
- [テンプレートのカスタマイズ](#テンプレートのカスタマイズ)
- [コントリビューション](#コントリビューション)

---

## 特徴

| | |
|---|---|
| ⚡ **高速** | Rust 製。数千ファイルを1秒未満で生成 |
| 🎯 **正確** | `$ref` 解決・`allOf` マージ・nullable・enum を完全サポート |
| 🐘 **PHP 8.1 〜 8.3** | readonly DTO・`BackedEnum`・union type をバージョンごとに最適化 |
| 🏗️ **フレームワーク対応** | `plain`（依存ゼロ）・`laravel`（FormRequest / JsonResource / Controller / routes stub）・`symfony`（WIP、plain にフォールバック） |
| 🔍 **差分モード** | `--diff` で生成物とディスクの差異があると終了コード 1（CI ゲートに最適） |
| 👀 **ウォッチモード** | `--watch` でスペック変更を検知して自動再生成 |
| 🧩 **テンプレート上書き** | `--templates` に Jinja2 テンプレートを置くだけでカスタマイズ |

---

## インストール

### 事前コンパイル済みバイナリ（最速）

[GitHub Releases](https://github.com/soipon05/openapi-php/releases) からお使いのプラットフォーム向けバイナリをダウンロードしてください:

| プラットフォーム | アセット名 |
|---|---|
| Linux x86_64 | `openapi-php-x86_64-unknown-linux-musl` |
| Linux aarch64 | `openapi-php-aarch64-unknown-linux-musl` |
| macOS x86_64 | `openapi-php-x86_64-apple-darwin` |
| macOS aarch64 (Apple Silicon) | `openapi-php-aarch64-apple-darwin` |
| Windows x86_64 | `openapi-php-x86_64-pc-windows-msvc.exe` |

### Cargo

```bash
cargo install openapi-php
```

### ソースからビルド

```bash
git clone https://github.com/soipon05/openapi-php.git
cd openapi-php
cargo build --release
# バイナリ: ./target/release/openapi-php
```

---

## クイックスタート

スペックファイル `openapi.yaml` がある場合:

```bash
# スペックの検証
openapi-php validate --input openapi.yaml

# プレーン PHP（モデル + クライアント）を生成
openapi-php generate --input openapi.yaml --output generated/

# Laravel ボイラープレートを生成（FormRequest・JsonResource・routes stub）
openapi-php generate --input openapi.yaml --framework laravel --output app/Generated/

# 生成プレビュー — ファイルへの書き込みなし
openapi-php generate --input openapi.yaml --dry-run

# CI ゲート: 生成物がディスクと異なれば終了コード 1
openapi-php generate --input openapi.yaml --diff

# スペック保存のたびに自動再生成
openapi-php generate --input openapi.yaml --watch
```

**`Pet` スキーマを含む `petstore.yaml` に対する Laravel 出力例:**

```
app/Generated/
  Models/
    Pet.php              # readonly DTO
    PetStatus.php        # BackedEnum
  Http/
    Controllers/
      PetController.php      # リソースコントローラースタブ (index/show/store/update/destroy)
    Requests/
      NewPetRequest.php      # FormRequest（バリデーションルール付き）
    Resources/
      PetResource.php        # JsonResource
  routes/
    api.php              # Route::apiResource スタブ
```

---

## 設定ファイル

プロジェクトルートに `openapi-php.toml` を置くと CLI フラグの繰り返しを省けます:

```toml
[input]
path = "openapi/api.yaml"

[generator]
output    = "app/Generated"
namespace = "App\\Generated"
framework = "laravel"        # plain | laravel | symfony (WIP)
php_version = "8.2"          # 8.1 | 8.2 | 8.3
```

優先順位: **CLI フラグ > openapi-php.toml > 組み込みデフォルト**

設定ファイルはカレントディレクトリから `.git` を含む祖先ディレクトリまで自動探索します。

---

## CLI リファレンス

```
openapi-php <COMMAND>

Commands:
  generate   OpenAPI スペックから PHP コードを生成
  validate   OpenAPI スペックファイルを検証

generate のオプション:
  -i, --input <PATH>         OpenAPI スペックファイル（YAML または JSON）
  -o, --output <DIR>         出力ディレクトリ  [デフォルト: generated/]
  -n, --namespace <NS>       PHP 名前空間     [デフォルト: App\Generated]
  -m, --mode <MODE>          models | client | all  [デフォルト: all]
      --framework <FW>       plain | laravel | symfony
      --php-version <VER>    8.1 | 8.2 | 8.3
      --templates <DIR>      Jinja2 テンプレート上書きディレクトリ
      --dry-run              書き込まずにファイルをプレビュー
      --diff                 ディスクと異なれば終了コード 1
      --watch                スペック変更を検知して自動再生成
```

---

## アーキテクチャ

```
openapi.yaml / openapi.json
        │
        ▼
  ┌─────────────┐
  │   parser    │  serde_yaml / serde_json → raw OpenAPI 型
  │  (+ resolve)│  $ref 解決・allOf マージ・インラインスキーマ
  └──────┬──────┘
         │  ResolvedSpec  (IR)
         ▼
  ┌─────────────┐
  │  generator  │  フレームワーク振り分け → CodegenBackend trait
  │             │  Plain PHP  │  Laravel  │  Symfony (WIP)
  │             │  minijinja テンプレートをファイルごとにレンダリング
  └──────┬──────┘
         │  Vec<RenderedFile>
         ▼
  ディスク書き込み  /  dry-run プレビュー  /  差分表示
```

**ソースレイアウト:**

```
src/
  main.rs          エントリーポイント（薄いラッパー）
  lib.rs           公開モジュール宣言 + パイプライン説明
  cli/             clap 引数定義と run() ディスパッチ
  config.rs        openapi-php.toml 読み込み + CLI マージ
  parser/
    mod.rs         load_and_resolve() — YAML/JSON → ResolvedSpec
    raw/           生 OpenAPI 3.x の serde デシリアライズ
    resolve/       $ref 解決・allOf・スキーマ正規化
  ir/              中間表現（ResolvedSpec・ResolvedSchema 等）
  generator/
    backend.rs     CodegenBackend trait + CodegenContext
    php/
      plain.rs     PlainPhpBackend
      laravel.rs   LaravelPhpBackend
      context.rs   IR → Jinja2 コンテキスト構造体
      helpers.rs   PHP 固有ヘルパー
      templates.rs 組み込み + 上書きテンプレート読み込み
  php_utils.rs     to_camel_case・to_pascal_case 等
tests/
  fixtures/        統合テスト用サンプル OpenAPI スペック
```

---

## テンプレートのカスタマイズ

すべての生成ファイルは [minijinja](https://github.com/mitsuhiko/minijinja) による Jinja2 テンプレートで駆動されます。  
`--templates` でディレクトリを指定するとテンプレートを上書きできます:

```bash
openapi-php generate \
  --input openapi.yaml \
  --framework laravel \
  --templates ./my-templates/
```

上書き対象のテンプレートファイル名:

| テンプレート | 対象ファイル |
|---|---|
| `model.php.j2` | DTO クラス |
| `enum.php.j2` | BackedEnum |
| `client.php.j2` | PSR-18 API クライアント |
| `laravel/form_request.php.j2` | FormRequest |
| `laravel/resource.php.j2` | JsonResource |
| `laravel/routes.php.j2` | routes/api.php |
| `laravel/controller.php.j2` | Resource Controller |

ディレクトリ内に一致するファイルがあれば組み込みテンプレートを置き換えます。  
一致しないファイルは組み込みデフォルトにフォールバックします。

---

## コントリビューション

1. リポジトリを Fork & Clone
2. `cargo test` — すべてのテストが通ること
3. `cargo clippy -- -D warnings` — 警告ゼロ
4. `main` ブランチに PR を出す

バグ報告・機能リクエストは [GitHub Issues](https://github.com/soipon05/openapi-php/issues) へ。

---

## ライセンス

MIT — [LICENSE](LICENSE) を参照。
