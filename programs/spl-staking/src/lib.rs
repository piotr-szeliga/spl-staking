mod ins;
mod state;

use crate::ins::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer};

declare_id!("9GAsSHWvHoHoqbk8tqHYCq3fcpyGmovgXD5GBkSo4p3f");

#[program]
pub mod spl_staking {

    use super::*;

    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        daily_payout_amount: u64,
        bump: u8,
    ) -> Result<()> {
        let mut vault = ctx.accounts.vault.load_init()?;
        vault.bump = bump;
        vault.stake_token_mint = ctx.accounts.stake_token_mint.key();
        vault.daily_payout_amount = daily_payout_amount;
        vault.authority = ctx.accounts.authority.key();

        Ok(())
    }

    pub fn update_vault(
        ctx: Context<UpdateVault>,
        new_authority: Pubkey,
        daily_payout_amount: u64,
    ) -> Result<()> {
        let mut vault = ctx.accounts.vault.load_mut()?;
        vault.stake_token_mint = ctx.accounts.stake_token_mint.key();
        vault.daily_payout_amount = daily_payout_amount;
        vault.authority = new_authority;

        Ok(())
    }

    pub fn fund(ctx: Context<Fund>, amount: u64) -> Result<()> {
        let mut vault = ctx.accounts.vault.load_mut()?;

        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.funder_ata.to_account_info(),
                    to: ctx.accounts.vault_ata.to_account_info(),
                    authority: ctx.accounts.funder.to_account_info(),
                },
            ),
            amount,
        )?;

        vault.reward_pool_amount = vault.reward_pool_amount.checked_add(amount).unwrap();

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let mut vault = ctx.accounts.vault.load_mut()?;
        let bump = vault.bump;
        let vault_bump = bump;
        let seeds = [b"vault".as_ref(), &[vault_bump]];
        let signer = &[&seeds[..]];
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_ata.to_account_info(),
                    to: ctx.accounts.authority_ata.to_account_info(),
                    authority: ctx.accounts.token_vault.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;

        vault.reward_pool_amount = vault.reward_pool_amount.checked_sub(amount).unwrap();

        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let mut vault = ctx.accounts.vault.load_mut()?;

        vault.stake(ctx.accounts.staker.key(), amount);

        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.staker_ata.to_account_info(),
                    to: ctx.accounts.vault_ata.to_account_info(),
                    authority: ctx.accounts.staker.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        let mut vault = ctx.accounts.vault.load_mut()?;
        let bump = vault.bump;
        let vault_bump = bump;
        let seeds = [b"vault".as_ref(), &[vault_bump]];
        let signer = &[&seeds[..]];
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_ata.to_account_info(),
                    to: ctx.accounts.staker_ata.to_account_info(),
                    authority: ctx.accounts.token_vault.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;

        vault.unstake(ctx.accounts.staker.key(), amount);

        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let mut vault = ctx.accounts.vault.load_mut()?;
        let bump = vault.bump;
        let vault_bump = bump;

        let amount = vault.claim(ctx.accounts.staker.key());

        let seeds = [b"vault".as_ref(), &[vault_bump]];
        let signer = &[&seeds[..]];
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_ata.to_account_info(),
                    to: ctx.accounts.staker_ata.to_account_info(),
                    authority: ctx.accounts.token_vault.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;

        Ok(())
    }

    pub fn close_pda(ctx: Context<ClosePda>) -> Result<()> {
        let dest_account_info = ctx.accounts.signer.to_account_info();
        let source_account_info = ctx.accounts.pda.to_account_info();
        let dest_starting_lamports = dest_account_info.lamports();
        **dest_account_info.lamports.borrow_mut() = dest_starting_lamports
            .checked_add(source_account_info.lamports())
            .unwrap();
        **source_account_info.lamports.borrow_mut() = 0;

        Ok(())
    }
}
