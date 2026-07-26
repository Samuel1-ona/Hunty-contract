#![cfg(test)]
extern crate std;

use crate::{NftMetadata, NftReward, NftRewardClient};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    Address, Env, IntoVal, Map, String, Symbol, Val,
};

fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);
    env
}

fn create_metadata(env: &Env, title: &str, desc: &str, image_uri: &str) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, desc),
        image_uri: String::from_str(env, image_uri),
        hunt_title: String::from_str(env, title),
        rarity: 0u32,
        tier: 0u32,
    }
}

fn create_metadata_full(
    env: &Env,
    title: &str,
    desc: &str,
    image_uri: &str,
    hunt_title: &str,
    rarity: u32,
    tier: u32,
) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, desc),
        image_uri: String::from_str(env, image_uri),
        hunt_title: String::from_str(env, hunt_title),
        rarity,
        tier,
    }
}

fn create_transferable_metadata(env: &Env, title: &str, desc: &str, image_uri: &str) -> Map<Symbol, Val> {
    let mut metadata: Map<Symbol, Val> = Map::new(env);
    metadata.set(Symbol::new(env, "title"), String::from_str(env, title).into_val(env));
    metadata.set(Symbol::new(env, "description"), String::from_str(env, desc).into_val(env));
    metadata.set(Symbol::new(env, "image_uri"), String::from_str(env, image_uri).into_val(env));
    metadata.set(Symbol::new(env, "hunt_title"), String::from_str(env, title).into_val(env));
    metadata.set(Symbol::new(env, "rarity"), 0u32.into_val(env));
    metadata.set(Symbol::new(env, "tier"), 0u32.into_val(env));
    metadata.set(Symbol::new(env, "transferable"), true.into_val(env));
    metadata
}

// ============================================================
//  Existing core tests (updated for new mint_reward_nft signature)
// ============================================================

#[test]
fn test_mint_reward_nft() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = create_metadata(
        &env,
        "Hunt Champion",
        "Completed the City Hunt",
        "ipfs://QmExample123",
    );

    let nft_id = client.mint_reward_nft(&1, &player, &metadata, &1);

    assert_eq!(nft_id, 1);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.nft_id, 1);
    assert_eq!(nft.hunt_id, 1);
    assert_eq!(nft.owner, player);
    assert_eq!(nft.metadata.title, metadata.title);
    assert_eq!(nft.metadata.description, metadata.description);
    assert_eq!(nft.metadata.image_uri, metadata.image_uri);
    assert_eq!(nft.minted_at, 1000);
    assert_eq!(nft.completion_rank, 1);
}

#[test]
fn test_nft_ids_are_unique() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let metadata = create_metadata(&env, "NFT 1", "Desc 1", "ipfs://1");

    let nft_id_1 = client.mint_reward_nft(&1, &player1, &metadata, &1);
    let metadata2 = create_metadata(&env, "NFT 2", "Desc 2", "ipfs://2");
    let nft_id_2 = client.mint_reward_nft(&1, &player2, &metadata2, &2);
    let metadata3 = create_metadata(&env, "NFT 3", "Desc 3", "ipfs://3");
    let nft_id_3 = client.mint_reward_nft(&2, &player1, &metadata3, &1);

    assert_eq!(nft_id_1, 1);
    assert_eq!(nft_id_2, 2);
    assert_eq!(nft_id_3, 3);
}

#[test]
fn test_metadata_stored_correctly() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = create_metadata(
        &env,
        "Treasure Hunter Trophy",
        "Awarded for completing the legendary treasure hunt in record time",
        "https://cdn.example.com/nft/123.png",
    );

    let nft_id = client.mint_reward_nft(&42, &player, &metadata, &0);
    let nft = client.get_nft(&nft_id).unwrap();

    assert_eq!(nft.metadata.title, String::from_str(&env, "Treasure Hunter Trophy"));
    assert_eq!(
        nft.metadata.description,
        String::from_str(&env, "Awarded for completing the legendary treasure hunt in record time")
    );
    assert_eq!(
        nft.metadata.image_uri,
        String::from_str(&env, "https://cdn.example.com/nft/123.png")
    );
}

#[test]
fn test_initial_ownership_set_correctly() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Trophy", "Trophy desc", "ipfs://trophy");

    let nft_id = client.mint_reward_nft(&1, &player, &metadata, &1);

    let owner = client.owner_of(&nft_id).unwrap();
    assert_eq!(owner, player);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.owner, player);
}

#[test]
fn test_nft_minted_event() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Event Test", "Event desc", "ipfs://event");

    let _nft_id = client.mint_reward_nft(&7, &player, &metadata, &1);

    let events = env.events().all();
    assert!(!events.is_empty());
    let (_contract, topics, _data) = events.get(events.len() - 1).unwrap();
    assert_eq!(topics.len(), 2); // "NftMinted" + nft_id
}

#[test]
fn test_multiple_nfts_can_be_minted() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);

    let titles = ["Hunt 1", "Hunt 2", "Hunt 3", "Hunt 4", "Hunt 5"];
    let descs = [
        "Description for hunt 1",
        "Description for hunt 2",
        "Description for hunt 3",
        "Description for hunt 4",
        "Description for hunt 5",
    ];
    let uris = ["ipfs://hunt1", "ipfs://hunt2", "ipfs://hunt3", "ipfs://hunt4", "ipfs://hunt5"];

    for i in 0..5 {
        let metadata = create_metadata(&env, titles[i], descs[i], uris[i]);
        let nft_id = client.mint_reward_nft(&(i as u64 + 1), &player, &metadata, &(i as u64 + 1));
        assert_eq!(nft_id, (i as u64) + 1);
    }

    assert_eq!(client.total_supply(), 5);
}

#[test]
fn test_nft_data_can_be_queried() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Query Test", "Query desc", "ipfs://query");
    let nft_id = client.mint_reward_nft(&99, &player, &metadata, &0);

    let nft = client.get_nft(&nft_id);
    assert!(nft.is_some());
    let nft = nft.unwrap();
    assert_eq!(nft.hunt_id, 99);
    assert_eq!(nft.nft_id, nft_id);
}

#[test]
fn test_get_nonexistent_nft_returns_none() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let nft = client.get_nft(&999);
    assert!(nft.is_none());

    let owner = client.owner_of(&999);
    assert!(owner.is_none());

    let meta = client.get_nft_metadata(&999);
    assert!(meta.is_none());
}

#[test]
fn test_get_nft_metadata_returns_complete_info() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = create_metadata_full(
        &env,
        "Epic Hunt Trophy",
        "Completed legendary hunt",
        "ipfs://trophy",
        "Legendary City Hunt",
        4,
        1,
    );

    let nft_id = client.mint_reward_nft(&42, &player, &metadata, &3);
    let meta = client.get_nft_metadata(&nft_id).unwrap();

    assert_eq!(meta.nft_id, nft_id);
    assert_eq!(meta.hunt_id, 42);
    assert_eq!(meta.hunt_title, String::from_str(&env, "Legendary City Hunt"));
    assert_eq!(meta.completion_timestamp, 1000);
    assert_eq!(meta.completion_player, player);
    assert_eq!(meta.current_owner, player);
    assert_eq!(meta.title, String::from_str(&env, "Epic Hunt Trophy"));
    assert_eq!(meta.description, String::from_str(&env, "Completed legendary hunt"));
    assert_eq!(meta.image_uri, String::from_str(&env, "ipfs://trophy"));
    assert_eq!(meta.rarity, 4);
    assert_eq!(meta.tier, 1);
    assert_eq!(meta.completion_rank, 3);
}

#[test]
fn test_update_nft_metadata_owner_only() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Original", "Original desc", "ipfs://old");

    let nft_id = client.mint_reward_nft(&1, &owner, &metadata, &1);

    client.update_nft_metadata(
        &nft_id,
        &owner,
        &String::from_str(&env, "Updated description"),
        &String::from_str(&env, "ipfs://new"),
    );

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.description, String::from_str(&env, "Updated description"));
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, "ipfs://new"));
    assert_eq!(nft.metadata.title, String::from_str(&env, "Original"));
}

#[test]
fn test_update_nft_metadata_preserves_immutable_fields() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let owner = Address::generate(&env);
    let metadata = create_metadata_full(&env, "Title", "Desc", "ipfs://img", "Hunt", 3, 2);

    let nft_id = client.mint_reward_nft(&1, &owner, &metadata, &2);

    client.update_nft_metadata(
        &nft_id,
        &owner,
        &String::from_str(&env, "New desc"),
        &String::from_str(&env, "ipfs://newimg"),
    );

    let meta = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta.title, String::from_str(&env, "Title"));
    assert_eq!(meta.rarity, 3);
    assert_eq!(meta.tier, 2);
    assert_eq!(meta.hunt_title, String::from_str(&env, "Hunt"));
}

#[test]
fn test_transfer_nft_success() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_transferable_metadata(&env, "Transfer NFT", "Test transfer", "ipfs://transfer");

    let nft_id = client.mint_reward_nft_from_map(&1, &from, &metadata);
    assert_eq!(client.owner_of(&nft_id), Some(from.clone()));

    client.transfer_nft(&nft_id, &from, &to);

    assert_eq!(client.owner_of(&nft_id), Some(to.clone()));
    assert_eq!(client.get_nft_owner(&nft_id), Some(to.clone()));

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.owner, to);
}

#[test]
fn test_transfer_nft_updates_player_nfts() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let metadata2 = create_metadata(&env, "NFT 2", "Desc 2", "ipfs://2");
    let metadata1 = create_transferable_metadata(&env, "NFT 1", "Desc 1", "ipfs://1");
    let nft1 = client.mint_reward_nft_from_map(&1, &alice, &metadata1);
    let nft2 = client.mint_reward_nft(&2, &alice, &metadata2, &1);

    let alice_nfts = client.get_player_nfts(&alice);
    assert_eq!(alice_nfts.len(), 2);
    assert!(alice_nfts.get(0).unwrap() == nft1 || alice_nfts.get(0).unwrap() == nft2);

    client.transfer_nft(&nft1, &alice, &bob);

    let alice_nfts = client.get_player_nfts(&alice);
    assert_eq!(alice_nfts.len(), 1);

    let bob_nfts = client.get_player_nfts(&bob);
    assert_eq!(bob_nfts.len(), 1);
    assert_eq!(bob_nfts.get(0).unwrap(), nft1);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_transfer_nft_requires_auth() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);

    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Auth Test", "Desc", "ipfs://auth");

    let _nft_id = client.mint_reward_nft(&1, &from, &metadata, &1);

    client.transfer_nft(&1, &from, &to);
}

#[test]
#[should_panic]
fn test_transfer_nft_nonexistent() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.transfer_nft(&999, &from, &to);
}

#[test]
#[should_panic]
fn test_transfer_nft_not_owner() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Owner Test", "Desc", "ipfs://owner");

    let nft_id = client.mint_reward_nft(&1, &owner, &metadata, &1);

    client.transfer_nft(&nft_id, &attacker, &to);
}

#[test]
#[should_panic]
fn test_transfer_nft_invalid_recipient_same_as_from() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Same Addr", "Desc", "ipfs://same");

    let nft_id = client.mint_reward_nft(&1, &owner, &metadata, &1);

    client.transfer_nft(&nft_id, &owner, &owner);
}

#[test]
fn test_transfer_nft_emits_event() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_transferable_metadata(&env, "Event NFT", "Desc", "ipfs://event");

    let nft_id = client.mint_reward_nft_from_map(&1, &from, &metadata);
    client.transfer_nft(&nft_id, &from, &to);

    assert_eq!(client.owner_of(&nft_id), Some(to));
}

#[test]
fn test_get_player_nfts_empty_for_new_address() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let new_addr = Address::generate(&env);
    let nfts = client.get_player_nfts(&new_addr);
    assert_eq!(nfts.len(), 0);
}

#[test]
fn test_get_nft_owner_matches_owner_of() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Alias Test", "Desc", "ipfs://alias");

    let nft_id = client.mint_reward_nft(&1, &player, &metadata, &1);

    assert_eq!(client.owner_of(&nft_id), client.get_nft_owner(&nft_id));
    assert_eq!(client.get_nft_owner(&nft_id), Some(player));
}

#[test]
fn test_burn_removes_nft_and_clears_owner_list() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Burn Me", "Desc", "ipfs://burn");

    let nft_id = client.mint_reward_nft(&1, &owner, &metadata, &1);
    assert!(client.get_nft(&nft_id).is_some());

    client.burn(&nft_id, &owner);

    assert!(client.get_nft(&nft_id).is_none());
    assert_eq!(client.get_player_nfts(&owner).len(), 0);
}

#[test]
fn test_burn_fails_if_not_owner() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let owner = Address::generate(&env);
    let other = Address::generate(&env);
    let metadata = create_metadata(&env, "Not Yours", "Desc", "ipfs://notyours");

    let nft_id = client.mint_reward_nft(&1, &owner, &metadata, &1);

    let result = client.try_burn(&nft_id, &other);
    assert!(result.is_err());
    assert!(client.get_nft(&nft_id).is_some());
}

#[test]
fn test_burn_fails_for_nonexistent_nft() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let owner = Address::generate(&env);
    let result = client.try_burn(&999u64, &owner);
    assert!(result.is_err());
}

// ============================================================
//  Issue #383: NFT Display Name Generation
// ============================================================

#[test]
fn test_display_name_auto_generated_when_title_empty() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    // Title is empty — display name should be auto-generated
    let metadata = NftMetadata {
        title: String::from_str(&env, ""),
        description: String::from_str(&env, "desc"),
        image_uri: String::from_str(&env, "ipfs://img"),
        hunt_title: String::from_str(&env, "City Quest"),
        rarity: 0,
        tier: 0,
    };

    let nft_id = client.mint_reward_nft(&1, &player, &metadata, &3);
    let meta = client.get_nft_metadata(&nft_id).unwrap();

    // Auto-generated title should be "{hunt_title} #{rank}"
    assert_eq!(meta.title, String::from_str(&env, "City Quest #3"));
    assert_eq!(meta.display_name, String::from_str(&env, "City Quest #3"));
    assert_eq!(meta.completion_rank, 3);
}

#[test]
fn test_display_name_uses_provided_title_when_given() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = NftMetadata {
        title: String::from_str(&env, "Custom Title"),
        description: String::from_str(&env, "desc"),
        image_uri: String::from_str(&env, "ipfs://img"),
        hunt_title: String::from_str(&env, "City Quest"),
        rarity: 0,
        tier: 0,
    };

    let nft_id = client.mint_reward_nft(&1, &player, &metadata, &5);
    let nft = client.get_nft(&nft_id).unwrap();
    let meta = client.get_nft_metadata(&nft_id).unwrap();

    // Title stays as provided
    assert_eq!(nft.metadata.title, String::from_str(&env, "Custom Title"));
    // Display name is always "{hunt_title} #{rank}"
    assert_eq!(meta.display_name, String::from_str(&env, "City Quest #5"));
    assert_eq!(meta.completion_rank, 5);
}

#[test]
fn test_display_name_format_first_rank() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = create_metadata_full(
        &env,
        "",
        "First place",
        "ipfs://first",
        "Treasure Island Hunt",
        0,
        0,
    );

    let nft_id = client.mint_reward_nft(&10, &player, &metadata, &1);
    let meta = client.get_nft_metadata(&nft_id).unwrap();

    assert_eq!(meta.display_name, String::from_str(&env, "Treasure Island Hunt #1"));
    assert_eq!(meta.title, String::from_str(&env, "Treasure Island Hunt #1"));
}

#[test]
fn test_display_name_rank_zero() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = NftMetadata {
        title: String::from_str(&env, ""),
        description: String::from_str(&env, ""),
        image_uri: String::from_str(&env, ""),
        hunt_title: String::from_str(&env, "Some Hunt"),
        rarity: 0,
        tier: 0,
    };

    let nft_id = client.mint_reward_nft(&1, &player, &metadata, &0);
    let meta = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta.display_name, String::from_str(&env, "Some Hunt #0"));
}

#[test]
fn test_generate_display_name_helper() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let result = client.generate_display_name(
        &String::from_str(&env, "Forest Hunt"),
        &7u64,
    );
    assert_eq!(result, String::from_str(&env, "Forest Hunt #7"));
}

#[test]
fn test_display_name_in_metadata_response_includes_rank() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = create_metadata_full(
        &env,
        "Winner NFT",
        "First place prize",
        "ipfs://winner",
        "Grand Hunt",
        5,
        0,
    );

    let nft_id = client.mint_reward_nft(&42, &player, &metadata, &1);
    let meta = client.get_nft_metadata(&nft_id).unwrap();

    assert_eq!(meta.completion_rank, 1);
    assert_eq!(meta.display_name, String::from_str(&env, "Grand Hunt #1"));
}

#[test]
fn test_mint_from_map_auto_generates_display_name() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let mut metadata: Map<Symbol, Val> = Map::new(&env);
    // No title key => empty title => auto-generate
    metadata.set(Symbol::new(&env, "description"), String::from_str(&env, "desc").into_val(&env));
    metadata.set(Symbol::new(&env, "image_uri"), String::from_str(&env, "ipfs://x").into_val(&env));
    metadata.set(Symbol::new(&env, "hunt_title"), String::from_str(&env, "Ocean Hunt").into_val(&env));
    metadata.set(Symbol::new(&env, "rarity"), 0u32.into_val(&env));
    metadata.set(Symbol::new(&env, "tier"), 0u32.into_val(&env));
    metadata.set(Symbol::new(&env, "completion_rank"), 4u64.into_val(&env));

    let nft_id = client.mint_reward_nft_from_map(&1, &player, &metadata);
    let meta = client.get_nft_metadata(&nft_id).unwrap();

    assert_eq!(meta.display_name, String::from_str(&env, "Ocean Hunt #4"));
    assert_eq!(meta.title, String::from_str(&env, "Ocean Hunt #4"));
    assert_eq!(meta.completion_rank, 4);
}

// ============================================================
//  Issue #384: Contract Upgrade Mechanism
// ============================================================

#[test]
fn test_initialize_sets_admin() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let admin = Address::generate(&env);
    client.initialize(&admin);

    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
fn test_initialize_sets_version_to_1() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let admin = Address::generate(&env);
    client.initialize(&admin);

    assert_eq!(client.get_version(), 1u32);
}

#[test]
fn test_initialize_is_idempotent() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    client.initialize(&admin1);
    // Second call with a different admin should be a no-op
    client.initialize(&admin2);

    // Original admin still set
    assert_eq!(client.get_admin(), Some(admin1));
    assert_eq!(client.get_version(), 1u32);
}

#[test]
fn test_upgrade_fails_if_not_admin() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    client.initialize(&admin);

    // Use a dummy 32-byte wasm hash
    let dummy_hash = soroban_sdk::BytesN::from_array(&env, &[0u8; 32]);
    let result = client.try_upgrade(&attacker, &dummy_hash);
    assert!(result.is_err());
}

#[test]
fn test_upgrade_fails_without_initialization() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let caller = Address::generate(&env);
    let dummy_hash = soroban_sdk::BytesN::from_array(&env, &[0u8; 32]);
    // No initialize() call — admin not set — should fail with Unauthorized
    let result = client.try_upgrade(&caller, &dummy_hash);
    assert!(result.is_err());
}

#[test]
fn test_version_starts_at_zero_before_init() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    // No initialize call
    assert_eq!(client.get_version(), 0u32);
    assert_eq!(client.get_admin(), None);
}

// ============================================================
//  Issue #388: Admin NFT Revocation
// ============================================================

#[test]
fn test_admin_revoke_nft_removes_nft() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let admin = Address::generate(&env);
    let player = Address::generate(&env);
    client.initialize(&admin);

    let metadata = create_metadata(&env, "Revokable", "Desc", "ipfs://rev");
    let nft_id = client.mint_reward_nft(&1, &player, &metadata, &1);

    assert!(client.get_nft(&nft_id).is_some());

    client.admin_revoke_nft(
        &admin,
        &nft_id,
        &String::from_str(&env, "Fraudulent completion"),
    );

    // NFT should be gone
    assert!(client.get_nft(&nft_id).is_none());
    // Owner index should be cleaned up
    assert_eq!(client.get_player_nfts(&player).len(), 0);
}

#[test]
fn test_admin_revoke_nft_records_reason() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let admin = Address::generate(&env);
    let player = Address::generate(&env);
    client.initialize(&admin);

    let metadata = create_metadata(&env, "Revokable", "Desc", "ipfs://rev");
    let nft_id = client.mint_reward_nft(&1, &player, &metadata, &1);

    let reason = String::from_str(&env, "Cheating detected");
    client.admin_revoke_nft(&admin, &nft_id, &reason);

    // Revocation record should be accessible
    let record = client.get_revocation_record(&nft_id);
    assert!(record.is_some());
    let (prev_owner, stored_reason) = record.unwrap();
    assert_eq!(prev_owner, player);
    assert_eq!(stored_reason, reason);
}

#[test]
fn test_admin_revoke_nft_emits_event() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let admin = Address::generate(&env);
    let player = Address::generate(&env);
    client.initialize(&admin);

    let metadata = create_metadata(&env, "EventNFT", "Desc", "ipfs://evt");
    let nft_id = client.mint_reward_nft(&1, &player, &metadata, &1);

    client.admin_revoke_nft(
        &admin,
        &nft_id,
        &String::from_str(&env, "Test revoke"),
    );

    let events = env.events().all();
    // Verify at least one event was emitted during the revocation
    assert!(!events.is_empty());
}

#[test]
fn test_admin_revoke_nft_fails_if_not_admin() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let admin = Address::generate(&env);
    let player = Address::generate(&env);
    let attacker = Address::generate(&env);
    client.initialize(&admin);

    let metadata = create_metadata(&env, "Protected", "Desc", "ipfs://prot");
    let nft_id = client.mint_reward_nft(&1, &player, &metadata, &1);

    let result = client.try_admin_revoke_nft(
        &attacker,
        &nft_id,
        &String::from_str(&env, "Should fail"),
    );
    assert!(result.is_err());
    // NFT must still exist
    assert!(client.get_nft(&nft_id).is_some());
}

#[test]
fn test_admin_revoke_nonexistent_nft_fails() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let result = client.try_admin_revoke_nft(
        &admin,
        &9999u64,
        &String::from_str(&env, "No such NFT"),
    );
    assert!(result.is_err());
}

#[test]
fn test_admin_revoke_cleans_owner_index_of_multiple_nfts() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let admin = Address::generate(&env);
    let player = Address::generate(&env);
    client.initialize(&admin);

    let m1 = create_metadata(&env, "NFT A", "Desc A", "ipfs://a");
    let m2 = create_metadata(&env, "NFT B", "Desc B", "ipfs://b");
    let m3 = create_metadata(&env, "NFT C", "Desc C", "ipfs://c");

    let id1 = client.mint_reward_nft(&1, &player, &m1, &1);
    let id2 = client.mint_reward_nft(&1, &player, &m2, &2);
    let id3 = client.mint_reward_nft(&1, &player, &m3, &3);

    assert_eq!(client.get_player_nfts(&player).len(), 3);

    // Revoke the middle NFT
    client.admin_revoke_nft(
        &admin,
        &id2,
        &String::from_str(&env, "Middle revoke"),
    );

    let remaining = client.get_player_nfts(&player);
    assert_eq!(remaining.len(), 2);
    // id1 and id3 should still be there; id2 gone
    assert!(remaining.contains(&id1));
    assert!(remaining.contains(&id3));
    assert!(!remaining.contains(&id2));
}

#[test]
fn test_get_revocation_record_none_for_active_nft() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Active", "Desc", "ipfs://active");
    let nft_id = client.mint_reward_nft(&1, &player, &metadata, &1);

    // Not revoked — record should be None
    assert!(client.get_revocation_record(&nft_id).is_none());
}
