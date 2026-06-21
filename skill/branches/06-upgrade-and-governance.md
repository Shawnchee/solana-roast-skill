# Branch 6 — Upgrade Authority & Governance

The keys that control the program *are* the program's security model. A perfect program with a
single hot upgrade key is one phishing email from a rug. This branch interrogates who can
change what, and how that power is constrained. **Run it before every mainnet deploy.**

> Load this always before launch — even for "simple" programs.

---

## 6.1 — Upgrade authority: who holds it `CRITICAL`

**Check:** You know exactly who holds the program's upgrade authority, it is **not** a single
hot wallet for anything holding real value, and the plan for it is deliberate.

**Why:** The upgrade authority can replace your entire program with arbitrary code — instant,
total compromise if that key leaks. This is the single highest-leverage key in your system.

**Question:**
> "Who holds the upgrade authority today, and who will hold it on mainnet? A dev hot wallet, a
> multisig, or will it be made immutable?
> → Recommended: a **Squads Protocol v4** multisig (M-of-N) for anything custodial, ideally
> behind a timelock. Never a single hot key on mainnet."

**Verify:** `solana program show <PROGRAM_ID>` reveals the current authority; check `Anchor.toml`
/ deploy scripts for who is set.

---

## 6.2 — Immutable vs. upgradeable trade-off `HIGH`

**Check:** The decision to keep the program upgradeable (flexibility, can patch bugs — but
trust assumption + rug vector) vs. make it immutable (`--final`: maximal trust, but bugs are
permanent) is explicit and matches the product's stage.

**Why:** Both extremes are defensible; what's dangerous is *not deciding*. Immutable-too-early
locks in bugs; upgradeable-forever leaves a permanent backdoor users must trust.

**Question:**
> "Is staying upgradeable a deliberate choice? What's the path to reduced trust over time?
> → Recommended: launch upgradeable behind a multisig+timelock, publish the policy, and move
> toward immutability or DAO-controlled upgrades as the program matures."

---

## 6.3 — Admin / config authority blast radius `HIGH`

**Check:** The on-chain admin (distinct from the upgrade authority) can only do bounded things.
Enumerate every admin-only instruction and ask: what is the worst an admin (or a stolen admin
key) can do? Can it drain user funds, set a 100% fee, change the oracle, or pause withdrawals
forever?

**Why:** "Admin can update config" often quietly includes "admin can point the oracle at a
fake feed and liquidate everyone." The blast radius is usually larger than intended.

**Question:**
> "List your admin instructions. For each, what's the worst-case if the admin key is malicious
> or stolen? Which of those should be bounded or removed?
> → Recommended: bound every admin power in code (fee caps, oracle allow-list, no direct
> user-fund withdrawal); put the dangerous ones behind multisig+timelock."

**Verify in code:** enumerate admin-gated instructions (from branch 2's access matrix); annotate
each with its worst-case effect.

---

## 6.4 — Multisig & timelock `HIGH`

**Check:** Privileged actions (upgrade, set-oracle, set-fee, withdraw-protocol-funds) go through
a multisig, and the highest-impact ones have a timelock giving users time to exit.

**Why:** Multisig removes single-key risk; timelock removes "instant rug" risk and is a strong
trust signal. **Squads Protocol v4** is the standard on Solana (it supports timelocks, spending
limits, and roles).

**Question:**
> "Which actions are multisig-gated, and do any have a timelock?
> → Recommended: Squads Protocol v4 M-of-N for all privileged actions; a timelock (hours–days)
> on upgrades, oracle changes, and anything that can move user funds."

---

## 6.5 — Pause / emergency stop `MEDIUM`

**Check:** There is a way to halt the program in an incident — and it can't itself be abused to
permanently trap user funds (users can still withdraw, or the pause is time-bounded).

**Why:** A circuit breaker limits exploit damage. But a pause that *also* blocks withdrawals is
a censorship/lock vector — get the asymmetry right.

**Question:**
> "Is there an emergency pause? Does it still let users withdraw, or can it trap funds?
> → Recommended: a guardian-triggered pause that stops *new risk* (deposits, new positions)
> but never blocks users from exiting; log every pause."

**Verify in code:** `Grep "paused", "pause", "guardian", "emergency"`.

---

## Branch exit

`threat-model.md` gets a **governance section**: who holds the upgrade authority (and target
state), the admin blast-radius table, multisig/timelock coverage, and the pause design. A live
single-key upgrade authority on a value-holding mainnet program is CRITICAL — it dominates the
risk profile regardless of how clean the code is.

---

**Sources:** the primary references behind every check in this branch are listed in [SOURCES.md](../../SOURCES.md). Do not assert a claim that isn't grounded there (or in a newer official source) — flag it instead.
