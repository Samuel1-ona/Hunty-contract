//! Regression tests for `NftMintedEvent.total_minted_for_hunt` (#1093).
//!
//! Run with:
//!
//! ```text
//! cargo test -p nft-reward --test nft_minted_event_per_hunt
//! ```
//!
//! `total_minted_for_hunt` used to be set to `Storage::get_nft_counter`, i.e.
//! the collection-wide supply, so a mint for a brand new hunt advertised the
//! total number of NFTs in the collection instead of the number minted for that
//! hunt. These tests assert the event now reports the per-hunt total that
//! `get_hunt_nft_count` exposes.
//!
//! They live in an integration target rather than the in-crate `src/test.rs`
//! module because that module is mid-refactor and does not compile against the
//! Soroban SDK 28 client API.

#![allow(deprecated)]

use nft_reward::{CollectionMetadata, NftMintedEvent, NftMetadata, NftReward, NftRewardClient};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    vec, Address, Env, IntoVal, Map, String, Symbol,
};

/// A 46-character IPFS CID is the shortest URI `image_uri_is_valid` accepts.
const IMAGE_URI: &str = "ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG";

fn metadata(env: &Env, hunt_title: &str) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, hunt_title),
        description: String::from_str(env, "Reward for completing a hunt"),
        image_uri: String::from_str(env, IMAGE_URI),
        hunt_title: String::from_str(env, hunt_title),
        rarity: 0,
        tier: 0,
        creator: None,
        royalty_bps: None,
        extensions: Map::new(env),
    }
}

fn setup<'a>(env: &'a Env) -> (Address, NftRewardClient<'a>, Address) {
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let minter = Address::generate(env);
    let collection = CollectionMetadata {
        name: String::from_str(env, "Hunty Rewards"),
        description: String::from_str(env, "Reward NFTs for completed hunts"),
        total_supply: 0,
        creator: None,
    };
    client.initialize(&admin, &minter, &None, &collection);
    (contract_id, client, minter)
}

/// Mints one NFT for `hunt_id` and asserts the emitted `NftMintedEvent` in full,
/// including the `total_minted_for_hunt` the caller expects.
fn mint_and_assert_event(
    env: &Env,
    contract_id: &Address,
    client: &NftRewardClient<'_>,
    minter: &Address,
    hunt_id: u64,
    owner: &Address,
    hunt_title: &str,
    expected_total_for_hunt: u32,
) -> u64 {
    let expected_nft_id = client.total_supply() + 1;
    let nft_id = client.mint_reward_nft(minter, &hunt_id, owner, &metadata(env, hunt_title));
    assert_eq!(nft_id, expected_nft_id);

    let expected = NftMintedEvent {
        nft_id,
        hunt_id,
        owner: owner.clone(),
        rarity: 0,
        tier: 0,
        minted_at: env.ledger().timestamp(),
        hunt_title: String::from_str(env, hunt_title),
        total_minted_for_hunt: expected_total_for_hunt,
        completion_rank: 0,
        collection_stats: String::from_str(
            env,
            "total_supply=tracked,total_hunts=tracked,total_owners=tracked",
        ),
    };

    // `mint_reward_nft` publishes exactly one event, so this pins the whole
    // payload rather than a single field.
    assert_eq!(
        env.events().all().filter_by_contract(contract_id),
        vec![
            env,
            (
                contract_id.clone(),
                (Symbol::new(env, "NftMinted"), nft_id).into_val(env),
                expected.into_val(env),
            ),
        ]
    );

    nft_id
}

#[test]
fn total_minted_for_hunt_is_scoped_to_the_hunt_not_the_collection() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);
    let (contract_id, client, minter) = setup(&env);
    let owner = Address::generate(&env);

    // Interleaved hunts: a collection-wide counter would report 1, 2, 3, 4 here.
    let first = mint_and_assert_event(&env, &contract_id, &client, &minter, 7, &owner, "Hunt 7", 1);
    let second = mint_and_assert_event(&env, &contract_id, &client, &minter, 8, &owner, "Hunt 8", 1);
    let third = mint_and_assert_event(&env, &contract_id, &client, &minter, 7, &owner, "Hunt 7", 2);
    let fourth = mint_and_assert_event(&env, &contract_id, &client, &minter, 8, &owner, "Hunt 8", 2);

    assert_eq!(client.total_supply(), 4);
    assert_eq!(client.get_hunt_nft_count(&7), 2);
    assert_eq!(client.get_hunt_nft_count(&8), 2);

    assert_ne!(first, second);
    assert_ne!(third, fourth);
}

#[test]
fn first_mint_for_a_fresh_hunt_reports_one_even_after_other_hunts() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);
    let (contract_id, client, minter) = setup(&env);
    let owner = Address::generate(&env);

    for hunt_id in 1..=3u64 {
        mint_and_assert_event(
            &env,
            &contract_id,
            &client,
            &minter,
            hunt_id,
            &owner,
            "Filler hunt",
            1,
        );
    }

    // Hunt 4 is brand new, so its total is 1 even though the collection now
    // holds three NFTs. This is the exact case the old code got wrong.
    mint_and_assert_event(&env, &contract_id, &client, &minter, 4, &owner, "Hunt 4", 1);
    assert_eq!(client.total_supply(), 4);
    assert_eq!(client.get_hunt_nft_count(&4), 1);
}
