# solana-roast 🔥

**Adversarial pre-ship interrogation for Solana programs.** Before you write the last line, and
long before you book an audit, `solana-roast` reads your Anchor/native program and roasts your
*design* — one question at a time, with a recommended answer for each — across the eight places
Solana programs actually get exploited.

> Inspired by the [`grill-me`](https://github.com/mattpocock/skills) interview skill, but
> Solana-native: it knows the SVM account model, reads your code, and is grounded in real
> exploit classes (Wormhole's ~$320M account-confusion bug, missing signer checks, user-supplied
> PDA bumps, share-inflation, unrevoked authorities, …) — current to the **Anchor 1.0 / Solana
> 3.x** stack.

> **Not to be confused with:** `roast-my-product` (critiques the *product* — business/UX) or
> `review-and-iterate` / Trail of Bits scanner (review *finished code* for vulnerabilities).
> `solana-roast` interrogates the *program design* **before the code is finished**. Three
> different gates, three different stages.

---

## The problem it solves

Look at the Solana tooling timeline:

```
idea validation  →  scaffold/build  →  [ ❓ THE GAP ❓ ]  →  audit / scanner  →  mainnet
```

There are great tools for validating an *idea* and great tools for auditing *finished code*.
But the most expensive Solana mistakes are made **in between** — at the design/architecture
stage — and they're invisible until an auditor finds them written into the code, where they're
expensive to undo.

- **Idea validators** (`validate-idea`, `find-next-crypto-idea`) are too early — they're about
  the business, not the program.
- **Auditors & scanners** (Trail of Bits' `solana-vulnerability-scanner`, the kit's
  `review-and-iterate`) are too late — they need finished code, and they're a gate, not a guide.
- **Generic interview skills** (`grill-me`, requirements interviewers) have **zero** Solana
  awareness — they can't tell you your `withdraw` authority never signs.

`solana-roast` fills the gap: a **Solana-aware design interrogator** that runs *before* the
auditor is useful, catches design-stage flaws while they're cheap to fix, and produces the exact
artifacts the auditors want as input.

## What it does

1. **Reads your program first.** It explores the codebase (`Anchor.toml`, `programs/*/src`,
   `#[derive(Accounts)]`, CPIs, math, token usage) and answers what it can from the code — so it
   never wastes your time asking what the code already shows.
2. **Roasts you, one question at a time**, down 9 branches in dependency order:

   | # | Branch | Catches |
   |---|--------|---------|
   | 1 | Accounts & PDAs | account substitution, missing owner check, user-supplied bump, PDA sharing |
   | 2 | Authority & Signers | missing `is_signer`, authority confusion, missing `has_one` |
   | 3 | CPI & Composability | arbitrary CPI, unverified program ID, signer propagation, reentrancy |
   | 4 | State & Data | type cosplay, re-init attacks, close/revival, realloc safety |
   | 5 | Economic Invariants | overflow, rounding leaks, share-inflation, oracle/slippage abuse |
   | 6 | Upgrade & Governance | live upgrade key, single-key admin, multisig/timelock gaps |
   | 7 | Compute & DoS | unbounded loops, fund-locking growth, tx-size limits, rent griefing |
   | 8 | Tokens (SPL/Token-2022) | mint/freeze authority, ATA spoofing, transfer fees & hooks, permanent delegate |
   | 9 | Client & Integration | blind signing, SIWS replay/phishing, RPC trust, relayer keys, durable-nonce abuse |

   Branches 1–8 roast the on-chain program; branch 9 adds the Solana-specific client↔chain seam
   (generic web2/infra → handed off to `cso`). Every question ends with a **recommended answer**.
3. **Triages findings** by severity into a running ledger, and gives a headline **Design Safety
   score** (`X/10`) so progress is legible — caveated as *design* risk, not an audit pass.
4. **Emits artifacts** into `.solana-roast/`:
   - `design-spec.md` — the resolved design (account model, PDA map, access matrix, invariants).
   - `threat-model.md` — every finding, severity, decision, residual risk, the score.
   - `pre-audit-checklist.md` — a checked-off list an external auditor can pick up cold.
   - `lecture.md` *(optional, `/roast-lecture`)* — **teaches** each finding: what breaks if
     unfixed + the real exploit it mirrors (e.g. *"that's the Cashio bug, $52M"*), so your team
     learns, not just patches. Grounded in a verified [exploit library](skill/exploit-library.md).
5. **Hands off** to the right next gate (scanner → audit → formal verification → devnet → mainnet).

It is honest about its scope: it **reduces design risk and produces a triaged checklist** — it is
**not** a substitute for a professional audit, and it says so.

## Example

```
You: roast my program

roast: I read programs/vault/src/. It's an upgradeable Anchor vault moving SPL tokens.
       I'll roast 6 of 8 branches (~18 Qs). I already spotted 2 likely CRITICALs.
       Full roast, or CRITICAL/HIGH only?

You: full

roast: [Q4/18 · Authority · CRITICAL]
       `withdraw` takes `authority: UncheckedAccount` (programs/vault/src/lib.rs) and never
       checks it signed — anyone can call withdraw and drain the vault. Missing signer checks
       are the #1 Solana exploit class.
       → Recommended: type it `Signer<'info>` + `#[account(has_one = authority)]`. Apply? [y/n/explain]
```

> A full worked example — the intentionally-vulnerable program above plus the complete roast
> output it produces — lives in [`examples/vulnerable-vault/`](examples/vulnerable-vault/).

## Install

### Option A — clone & run the installer (recommended)

```bash
git clone https://github.com/Shawnchee/solana-roast-skill
cd solana-roast-skill
./install.sh                 # personal install to ~/.claude
# or:
./install-custom.sh          # menu: project-local install, Codex support
```

### Option B — skills.sh

```bash
npx skills add https://github.com/Shawnchee/solana-roast-skill
```

### Option C — as a submodule in the Solana AI Kit

```bash
git submodule add https://github.com/Shawnchee/solana-roast-skill .claude/skills/ext/solana-roast
```

The installer copies the skill to `~/.claude/skills/solana-roast/`, the agent to
`~/.claude/agents/`, and the commands to `~/.claude/commands/`. No network calls, no downloads —
it only copies this repo's files.

## Usage

- **Natural language:** "roast my Solana program", "review my program design", "threat model
  this", "am I missing any signer checks", "is this safe to deploy".
- **Command:** `/roast [path-to-program]` — start. `/roast-resume` — continue a saved session.
  `/roast-lecture` — turn the findings into a teaching lecture (what breaks + the real exploit).
- **Agent:** delegate a full self-driving review to the `solana-design-interrogator` agent.

## Structure

```
solana-roast-skill/
├── skill/
│   ├── SKILL.md                       # entry/router — progressive loading
│   ├── interrogation-protocol.md      # how to run the roast
│   ├── branches/01..09-*.md           # the 9 decision-tree branches
│   ├── exploit-library.md             # verified real Solana hacks, mapped to branches
│   ├── SOURCES.md                     # per-branch primary sources (no-guessing policy)
│   └── templates/                     # design-spec / threat-model / pre-audit-checklist / lecture
├── agents/solana-design-interrogator.md, openai.yaml
├── commands/roast.md, roast-resume.md, roast-lecture.md
├── rules/interrogation-rules.md
├── examples/vulnerable-vault/         # intentionally-vulnerable demo + sample roast output
├── install.sh, install-custom.sh
├── LICENSE (MIT)
└── README.md
```

The `SKILL.md` is a thin router; branch files load **only when that branch is in play**, keeping
context small (token-efficient progressive loading).

## How it fits the Solana AI Kit

`solana-roast` is the **design gate before the audit gate**. Its `pre-audit-checklist.md` is
designed to be the direct input to:
- Trail of Bits `solana-vulnerability-scanner` — pattern scan
  (`/plugin marketplace add trailofbits/skills`).
- `review-and-iterate` (ships in the kit) — deep audit. (A dedicated `solana-auditor` skill is a
  proposed kit *seed*; use it if/when it's published.)
- QEDGen's Lean-4 skills (`npx skills add qedgen/solana-skills`) — formal verification of the
  invariants it surfaces.
- `deploy-to-mainnet` — once the checklist is green and devnet-tested.

It **complements** rather than duplicates the existing security skills: it produces the input
they consume, at the design stage they can't reach.

## Sources & grounding

Every check traces to a primary, authoritative source. The full per-branch mapping lives in
**[SOURCES.md](skill/SOURCES.md)** (and each branch file links to it); the skill is instructed not to
assert anything it can't ground there or in a newer official source. Headline references:
- [Helius — A Hitchhiker's Guide to Solana Program Security](https://www.helius.dev/blog/a-hitchhikers-guide-to-solana-program-security)
- [Zealynx — Solana Security Guide: 45 Exploit Checks](https://www.zealynx.io/blogs/solana-security-checklist)
- [Solana Program Security Checklist: 14 Critical Checks Before Mainnet](https://dev.to/ohmygod/solana-program-security-checklist-14-critical-checks-before-you-deploy-to-mainnet-2d66)
- [Solana docs — accounts, PDAs, CPI](https://solana.com/docs/core), [Anchor Book](https://book.anchor-lang.com)

## License

MIT — see [LICENSE](LICENSE). Built to be merged or submoduled into the
[Solana AI Kit](https://github.com/solanabr/solana-ai-kit).
