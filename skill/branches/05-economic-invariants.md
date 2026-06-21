# Branch 5 — Economic Invariants

Where the money logic lives. This is where 8-figure DeFi exploits happen — not from a missing
signer but from math that rounds the wrong way, an invariant nobody wrote down, or an oracle
that lied. This branch forces you to *state your invariants* and prove the code enforces them.

> Load this whenever the program moves, mints, or accounts for value. Skip the heavy parts
> (oracle, peg) for non-financial programs, but still do overflow checks on any counter.

---

## 5.1 — Name the invariants `HIGH`

**Check:** You can state, in one sentence each, the properties that must **always** hold —
and the code enforces each one explicitly.

**Why:** You can't protect what you haven't named. "Total deposits == sum of user balances",
"shares minted == f(assets in)", "no withdraw exceeds collateral" — these are the things an
exploit breaks. Writing them down is half the defense and is exactly what the formal-verification
hand-off (QEDGen) needs.

**Question (drives the threat-model):**
> "What must *always* be true about your accounting? Give me the conservation laws. I'll
> phrase them as invariants and we'll check each is enforced on-chain.
> → Recommended starting set: value conservation (in == out + fees), supply integrity
> (shares ↔ assets monotonic), and no-negative-balance. List yours."

**Verify in code:** for each stated invariant, locate the `require!`/`checked_*` that enforces
it. An invariant with no enforcing line is a finding.

---

## 5.2 — Integer overflow / underflow `CRITICAL`

**Check:** All arithmetic on value-bearing quantities uses `checked_*` / `saturating_*` (or the
program is built with `overflow-checks = true` in `Cargo.toml` for release). No bare `+ - *` on
balances; no silent `as` truncation.

**Why:** In release builds Rust **wraps** on overflow by default — a subtraction that underflows
gives a huge `u64`, turning "you have 0, withdraw 1" into "you now have 18 quintillion".

**Question:**
> "Is `overflow-checks = true` set for release in `Cargo.toml`, and do balance computations use
> `checked_*`? I see bare arithmetic at `<file:line>`.
> → Recommended: set `overflow-checks = true` **and** use `checked_*` on value math (defense in
> depth). Avoid `as u64` casts that can truncate `u128` intermediates."

**Verify in code:** check `[profile.release] overflow-checks`; `Grep "checked_", " as u", "+", "-", "*"`
in math on balances.

---

## 5.3 — Rounding direction `HIGH`

**Check:** Every division/rounding step rounds in the **protocol's favor**, consistently, and
the direction is deliberate (deposits round shares *down*, debts round *up*).

**Why:** Rounding the wrong way leaks value every transaction; attackers amplify it with many
tiny operations (dust attacks, share-inflation). "It's just 1 lamport" becomes a drain at scale.

**Question:**
> "For each ratio (shares↔assets, fees, interest) — which way does it round, and is that toward
> the protocol or the user?
> → Recommended: round so the protocol never loses (mint fewer shares, charge more debt); be
> consistent and document each direction."

**Verify in code:** find every division; confirm rounding direction and order-of-operations
(multiply before divide to preserve precision).

---

## 5.4 — First-depositor / share-inflation attack `HIGH`

**Check:** Vault/pool share math resists the classic first-depositor inflation attack (attacker
deposits 1 base unit, donates a large amount directly to the vault, then later depositors'
shares round to zero).

**Why:** A well-known vault drain. Mitigations: mint dead shares on init, use virtual
offsets/shares, or enforce a minimum deposit.

**Question:**
> "How are shares priced for the very first deposit, and what stops a 1-unit deposit + direct
> donation from inflating share price?
> → Recommended: seed initial liquidity / mint locked dead shares, or use virtual-shares
> accounting (ERC4626-style offset adapted to your token math)."

**Verify in code:** inspect the init/first-deposit branch of share minting.

---

## 5.5 — Oracle / price input integrity `CRITICAL` (if used)

**Check:** External prices (Pyth, Switchboard, AMM TWAP) are validated for **staleness** and
**confidence**, and spot prices aren't used where they can be flash-manipulated.

**Why:** Stale or manipulable prices cause bad liquidations and under-collateralized loans —
a top DeFi exploit category. A single un-checked price read is a CRITICAL.

**Question:**
> "Where do prices come from, and how do you bound staleness + confidence? Any use of a spot AMM
> price that a flash loan could move?
> → Recommended: Pyth is now a **pull** oracle (the price update is posted via Hermes in the
> same tx) — consume it with `pyth-solana-receiver-sdk` and
> `get_price_no_older_than(clock, max_age, feed_id)`, and reject on a wide `conf` (confidence)
> band. Switchboard is now **Switchboard On-Demand** (pull feeds) — apply the same
> staleness/confidence checks. Prefer a TWAP over spot for anything liquidation-relevant."

**Verify in code:** `Grep "pyth", "Switchboard", "get_price", "price", "oracle", "publish_time", "conf"`;
flag any price used without a staleness + confidence guard.

---

## 5.6 — Slippage / MEV / sandwich surface `MEDIUM`

**Check:** Swaps and price-sensitive actions take a user-supplied `min_out` / `max_in` and
enforce it; deadlines are considered.

**Why:** Without slippage bounds, searchers sandwich every trade. The program can't stop MEV
but must let users bound their loss.

**Question:**
> "Do value-moving instructions accept and enforce a slippage limit (`min_amount_out`)?
> → Recommended: require `min_out`/`max_in` on every swap-like instruction and `require!` the
> realized amount satisfies it."

**Verify in code:** find swap/trade instructions; confirm a min/max bound argument is enforced.

---

## 5.7 — Fee & accounting consistency `MEDIUM`

**Check:** Fees are computed once, can't exceed 100%, are bounded by admin limits, and the
accounting after fees still satisfies the invariants in 5.1.

**Question:**
> "How are fees bounded and where do they accrue? Can an admin set a 100% fee and seize funds?
> → Recommended: cap fee rate in code (e.g. ≤ some max bps), accrue to a dedicated PDA, and keep
> fee math inside the conservation invariant."

**Verify in code:** locate fee math; confirm bounds and accrual destination.

---

## Branch exit

`threat-model.md` should now list each named invariant, the line that enforces it, and any
gap. Overflow on value math and unchecked oracles are CRITICAL. The invariant list you produce
is the direct input to the formal-verification hand-off.
