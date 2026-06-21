# Branch 7 — Compute, Limits & Denial of Service

Solana's hard limits (1.4M compute units, 1,232-byte transactions, account size caps) turn
"works on devnet with 3 items" into "permanently stuck at scale". This branch finds the places
where growth, loops, or an attacker can make an instruction un-runnable — locking funds or
bricking the program.

> Load this when the program has loops, lists, per-user growth, or anything that scales with
> input/usage.

---

## 7.1 — Unbounded iteration `HIGH`

**Check:** No instruction loops over a collection whose size a user/attacker can grow without
bound. Every loop has a provable upper bound that fits in the compute budget.

**Why:** If processing requires iterating all positions/holders/orders and that list grows, you
eventually exceed 1.4M CU and the instruction **can never succeed** — funds tied to it are
locked. An attacker can intentionally bloat the list to trigger this (*griefing DoS*).

**Question:**
> "Any instruction iterate over a `Vec` / list of accounts whose length grows with usage? What's
> the max iterations, and the CU at that max?
> → Recommended: cap the iterable, or redesign to O(1) per instruction (process one item per
> tx, use a crank/pagination, or per-item PDAs instead of one big list)."

**Verify in code:** `Grep "for ", ".iter()", "remaining_accounts", "Vec<"` inside handlers;
estimate worst-case CU.

---

## 7.2 — State that only grows `HIGH`

**Check:** Accounts and collections have a retirement/close path; they don't grow forever until
they hit the account-size cap (10 MB) or become too expensive to touch.

**Why:** An ever-growing account eventually can't be read/written within limits — the same lock
as 7.1, by accretion. Per-user data crammed into one global account is the classic mistake.

**Question:**
> "Does any account accumulate entries indefinitely? How does old data get removed?
> → Recommended: one PDA per item (closeable) instead of a growing global list; add explicit
> cleanup/close instructions that reclaim rent."

**Verify in code:** find global accounts with `Vec`/append patterns and no removal path.

---

## 7.3 — Transaction size & account count `MEDIUM`

**Check:** The instructions a user must send fit in the 1,232-byte transaction limit — or you
use Versioned (v0) transactions + Address Lookup Tables deliberately.

**Why:** A legacy transaction tops out around **32–35 accounts** in practice (byte-limited, not
a protocol cap; the hard cap is **256** accounts via the u8 account index). A flow that needs
many accounts can't be built as a legacy tx; v0 + ALTs raise the reachable count toward 256, but
must be designed in, not bolted on.

**Question:**
> "What's the largest number of accounts a single instruction needs? Approaching ~32?
> → Recommended: keep core instructions well under the byte limit; if a flow needs many
> accounts, design for v0 transactions + Address Lookup Tables from the start."

**Verify in code:** count accounts per `#[derive(Accounts)]`; flag large ones.

---

## 7.4 — Compute budget headroom `MEDIUM`

**Check:** Heavy instructions request an explicit compute limit and have headroom; they don't
silently rely on the 200k default and break when logic grows.

**Why:** The default is 200k CU per instruction; a complex DeFi op can need 400k+. Without
`setComputeUnitLimit`, it fails. Excess logging (`msg!`) also burns CU.

**Question:**
> "Do heavy instructions set a compute limit client-side, and have you measured peak CU?
> → Recommended: measure worst-case CU, request limit with headroom via the Compute Budget
> program, and strip noisy `msg!` logs in hot paths."

**Verify in code:** look for `ComputeBudgetProgram.setComputeUnitLimit` in clients; `Grep "msg!"`
in hot handlers.

---

## 7.5 — Rent-griefing & account-creation DoS `MEDIUM`

**Check:** An attacker can't force *you* (a PDA/protocol) to pay rent for accounts they spawn,
and can't exhaust a bounded resource (e.g. fixed-size order slots) to lock out real users.

**Why:** If the protocol pays rent for caller-created accounts, an attacker spams creation to
drain the treasury. Fixed slot tables can be squatted.

**Question:**
> "Who pays rent for newly created accounts — the caller or a protocol PDA? Any fixed-size slot
> table a spammer could fill?
> → Recommended: the **caller** always funds their own accounts' rent; make squattable
> resources reclaimable or economically costly to hold."

**Verify in code:** check `payer = ` on `init` constraints; confirm it's the user, not a PDA.

---

## Branch exit

`threat-model.md` records the scaling limits: max iterations per instruction (with CU estimate),
any unbounded growth, the tx-size/account-count posture, and who pays rent. A fund-locking
unbounded loop is HIGH→CRITICAL because it can permanently trap value with no recovery.
