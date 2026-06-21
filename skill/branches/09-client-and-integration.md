# Branch 9 — Client & Integration Boundary

The program can be flawless and the dapp still drains users — at the **seam** where the
frontend, wallet, RPC, and backend meet the chain. This branch roasts that boundary. It covers
only the **Solana-specific** surface; generic web2 security (XSS, CSRF, secrets-in-env,
dependency CVEs, CI/CD) is **out of scope — hand off to the `cso` skill.**

> Load this when the repo has a frontend / wallet integration / backend signer, not just
> `programs/`. Skip it for a pure on-chain library.

---

## 9.1 — Blind signing & transaction simulation `HIGH`

**Check:** Transactions are built so the wallet can simulate them and show real effects (asset
deltas, ownership changes, approvals); the app never pushes users to enable "blind signing" or
buries effects in opaque instruction data.

**Why:** Blind signing = the user approves a payload they can't read — a primary wallet-drain
vector (hidden token approvals, `SetAuthority`, account-ownership transfers). Modern wallets
simulate + scan (Phantom bundles Blowfish), but that's a backstop, not your security model — it
has missed cases before.

**Question:**
> "Does any flow ask users to approve a transaction whose effects the wallet can't surface
> (custom/opaque instructions, or UI that nudges 'enable blind signing')?
> → Recommended: keep instructions minimal and legible so wallet simulation shows true
> asset/ownership/approval deltas; never tell users to enable blind signing; clearly label
> high-risk instructions (`SetAuthority`, approvals) in-app."

**Verify in code:** inspect transaction assembly before `signTransaction`/`sendTransaction`; flag
opaque custom instruction data and any "enable blind signing" UI copy.

---

## 9.2 — Wallet auth via Sign-In With Solana (SIWS) `HIGH`

**Check:** Wallet login uses the **SIWS** standard (Wallet Standard `signIn`), not ad-hoc
`connect` + `signMessage`. The `SolanaSignInInput` carries `domain`, a server-issued `nonce`,
and `issuedAt`/`expirationTime`, and the **server** verifies the signed output
(`verifySignIn`).

**Why:** With SIWS the *wallet* builds the message from structured fields, so it can render it
safely and **domain-bind** it (defeats phishing), while `nonce` + `issuedAt` defeat replay. SIWS
is an open standard (Solana Labs + Phantom), supported by `@solana/wallet-adapter` and any
Wallet-Standard wallet.

**Question:**
> "Is login SIWS (`signIn`) or a custom `signMessage`? Does the input carry `domain`, a
> server-issued single-use `nonce`, and `issuedAt`, and does the backend verify it?
> → Recommended: use SIWS via wallet-adapter; server issues + stores a one-time `nonce`, sets
> `domain` to your real origin, then verifies with `verifySignIn` and rejects any mismatch."

**Verify in code:** `Grep "signIn", "SolanaSignInInput", "verifySignIn", "signMessage"`. If you
find `signMessage`-based auth → go to 9.3.

---

## 9.3 — `signMessage` auth done wrong (replay / no domain binding) `HIGH`

**Check:** If auth uses raw `signMessage`, the message includes a server-issued **single-use
nonce**, a **timestamp/expiry**, and the **domain/origin**; the backend verifies the ed25519
signature and rejects reused nonces and foreign domains.

**Why:** A static/client-chosen message with no nonce is **replayable** forever; with no domain,
a phishing site can harvest a signature and replay it against the real app (the wallet can't warn
the user — to it, a raw `signMessage` is just opaque bytes). Verifying the signature alone is not
enough; the *binding* is what stops replay.

**Question:**
> "Is the signed message fixed/client-supplied, or does it carry a server nonce + expiry + your
> domain — and does the server verify the signature and burn the nonce?
> → Recommended: prefer SIWS (9.2). If staying on `signMessage`: embed a server nonce,
> `issuedAt`/expiry, and origin; verify ed25519 server-side; reject reused nonces / wrong domain."

**Verify in code:** find the `signMessage` call + backend verify path; flag constant messages, no
nonce store, client-controlled nonces, missing expiry, no domain check, or "wallet connected" =
authenticated.

---

## 9.4 — Don't trust RPC reads for authorization `HIGH`

**Check:** Security-critical decisions (ownership, eligibility, value release) are enforced
**on-chain by the program**, not derived from a frontend/backend RPC read. RPC reads are
untrusted UI hints.

**Why:** A malicious, compromised, or stale RPC can return false account data (fake balance,
wrong owner, spoofed state). If authorization trusts that read, an attacker controlling the RPC
response manipulates the client. The chain is the source of truth; only an account's owner
program can mutate its data — so re-check every invariant on-chain at execution time.

**Question:**
> "Is any authorization / eligibility / value-movement decision made from an RPC read, or is it
> all re-validated by the program on-chain?
> → Recommended: never authorize off an RPC read; enforce every invariant in the program; treat
> client `getAccountInfo`/balance reads as advisory display only."

**Verify in code:** find frontend/backend logic that reads an account (owner, balance, flag,
allowlist) and then grants access / signs / releases value without the program re-checking it.

---

## 9.5 — Backend signer / fee-payer / relayer key management `CRITICAL`

**Check:** Server-side signers (fee payers, relayers/gasless, co-signing backends) keep the key
in a KMS/HSM (not a file/env), **and** the relayer validates every transaction it co-signs —
whitelisting allowed programs/accounts, capping fees, rejecting unrecognized instructions.

**Why:** A server fee-payer/relayer key is a live hot wallet. The Solana-specific footgun isn't
just key storage (defer pure secrets-mgmt to `cso`) — it's **what the relayer agrees to sign**: a
naive relayer that blind-`partialSign`s any submitted transaction can be tricked into sponsoring
or lending its signature to malicious instructions. Production relayers (Kora, Octane) validate +
whitelist before co-signing.

**Question:**
> "Where does the server signer live, and what does it agree to sign? Raw keypair in env/file, or
> KMS/HSM? Does the relayer inspect each transaction (allowed programs, fee caps, expected
> accounts) before adding its signature?
> → Recommended: signer in KMS/HSM; relayer validates every instruction it co-signs (whitelist
> programs/tokens, cap priority fee, reject unknown instructions) — never blind `partialSign`."

**Verify in code:** locate where the fee-payer/relayer keypair loads (flag
`Keypair.fromSecretKey(... process.env / fs.readFile ...)`) and the signing path; confirm
instruction/program validation + fee caps before co-signing.

---

## 9.6 — Transaction landing & durable nonces `MEDIUM` (reliability) / `HIGH` (durable nonce)

**Check:** The app fetches a fresh blockhash near send time (valid only ~60–90s / ~150 slots),
sets a priority fee under congestion, and retries/rebuilds on expiry. **Separately:** if it uses
**durable nonces** (offline/pre-signed transactions), it treats each pre-signed transaction as a
long-lived **bearer credential** — tightly scoped, short-lived, nonce advanced after use.

**Why:** *Reliability* — a stale-blockhash transaction is permanently dropped. *Security* —
durable nonces remove the time limit, separating *approval* from *execution*: the **Drift
Protocol incident (~$285M, April 2026)** abused pre-signed durable-nonce admin transactions that
stayed valid for over a week and executed after the context that justified them had changed.

**Question:**
> "How do transactions land — fresh blockhash + priority fee + retry, or stale-and-hope? Do you
> use durable nonces anywhere, and if so who holds the pre-signed tx, and for how long?
> → Recommended (reliability): `getLatestBlockhash` just before signing, priority fee under load,
> rebuild on expiry. (Security): treat durable-nonce pre-signed txs as bearer credentials —
> minimize lifetime, advance/cancel the nonce after use, scope tightly, re-verify context at
> execution."

**Verify in code:** check `getLatestBlockhash` timing vs signing; `ComputeBudgetProgram
.setComputeUnitPrice`; retry logic. `Grep "nonceAccount", "advanceNonceAccount"` — if durable
nonces gate any approval flow, escalate to HIGH and check lifetime/scoping.

---

## 9.7 — Other client↔chain footguns (quick checks)

- **Wallet connection ≠ authentication `HIGH`.** A connected/claimed pubkey only proves the user
  selected an address, not that they control it. APIs that return user-scoped data from
  `?wallet=<pubkey>` with no signature challenge are spoofable. *Verify:* backend routes taking a
  pubkey param and returning private data without a verified signature/session.
- **UI amount ≠ encoded instruction `MEDIUM`.** The instruction the program executes is the only
  truth; never authorize from what the UI *intended*.
- **Decimals/lamports at the seam `LOW`.** Mixing lamports (1 SOL = 1e9) or raw base units with
  display units → off-by-10^9 transfers. Frequent real-world bug.

---

## Branch exit

`threat-model.md` gains a **client/integration** section: auth method (SIWS vs raw), RPC-trust
posture, backend-signer custody + relayer validation, and durable-nonce handling. Then **hand off
generic web2/infra security to `cso`** — say so explicitly; don't pretend to cover it.

---

**Sources:** the primary references behind every check in this branch are listed in [SOURCES.md](../../SOURCES.md). Do not assert a claim that isn't grounded there (or in a newer official source) — flag it instead.
