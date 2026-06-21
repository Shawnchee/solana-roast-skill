# Branch 2 — Authority & Signers

Authentication failures are the **#1 cause of Solana exploits**. The Wormhole hack (~$320M,
Feb 2022) is the canonical example: the bridge failed to verify that the account it used for
signature verification was the *genuine* "instructions" sysvar, so a spoofed account let the
attacker forge the guardians' signature check. The lesson spans this branch and branch 1 —
verify *who signed* **and** that the accounts behind every check are the real ones. This branch
makes sure every privileged action is authorized by the right, actually-signing party.

> Load this for any instruction that mutates state, moves value, or changes config.

---

## 2.1 — Signer is actually required `CRITICAL`

**Check:** Every authority/admin/owner account that gates a privileged instruction is a
`Signer` (Anchor) or has `is_signer == true` verified (native). Checking the *pubkey* alone
is not enough — anyone can pass any pubkey.

**Why:** Checking a pubkey without verifying it *signed* lets an attacker pass the real
authority's pubkey (public information) without its signature and be treated as authorized —
the textbook missing-signer bug. (Wormhole was a cousin of this: a missing check on the
*account used to verify signatures*; see the branch intro.)

**Question:**
> "`<instruction>`'s authority is typed `<AccountInfo / UncheckedAccount>` at `<file:line>` —
> nothing forces it to sign. Is that intended?
> → Recommended: type it `Signer<'info>` (Anchor) or assert `account.is_signer` (native) on
> **every** privileged account. Comparing keys without a signature check is the missing-signer bug."

**Verify in code:** `Grep "AccountInfo", "UncheckedAccount", "is_signer"` and cross-reference
every authority used in a `require_keys_eq!` / `has_one` — it must *also* be a signer.

---

## 2.2 — Authority is bound to the resource `CRITICAL`

**Check:** The signer is verified to be *the* authority for *this* resource, via `has_one`
(Anchor) or `require_keys_eq!(state.authority, signer.key())` (native) — not just *a* valid
signer.

**Why:** Requiring *a* signature without binding it to the resource lets any user sign for
*someone else's* account. The signature is real; the authorization is wrong.

**Question:**
> "On `<instruction>`, the signer signs — but is it checked against `<state>.authority`? I see a
> `Signer` but no `has_one = authority`.
> → Recommended: `#[account(mut, has_one = authority)]` on the state + `authority: Signer`.
> One without the other is a bypass."

**Verify in code:** for each privileged instruction, confirm both (a) signer present and
(b) signer == the resource's stored authority.

---

## 2.3 — Privilege separation across instructions `HIGH`

**Check:** Admin/config instructions (set fee, set authority, pause, withdraw protocol funds)
require the *admin/global* authority, while user instructions require the *user's* authority —
they are not collapsed into one check.

**Why:** A single "is signer" check reused across both user and admin paths lets a user invoke
admin paths. Roles must be distinct.

**Question:**
> "Which authority gates each instruction — global admin vs. per-user owner? Map them for me.
> → Recommended: separate `admin` (on the global config PDA) from `owner` (on the user's
> account); never let a user-owned signer authorize a global-config change."

**Verify in code:** group instructions by which authority they check; flag any admin action
gated only by a user-level signer.

---

## 2.4 — Authority transfer is two-step `MEDIUM`

**Check:** Changing an authority/admin uses a propose→accept (two-step) flow, or at minimum
validates the new authority is non-default and not the zero/`Pubkey::default()` address.

**Why:** A one-step `set_authority(new)` to a typo'd or zero address permanently bricks
control of the account/program. Two-step transfer (new authority must accept) prevents
fat-finger and hostile lock-out.

**Question:**
> "How is authority handed over? One-step or propose/accept?
> → Recommended: two-step (`pending_authority` set by current admin, then `accept_authority`
> signed by the new one). Reject `Pubkey::default()` either way."

**Verify in code:** `Grep "authority ="` in handlers; look for a single assignment with no
acceptance step.

---

## 2.5 — Instruction-level access matrix `HIGH`

**Check:** There exists a clear, complete mapping of *instruction → who may call it → what
they may touch*. No instruction is "anyone can call and it's fine" unless that's deliberate
and safe.

**Why:** Most authorization bugs are *omissions* — an instruction nobody remembered to gate.
An explicit matrix surfaces the gap that ad-hoc review misses.

**Question (drives the design-spec):**
> "Let's build the access matrix. For each instruction: caller, required signer, accounts it
> mutates, and the invariant that must hold after. I'll fill what the code shows and ask you
> for the rest."
> → Recommended: anything that mutates global state or moves value is admin- or owner-gated by
> default; justify every public/un-gated instruction explicitly.

**Verify in code:** enumerate `pub fn` in the `#[program]` mod; for each, record the signer
and mutated accounts into the access matrix in `design-spec.md`.

---

## Branch exit

You should now have a complete **authority model** for `design-spec.md`: every instruction's
caller, required signer, and resource binding. Any instruction where a privileged action is
reachable without the correct *signing* authority is CRITICAL — do not let it leave this
branch unresolved.
