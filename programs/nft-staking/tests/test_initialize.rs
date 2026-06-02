use anchor_lang::AccountDeserialize;
use solana_keypair::Keypair;
use solana_signer::Signer;
mod utils;

use utils::*;

#[test]
fn test_initialize() {
    let (mut svm, payer) = setup();

    let collection = Keypair::new();

    // Create collection first
    send_tx(
        &mut svm,
        &payer,
        &[&collection],
        vec![create_collection_ix(
            payer.pubkey(),
            collection.pubkey(),
            "Test Collection".to_string(),
            "https://example.com".to_string(),
        )],
    );

    send_tx(
        &mut svm,
        &payer,
        &[],
        vec![initialize_ix(
            payer.pubkey(),
            collection.pubkey(),
            500, // 5%
            7,   // 7 days
        )],
    );

    let config = get_config(&collection.pubkey());

    let config_account = svm.get_account(&config).unwrap();

    let config_data =
        nft_staking::state::Config::try_deserialize(
            &mut config_account.data.as_slice(),
        )
        .unwrap();

    assert_eq!(config_data.reward_bps, 500);
    assert_eq!(config_data.freeze_period, 7);
}
