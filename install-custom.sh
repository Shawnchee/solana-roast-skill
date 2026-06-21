#!/usr/bin/env bash
# solana-roast — custom installer with a menu.
# Choose: install scope (personal ~/.claude vs project ./.claude) and agent target (Claude / Codex).
# Transparent: it only copies this repo's files. No network calls, no downloads.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

SKILL_SRC="$SCRIPT_DIR/skill"
AGENTS_SRC="$SCRIPT_DIR/agents"
COMMANDS_SRC="$SCRIPT_DIR/commands"
RULES_SRC="$SCRIPT_DIR/rules"

if [ ! -f "$SKILL_SRC/SKILL.md" ]; then
  echo "ERROR: $SKILL_SRC/SKILL.md not found. Run this from the repo root." >&2
  exit 1
fi

echo "solana-roast — custom install"
echo ""
echo "1) Install scope:"
echo "   [1] Personal  (\$HOME/.claude)         — available in every project"
echo "   [2] Project   (./.claude)             — only this repo"
read -r -p "Choose 1 or 2 [1]: " SCOPE
SCOPE="${SCOPE:-1}"

if [ "$SCOPE" = "2" ]; then
  BASE="$(pwd)/.claude"
else
  BASE="$HOME/.claude"
fi

echo ""
echo "2) Also install the agent definition for Codex (\$HOME/.codex)? [y/N]"
read -r -p "> " CODEX
CODEX="${CODEX:-n}"

SKILL_DEST="$BASE/skills/solana-roast"
AGENTS_DEST="$BASE/agents"
COMMANDS_DEST="$BASE/commands"

echo ""
echo "Installing to: $BASE"
mkdir -p "$SKILL_DEST" "$AGENTS_DEST" "$COMMANDS_DEST"

echo "→ skill   → $SKILL_DEST"
cp -R "$SKILL_SRC/." "$SKILL_DEST/"
mkdir -p "$SKILL_DEST/rules"
cp -R "$RULES_SRC/." "$SKILL_DEST/rules/"

echo "→ agent   → $AGENTS_DEST"
cp "$AGENTS_SRC/"*.md "$AGENTS_DEST/"

echo "→ commands→ $COMMANDS_DEST"
cp "$COMMANDS_SRC/"*.md "$COMMANDS_DEST/"

if [[ "$CODEX" =~ ^[Yy]$ ]]; then
  CODEX_SKILL_DEST="$HOME/.codex/skills/solana-roast"
  mkdir -p "$CODEX_SKILL_DEST"
  echo "→ codex skill → $CODEX_SKILL_DEST"
  cp -R "$SKILL_SRC/." "$CODEX_SKILL_DEST/"
  mkdir -p "$CODEX_SKILL_DEST/rules"
  cp -R "$RULES_SRC/." "$CODEX_SKILL_DEST/rules/"
fi

echo ""
echo "✅ Done."
echo "Trigger it with: \"roast my Solana program\"  or  /roast [path]"
