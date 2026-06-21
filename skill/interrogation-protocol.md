# Interrogation Protocol

How to actually run the roast. Read this before asking the first question. These rules are
what separate a useful interrogation from a generic checklist read-back.

---

## The prime directive: explore before you ask

If there is **any** code — an Anchor program, a `#[derive(Accounts)]` struct, a native
`process_instruction`, even a half-written lib.rs — **read it first**. Then for every
candidate question, ask yourself: *can the code answer this?*

- **Code answers it** → state the answer and confirm. "Your `withdraw` takes `authority` but
  it's `AccountInfo`, not `Signer` — so it's never required to sign. That's the bug; confirm
  you want me to treat this as CRITICAL?" (Do **not** ask "does authority sign?")
- **Code is silent / it's a human decision** → ask the human. "Should the config be a
  singleton PDA, or one-per-deployer? Recommended: singleton `["config"]` unless you need
  multi-tenant."

How to explore quickly:

```
- Find programs:        Glob "**/programs/**/src/lib.rs", "**/src/lib.rs", "**/Anchor.toml"
- Find account structs: Grep "#\[derive\(Accounts\)\]", "#\[account"
- Find instructions:    Grep "pub fn " inside the #[program] mod
- Find CPIs:            Grep "invoke", "invoke_signed", "CpiContext"
- Find authority/PDA:   Grep "Signer", "has_one", "seeds", "bump", "AccountInfo"
- Find math:            Grep "checked_", " + ", " - ", " * ", "as u64", "as u128"
- Find token usage:     Grep "TokenAccount", "Mint", "token::", "anchor_spl", "Token2022"
```

If `Anchor.toml` / `Cargo.toml` exists, note the Anchor and `anchor-spl` versions — security
defaults differ by version (e.g. `init_if_needed`, custom discriminators since 0.31, and Anchor
1.0's default rejection of duplicate mutable accounts). The current stack is **Anchor 1.0.x on
the Solana 3.x toolchain** (the TS client package is `@anchor-lang/core`). For tests, the current
tools are **LiteSVM** (fast unit), **Mollusk** (instruction-level), and **Surfpool** (integration
/ mainnet-fork) — `solana-bankrun` is deprecated.

---

## The five rules of every question

1. **One at a time.** Ask, wait, resolve, then the next. A wall of questions makes the user
   skim; the point is to walk one branch of the decision tree to the bottom.
2. **Recommend.** End every question with `→ Recommended: <answer> — <one-line why>`. The
   user should be able to answer "yep" and keep moving. You are a consultant with an opinion,
   not a form.
3. **Ground it.** Quote the file + line or the exact construct. "In `state.rs:42`, `Vault`
   has no `bump` field…" beats "do you store your bump?".
4. **Explain the blast radius, briefly.** One sentence on what goes wrong if ignored, tied to
   a real exploit class when one fits ("this is the Wormhole bug class").
5. **Severity-tag the outcome.** When a question resolves into a finding, tag it
   `CRITICAL / HIGH / MEDIUM / LOW` (see scale below) and write it to the ledger.

---

## Dependency ordering

Some answers gate whole branches. Ask gating questions first so you don't waste the user's
time down a branch that doesn't apply.

```
upgradeable?  ── no ──→ skip most of branch 6, but flag "immutable = bugs are permanent"
              └─ yes ─→ full governance branch (who holds authority, multisig, timelock)

moves value?  ── no ──→ branch 5 is light (still check overflow on any counters)
              └─ yes ─→ full economic-invariant branch

uses tokens?  ── no ──→ skip branch 8
              └─ yes ─→ SPL vs Token-2022 first (it changes every later token question)

calls other programs? ── no ──→ skip branch 3
                      └─ yes ─→ full CPI branch
```

Within a branch, resolve the CRITICAL checks before the MEDIUM ones — if the account model
is broken, the rounding question can wait.

---

## Severity scale

Use this consistently so the threat-model triages itself.

| Severity | Meaning | Examples |
|----------|---------|----------|
| **CRITICAL** | Direct loss of funds / full bypass, trivially reachable | Missing signer on a withdraw; arbitrary CPI; unverified account owner on a state account |
| **HIGH** | Loss/lock of funds or auth bypass under realistic conditions | Non-canonical bump accepted; re-init attack; oracle with no staleness check |
| **MEDIUM** | Exploitable with constraints, or degraded safety | Unbounded loop that *can* be kept small; rounding that leaks dust; single-key admin on a small program |
| **LOW** | Hygiene / defense-in-depth | Missing event logs; unused mutable; non-blocking naming |
| **INFO** | Design note, not a vuln | "Consider Token-2022 transfer fees if you later add them" |

When unsure between two levels, pick the higher one and say why — under-rating design risk is
the failure mode that costs money.

---

## The findings ledger (your working memory)

Keep this updated in your head / scratchpad through the session, and persist it to
`.solana-roast/session.md` so the roast can resume. One row per resolved question:

```
ID    | Branch          | Severity | Finding                                  | Decision                         | Status
F-01  | authority       | CRITICAL | withdraw authority is AccountInfo        | switch to Signer + has_one        | accepted
F-02  | pdas            | HIGH     | bump not stored, re-derived each ix      | store canonical bump in state     | accepted
F-03  | economic        | MEDIUM   | fee math rounds down, leaks 1 lamport    | round in protocol's favor         | deferred
```

`Status` is one of: `accepted` (will fix), `deferred` (acknowledged, fixing later),
`wontfix` (user accepts the risk — record their rationale), `n/a` (didn't apply).

---

## Opening the session

Don't start roasting cold. Open with scope so the user knows the shape:

> "I read your program (`programs/vault/src/`). It's an Anchor vault that moves SPL tokens
> and is upgradeable. I'll roast you across 6 of the 8 branches — accounts/PDAs, authority,
> CPI, state, economic invariants, and governance — skipping the token-2022 and pure-compute
> ones since they don't apply. Roughly 18 questions. I already spotted 2 likely CRITICALs
> while reading. Full roast, or CRITICAL/HIGH only?"

Then go one question at a time.

## Closing the session

When every applicable branch is resolved:

1. Summarize the ledger: counts by severity.
2. Write the three artifacts (`templates/`) into `.solana-roast/`.
3. State the honest bottom line: "Design risk is materially reduced. This is **not** an
   audit — here's your hand-off." Then route per SKILL.md's hand-off section.
