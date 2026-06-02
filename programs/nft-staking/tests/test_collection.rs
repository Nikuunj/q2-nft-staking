use solana_signer::Signer;
use solana_keypair::Keypair;
mod utils;

use utils::*;

#[test]
fn test_create_collection() {
    let (mut svm, payer) = setup();

    let collection = Keypair::new();

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

    assert!(svm.get_account(&collection.pubkey()).is_some());
}