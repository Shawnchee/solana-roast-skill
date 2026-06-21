# Sources

Every check in `solana-roast` traces to a primary, authoritative source. This file maps each
branch's claims to where they come from, so nothing in the skill is asserted from memory. All
links were verified against the live docs during the skill's accuracy pass (Anchor 1.0.x /
Solana 3.x, mid-2026).

> **Rule the skill follows:** if a claim can't be grounded in one of these (or a newer official
> source), it says so rather than guessing. When the stack moves, update the link *and* the
> branch text together.

## Foundational references (apply across all branches)

- Helius — *A Hitchhiker's Guide to Solana Program Security* — https://www.helius.dev/blog/a-hitchhikers-guide-to-solana-program-security
- Zealynx — *Solana Security Guide: 45 Exploit Checks for Anchor & Native Programs* — https://www.zealynx.io/blogs/solana-security-checklist
- *Solana Program Security Checklist: 14 Critical Checks Before Mainnet* — https://dev.to/ohmygod/solana-program-security-checklist-14-critical-checks-before-you-deploy-to-mainnet-2d66
- Solana docs — Core concepts — https://solana.com/docs/core
- Anchor docs — https://www.anchor-lang.com/docs · Anchor Book — https://book.anchor-lang.com
- Anchor 1.0 release notes — https://www.anchor-lang.com/docs/updates/release-notes/1-0-0 · changelog — https://www.anchor-lang.com/docs/updates/changelog
- Anchor 0.31 release notes (custom discriminators) — https://www.anchor-lang.com/docs/updates/release-notes/0-31-0
- `anchor-lang` API — https://docs.rs/anchor-lang/latest/anchor_lang/

## Branch 1 — Accounts & PDAs

- Owner check / typed accounts (`Account<T>` vs `AccountInfo`) — https://www.anchor-lang.com/docs/references/account-types
- Canonical PDA bump (bare `bump` re-derives canonical; danger is a *user-supplied* bump) — https://www.anchor-lang.com/docs/pdas · https://www.anchor-lang.com/docs/references/account-constraints · Helius guide (PDA section)
- PDA sharing / seed uniqueness — Helius guide
- `has_one` relationship enforcement — https://www.anchor-lang.com/docs/references/account-constraints
- Duplicate mutable accounts rejected by default + `dup` constraint (Anchor 1.0) — https://www.anchor-lang.com/docs/updates/release-notes/1-0-0 · changelog
- `init` vs `init_if_needed` (feature-gated; re-init risk) — https://www.anchor-lang.com/docs/references/account-constraints

## Branch 2 — Authority & Signers

- Missing-signer as the #1 exploit class — Helius guide; Zealynx checklist
- Wormhole (~$320M, Feb 2022 — account-confusion on the signature-verification path) — https://www.halborn.com/blog/post/explained-the-wormhole-hack-february-2022 · https://research.kudelskisecurity.com/2022/02/03/quick-analysis-of-the-wormhole-attack/
- Binding the signer to the resource (`has_one` / `require_keys_eq!`) — https://www.anchor-lang.com/docs/references/account-constraints

## Branch 3 — CPI & Composability

- Cross-Program Invocations & arbitrary-CPI risk — https://solana.com/docs/core/cpi · Helius guide
- CPI nesting depth (historically 4; SIMD-0268 raises to 8, "Accepted", contingent on SIMD-0219) — https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0268-raise-cpi-nesting-limit.md · https://solana.com/docs/programs/limitations
- Transfer-hook reentrancy surface (read-only + signer-drop during hook) — https://neodyme.io/en/blog/token-2022/
- Crediting the actual delta around CPIs (fee-on-transfer) — https://neodyme.io/en/blog/token-2022/

## Branch 4 — State & Data

- Account discriminator (8 bytes by default; customizable since Anchor 0.31) — https://www.anchor-lang.com/docs/updates/release-notes/0-31-0 · https://docs.rs/anchor-lang/latest/anchor_lang/attr.account.html
- Type discrimination / type cosplay — https://www.anchor-lang.com/docs/references/account-types · Helius guide
- Re-initialization safety — https://www.anchor-lang.com/docs/references/account-constraints
- Account close & revival (`close =`; `CLOSED_ACCOUNT_DISCRIMINATOR` no longer a public constant) — https://www.anchor-lang.com/docs/references/account-constraints · https://docs.rs/anchor-lang/latest/anchor_lang/
- `realloc` (`realloc::zero`, payer) — https://www.anchor-lang.com/docs/references/account-constraints
- Space / `InitSpace` / `#[max_len]` — https://www.anchor-lang.com/docs/references/space

## Branch 5 — Economic Invariants

- Release builds wrap on overflow; `overflow-checks` off by default — https://doc.rust-lang.org/cargo/reference/profiles.html
- First-depositor / share-inflation (virtual-shares offset, dead shares, min deposit) — https://blog.openzeppelin.com/a-novel-defense-against-erc4626-inflation-attacks · https://docs.openzeppelin.com/contracts/5.x/erc4626
- Pyth as a pull oracle (`get_price_no_older_than`, confidence) — https://docs.pyth.network/price-feeds/core/use-real-time-data/pull-integration/solana · https://docs.pyth.network/price-feeds/core/best-practices
- Switchboard On-Demand (pull feeds) — https://github.com/switchboard-xyz/on-demand

## Branch 6 — Upgrade Authority & Governance

- Upgradeable programs, upgrade authority, `--final` immutability — https://solana.com/docs/programs/deploying
- Squads Protocol v4 (multisig, timelocks, spending limits, roles) — https://github.com/Squads-Protocol/v4 · https://docs.squads.so

## Branch 7 — Compute, Limits & DoS

- Compute budget (1,400,000 CU/tx; 200k default per instruction) — https://solana.com/docs/core/fees/compute-budget
- Transaction size (1,232 bytes) — https://solana.com/docs/core/transactions
- Account count: 256 hard cap (u8 index); Address Lookup Tables — https://docs.anza.xyz/proposals/versioned-transactions · https://solana.com/developers/guides/advanced/lookup-tables
- Block limit context (SIMD-0286, 60M→100M) — https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0286-raise-block-limits-to-100M.md

## Branch 8 — Tokens (SPL & Token-2022)

- Token-2022 extension set (transfer hook, transfer fee, permanent delegate, default state, pausable, …) — https://solana.com/docs/tokens/extensions
- Token-2022 security pitfalls (permanent-delegate clawback, hooks, fees) — https://neodyme.io/en/blog/token-2022/
- `anchor-spl` `token_interface` (`InterfaceAccount`, `Interface<TokenInterface>`) — https://docs.rs/anchor-spl/latest/anchor_spl/ · https://www.anchor-lang.com/docs/tokens/extensions
- `transfer_checked` vs deprecated `Transfer` (Token-2022) — https://docs.rs/spl-token-2022/latest/spl_token_2022/instruction/index.html

## Testing & toolchain (interrogation protocol)

- LiteSVM (fast unit tests) — https://github.com/LiteSVM/litesvm
- Mollusk (instruction-level harness) — https://github.com/buffalojoec/mollusk
- Surfpool (integration / mainnet-fork) — https://github.com/solana-foundation/surfpool
- `solana-bankrun` (deprecated → use LiteSVM) — https://github.com/kevinheavey/solana-bankrun
- Anchor releases (current 1.0.x) — https://github.com/coral-xyz/anchor/releases

---

### Items deliberately left unasserted (no guessing)

- **Exact mainnet activation epoch of SIMD-0268 (CPI depth 4→8).** The SIMD is "Accepted" and
  contingent on SIMD-0219; the trackers disagree on past/future tense. The skill therefore says
  "historically 4, being raised to 8 by SIMD-0268 (Agave 3.x)" rather than asserting a live value.
- **The precise closed-account sentinel byte value in Anchor 1.0.** `CLOSED_ACCOUNT_DISCRIMINATOR`
  is no longer a public exported constant; the skill recommends the `close =` constraint and does
  not assert the internal byte value.
