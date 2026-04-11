# openapi-php

[![CI](https://github.com/soipon05/openapi-php/actions/workflows/ci.yml/badge.svg)](https://github.com/soipon05/openapi-php/actions)
[![Crates.io](https://img.shields.io/crates/v/openapi-php.svg)](https://crates.io/crates/openapi-php)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Generate PHP models, API clients, and Laravel boilerplate from any OpenAPI 3.x spec — in milliseconds.**

```
openapi-php generate --input openapi.yaml --framework laravel
```

---

## Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration file](#configuration-file)
- [CLI Reference](#cli-reference)
- [Architecture](#architecture)
- [Template overrides](#template-overrides)
- [Contributing](#contributing)

---

## Features

| | |
|---|---|
| ⚡ **Fast** | Written in Rust; generates thousands of files in under a second |
| 🎯 **Precise** | Respects `$ref` resolution, `allOf`, nullable types, and enums |
| 🐘 **PHP 8.1 – 8.3** | Readonly DTOs, `BackedEnum`, union types — tuned per version |
| 🏗️ **Framework-aware** | `plain` (zero dependencies), `laravel` (FormRequest, JsonResource, Controller, routes stub), `symfony` (WIP — falls back to plain) |
| 🔍 **Diff mode** | `--diff` exits 1 when generated output diverges from disk (great for CI) |
| 👀 **Watch mode** | `--watch` re-runs generation whenever the spec file changes |
| 🧩 **Override templates** | Drop a Jinja2 template into `--templates` to customise any file |

---

## Installation

### Pre-compiled binary (fastest)

Download a release binary for your platform from [GitHub Releases](https://github.com/soipon05/openapi-php/releases):

| Platform | Asset |
|---|---|
| Linux x86_64 | `openapi-php-x86_64-unknown-linux-musl` |
| Linux aarch64 | `openapi-php-aarch64-unknown-linux-musl` |
| macOS x86_64 | `openapi-php-x86_64-apple-darwin` |
| macOS aarch64 (M-series) | `openapi-php-aarch64-apple-darwin` |
| Windows x86_64 | `openapi-php-x86_64-pc-windows-msvc.exe` |

### Cargo

```bash
cargo install openapi-php
```

### Build from source

```bash
git clone https://github.com/soipon05/openapi-php.git
cd openapi-php
cargo build --release
# Binary at ./target/release/openapi-php
```

---

## Quick Start

Given a spec file `openapi.yaml`:

```bash
# Validate the spec
openapi-php validate --input openapi.yaml

# Generate plain PHP (models + client)
openapi-php generate --input openapi.yaml --output generated/

# Generate Laravel boilerplate (FormRequest, JsonResource, routes stub)
openapi-php generate --input openapi.yaml --framework laravel --output app/Generated/

# Preview what would be written — nothing touches disk
openapi-php generate --input openapi.yaml --dry-run

# CI gate: fail if generated code is out of date
openapi-php generate --input openapi.yaml --diff

# Auto-regenerate on every save
openapi-php generate --input openapi.yaml --watch
```

**Laravel output** for a `petstore.yaml` with a `Pet` schema looks like:

```
app/Generated/
  Models/
    Pet.php              # readonly DTO
    PetStatus.php        # BackedEnum
  Http/
    Controllers/
      PetController.php      # Resource controller stub (index/show/store/update/destroy)
    Requests/
      NewPetRequest.php      # FormRequest with validation rules
    Resources/
      PetResource.php        # JsonResource
  routes/
    api.php              # Route::apiResource stubs
```

---

## Configuration file

Place an `openapi-php.toml` in your project root to avoid repeating CLI flags:

```toml
[input]
path = "openapi/api.yaml"

[generator]
output    = "app/Generated"
namespace = "App\\Generated"
framework = "laravel"        # plain | laravel | symfony (WIP)
php_version = "8.2"          # 8.1 | 8.2 | 8.3
```

CLI flags always override the config file. Options precedence:  
**CLI flag > openapi-php.toml > built-in default**

---

## CLI Reference

```
openapi-php <COMMAND>

Commands:
  generate   Generate PHP code from an OpenAPI spec
  validate   Validate an OpenAPI spec file

Options for `generate`:
  -i, --input <PATH>         OpenAPI spec file (YAML or JSON)
  -o, --output <DIR>         Output directory  [default: generated/]
  -n, --namespace <NS>       PHP namespace     [default: App\Generated]
  -m, --mode <MODE>          models | client | all  [default: all]
      --framework <FW>       plain | laravel | symfony
      --php-version <VER>    8.1 | 8.2 | 8.3
      --templates <DIR>      Directory of Jinja2 template overrides
      --dry-run              Print files without writing
      --diff                 Exit 1 if output differs from disk
      --watch                Re-run on spec file changes
```

---

## Architecture

```
openapi.yaml / openapi.json
        │
        ▼
  ┌─────────────┐
  │   parser    │  serde_yaml / serde_json → raw OpenAPI types
  │  (+ resolve)│  $ref resolution, allOf merging, inline schemas
  └──────┬──────┘
         │  ResolvedSpec  (IR)
         ▼
  ┌─────────────┐
  │  generator  │  Framework dispatch → CodegenBackend trait
  │             │  Plain PHP  │  Laravel  │  Symfony (WIP)
  │             │  minijinja templates rendered per file
  └──────┬──────┘
         │  Vec<RenderedFile>
         ▼
  write to disk  /  dry-run print  /  diff against existing
```

**Source layout:**

```
src/
  main.rs          Entry point (thin)
  lib.rs           Public module declarations + pipeline doc
  cli/             Clap argument definitions and run() dispatch
  config.rs        openapi-php.toml loading + CLI merge
  parser/
    mod.rs         load_and_resolve() — YAML/JSON → ResolvedSpec
    raw/           Serde deserialization of raw OpenAPI 3.x
    resolve/       $ref resolution, allOf, schema normalisation
  ir/              Intermediate representation (ResolvedSpec, ResolvedSchema, …)
  generator/
    backend.rs     CodegenBackend trait + CodegenContext
    php/
      plain.rs     PlainPhpBackend
      laravel.rs   LaravelPhpBackend
      context.rs   IR → Jinja2 context structs
      helpers.rs   PHP-specific helpers
      templates.rs Embedded + override template loading
  php_utils.rs     to_camel_case, to_pascal_case, …
tests/
  fixtures/        Sample OpenAPI specs used by integration tests
  snapshots/       insta snapshot files
```

---

## Template overrides

Every generated file is driven by a Jinja2 template (via [minijinja](https://github.com/mitsuhiko/minijinja)).  
To customise output, copy a template and pass the directory with `--templates`:

```bash
openapi-php generate \
  --input openapi.yaml \
  --framework laravel \
  --templates ./my-templates/
```

Files in `./my-templates/` that match a built-in template name replace the default.  
Unmatched files fall back to the embedded defaults.

---

## Contributing

1. Fork & clone the repo
2. `cargo test` — all tests must pass
3. `cargo clippy -- -D warnings` — no new warnings
4. Open a PR against `main`

Bug reports and feature requests are welcome via [GitHub Issues](https://github.com/soipon05/openapi-php/issues).

---

## License

MIT — see [LICENSE](LICENSE).
