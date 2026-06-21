# Branch 1 — Accounts & PDAs

Start here, always. On Solana **everything is an account**, and the program does not choose
which accounts it receives — the *caller* does. Every account your program trusts is an
account an attacker can try to substitute. This branch closes the substitution surface.

> Load this branch first. It gates the others: if the account model is wrong, nothing built
> on top of it is safe.

---

## 1.1 — Owner check before deserialization `CRITICAL`

**Check:** Every account whose *data* you read is verified to be owned by the expected
program before you trust its contents.

**Why:** An attacker can create an account with arbitrary bytes under a program they control
and pass it where you expect your own state. If you deserialize without checking `owner`, you
deserialize attacker-controlled data as trusted state. This is the classic *account
substitution* / *owner-check* bug.

**Question:**
> "For each account you deserialize — is the owner verified? In Anchor, `Account<'info, T>`
> checks owner + discriminator for you, but `AccountInfo` / `UncheckedAccount` does **not**.
> I see `<X>` typed as `<AccountInfo / UncheckedAccount>` at `<file:line>` — is its owner
> checked manually?
> → Recommended: use `Account<'info, T>` (or `InterfaceAccount` for tokens) wherever you
> read data. Only use `UncheckedAccount` with an explicit `owner =`/`address =` constraint and
> a `/// CHECK:` justifying it."

**Verify in code:** `Grep "AccountInfo<'info>"`, `"UncheckedAccount"`, `"#[account("` —
flag any data-bearing account not using a typed `Account`/`InterfaceAccount` wrapper.

---

## 1.2 — Canonical PDA bump `HIGH`

**Check:** PDAs use the **canonical** bump, and the program never accepts a **user-supplied
bump argument**. In Anchor, a bare `bump` constraint re-derives the canonical bump for you
(safe); `bump = state.bump` reuses a stored canonical bump (also safe, and cheaper).

**Why:** For one seed set there are multiple valid bumps. The danger is accepting the bump as
*instruction input* (`bump = some_arg`) — an attacker supplies a different valid `(seed, bump)`
→ a different address → an account you didn't intend to trust. Re-deriving the canonical bump
(bare `bump`) is **not** a vulnerability; storing the bump is a **compute optimization** (~1.5k
CU saved by skipping `find_program_address`), not a security requirement.

**Question:**
> "How is the bump for `<PDA>` obtained — Anchor's bare `bump` (re-derives canonical), a stored
> `bump = state.bump`, or is it passed in as an instruction argument?
> → Recommended: use bare `bump` (or a stored canonical bump on hot paths). **Never** accept a
> bump as an instruction argument."

**Verify in code:** `Grep "bump"` — the red flag is a `bump: u8` in the **instruction args** /
`bump = <arg>`. A bare `bump` or `bump = <state>.bump` is fine; a *missing stored* bump is a CU
note, not a vuln.

---

## 1.3 — PDA seed uniqueness & no cross-domain sharing `HIGH`

**Check:** Seeds uniquely identify the account's role, and the same PDA is **not** reused as
the authority/signer across unrelated domains.

**Why:** *PDA sharing* — using one PDA (e.g. a single global authority) for multiple roles —
lets a caller use access in one context to act in another, because the program will
`invoke_signed` for that PDA regardless of which role is intended. Overlapping or
attacker-influenced seeds also let two logical accounts collide.

**Question:**
> "Walk me through your seeds. Does each account type have seeds that pin it to exactly one
> owner/market/role? And is any single PDA used as the signing authority for more than one
> kind of operation?
> → Recommended: include the owning entity in the seeds (`["vault", market.key()]`,
> `["position", market.key(), user.key()]`); give each authority domain its own PDA."

**Verify in code:** list every `seeds = [...]`; check user-controlled seed components can't be
chosen to collide, and that one authority PDA isn't `invoke_signed` across distinct domains.

---

## 1.4 — Account relationships (`has_one` / address constraints) `HIGH`

**Check:** When account A must belong to / reference account B, that link is enforced
(`has_one`, `address =`, or an explicit `require_keys_eq!`).

**Why:** Without it, a caller passes *their* legitimately-owned account A alongside *someone
else's* B. Each account is individually valid; the **relationship** is the thing that's
unchecked. This is how "withdraw from vault X using position from vault Y" attacks work.

**Question:**
> "Your `<Position>` has a `vault` field and the instruction also takes a `vault` account —
> are they constrained to be the same? I don't see a `has_one = vault`.
> → Recommended: `#[account(has_one = vault)]` on the position, and `has_one = owner` /
> `has_one = authority` for every ownership link."

**Verify in code:** for each struct field that is a `Pubkey` of a related account, confirm a
matching `has_one`/constraint exists in the `#[derive(Accounts)]`.

---

## 1.5 — Duplicate mutable accounts `MEDIUM`

**Check:** Instructions that take two accounts of the same type which must differ (e.g.
`from` and `to`) reject the case where the caller passes the same account for both.

**Why:** Passing the same account as source and destination can let a transfer credit and
debit the same balance, or bypass a check that assumed two distinct accounts.

**Question:**
> "In `<transfer/settle>` you take `from` and `to` of the same type — what happens if they're
> the same account?
> → Recommended: Anchor 1.0 **rejects duplicate mutable accounts by default** (runtime error);
> intentional duplicates must opt in with the `dup` constraint (`#[account(mut, dup)]`). Still
> add an explicit `constraint = from.key() != to.key()` for non-mut / cross-type cases and for
> any pre-1.0 program."

**Verify in code:** find instructions with ≥2 same-type writable accounts; on Anchor 1.0 confirm
no unintended `dup`; on older Anchor confirm an explicit distinctness constraint.

---

## 1.6 — `init` vs `init_if_needed` re-init surface `HIGH`

**Check:** Account creation uses `init` (fails if it already exists). `init_if_needed` is used
only when truly needed and the handler treats an already-initialized account safely.

**Why:** `init_if_needed` silently runs your init logic again on an existing account → a
*re-initialization attack* that can reset authority/state. (It also requires explicitly
enabling the feature, which is a signal to scrutinize it.)

**Question:**
> "I see `init_if_needed` on `<account>`. Is re-running init on an existing account safe — does
> it reset any authority or balance field?
> → Recommended: use plain `init` unless idempotent creation is genuinely required; if you
> keep `init_if_needed`, guard fields so a second call can't overwrite ownership/state."

**Verify in code:** `Grep "init_if_needed"` — for each, check the handler doesn't blindly
re-assign authority/critical fields.

---

## Branch exit

Before leaving branch 1 you should be able to fill the **account model** and **PDA map**
sections of `design-spec.md`:

- Every account: type, owner, who can create it, who can write it.
- Every PDA: seeds, canonical bump storage, signing authority, role (one role only).
- Every cross-account relationship: how it's enforced.

Open findings here are almost always CRITICAL/HIGH — resolve them before moving on.

---

**Sources:** the primary references behind every check in this branch are listed in [SOURCES.md](../SOURCES.md). Do not assert a claim that isn't grounded there (or in a newer official source) — flag it instead.
