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
- [生成コードの詳細](#生成コードの詳細)
  - [タグ別分割（Split by Tag）](#タグ別分割split-by-tag)
  - [認証なし PSR-18 クライアント](#認証なし-psr-18-クライアント)
  - [OpenAPI 3.1 nullable 型](#openapi-31-nullable-型)
  - [判別共用体型（Discriminated Union）](#判別共用体型discriminated-union)
  - [PHP バージョンと readonly](#php-バージョンと-readonly)
- [設定ファイル](#設定ファイル)
- [CLI リファレンス](#cli-リファレンス)
- [アーキテクチャ](#アーキテクチャ)
- [テンプレートのカスタマイズ](#テンプレートのカスタマイズ)
- [コントリビューション](#コントリビューション)

---

## 特徴

| 機能 | 説明 |
|---|---|
| 高速 | Rust 製。数千ファイルを1秒未満で生成 |
| 正確 | `$ref` 解決・`allOf` マージ・nullable・enum を完全サポート |
| OpenAPI 3.0 & 3.1 | `nullable: true`（OAS 3.0）と `type: ["string", "null"]`（OAS 3.1）の両スタイルに対応。どちらも `?T` を生成 |
| PHP 8.1 〜 8.4 | readonly DTO・`BackedEnum`・union type |
| フレームワーク対応 | `plain`（依存ゼロ）・`laravel`（FormRequest / JsonResource / Controller スタブ / routes stub — Laravel 12+ 対象）・`symfony`（WIP、plain にフォールバック） |
| タグ別分割 | `--split-by-tag` で OpenAPI タグごとに `{Tag}Client.php` を分割生成（デフォルトは `ApiClient.php` に集約） |
| 認証なし PSR-18 クライアント | Bearer/ApiKey の注入なし。認証は呼び出し側が PSR-18 ミドルウェアとして実装して注入 |
| PHPStan 型エイリアス | DTO に `@phpstan-type PetData array{…}` を自動生成 — PHPStan strict モード対応 |
| Enum ラベル | `x-enum-descriptions` ベンダー拡張から `label(): string` メソッドを生成 |
| 非推奨プロパティ | OpenAPI `deprecated: true` のプロパティに `#[\Deprecated]` 属性を付与 |
| FormRequest ルール | `minLength`/`maxLength`/`pattern`/`minimum`/`maximum` から Laravel バリデーションルールを自動導出 |
| 差分モード | `--diff` で生成物とディスクの差異があると終了コード 1（CI ゲートに活用） |
| ウォッチモード | `--watch` でスペック変更を検知して自動再生成 |
| テンプレート上書き | `--templates` に Jinja2 テンプレートを置くだけでカスタマイズ |

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
    api.php              # Route::get/post/… スタブ（use import 付き）
```

`routes/api.php` の出力例:

```php
use Illuminate\Support\Facades\Route;
use App\Generated\Http\Controllers\PetController;

// GET /pets → PetController@index
Route::get('/pets', [PetController::class, 'index']);
// POST /pets → PetController@store
Route::post('/pets', [PetController::class, 'store']);
```

---

## 生成コードの詳細

### タグ別分割（Split by Tag）

デフォルトでは全エンドポイントが `Client/ApiClient.php` 1 ファイルにまとめられます。  
`--split-by-tag` を指定すると、OpenAPI タグごとに独立したクライアントファイルを生成します:

```bash
openapi-php generate --input openapi.yaml --split-by-tag
```

**出力例（petstore — タグ: `pets`・`store`）:**

```
Client/
  PetsClient.php    ← tag: pets のエンドポイントだけ
  StoreClient.php   ← tag: store のエンドポイントだけ
```

タグなしのエンドポイントは `DefaultClient.php` にまとめられます。

`openapi-php.toml` でも設定できます:

```toml
[generator]
split_by_tag = true
```

---

### 認証なし PSR-18 クライアント

生成される `ApiClient.php` / `{Tag}Client.php` は Bearer トークンや API キーを一切注入しません。  
認証はアプリ側で PSR-18 ミドルウェアとして実装し、`ClientInterface` として注入してください:

```php
// 認証を PSR-18 ミドルウェアで実装（Guzzle HandlerStack の例）
$stack = HandlerStack::create();
$stack->push(new BearerAuthMiddleware($token));
$psr18 = new GuzzleAdapter(new GuzzleClient(['handler' => $stack]));

$client = new ApiClient(
    httpClient: $psr18,
    requestFactory: new Psr17Factory(),
    streamFactory: new Psr17Factory(),
);
```

---

### OpenAPI 3.1 nullable 型

2 種類の nullable 表記をどちらもサポートし、どちらも PHP では `?T` を生成します:

```yaml
# OpenAPI 3.1 スタイル
description:
  type: ["string", "null"]

# OpenAPI 3.0 スタイル（引き続き動作）
description:
  type: string
  nullable: true
```

---

### 判別共用体型（Discriminated Union）

`discriminator.propertyName` を持つ `oneOf` スキーマに対しては、PHP の `final class` が生成されます。`fromArray()` ファクトリメソッドが判別フィールドの値を元に正しいサブクラスへディスパッチします。

**入力（OpenAPI YAML）:**

```yaml
components:
  schemas:
    Shape:
      oneOf:
        - $ref: '#/components/schemas/Circle'
        - $ref: '#/components/schemas/Rectangle'
      discriminator:
        propertyName: type
        mapping:
          circle: '#/components/schemas/Circle'
          rectangle: '#/components/schemas/Rectangle'
```

**生成される PHP:**

```php
final class Shape
{
    private function __construct(
        public readonly Circle|Rectangle $value,
    ) {}

    /** @param array<string, mixed> $data */
    public static function fromArray(array $data): self
    {
        return match ((string) ($data['type'] ?? '')) {
            'circle'    => new self(Circle::fromArray($data)),
            'rectangle' => new self(Rectangle::fromArray($data)),
            default     => throw new \UnexpectedValueException(
                'Shape: unknown discriminator value "' . ($data['type'] ?? '') . '"',
            ),
        };
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return $this->value->toArray();
    }
}
```

`mapping` を省略した場合は、OpenAPI 仕様のデフォルトに従いスキーマ名がそのままマッチキーになります。`discriminator` を持たない `oneOf` および `anyOf` は union クラスを生成しません。

> **nullable 省略記法** — `oneOf: [{$ref: '#/components/schemas/T'}, {nullable: true}]` は union クラスを生成せず、`?T` 型のプロパティとして解決されます。

---

### PHP バージョンと readonly

`--php-version` フラグ（または `openapi-php.toml` の `php_version`）によって、readonly プロパティの出力形式が変わります。

| バージョン | 効果 |
|-----------|------|
| `8.1`（デフォルト） | 各プロパティに個別で `public readonly` を付与 |
| `8.2`・`8.3`・`8.4` | クラス宣言が `readonly final class` になり、各プロパティの `readonly` は省略 |

**PHP 8.1 出力（デフォルト）:**

```php
final class Pet
{
    public function __construct(
        public readonly string $name,
        public readonly ?int $age = null,
    ) {}
}
```

**PHP 8.2 以上の出力（`--php-version 8.2`）:**

```php
readonly final class Pet
{
    public function __construct(
        public string $name,
        public ?int $age = null,
    ) {}
}
```

フラグの繰り返しを避けるには `openapi-php.toml` に記載します:

```toml
[generator]
php_version = "8.2"
```

---

## 設定ファイル

プロジェクトルートに `openapi-php.toml` を置くと CLI フラグの繰り返しを省けます:

```toml
[input]
path = "openapi/api.yaml"

[generator]
output       = "app/Generated"
namespace    = "App\\Generated"
framework    = "laravel"        # plain | laravel | symfony (WIP)
php_version  = "8.2"           # 8.1 | 8.2 | 8.3 | 8.4
split_by_tag = true            # タグごとに {Tag}Client.php を分割生成
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
      --php-version <VER>    8.1 | 8.2 | 8.3 | 8.4
      --templates <DIR>      Jinja2 テンプレート上書きディレクトリ
      --split-by-tag         OpenAPI タグごとに {Tag}Client.php を分割生成
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

## サンプル

[`examples/`](examples/) ディレクトリにサンプル OpenAPI スペックと生成済み PHP ファイルをコミット済みです。ツールを実行しなくても生成物を確認できます。

| サンプル | スペック | plain 出力 | Laravel 出力 | split 出力 |
|---|---|---|---|---|
| simple | [openapi.yaml](examples/simple/openapi.yaml) | [output/](examples/simple/output/) | [output-laravel/](examples/simple/output-laravel/) | — |
| petstore | [openapi.yaml](examples/petstore/openapi.yaml) | [output/](examples/petstore/output/) | [output-laravel/](examples/petstore/output-laravel/) | [output-split/](examples/petstore/output-split/) |
| discriminated-union | [openapi.yaml](examples/discriminated-union/openapi.yaml) | [output/](examples/discriminated-union/output/) | [output-laravel/](examples/discriminated-union/output-laravel/) | — |

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
