---
name: roast
description: Roast the design & security of a Solana program before audit — one question at a time, with recommendations.
argument-hint: "[path to program or program description]"
---

Run the **solana-roast** design & security interrogation on the target Solana program.

Target: `$ARGUMENTS` (a path to a program/workspace, or a short description. If empty, look for
an Anchor workspace in the current directory: `Anchor.toml`, `programs/*/src/lib.rs`.)

Follow the `solana-roast` skill exactly:

1. **Scope** — Read the program first. Identify which of the 8 branches apply. State the plan
   and rough question count, then ask: **"Full roast, or CRITICAL/HIGH only?"**
2. **Interrogate** — Walk branches in dependency order (accounts/PDAs → authority → CPI → state
   → economic → governance → compute → tokens). One question at a time. Explore the code to
   self-answer; only ask the human what the code can't tell you. Always end each question with a
   recommended answer and a one-line why.
3. **Triage** — Maintain the findings ledger (severity + decision). Persist to
   `.solana-roast/session.md`.
4. **Emit** — Write `design-spec.md`, `threat-model.md`, and `pre-audit-checklist.md` into
   `.solana-roast/`. Summarize severity counts and give the honest bottom line.
5. **Hand off** — Route to the scanner/audit/formal-verification/deploy skills.

Load skill files progressively — read `skill/interrogation-protocol.md` first, then only the
branch file you're currently working through. Do not read all branches up front.

Remember: you reduce design risk and produce a triaged checklist. This is **not** an audit —
state that plainly at the end.
