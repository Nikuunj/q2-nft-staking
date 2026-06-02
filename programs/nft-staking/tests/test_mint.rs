use solana_signer::Signer;
use solana_keypair::Keypair;
mod utils;

use utils::*;

#[test]
fn test_mint_asset() {
    let (mut svm, payer) = setup();

    let collection = Keypair::new();
    let asset = Keypair::new();

    send_tx(
        &mut svm,
        &payer,
        &[&collection],
        vec![create_collection_ix(
            payer.pubkey(),
            collection.pubkey(),
            "Collection".into(),
            "uri".into(),
        )],
    );

    send_tx(
        &mut svm,
        &payer,
        &[&asset],
        vec![mint_asset_ix(
            payer.pubkey(),
            asset.pubkey(),
            collection.pubkey(),
            "NFT".into(),
            "uri".into(),
        )],
    );

    let account = svm.get_account(&asset.pubkey());
    assert!(account.is_some());
}