#!/usr/bin/env bash
# bench-vs-competitors.sh — compare openapi-php generation speed against
# jane-php and OpenAPI Generator on the same synthetic 100-endpoint spec.
#
# Dependencies:
#   - hyperfine (mise install hyperfine)
#   - docker (for jane-php + openapi-generator — avoids JVM / PHP setup)
#   - ./target/release/openapi-php (cargo build --release)
#
# Output: prints a results table to stdout. Also dumps raw timings to
#         target/bench-results.md for pasting into BENCHMARKS.md.

set -euo pipefail

cd "$(dirname -- "${BASH_SOURCE[0]}")/.."

SPEC="tests/fixtures/large_api.yaml"
BIN="./target/release/openapi-php"

if [ ! -f "$BIN" ]; then
    echo "building release binary..."
    cargo build --release
fi

# Ensure the bench spec exists. It is committed so numbers are reproducible.
if [ ! -f "$SPEC" ]; then
    echo "bench spec missing — regenerating via scripts/generate-bench-spec.py"
    python3 scripts/generate-bench-spec.py 20 > "$SPEC"
fi

# ── Prepare competitor scratch dirs ────────────────────────────────────────
OPENAPI_PHP_OUT=/tmp/bench-openapi-php
OPENAPIGEN_OUT=/tmp/bench-openapigen
JANE_DIR=/tmp/bench-jane

mkdir -p "$OPENAPI_PHP_OUT" "$OPENAPIGEN_OUT" "$JANE_DIR/out"

# jane-php workspace (composer install cached between runs)
if [ ! -d "$JANE_DIR/vendor" ]; then
    cat > "$JANE_DIR/composer.json" <<'EOF'
{
  "require": {
    "jane-php/open-api-3": "^7.11",
    "jane-php/open-api-runtime": "^7.11"
  },
  "minimum-stability": "stable"
}
EOF
    cat > "$JANE_DIR/.jane-openapi" <<'EOF'
<?php
return [
    'openapi-file' => __DIR__ . '/large_api.yaml',
    'namespace' => 'App\\Bench',
    'directory' => __DIR__ . '/out',
];
EOF
    cp "$SPEC" "$JANE_DIR/large_api.yaml"
    docker run --rm -v "$JANE_DIR":/app -w /app composer:latest \
        install --no-progress --no-interaction
else
    cp "$SPEC" "$JANE_DIR/large_api.yaml"
fi

# ── Warmups + pulls so first run isn't skewed by image download ────────────
docker pull -q openapitools/openapi-generator-cli:latest > /dev/null || true
docker pull -q php:8.4-cli > /dev/null || true

# ── Run benchmarks ─────────────────────────────────────────────────────────
# Combine all three commands into a single hyperfine run so the markdown
# export is a single comparison table. Warmup and runs apply to every command;
# the slow competitors (jane, openapi-generator) dominate the wall-clock time
# so we keep --runs conservative to avoid a 10-minute benchmark.
mkdir -p target
hyperfine \
    --export-markdown target/bench-results.md \
    --warmup 2 --runs 10 \
    --command-name "openapi-php (rust native)" \
    --prepare "find $OPENAPI_PHP_OUT -type f -delete 2>/dev/null || true" \
    "$BIN generate --input $SPEC --output $OPENAPI_PHP_OUT --namespace App\\\\Bench" \
    --command-name "jane-php (php 8.4 via docker)" \
    --prepare "find $JANE_DIR/out -type f -delete 2>/dev/null || true" \
    "docker run --rm -v $JANE_DIR:/app -w /app php:8.4-cli php vendor/bin/jane-openapi generate" \
    --command-name "openapi-generator (java via docker)" \
    --prepare "find $OPENAPIGEN_OUT -type f -delete 2>/dev/null || true" \
    "docker run --rm -v $(pwd):/local -v $OPENAPIGEN_OUT:/out openapitools/openapi-generator-cli:latest generate -i /local/$SPEC -g php -o /out"

echo
echo "Results written to target/bench-results.md"
