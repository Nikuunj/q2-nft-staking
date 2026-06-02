#[allow(dead_code)]

use anchor_lang::{
    solana_program::{instruction::Instruction, pubkey::Pubkey},
    system_program::ID as SYSTEM_PROGRAM_ID,
    InstructionData, Key, ToAccountMetas,
};
use anchor_spl::associated_token;
use litesvm::LiteSVM;
use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo, TOKEN_ID};
use mpl_core::{
    instructions::{CreateCollectionV1Builder, CreateV1Builder},
    ID as MPL_CORE_ID,
};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

pub fn setup() -> (LiteSVM, Keypair) {
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/nft_staking.so");
    let bytes_mpl = include_bytes!("./mpl_core_program.so");
    svm.add_program(Pubkey::from(MPL_CORE_ID.to_bytes()), bytes_mpl);
    svm.add_program(nft_staking::ID, bytes);
    let payer_address = payer.pubkey().to_bytes().into();
    svm.airdrop(&payer_address, 100_000_000_000).unwrap();
    (svm, payer)
}


pub fn send_tx(
    svm: &mut LiteSVM,
    payer: &Keypair,
    signers: &[&Keypair],
    instructions: Vec<Instruction>,
) {
    let mut all_signers = vec![payer];
    all_signers.extend_from_slice(signers);

    let tx = Transaction::new(
        &all_signers,
        Message::new(&instructions, Some(&payer.pubkey())),
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx).unwrap();
}

pub fn get_update_authority(collection: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"update_authority", collection.as_ref()],
        &nft_staking::ID,
    )
    .0
}

pub fn get_config(collection: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"config", collection.as_ref()],
        &nft_staking::ID,
    )
    .0
}

pub fn get_reward_mint(collection: &Pubkey) -> Pubkey {
    let config = get_config(collection);

    Pubkey::find_program_address(
        &[b"reward_mint", config.as_ref()],
        &nft_staking::ID,
    )
    .0
}


pub fn create_collection_ix(
    payer: Pubkey,
    collection: Pubkey,
    name: String,
    uri: String,
) -> Instruction {
    Instruction {
        program_id: nft_staking::ID,
        accounts: nft_staking::accounts::CreateCollection {
            payer,
            collection,
            update_authority: get_update_authority(&collection),
            system_program: SYSTEM_PROGRAM_ID,
            mpl_core_program: MPL_CORE_ID,
        }
        .to_account_metas(None),
        data: nft_staking::instruction::CreateCollection {
            name,
            uri,
        }
        .data(),
    }
}


pub fn mint_asset_ix(
    user: Pubkey,
    asset: Pubkey,
    collection: Pubkey,
    name: String,
    uri: String,
) -> Instruction {
    Instruction {
        program_id: nft_staking::ID,
        accounts: nft_staking::accounts::MintAsset {
            user,
            asset,
            collection,
            update_authority: get_update_authority(&collection),
            system_program: SYSTEM_PROGRAM_ID,
            mpl_core_program: MPL_CORE_ID,
        }
        .to_account_metas(None),
        data: nft_staking::instruction::MintAsset {
            name,
            uri,
        }
        .data(),
    }
}

pub fn initialize_ix(
    admin: Pubkey,
    collection: Pubkey,
    reward_bps: u16,
    freeze_period: u16,
) -> Instruction {
    Instruction {
        program_id: nft_staking::ID,
        accounts: nft_staking::accounts::Initialize {
            admin,
            config: get_config(&collection),
            collection,
            update_authority: get_update_authority(&collection),
            reward_mint: get_reward_mint(&collection),
            token_program: TOKEN_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: nft_staking::instruction::Initialize {
            rewards_bps: reward_bps,
            freeze_preriod: freeze_period,
        }
        .data(),
    }
}

pub fn stake_ix(
    owner: Pubkey,
    collection: Pubkey,
    asset: Pubkey,
) -> Instruction {
    Instruction {
        program_id: nft_staking::ID,
        accounts: nft_staking::accounts::Stake {
            owner,
            config: get_config(&collection),
            collection,
            asset,
            update_authority: get_update_authority(&collection),
            system_program: SYSTEM_PROGRAM_ID,
            mpl_core_program: MPL_CORE_ID,
        }
        .to_account_metas(None),
        data: nft_staking::instruction::Stake {}.data(),
    }
}


pub fn unstake_ix(
    owner: Pubkey,
    collection: Pubkey,
    asset: Pubkey,
) -> Instruction {
    let reward_mint = get_reward_mint(&collection);

    Instruction {
        program_id: nft_staking::ID,
        accounts: nft_staking::accounts::UnStake {
            owner,
            config: get_config(&collection),
            collection,
            asset,
            update_authority: get_update_authority(&collection),
            reward_mint,
            user_reward_ata: associated_token::get_associated_token_address(
                &owner,
                &reward_mint,
            ),
            associated_token_program: associated_token::ID,
            token_program: TOKEN_ID,
            system_program: SYSTEM_PROGRAM_ID,
            mpl_core_program: MPL_CORE_ID,
        }
        .to_account_metas(None),
        data: nft_staking::instruction::Unstake {}.data(),
    }
}


pub fn claim_ix(
    owner: Pubkey,
    collection: Pubkey,
    asset: Pubkey,
) -> Instruction {
    let reward_mint = get_reward_mint(&collection);

    Instruction {
        program_id: nft_staking::ID,
        accounts: nft_staking::accounts::Claim {
            owner,
            config: get_config(&collection),
            collection,
            asset,
            update_authority: get_update_authority(&collection),
            reward_mint,
            user_reward_ata: associated_token::get_associated_token_address(
                &owner,
                &reward_mint,
            ),
            associated_token_program: associated_token::ID,
            token_program: TOKEN_ID,
            system_program: SYSTEM_PROGRAM_ID,
            mpl_core_program: MPL_CORE_ID,
        }
        .to_account_metas(None),
        data: nft_staking::instruction::Claim {}.data(),
    }
}