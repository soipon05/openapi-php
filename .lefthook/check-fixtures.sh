#!/usr/bin/env bash
# Verify that every fixture file referenced in test code actually exists on disk.
# Prevents CI failures caused by forgetting to git-add a new fixture.

set -euo pipefail

TESTS_DIR="tests"
FIXTURES_DIR="tests/fixtures"
missing=0

# Extract all strings passed to fixture("...") across all test files
referenced=$(grep -rh -oE 'fixture\("[^"]+"\)' "$TESTS_DIR"/*.rs 2>/dev/null \
  | grep -oE '"[^"]+"' | tr -d '"' | sort -u)

for name in $referenced; do
  if [ ! -f "${FIXTURES_DIR}/${name}" ]; then
    echo "❌ check-fixtures: missing fixture file: ${FIXTURES_DIR}/${name}"
    missing=1
  fi
done

if [ "$missing" -eq 0 ]; then
  echo "✅ check-fixtures: all referenced fixtures exist"
fi

exit "$missing"
