use anchor_lang::prelude::*;
use mpl_core::{accounts::BaseCollectionV1, instructions::CreateV2CpiBuilder, ID as MPL_CORE_ID};

#[derive(Accounts)]
pub struct MintAsset<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub asset: Signer<'info>,

    #[account(mut)]
    pub collection: Account<'info, BaseCollectionV1>,
    /// CHECK: This is the Update authority Account
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: This is the Metaplex Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> MintAsset<'info> {
    pub fn mint_asset(&mut self, name: String, uri: String, bumps: MintAssetBumps) -> Result<()> {
        let collection_key = self.collection.key();

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ]];

        let collection = &self.collection.to_account_info();

        CreateV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(collection))
            .authority(Some(&self.update_authority.to_account_info()))
            .payer(&self.user.to_account_info())
            .owner(Some(&self.user.to_account_info()))
            .update_authority(None)
            .system_program(&self.system_program.to_account_info())
            .name(name)
            .uri(uri)
            .invoke_signed(signer_seeds)?;

        Ok(())
    }
}
