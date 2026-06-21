#!/usr/bin/env bash
# solana-roast — installer (defaults). Installs the skill, agent, and commands for Claude Code.
# Safe and transparent: it only copies this repo's files into your ~/.claude directory.
# For a menu (project-local install, Codex support), run ./install-custom.sh instead.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_HOME="${CLAUDE_HOME:-$HOME/.claude}"

SKILL_SRC="$SCRIPT_DIR/skill"
AGENTS_SRC="$SCRIPT_DIR/agents"
COMMANDS_SRC="$SCRIPT_DIR/commands"
RULES_SRC="$SCRIPT_DIR/rules"

SKILL_DEST="$CLAUDE_HOME/skills/solana-roast"
AGENTS_DEST="$CLAUDE_HOME/agents"
COMMANDS_DEST="$CLAUDE_HOME/commands"

echo "solana-roast installer"
echo "  source: $SCRIPT_DIR"
echo "  target: $CLAUDE_HOME"
echo ""

if [ ! -f "$SKILL_SRC/SKILL.md" ]; then
  echo "ERROR: $SKILL_SRC/SKILL.md not found. Run this from the repo root." >&2
  exit 1
fi

mkdir -p "$SKILL_DEST" "$AGENTS_DEST" "$COMMANDS_DEST"

echo "→ Installing skill to $SKILL_DEST"
cp -R "$SKILL_SRC/." "$SKILL_DEST/"
# bundle the rules alongside the skill so they travel with it
mkdir -p "$SKILL_DEST/rules"
cp -R "$RULES_SRC/." "$SKILL_DEST/rules/"

echo "→ Installing agent to $AGENTS_DEST"
cp "$AGENTS_SRC/"*.md "$AGENTS_DEST/"

echo "→ Installing commands to $COMMANDS_DEST"
cp "$COMMANDS_SRC/"*.md "$COMMANDS_DEST/"

echo ""
echo "✅ Installed solana-roast."
echo ""
echo "Use it by saying:  \"roast my Solana program\"  /  \"review my program design\""
echo "Or run the command: /roast [path-to-program]    (resume with /roast-resume)"
echo ""
echo "The skill loads progressively — only the branch in play is read into context."
