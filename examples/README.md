# Examples

This directory follows the common OSS code-generator pattern:

- `openapi.yaml` is the committed input spec.
- `output/` and `output-laravel/` are committed generated outputs.
- The generated output is committed so users can inspect what `openapi-php` produces without running the tool first.

## `simple/`

Demonstrates a small API with a compact model set and both supported framework targets.

Input:

- `examples/simple/openapi.yaml`

Regenerate plain output:

```bash
./target/release/openapi-php generate --input examples/simple/openapi.yaml --output examples/simple/output --namespace 'App\Generated' --framework plain
```

Regenerate Laravel output:

```bash
./target/release/openapi-php generate --input examples/simple/openapi.yaml --output examples/simple/output-laravel --namespace 'App\Generated' --framework laravel
```

## `petstore/`

Demonstrates a larger multi-model API, including richer schema relationships and the generated client/controller structure for both frameworks.

Input:

- `examples/petstore/openapi.yaml`

Regenerate plain output:

```bash
./target/release/openapi-php generate --input examples/petstore/openapi.yaml --output examples/petstore/output --namespace 'App\Petstore' --framework plain
```

Regenerate Laravel output:

```bash
./target/release/openapi-php generate --input examples/petstore/openapi.yaml --output examples/petstore/output-laravel --namespace 'App\Petstore' --framework laravel
```
