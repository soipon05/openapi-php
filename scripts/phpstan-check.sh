#!/usr/bin/env bash
# phpstan-check.sh — run PHPStan level 9 + strict-rules against generated PHP.
#
# Usage:
#   ./scripts/phpstan-check.sh DIR [DIR ...]
#
# Environment variables:
#   PHPSTAN_USE_DOCKER=1  Run PHPStan inside a docker container (for hosts
#                         without a local PHP + composer toolchain).
#
# Each DIR must contain one or more openapi-php-generated PHP trees (typically
# laid out as Client/, Models/, Exceptions/). Directories are analysed together
# inside a composer-backed PHPStan workspace.
#
# Exit code: 0 = clean, non-zero = PHPStan found issues or setup failed.

set -euo pipefail

if [ $# -eq 0 ]; then
    echo "usage: $0 DIR [DIR ...]" >&2
    exit 2
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE="$SCRIPT_DIR/.phpstan-workspace"

mkdir -p "$WORKSPACE/src"
cp "$SCRIPT_DIR/composer.json" "$WORKSPACE/composer.json"
cp "$SCRIPT_DIR/phpstan.neon" "$WORKSPACE/phpstan.neon"

# Install PHPStan once per CI run. Cached by composer.lock if present.
if [ ! -d "$WORKSPACE/vendor" ]; then
    if [ -n "${PHPSTAN_USE_DOCKER:-}" ]; then
        docker run --rm -v "$WORKSPACE":/app -w /app composer:latest \
            install --no-progress --no-interaction
    else
        ( cd "$WORKSPACE" && composer install --no-progress --no-interaction )
    fi
fi

# Wipe and populate src/ with each generated directory under its own subtree
# so classes in separate trees don't shadow each other.
find "$WORKSPACE/src" -mindepth 1 -delete

idx=0
for d in "$@"; do
    if [ ! -d "$d" ]; then
        echo "error: not a directory: $d" >&2
        exit 2
    fi
    idx=$((idx + 1))
    slug="tree${idx}"
    mkdir -p "$WORKSPACE/src/$slug"
    cp -R "$d"/* "$WORKSPACE/src/$slug/" 2>/dev/null || true
done

if [ -n "${PHPSTAN_USE_DOCKER:-}" ]; then
    docker run --rm -v "$WORKSPACE":/app -w /app php:8.3-cli \
        php vendor/bin/phpstan analyse --no-progress
else
    ( cd "$WORKSPACE" && php vendor/bin/phpstan analyse --no-progress )
fi
