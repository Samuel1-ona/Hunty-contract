//! Regression tests for the indexed global all-NFT index (#1091).
//!
//! Run with:
//!
//! ```text
//! cargo test -p nft-reward --test global_nft_index
//! ```
//!
//! These live in an integration target rather than the in-crate `src/test.rs`
//! module so they stay runnable on their own — `src/test.rs` is mid-refactor and
//! does not compile against the Soroban SDK 28 client API yet.
//!
//! The global index used to live in one `ALLNFT` `Vec<u64>` entry that every
//! `Storage::save_nft` call loaded and scanned with `first_index_of`, so any
//! write (transfer, extension edit, metadata refresh) cost O(total supply) and
//! the single entry eventually exceeded the ledger entry size limit. These
//! tests pin the replacement contract: the index is per-position storage
//! entries, and it only grows on mint.

#![allow(deprecated)]

use nft_reward::{CollectionMetadata, NftMetadata, NftReward, NftRewardClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env, IntoVal, Map, String, Symbol, Val,
};

/// A 46-character IPFS CID is the shortest URI `image_uri_is_valid` accepts.
const IMAGE_URI: &str = "ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG";

fn metadata(env: &Env, title: &str) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, "Reward for completing a hunt"),
        image_uri: String::from_str(env, IMAGE_URI),
        hunt_title: String::from_str(env, title),
        rarity: 0,
        tier: 0,
        creator: None,
        royalty_bps: None,
        extensions: Map::new(env),
    }
}

fn setup<'a>(env: &'a Env) -> (NftRewardClient<'a>, Address) {
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
    (client, minter)
}

/// Mints through the map entrypoint with `transferable = true` so the transfer
/// flow below can be exercised.
fn mint_transferable(
    env: &Env,
    client: &NftRewardClient<'_>,
    minter: &Address,
    hunt_id: u64,
    owner: &Address,
    title: &str,
) -> u64 {
    let mut map: Map<Symbol, Val> = Map::new(env);
    map.set(
        Symbol::new(env, "title"),
        String::from_str(env, title).into_val(env),
    );
    map.set(
        Symbol::new(env, "description"),
        String::from_str(env, "Reward for completing a hunt").into_val(env),
    );
    map.set(
        Symbol::new(env, "image_uri"),
        String::from_str(env, IMAGE_URI).into_val(env),
    );
    map.set(Symbol::new(env, "transferable"), true.into_val(env));
    client.mint_reward_nft_from_map(minter, &hunt_id, owner, &map)
}

/// Reads the global index through its public read path (`list_all_nfts`).
fn listed_ids(client: &NftRewardClient<'_>) -> Vec<u64> {
    let listed = client.list_all_nfts(&0, &200);
    let mut ids = Vec::new();
    for i in 0..listed.len() {
        ids.push(listed.get(i).unwrap().nft_id);
    }
    ids
}

#[test]
fn index_records_every_minted_nft_in_mint_order() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);
    let (client, minter) = setup(&env);
    let owner = Address::generate(&env);

    let id1 = client.mint_reward_nft(&minter, &1, &owner, &metadata(&env, "Hunt 1 reward"));
    let id2 = client.mint_reward_nft(&minter, &1, &owner, &metadata(&env, "Hunt 1 reward"));
    let id3 = client.mint_reward_nft(&minter, &2, &owner, &metadata(&env, "Hunt 2 reward"));

    assert_eq!(listed_ids(&client), vec![id1, id2, id3]);
    assert_eq!(client.get_hunt_nft_count(&1), 2);
    assert_eq!(client.get_hunt_nft_count(&2), 1);
}

#[test]
fn index_is_not_touched_by_extension_edits_or_transfers() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);
    let (client, minter) = setup(&env);
    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);

    let id1 = client.mint_reward_nft(&minter, &1, &owner, &metadata(&env, "Hunt 1 reward"));
    let id2 = mint_transferable(&env, &client, &minter, 1, &owner, "Hunt 1 reward");

    // Both writes below go through `Storage::save_nft` for an existing NFT.
    client.set_nft_extension(
        &id1,
        &owner,
        &String::from_str(&env, "badge"),
        &String::from_str(&env, "gold"),
    );
    client.transfer_nft(&id2, &owner, &recipient, &owner);

    // ...and neither of them may append to (or disturb) the global index.
    assert_eq!(listed_ids(&client), vec![id1, id2]);
    assert_eq!(client.get_nft(&id1).unwrap().metadata.extensions.len(), 1);
    assert_eq!(client.owner_of(&id2).unwrap(), recipient);
}

#[test]
fn burning_swaps_the_last_index_entry_and_keeps_the_rest_discoverable() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);
    let (client, minter) = setup(&env);
    let owner = Address::generate(&env);

    let id1 = client.mint_reward_nft(&minter, &1, &owner, &metadata(&env, "Hunt 1 reward"));
    let id2 = client.mint_reward_nft(&minter, &1, &owner, &metadata(&env, "Hunt 1 reward"));
    let id3 = client.mint_reward_nft(&minter, &1, &owner, &metadata(&env, "Hunt 1 reward"));

    client.burn_nft(&id2, &owner);

    // Swap-remove: the last entry takes the freed position, nothing is lost.
    assert_eq!(listed_ids(&client), vec![id1, id3]);
    assert!(client.get_nft(&id2).is_none());
    // total_supply tracks the live count, so burning one leaves two.
    assert_eq!(client.total_supply(), 2);
    assert_eq!(client.get_hunt_nft_count(&1), 2);

    // The freed slot is reused rather than skipped by the next mint.
    let id4 = client.mint_reward_nft(&minter, &1, &owner, &metadata(&env, "Hunt 1 reward"));
    assert_eq!(listed_ids(&client), vec![id1, id3, id4]);
}

#[test]
fn indexed_global_index_supports_offset_paging() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);
    let (client, minter) = setup(&env);
    let owner = Address::generate(&env);

    let id1 = client.mint_reward_nft(&minter, &1, &owner, &metadata(&env, "Hunt 1 reward"));
    let id2 = client.mint_reward_nft(&minter, &2, &owner, &metadata(&env, "Hunt 2 reward"));
    let id3 = client.mint_reward_nft(&minter, &3, &owner, &metadata(&env, "Hunt 3 reward"));

    let page = client.list_all_nfts(&1, &1);
    assert_eq!(page.len(), 1);
    assert_eq!(page.get(0).unwrap().nft_id, id2);

    let tail = client.list_all_nfts(&2, &5);
    assert_eq!(tail.len(), 1);
    assert_eq!(tail.get(0).unwrap().nft_id, id3);

    assert_eq!(client.list_all_nfts(&3, &5).len(), 0);
    // Guards against the index silently reordering: full list is still 1,2,3.
    assert_eq!(listed_ids(&client), vec![id1, id2, id3]);
}
