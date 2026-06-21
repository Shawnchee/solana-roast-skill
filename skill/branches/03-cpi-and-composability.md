# Branch 3 — CPI & Composability

Cross-Program Invocations are how Solana composes — and how trust leaks. When your program
calls another, you extend your transaction's authority into code you may not control. This
branch makes sure you only call what you mean to, and only lend your PDA's signature where
it's safe.

> Load this when the program contains `invoke`, `invoke_signed`, or `CpiContext`.

---

## 3.1 — Target program ID is verified `CRITICAL`

**Check:** The program you CPI into is pinned to the expected ID — via Anchor's
`Program<'info, Token>` / typed CPI, or an explicit `require_keys_eq!(program.key(), expected)`
in native.

**Why:** *Arbitrary CPI* — if the "token program" is just an `AccountInfo` the caller supplies,
an attacker passes their own program at that slot and your `invoke` runs **their** code with
your accounts (and any propagated signatures). Pin the ID.

**Question:**
> "The program you invoke for `<token transfer / swap>` — is it a typed `Program<...>` or a
> caller-supplied `AccountInfo` at `<file:line>`?
> → Recommended: use the typed program account (`Program<'info, Token>`,
> `Interface<'info, TokenInterface>`) or assert the program ID. Never CPI into an unverified
> program account."

**Verify in code:** `Grep "invoke", "AccountInfo"` near CPIs; confirm the program account is
typed or its key is checked.

---

## 3.2 — PDA signing scope is minimal `HIGH`

**Check:** When you `invoke_signed` with a PDA, the PDA you sign with authorizes *only* the
operation intended, and the seeds passed match exactly that authority's domain.

**Why:** If one PDA is the signing authority for many things (see branch 1.3 PDA sharing),
lending its signature in a CPI can authorize more than you meant. Over-broad signing authority
is how a "transfer from vault" CPI becomes "drain any vault this PDA controls."

**Question:**
> "The vault-authority PDA you `invoke_signed` with — does it sign for exactly one vault, or is
> it a global authority over many?
> → Recommended: scope the signing PDA to the specific resource (seeds include the market/vault
> key) so its signature can't be reused across resources."

**Verify in code:** inspect the `signer_seeds` in each `invoke_signed`; confirm the authority is
resource-scoped.

---

## 3.3 — Account & signer/writable propagation is intended `HIGH`

**Check:** You understand that signer and writable privileges **propagate** into CPIs, and you
don't forward a user's signature/write access to a callee that could abuse it.

**Why:** If account A signed the outer tx, it is still a signer inside your CPIs; if it's
writable, the callee can write it. Forwarding the wrong account into an untrusted CPI hands
the callee privileges over it.

**Question:**
> "For each CPI, list the accounts you forward and whether each is signer/writable downstream.
> Any user account being forwarded into a less-trusted program?
> → Recommended: forward the minimum set; never forward a user's signing authority into a
> program you don't fully trust."

**Verify in code:** match the accounts vector of each CPI against what the callee needs;
flag superfluous signer/writable forwards.

---

## 3.4 — State-mutation ordering vs. external calls (reentrancy posture) `MEDIUM`

**Check:** State is updated **before** external CPIs that could re-enter, or the design is
provably safe under re-entry. (Solana's shallow CPI depth limit — historically 4, being raised
to 8 by SIMD-0268 with Agave 3.x — and lack of EVM-style fallback make classic reentrancy
rarer, but it is **not** eliminated: **read-only reentrancy** and stale-state-during-callback
bugs exist, and Token-2022 **transfer hooks** are now a common CPI-back surface. During a
transfer hook, Token-2022 makes the passed accounts read-only and drops the sender's signer
privilege — which blocks classic write/signature reentrancy — but the hook can still CPI out
and read your not-yet-updated state, and an attacker can invoke your hook from theirs with
spoofed mint/accounts.)

**Why:** If you transfer out, then update the balance, a program you called (or a transfer hook)
can observe or act on stale state mid-instruction. Checks-effects-interactions still applies on
Solana — and matters *more* as the depth limit rises and hooks proliferate.

**Question:**
> "In `<instruction>`, do you mutate your state before or after the external CPI? Could the
> callee read your state mid-call?
> → Recommended: apply state changes (debit, mark-consumed) **before** the CPI; treat any
> token program with transfer hooks (Token-2022) as potentially re-entrant."

**Verify in code:** for each handler with a CPI, check the order of state writes vs. the
`invoke`/`invoke_signed`.

---

## 3.5 — Return-value / result handling `MEDIUM`

**Check:** You don't blindly trust a CPI's *effects* without verifying them when it matters
(e.g. read the token account balance *after* a transfer rather than assuming `amount` moved,
especially with fee-on-transfer / Token-2022).

**Why:** With transfer fees or hooks, the amount that *arrives* differs from the amount
*requested*. Crediting the requested amount over-credits the user.

**Question:**
> "After a token transfer in, do you credit the requested amount or the *actual* delta?
> → Recommended: snapshot balance before/after and credit the real delta when the mint could
> have transfer fees (see branch 8)."

**Verify in code:** look for `amount`-based accounting around token CPIs without a
balance-delta read.

---

## Branch exit

For `design-spec.md`, record: every external program called (with pinned ID), every PDA used
as a CPI signer (with its scoped seeds), and the state-mutation ordering guarantee. Unverified
target program IDs are CRITICAL.

---

**Sources:** the primary references behind every check in this branch are listed in [SOURCES.md](../../SOURCES.md). Do not assert a claim that isn't grounded there (or in a newer official source) — flag it instead.
