# Branch 8 — Tokens (SPL & Token-2022)

Anything touching tokens inherits a second security surface: mint authorities, account
spoofing, decimals, and — with Token-2022 — programmable behaviors (transfer hooks, fees,
freezing) that can break assumptions your program silently makes. This branch hardens token
handling.

> Load this when the program creates, holds, or moves tokens. Resolve **SPL vs Token-2022
> first** — it changes every later question.

---

## 8.0 — SPL Token vs Token-2022 (gating question) `HIGH`

**Check:** You know which token program(s) your program must support, and your account types
match (`Token` vs `Token2022`, `Account` vs `InterfaceAccount`).

**Why:** Token-2022 mints are owned by a *different* program ID and carry extensions that change
transfer semantics. A program hard-coded to SPL Token will reject or mishandle Token-2022 mints;
one that accepts both must handle the extensions safely.

**Question:**
> "Do you support classic SPL Token, Token-2022, or both? Today your accounts are typed `<...>`.
> → Recommended: if you only need SPL, pin the Token program ID and reject others. If you
> support Token-2022, use `InterfaceAccount<'info, Mint/TokenAccount>` + `TokenInterface` and
> explicitly decide which extensions you allow."

**Verify in code:** `Grep "anchor_spl", "Token2022", "TokenInterface", "InterfaceAccount", "token::", "token_2022::"`.

---

## 8.1 — Token account ownership & mint match `CRITICAL`

**Check:** Every token account you read/credit is verified to (a) be owned by the expected
authority and (b) hold the expected mint — via `token::mint =`, `token::authority =`, or the
associated-token constraints.

**Why:** Otherwise a caller passes a token account of the *wrong mint* or one they don't own,
and your accounting credits/debits the wrong asset — a direct theft vector (the token-account
equivalent of account substitution).

**Question:**
> "For each token account, are mint and authority constrained? I see `<token_account>` without a
> `token::mint =` at `<file:line>`.
> → Recommended: `#[account(associated_token::mint = mint, associated_token::authority = user)]`
> for ATAs, or explicit `token::mint`/`token::authority` constraints for non-ATAs."

**Verify in code:** for each `TokenAccount`/`InterfaceAccount`, confirm mint+authority constraints.

---

## 8.2 — Mint & freeze authority of *your* token `HIGH`

**Check:** If your program issues a token, the disposition of **mint authority** and **freeze
authority** is deliberate and trust-appropriate.

**Why:** A retained mint authority means unlimited supply can be minted (reads as a rug; breaks
fixed-supply guarantees). A retained freeze authority means you can freeze any holder's tokens —
a censorship/trust liability. Holders cannot verify your intentions, only the on-chain authority.

**Question:**
> "For the token your program controls: who holds mint authority and freeze authority, and what's
> the end state?
> → Recommended: revoke mint authority once supply is final (or hold it in a multisig with a
> public emission schedule); set freeze authority to `None` unless you have a regulated,
> documented reason to keep it. For **Token-2022** mints, also audit the extension set: a
> **permanent delegate** can transfer/burn from *any* holder account indefinitely (clawback)
> and **cannot** be revoked like mint/freeze authority — a rug vector equal to or worse than
> freeze; **default account state** can freeze new holders by default; **pausable** can halt all
> transfers."

**Verify:** check mint creation / `set_authority` calls and, for Token-2022, the enabled
extensions; for live mints, inspect on-chain via the DAS/RPC.

---

## 8.3 — Transfer fees & fee-on-transfer (Token-2022) `HIGH`

**Check:** If you accept Token-2022 mints with the transfer-fee extension, accounting uses the
**actual received amount** (balance delta), not the requested amount.

**Why:** With a transfer fee, less arrives than was sent. Crediting the requested amount
over-credits the user — drainable. (Cross-references branch 3.5.)

**Question:**
> "Could any accepted mint have a transfer fee? Do you credit the requested amount or the real
> delta received?
> → Recommended: snapshot the destination balance before/after and credit the difference; or
> disallow fee-bearing mints explicitly."

**Verify in code:** look for `amount`-based crediting around transfers without a delta read.

---

## 8.4 — Transfer hooks (Token-2022) `HIGH`

**Check:** If you allow mints with the transfer-hook extension, you understand a hook runs
arbitrary program code on every transfer — treat token transfers as potential CPI/reentrancy
points (branch 3.4).

**Why:** A transfer-hook mint runs mint-defined code on every transfer. Token-2022 makes the
passed accounts read-only and drops the sender's signer privilege during the hook (limiting
classic reentrancy), but the hook can still CPI out and read your not-yet-updated state, and an
attacker can invoke *your* hook from *their* hook with spoofed mint/accounts. Programs that
assume "a token transfer is inert" are wrong under Token-2022. (Note: a mint can't have both
transfer-hook and confidential-transfer — they're mutually exclusive.)

**Question:**
> "Do you accept transfer-hook mints? If so, is your state updated before transfers, and do you
> trust the hook program?
> → Recommended: for value-critical flows, either disallow transfer-hook mints (allow-list
> known-safe mints) or apply strict checks-effects-interactions ordering."

**Verify in code:** confirm whether the design allow-lists mints or accepts arbitrary ones.

---

## 8.5 — Decimals & amount units `MEDIUM`

**Check:** Amounts respect each mint's `decimals`; you never mix raw and UI amounts, and
cross-mint math normalizes decimals.

**Why:** Treating a 6-decimal and a 9-decimal token as comparable raw integers misprices by
1000×. Use the `*_checked` transfer variants: in **Token-2022** the plain `Transfer` is
**deprecated** in favor of `TransferChecked`; in classic SPL Token `transfer` still works but
`transfer_checked` (which verifies decimals + mint) is preferred.

**Question:**
> "Do you assume fixed decimals anywhere, or read each mint's `decimals`? Using `transfer` or
> `transfer_checked`?
> → Recommended: read `decimals` from the mint, normalize in cross-mint math, and use
> `transfer_checked`/`*_checked` variants."

**Verify in code:** `Grep "decimals", "transfer(", "transfer_checked"`.

---

## 8.6 — ATA creation & existence `LOW`

**Check:** Destination ATAs are created (or required to exist) safely, with the caller funding
rent, and you don't assume an ATA exists when it might not.

**Question:**
> "Do you create destination ATAs (`init_if_needed` / `associated_token`) or assume they exist?
> Who pays?
> → Recommended: use the associated-token constraints with the **caller** as payer; handle the
> not-yet-created case explicitly."

**Verify in code:** `Grep "associated_token", "init_if_needed"` on token accounts; confirm payer.

---

## Branch exit

`design-spec.md` records the token model: which token program(s) supported, the mint/freeze
authority plan for issued tokens, and the extension policy (fees/hooks allowed or not).
Unconstrained token accounts (8.1) are CRITICAL; retained mint authority and fee/hook
mishandling are HIGH.

---

**Sources:** the primary references behind every check in this branch are listed in [SOURCES.md](../../SOURCES.md). Do not assert a claim that isn't grounded there (or in a newer official source) — flag it instead.
