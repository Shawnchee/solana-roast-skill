# Example: `vulnerable-vault` — a worked `solana-roast` session

> ⚠️ **The program in `programs/vault/src/lib.rs` is intentionally vulnerable.**
> It exists only to demonstrate what `solana-roast` catches. **Do not deploy it.**

This folder is the proof that the skill works end-to-end on a real (if deliberately broken)
Anchor program. It contains:

```
vulnerable-vault/
├── Anchor.toml                       # Anchor 1.0.x / Solana 3.x
├── programs/vault/
│   ├── Cargo.toml                    # overflow-checks = false (a flaw, on purpose)
│   └── src/lib.rs                    # 8 labelled VULN-n design flaws
└── sample-roast-output/              # what `/roast` writes to .solana-roast/
    ├── transcript.md                 # the interrogation, one question at a time
    ├── design-spec.md                # the resolved design + access matrix
    ├── threat-model.md               # findings ledger, severity-triaged
    └── pre-audit-checklist.md        # the hand-off artifact for auditors
```

## The 8 planted flaws

| ID | Branch | Severity | Flaw |
|----|--------|----------|------|
| VULN-1 | 1.2 accounts/PDAs | HIGH | `initialize` takes the PDA bump as an **instruction argument** instead of using Anchor's canonical `bump`. |
| VULN-2 | 5.2 economic | CRITICAL | `deposit` uses unchecked `+` on `total_deposited` (overflow). |
| VULN-3 | 5.2 economic | CRITICAL | `withdraw` uses unchecked `-` on `total_deposited` (underflow → huge balance). |
| VULN-4 | 8.1 tokens | CRITICAL | `deposit.vault_token_account` has no `token::mint` / `token::authority` constraint. |
| VULN-5 | 2.2 authority | CRITICAL | `withdraw` never binds the signer to `vault.authority` (no `has_one`). |
| VULN-6 | 2.1 authority | CRITICAL | `withdraw.authority` is an `UncheckedAccount` — not a `Signer`. Anyone can withdraw. |
| VULN-7 | 8.1 tokens | CRITICAL | `withdraw` token accounts are unconstrained. |
| VULN-8 | 3.1 CPI | CRITICAL | `withdraw.token_program` is unverified → arbitrary CPI. |

Plus a process flaw: `Cargo.toml` ships `overflow-checks = false` (the release default), which is
what makes VULN-2/VULN-3 exploitable.

## How to reproduce

In a project that has `solana-roast` installed:

```
/roast examples/vulnerable-vault
```

or just say: *"roast my Solana program"* with this folder open. The skill reads `lib.rs`, walks
the branches one question at a time, and writes the four files you see in `sample-roast-output/`
(it writes them to `.solana-roast/` in a real run; they're copied here, un-ignored, so you can
read them without running anything).
