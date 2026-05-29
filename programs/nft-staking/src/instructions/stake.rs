use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{AddPluginV1CpiBuilder, UpdatePluginV1CpiBuilder},
    types::{
        Attribute, Attributes, FreezeDelegate, Plugin, PluginAuthority, PluginType, UpdateAuthority,
    },
    ID as MPL_CORE_ID,
};

use crate::{error::ErrorCode, state::Config};

#[derive(Accounts)]
pub struct Stake<'info> {
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

    pub system_program: Program<'info, System>,

    /// CHECK: This is the Metaplex Core program
    #[account(
        address = MPL_CORE_ID
    )]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> Stake<'info> {
    pub fn stake(&mut self, bumps: StakeBumps) -> Result<()> {
        let attributes_fetched: Option<Attributes> = fetch_plugin::<BaseAssetV1, Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes,
        )
        .ok()
        .map(|(_, attrs, _)| attrs);

        let mut attributes_list: Vec<Attribute> = Vec::new();

        if let Some(attributes) = &attributes_fetched {
            for attribute in &attributes.attribute_list {
                if attribute.key == "staked" {
                    require!(attribute.value == "false", ErrorCode::CustomError);
                } else if attribute.key != "staked_at" {
                    attributes_list.push(attribute.clone());
                }
            }
        }

        attributes_list.push(Attribute {
            key: "staked".to_string(),
            value: "true".to_string(),
        });

        attributes_list.push(Attribute {
            key: "staked_at".to_string(),
            value: Clock::get()?.unix_timestamp.to_string(),
        });

        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ]];

        if attributes_fetched.is_none() {
            AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .asset(&self.asset.to_account_info())
                .collection(Some(&self.collection.to_account_info()))
                .payer(&self.owner.to_account_info())
                .authority(Some(&self.update_authority.to_account_info()))
                .system_program(&self.system_program.to_account_info())
                .plugin(Plugin::Attributes(Attributes {
                    attribute_list: attributes_list,
                }))
                .init_authority(PluginAuthority::UpdateAuthority)
                .invoke_signed(signer_seeds)?;
        } else {
            UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .asset(&self.asset.to_account_info())
                .collection(Some(&self.collection.to_account_info()))
                .payer(&self.owner.to_account_info())
                .authority(Some(&self.update_authority.to_account_info()))
                .system_program(&self.system_program.to_account_info())
                .plugin(Plugin::Attributes(Attributes {
                    attribute_list: attributes_list,
                }))
                .invoke_signed(signer_seeds)?;
        }

        AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.owner.to_account_info())
            .system_program(&self.system_program.to_account_info())
            .authority(Some(&&self.owner.to_account_info()))
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
            .init_authority(PluginAuthority::UpdateAuthority)
            .invoke()?;

        Ok(())
    }
}
