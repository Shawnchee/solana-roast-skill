# Roast transcript — `vulnerable-vault`

> Illustrative session showing how `solana-roast` runs: it reads the code first, asks one
> question at a time, grounds each in a real line, and recommends an answer. Abridged to the
> CRITICAL/HIGH findings; MEDIUM/LOW omitted for brevity.

---

**roast:** I read `programs/vault/src/lib.rs` (Anchor 1.0.x). It's a single-PDA SPL-token vault
with `initialize` / `deposit` / `withdraw`, and it's upgradeable. Applicable branches: 1 (accounts
/PDAs), 2 (authority), 3 (CPI), 4 (state), 5 (economic), 6 (governance), 8 (tokens). Branch 7
(compute/DoS) is light — no loops or growing collections.

I already spotted **5 likely CRITICALs** while reading. ~16 questions. Full roast, or
CRITICAL/HIGH only?

**you:** full

---

**roast:** `[Q3/16 · Branch 2 Authority · CRITICAL]`
`withdraw.authority` is `UncheckedAccount<'info>` (`lib.rs`, `Withdraw` struct) — it is neither a
`Signer` nor checked against `vault.authority`. So **anyone** can call `withdraw` and drain the
vault; the `authority` account is decorative. This is the #1 Solana exploit class (missing signer).
→ **Recommended:** type it `authority: Signer<'info>` **and** add `#[account(has_one = authority)]`
to `vault`. Accept?

**you:** yes

**roast:** Logged **F-01 (CRITICAL, accepted)**. Note this resolves two checks at once — 2.1
(signer required) and 2.2 (signer bound to resource).

---

**roast:** `[Q5/16 · Branch 3 CPI · CRITICAL]`
`withdraw.token_program` is `UncheckedAccount` and you `CpiContext::new_with_signer(... token_program
.to_account_info() ...)`. An attacker can pass **their own program** at that slot — your vault PDA
then signs a CPI into attacker code (arbitrary CPI / drain).
→ **Recommended:** type it `token_program: Program<'info, Token>` (or `Interface<'info, TokenInterface>`
if you want Token-2022 too). Accept?

**you:** yes — and I do want Token-2022 support

**roast:** Logged **F-02 (CRITICAL, accepted)** → use `Interface<'info, TokenInterface>` +
`InterfaceAccount` for the token accounts. That opens branch 8 questions (fees/hooks) — we'll hit
those. Flagging now: with Token-2022 you must credit the **actual** delta on deposit, not the
requested amount (transfer-fee mints).

---

**roast:** `[Q6/16 · Branch 8 Tokens · CRITICAL]`
None of your token accounts are constrained. `deposit.vault_token_account` and both `withdraw`
token accounts are bare `Account<'info, TokenAccount>` with no `token::mint` / `token::authority`.
A caller can substitute a token account of the wrong mint or one they control.
→ **Recommended:** constrain the vault ATA with `token::mint = mint, token::authority = vault` (the
vault PDA), and the user/destination accounts with the expected mint. Accept?

**you:** yes

**roast:** Logged **F-03 (CRITICAL, accepted)**.

---

**roast:** `[Q8/16 · Branch 5 Economic · CRITICAL]`
`deposit` does `vault.total_deposited = vault.total_deposited + amount` and `withdraw` does `... -
amount` — both unchecked. Your `Cargo.toml` has `overflow-checks = false` (the release default),
so the subtraction **underflows to a huge u64** ("withdraw more than you hold" succeeds in
accounting) and the add can wrap.
→ **Recommended:** set `overflow-checks = true` in `[profile.release]` **and** use
`checked_add` / `checked_sub` with an explicit `require!` (defense in depth). Also add a balance
check: you can't withdraw more than `total_deposited`. Accept?

**you:** yes

**roast:** Logged **F-04 (CRITICAL, accepted)**. Naming the invariant for the threat-model:
*"`total_deposited` equals the sum of net deposits and never exceeds the vault token balance."*

---

**roast:** `[Q9/16 · Branch 1 Accounts/PDAs · HIGH]`
`initialize(vault_bump: u8)` stores a **user-supplied** bump (`vault.bump = vault_bump`). A caller
can pass a non-canonical-but-valid bump. (To be clear: re-deriving the canonical bump via bare
`bump` is fine — the problem is taking it as *input*.)
→ **Recommended:** drop the argument; use Anchor's bare `bump` at init and `bump = vault.bump`
afterwards (Anchor stores the canonical bump for you). Accept?

**you:** yes

**roast:** Logged **F-05 (HIGH, accepted)**.

---

**roast:** `[Q12/16 · Branch 6 Governance · HIGH]`
The program is upgradeable and I don't see a documented upgrade-authority plan. On mainnet, a
single hot upgrade key can replace the whole program — it dominates your risk profile.
→ **Recommended:** hold the upgrade authority in a **Squads Protocol v4** multisig behind a
timelock; publish the policy; plan a path to immutability or DAO control. Accept?

**you:** yes, Squads multisig planned

**roast:** Logged **F-06 (HIGH, accepted)**.

---

**roast:** Done. Applicable branches resolved. Tally: **5 CRITICAL, 2 HIGH** (F-07 is the
overflow-checks process flaw, grouped with F-04). Writing artifacts to `.solana-roast/`:
`design-spec.md`, `threat-model.md`, `pre-audit-checklist.md`.

**Bottom line:** design risk is now materially reduced — but this is **not** an audit. Next:
run the Trail of Bits `solana-vulnerability-scanner` on the fixed code, then `review-and-iterate`
for a deep pass, formal-verify the `total_deposited` invariant, devnet-test, then `deploy-to-mainnet`.
