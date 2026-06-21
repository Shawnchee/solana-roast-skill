# Branch 4 — State & Data

How your program stores, identifies, resizes, and closes account data. The bugs here are
subtle: an account that *looks* like the right type but isn't, an account that gets
re-initialized, or one that's closed but revived. This branch hardens the data layer.

> Load this when the program defines and stores its own account state.

---

## 4.1 — Type discrimination (no "type cosplay") `HIGH`

**Check:** Two account types with the same byte layout can't be confused for one another.
Anchor's account discriminator (8 bytes by **default**, customizable since Anchor 0.31) handles
this when you use `Account<'info, T>`; native programs need an explicit type tag.

**Why:** *Type cosplay* — if `User` and `Admin` serialize to the same shape and you read with a
raw deserializer, an attacker passes a `User` account where `Admin` is expected. The
discriminator is what prevents this; bypassing it (raw `try_from_slice` on `AccountInfo.data`)
reopens the hole.

**Question:**
> "Do you ever deserialize account data manually (`try_from_slice` on raw bytes) instead of via
> `Account<'info, T>`? Any two account types with identical fields?
> → Recommended: always go through the typed wrapper so the discriminator is checked; in native,
> prefix every account with a unique type/version byte and verify it."

**Verify in code:** `Grep "try_from_slice", "deserialize", "AccountInfo"` — flag manual
deserialization of program-owned data.

---

## 4.2 — Re-initialization safety `HIGH`

**Check:** An already-initialized account cannot be initialized again to reset its state
(ties to branch 1.6 `init_if_needed`, viewed from the data side).

**Why:** A re-init resets authority/balances/flags to defaults — an attacker re-inits a vault
to make themselves the authority. Even without `init_if_needed`, a hand-rolled init that
doesn't check an `is_initialized` flag is vulnerable.

**Question:**
> "Can `initialize` be called twice on the same account? Is there an `is_initialized` guard or
> does `init` (which fails on existing accounts) protect you?
> → Recommended: use Anchor `init` (fails if exists). For native/idempotent paths, set and
> check an `is_initialized: bool` / `version: u8`."

**Verify in code:** find init handlers; confirm they fail on an already-existing/initialized
account.

---

## 4.3 — Account closing & revival `HIGH`

**Check:** Closing an account zeroes its data, transfers all lamports out (so it's below
rent-exempt and gets garbage-collected), and sets the discriminator to the closed sentinel —
so it can't be "revived" within the same transaction and reused as valid state.

**Why:** A naive close that just moves lamports leaves the data intact for the rest of the tx;
an attacker tops it back up to rent-exempt and reuses the stale, now-"valid" account
(*closing/revival attack*). Anchor's `close = ` does this correctly; manual closes often don't.

**Question:**
> "How do you close accounts — Anchor `close = recipient`, or manual? After close, is the data
> zeroed and the discriminator invalidated?
> → Recommended: use Anchor's `#[account(mut, close = recipient)]` — it drains lamports, zeroes
> the data, and writes the closed-account sentinel discriminator that blocks revival. Do **not**
> hand-roll closes: the old `CLOSED_ACCOUNT_DISCRIMINATOR` constant is no longer exported by
> `anchor-lang`, and a manual close of a typed `Account<T>` is unsafe (the `mut` exit re-writes
> the discriminator). In native code, drain lamports, zero the data, **and** reassign the
> account to the System Program."

**Verify in code:** `Grep "close"` and manual lamport-draining patterns; prefer the `close =`
constraint over any hand-rolled close.

---

## 4.4 — `realloc` safety `MEDIUM`

**Check:** Account resizing validates the new size, zero-initializes new bytes when needed,
charges the right rent, and can't be driven to an attacker-chosen size that corrupts adjacent
interpretation.

**Why:** `realloc` with `realloc::zero = false` leaves stale bytes that deserialize as
attacker-influenced data; growing without funding rent fails or, done wrong, mis-accounts.

**Question:**
> "Do you `realloc` any accounts? Are new bytes zeroed and is rent topped up by the right payer?
> → Recommended: `realloc::zero = true` unless you have a specific reason; bound the max size;
> charge rent to the caller, not a PDA."

**Verify in code:** `Grep "realloc"` — check `realloc::zero` and payer.

---

## 4.5 — Account space & rent are correct `LOW`

**Check:** Allocated `space` matches the serialized struct (the discriminator — 8 bytes by
default — plus fields, with `Vec`/`String` bounded), and accounts are rent-exempt.

**Why:** Under-allocation panics on write or truncates state; unbounded `Vec`/`String` lets a
caller grow an account without limit (also a DoS vector — branch 7).

**Question:**
> "Is `space` computed with `InitSpace`/explicitly, and are all `Vec`/`String` fields
> `#[max_len(...)]` bounded?
> → Recommended: derive `InitSpace`, bound every dynamic field, and never store unbounded
> collections in a single account — use one-account-per-item PDAs instead."

**Verify in code:** `Grep "space =", "InitSpace", "max_len", "Vec<", "String"` — flag unbounded
collections.

---

## 4.6 — Sensitive data isn't trusted from the client `MEDIUM`

**Check:** Values the program must trust (balances, prices, totals) are read from program-owned
state or verified on-chain — never accepted as instruction arguments without validation.

**Why:** Anything in `instruction_data` is attacker-chosen. Accepting a `current_balance: u64`
argument and trusting it is a free exploit.

**Question:**
> "Which values come in as instruction args vs. read from on-chain state? Any trusted number
> taken as an argument?
> → Recommended: derive trusted values from accounts you own; use args only for intents
> (amounts, ids) and re-validate them against state."

**Verify in code:** review each instruction's args; flag any that are trusted without
on-chain cross-check.

---

## Branch exit

For `design-spec.md`, the state layer should be fully specified: each account's fields, size,
type tag/discriminator, init guard, and close semantics. Re-init and revival findings are
HIGH — they reset authority and are favorites of attackers.
