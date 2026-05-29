
use anchor_lang::{prelude::*, solana_program::clock::SECONDS_PER_DAY};
use anchor_spl::{associated_token::AssociatedToken,  token_interface::{Mint, MintToChecked, TokenAccount, TokenInterface, mint_to_checked}};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{ UpdatePluginV1CpiBuilder},
    types::{
        Attribute, Attributes, FreezeDelegate, Plugin, PluginType, UpdateAuthority,
    },
    ID as MPL_CORE_ID,
};

use crate::{error::ErrorCode, state::Config};

#[derive(Accounts)]
pub struct UnStake<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        has_one = update_authority
    )]
    pub collection: Account<'info, BaseCollectionV1>,

    #[account(
        mut,
        has_one = owner,
        constraint = asset.update_authority == UpdateAuthority::Collection(collection.key())
    )]
    pub asset: Account<'info, BaseAssetV1>,

    /// CHECK: This is the Update authority Account
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"reward_mint", config.key().as_ref()],
        bump = config.rewards_bump
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = reward_mint,
        associated_token::authority = owner
        
    )]
    pub user_reward_ata: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,

    /// CHECK: This is the Metaplex Core program
    #[account(
        address = MPL_CORE_ID
    )]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> UnStake<'info> {
    pub fn unstake(&mut self, bumps: UnStakeBumps) -> Result<()> {
        let attributes_fetched: Option<Attributes> = fetch_plugin::<BaseAssetV1, Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes,
        )
        .ok()
        .map(|(_, attrs, _)| attrs);


        require!(attributes_fetched.is_some(), ErrorCode::CustomError);

        let attributes = attributes_fetched.unwrap();
        let mut attributes_list: Vec<Attribute> = Vec::with_capacity(attributes.attribute_list.len());

        let current_timestamp = Clock::get()?.unix_timestamp;
        let mut staked_timestamp: i64 = 0;
        let mut staked_time: i64 = 0;

        for attirbute in &attributes.attribute_list {
            if attirbute.key == "staked" {
                require!(attirbute.value == "true", ErrorCode::CustomError);
            }
            else if attirbute.key == "staked_at" {
                staked_timestamp = staked_timestamp.checked_add(attirbute.value.parse::<i64>().map_err(|_| ErrorCode::CustomError)?).ok_or(ErrorCode::CustomError)?;

                staked_time = current_timestamp.checked_sub(staked_timestamp).ok_or(ErrorCode::CustomError)?;

                staked_time = staked_time.checked_div(SECONDS_PER_DAY as i64).ok_or(ErrorCode::CustomError)?;

                require!(staked_time >= self.config.freeze_period as i64, ErrorCode::CustomError);

            } 
            else {
                attributes_list.push(attirbute.clone());
            }
        }

        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ]];

        attributes_list.push(Attribute {
            key: "staked".to_string(),
            value: "false".to_string(),
        });

        attributes_list.push(Attribute {
            key: "staked_at".to_string(),
            value: "0".to_string(),
        });


        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
        .asset(&self.asset.to_account_info())
        .collection(Some(&self.collection.to_account_info()))
        .payer(&self.owner.to_account_info())
        .authority(Some(&self.update_authority.to_account_info()))
        .system_program(&self.system_program.to_account_info())
        .plugin(Plugin::Attributes(Attributes { attribute_list: attributes_list }))
        .invoke_signed(signer_seeds)?;


        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
        .asset(&self.asset.to_account_info())
        .collection(Some(&self.collection.to_account_info()))
        .payer(&self.owner.to_account_info())
        .authority(Some(&self.update_authority.to_account_info()))
        .system_program(&self.system_program.to_account_info())
        .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
        .invoke_signed(signer_seeds)?;

        let amount = (staked_time as u64)  
        .checked_mul(self.config.reward_bps as u64)
        .ok_or(ErrorCode::CustomError)? 
        .checked_mul(10u64.pow(self.reward_mint.decimals as u32))
        .ok_or(ErrorCode::CustomError)?
        .checked_div(10000u64)
        .ok_or(ErrorCode::CustomError)?;

        let config_signer_seeds: &[&[&[u8]]] = &[&[
            b"config",
            collection_key.as_ref(),
            &[self.config.bump]
        ]];

        mint_to_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(), 
                MintToChecked { 
                    mint: self.reward_mint.to_account_info(), 
                    to: self.user_reward_ata.to_account_info(), 
                    authority: self.config.to_account_info() 
                }, config_signer_seeds), 
                amount, 
            self.reward_mint.decimals
        )
    }
}
