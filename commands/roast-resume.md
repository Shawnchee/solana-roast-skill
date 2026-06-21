---
name: roast-resume
description: Resume a paused solana-roast interrogation from its saved session ledger.
argument-hint: "[optional: branch to jump to]"
---

Resume an in-progress **solana-roast** interrogation.

1. Read `.solana-roast/session.md` (the findings ledger + decisions so far). If it doesn't
   exist, tell the user there's no saved session and offer to start a fresh `/roast`.
2. Summarize where things stand: branches completed, open findings by severity, and the next
   branch/question.
3. If `$ARGUMENTS` names a branch, jump there; otherwise continue from the next unresolved
   question in dependency order.
4. Continue the interrogation per the `solana-roast` skill — one question at a time, always with
   a recommended answer — updating the ledger as you go.
5. When all applicable branches are resolved, (re)emit `design-spec.md`, `threat-model.md`, and
   `pre-audit-checklist.md`, then give the hand-off.

Re-read the program first in case it changed since the last session, and reconcile any findings
that the user has already fixed (mark them resolved in the ledger).
