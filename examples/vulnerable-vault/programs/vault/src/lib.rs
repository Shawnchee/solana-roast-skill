//! ⚠️  INTENTIONALLY VULNERABLE — teaching fixture for the `solana-roast` skill.
//! DO NOT DEPLOY. Every `VULN-n` comment marks a deliberate design flaw that
//! `solana-roast` is meant to catch at the design stage. The roast this program
//! produces is committed in ../../sample-roast-output/.
//!
//! Target: Anchor 1.0.x / Solana 3.x. This is an illustrative fixture; it is kept
//! minimal to make the flaws obvious, not to be a complete, deployable program.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("RoastVu1n111111111111111111111111111111111");

#[program]
pub mod vulnerable_vault {
    use super::*;

    /// Create the vault PDA and record its authority.
    pub fn initialize(ctx: Context<Initialize>, vault_bump: u8) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.authority.key();
        // VULN-1 (branch 1.2, HIGH): bump comes from instruction input instead of
        // Anchor's canonical `bump`. A caller can pass a non-canonical valid bump.
        vault.bump = vault_bump;
        vault.total_deposited = 0;
        Ok(())
    }

    /// Deposit SPL tokens into the vault.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        // VULN-2 (branch 5.2, CRITICAL): unchecked add. With overflow-checks off
        // (the release default) this wraps.
        vault.total_deposited = vault.total_deposited + amount;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }

    /// Withdraw SPL tokens from the vault. This handler is the disaster zone.
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        // VULN-3 (branch 5.2, CRITICAL): unchecked sub. Underflows to a huge u64,
        // so "withdraw more than you have" silently succeeds in accounting.
        vault.total_deposited = vault.total_deposited - amount;

        // The vault PDA signs the token transfer out.
        let seeds: &[&[u8]] = &[b"vault", &[vault.bump]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_token_account.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.vault.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Vault::INIT_SPACE,
        seeds = [b"vault"],
        bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"vault"], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    // VULN-4 (branch 8.1, CRITICAL): no `token::mint` / `token::authority` constraint.
    // A caller can pass any token account as the "vault" account.
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // VULN-5 (branch 1.4 / 2.2, CRITICAL): `vault` carries an `authority` field but it is
    // never bound here — no `has_one = authority`. Combined with VULN-6, withdraw is open.
    #[account(mut, seeds = [b"vault"], bump = vault.bump)]
    pub vault: Account<'info, Vault>,

    // VULN-6 (branch 2.1, CRITICAL): `authority` gates the withdraw but is neither a
    // `Signer` nor checked against `vault.authority`. Anyone can call withdraw.
    /// CHECK: intentionally unvalidated for the demo
    pub authority: UncheckedAccount<'info>,

    // VULN-7 (branch 8.1, CRITICAL): unconstrained token accounts again.
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,

    // VULN-8 (branch 3.1, CRITICAL): the token program is an unverified account → an
    // attacker can substitute their own program (arbitrary CPI).
    /// CHECK: intentionally unvalidated for the demo
    pub token_program: UncheckedAccount<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub authority: Pubkey,
    pub total_deposited: u64,
    pub bump: u8,
}
