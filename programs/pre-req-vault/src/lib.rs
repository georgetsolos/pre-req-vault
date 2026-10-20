use anchor_lang::prelude::*;

declare_id!("Gx5it45U9NS2eswoy1J3ahEr4B8FYm6xAFDmBiKozkQA");

declare_program!(registration);

use registration::cpi::accounts::Initialize as RegistrationInitialize;
use registration::cpi::initialize as registration_initialize;

#[program]
pub mod pre_req_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let state = &mut ctx.accounts.vault_state;
        state.state_bump = ctx.bumps.vault_state;
        state.vault_bump = ctx.bumps.vault;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let ix = anchor_lang::system_program::Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.system_program.key(), ix);
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        amount: u64,
        github_username: String,
    ) -> Result<()> {
        // Signer seeds must match the vault PDA derivation:
        //   seeds = [b"vault", vault_state.key(), bump]
        let vault_state_key = ctx.accounts.vault_state.key();
        let vault_bump = ctx.accounts.vault_state.vault_bump;
        let seeds: &[&[u8]] = &[b"vault", vault_state_key.as_ref(), &[vault_bump]];
        let signer = &[seeds];

        let ix = anchor_lang::system_program::Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.key(),
            ix,
            signer,
        );
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;

        // CPI into registration::initialize
        require_keys_eq!(
            ctx.accounts.application_program.key(),
            registration::ID,
            anchor_lang::error::ErrorCode::ConstraintOwner,
        );

        let cpi_accounts = RegistrationInitialize {
            user: ctx.accounts.user.to_account_info(),
            account: ctx.accounts.application_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.application_program.key(),
            cpi_accounts,
        );
        registration_initialize(cpi_ctx, github_username)?;

        Ok(())
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        let vault_state_key = ctx.accounts.vault_state.key();
        let vault_bump = ctx.accounts.vault_state.vault_bump;
        let seeds: &[&[u8]] = &[b"vault", vault_state_key.as_ref(), &[vault_bump]];
        let signer = &[seeds];

        let ix = anchor_lang::system_program::Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.key(),
            ix,
            signer,
        );
        anchor_lang::system_program::transfer(cpi_ctx, ctx.accounts.vault.lamports())?;
        Ok(())
    }
}

#[account]
pub struct VaultState {
    pub state_bump: u8,
    pub vault_bump: u8,
}

impl VaultState {
    pub const LEN: usize = 8 + 1 + 1;
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = VaultState::LEN,
        seeds = [b"state", user.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    /// CHECK: PDA validated against registration program seeds.
    #[account(
        mut,
        seeds = [b"prereqs", user.key().as_ref()],
        bump,
        seeds::program = registration::ID,
    )]
    pub application_account: UncheckedAccount<'info>,

    /// CHECK: Manually checked in the instruction body.
    pub application_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        close = user,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}
