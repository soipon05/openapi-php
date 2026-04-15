#!/usr/bin/env bash
# Validate commit message format.
# Called by lefthook as: validate-commit-msg.sh <message-file>

set -euo pipefail

MSG_FILE="${1}"
MSG=$(cat "$MSG_FILE")
SUBJECT=$(echo "$MSG" | head -1)

# ── 1. Conventional Commits prefix ────────────────────────────────────────────
VALID_PREFIXES="^(feat|fix|refactor|test|docs|chore|perf|build|ci)(\(.+\))?!?: .+"
if ! echo "$SUBJECT" | grep -qE "$VALID_PREFIXES"; then
  echo "❌ commit-msg: subject must start with a Conventional Commits prefix."
  echo "   Valid: feat|fix|refactor|test|docs|chore|perf|build|ci"
  echo "   Got: ${SUBJECT}"
  exit 1
fi

# ── 2. Subject length ≤ 72 chars ──────────────────────────────────────────────
LEN=${#SUBJECT}
if [ "$LEN" -gt 72 ]; then
  echo "❌ commit-msg: subject too long (${LEN} chars, max 72)."
  echo "   Got: ${SUBJECT}"
  exit 1
fi

# ── 3. No trailing period ─────────────────────────────────────────────────────
if echo "$SUBJECT" | grep -qE "\.$"; then
  echo "❌ commit-msg: subject must not end with a period."
  exit 1
fi

# ── 4. No internal tracking references ───────────────────────────────────────
# Reject: BUG-1, ISSUE-3, BUG-1~4 style references (internal shorthand)
if echo "$SUBJECT" | grep -qiE "(BUG|ISSUE)-[0-9]"; then
  echo "❌ commit-msg: do not use internal bug/issue numbers in commit subjects."
  echo "   Describe *what* was fixed instead."
  echo "   Got: ${SUBJECT}"
  exit 1
fi

# ── 5. No reviewer/tool mentions ─────────────────────────────────────────────
# Tools used during development are not part of what changed.
if echo "$SUBJECT" | grep -qiE "\b(codex|copilot|difit|chatgpt|claude)\b"; then
  echo "❌ commit-msg: do not mention review tools in commit subjects."
  echo "   Describe *what* was changed, not how it was found."
  echo "   Got: ${SUBJECT}"
  exit 1
fi

echo "✅ commit-msg: OK"
