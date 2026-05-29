use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};
use mpl_core::accounts::BaseCollectionV1;

use crate::state::Config;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = Config::INIT_SPACE + Config::DISCRIMINATOR.len(),
        seeds = [b"config", collection.key().as_ref()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        has_one = update_authority
    )]
    pub collection: Account<'info, BaseCollectionV1>,

    /// CHECK: This is the Update authority Account
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = config,
        seeds = [b"reward_mint", config.key().as_ref()],
        bump
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(
        &mut self,
        reward_bps: u16,
        freeze_period: u16,
        bumps: InitializeBumps,
    ) -> Result<()> {
        self.config.set_inner(Config {
            reward_bps,
            freeze_period,
            rewards_bump: bumps.reward_mint,
            bump: bumps.config,
        });
        Ok(())
    }
}
