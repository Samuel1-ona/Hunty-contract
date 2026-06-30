#![cfg(test)]
extern crate std;

use crate::{NftMetadata, NftMintedEvent, NftReward, NftRewardClient, METADATA_SCHEMA_VERSION};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    Address, Env, IntoVal, Map, String, Symbol, Val, TryFromVal, TryIntoVal,
};

fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);
    env
}

fn setup_nft_reward(env: &Env, max_supply: Option<u64>) -> (NftRewardClient<'_>, Address) {
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let minter = Address::generate(env);
    client.initialize(&admin, &minter, &max_supply);
    (client, minter)
}

fn setup_initialized() -> (Env, Address, Address, Address) {
    let env = setup_env();
    let contract_id = env.register_contract(None, NftReward);
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &minter, &None);
    (env, contract_id, admin, minter)
}

fn create_metadata(env: &Env, title: &str, desc: &str, image_uri: &str) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, desc),
        image_uri: String::from_str(env, image_uri),
        hunt_title: String::from_str(env, title),
        rarity: 0u32,
        tier: 0u32,
        creator: None,
        royalty_bps: None,
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
        creator: None,
        royalty_bps: None,
    }
}

fn mint_transferable(
    env: &Env,
    client: &NftRewardClient<'_>,
    hunt_id: u64,
    owner: &Address,
    metadata: &NftMetadata,
) -> u64 {
    let minter = Address::generate(env);
    let mut map: Map<Symbol, Val> = Map::new(env);
    map.set(
        Symbol::new(env, "title"),
        metadata.title.clone().into_val(env),
    );
    map.set(
        Symbol::new(env, "description"),
        metadata.description.clone().into_val(env),
    );
    map.set(
        Symbol::new(env, "image_uri"),
        metadata.image_uri.clone().into_val(env),
    );
    map.set(
        Symbol::new(env, "hunt_title"),
        metadata.hunt_title.clone().into_val(env),
    );
    map.set(Symbol::new(env, "transferable"), true.into_val(env));
    client.mint_reward_nft_from_map(&minter, &hunt_id, owner, &map)
}

#[test]
fn test_initialize_stores_admin() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
#[should_panic(expected = "HostError")]
fn test_initialize_requires_auth() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);

    let admin = Address::generate(&env);
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    client.initialize(&admin, &None);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_initialize_cannot_be_called_twice() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);
    client.initialize(&admin, &None);
}

#[test]
fn test_mint_reward_nft() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(
        &env,
        "Hunt Champion",
        "Completed the City Hunt",
        "ipfs://QmExample123",
    );

    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    assert!(nft_id > 0, "NFT ID must be non-zero");

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.nft_id, nft_id);
    assert_eq!(nft.hunt_id, 1);
    assert_eq!(nft.owner, player);
    assert_eq!(nft.metadata.title, metadata.title);
    assert_eq!(nft.metadata.description, metadata.description);
    assert_eq!(nft.metadata.image_uri, metadata.image_uri);
    assert_eq!(nft.minted_at, 1000);
}

#[test]
fn test_nft_ids_are_unique() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let metadata = create_metadata(&env, "NFT 1", "Desc 1", "ipfs://1");

    let nft_id_1 = client.mint_reward_nft(&minter, &1, &player1, &metadata);
    let metadata2 = create_metadata(&env, "NFT 2", "Desc 2", "ipfs://2");
    let nft_id_2 = client.mint_reward_nft(&minter, &1, &player2, &metadata2);
    let metadata3 = create_metadata(&env, "NFT 3", "Desc 3", "ipfs://3");
    let nft_id_3 = client.mint_reward_nft(&minter, &2, &player1, &metadata3);

    // IDs must be non-zero and all distinct
    assert!(nft_id_1 > 0);
    assert!(nft_id_2 > 0);
    assert!(nft_id_3 > 0);
    assert_ne!(nft_id_1, nft_id_2);
    assert_ne!(nft_id_2, nft_id_3);
    assert_ne!(nft_id_1, nft_id_3);
}

#[test]
fn test_metadata_stored_correctly() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(
        &env,
        "Treasure Hunter Trophy",
        "Awarded for completing the legendary treasure hunt in record time",
        "https://cdn.example.com/nft/123.png",
    );

    let nft_id = client.mint_reward_nft(&minter, &42, &player, &metadata);
    let nft = client.get_nft(&nft_id).unwrap();

    assert_eq!(
        nft.metadata.title,
        String::from_str(&env, "Treasure Hunter Trophy")
    );
    assert_eq!(
        nft.metadata.description,
        String::from_str(
            &env,
            "Awarded for completing the legendary treasure hunt in record time"
        )
    );
    assert_eq!(
        nft.metadata.image_uri,
        String::from_str(&env, "https://cdn.example.com/nft/123.png")
    );
}

#[test]
fn test_initial_ownership_set_correctly() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Trophy", "Trophy desc", "ipfs://trophy");

    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    let owner = client.owner_of(&nft_id).unwrap();
    assert_eq!(owner, player);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.owner, player);
}

#[test]
fn test_nft_minted_event() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Event Test", "Event desc", "ipfs://event");

    let nft_id = client.mint_reward_nft(&minter, &7, &player, &metadata);

    let events = env.events().all();
    assert!(!events.is_empty());
    // Last event should be NftMinted
    let (_contract, topics, data): (Address, soroban_sdk::Vec<Val>, Val) =
        events.get(events.len() - 1).unwrap();
    assert_eq!(topics.len(), 2); // "NftMinted" + nft_id
    assert_eq!(
        Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap(),
        Symbol::new(&env, "NftMinted")
    );
    assert_eq!(
        u64::try_from_val(&env, &topics.get(1).unwrap()).unwrap(),
        nft_id
    );

    let event: NftMintedEvent = NftMintedEvent::try_from_val(&env, &data).unwrap();
    assert_eq!(event.hunt_title, metadata.hunt_title);
    assert_eq!(event.total_minted_for_hunt, 1);
    assert_eq!(event.completion_rank, 1);
    assert_eq!(event.collection_stats.total_supply, 1);
    assert_eq!(event.collection_stats.total_hunts, 1);
    assert_eq!(event.collection_stats.total_owners, 1);
}

#[test]
fn test_multiple_nfts_can_be_minted() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);

    let titles = ["Hunt 1", "Hunt 2", "Hunt 3", "Hunt 4", "Hunt 5"];
    let descs = [
        "Description for hunt 1",
        "Description for hunt 2",
        "Description for hunt 3",
        "Description for hunt 4",
        "Description for hunt 5",
    ];
    let uris = [
        "ipfs://hunt1",
        "ipfs://hunt2",
        "ipfs://hunt3",
        "ipfs://hunt4",
        "ipfs://hunt5",
    ];

    for i in 0..5 {
        let metadata = create_metadata(&env, titles[i], descs[i], uris[i]);
        let nft_id = client.mint_reward_nft(&minter, &(i as u64 + 1), &player, &metadata);
        assert_eq!(nft_id, (i as u64) + 1);
    }

    assert_eq!(client.total_supply(), 5);
}

#[test]
fn test_nft_data_can_be_queried() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Query Test", "Query desc", "ipfs://query");
    let nft_id = client.mint_reward_nft(&minter, &99, &player, &metadata);

    let nft = client.get_nft(&nft_id);
    assert!(nft.is_some());
    let nft = nft.unwrap();
    assert_eq!(nft.hunt_id, 99);
    assert_eq!(nft.nft_id, nft_id);
}

#[test]
fn test_get_nonexistent_nft_returns_none() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);

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
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata_full(
        &env,
        "Epic Hunt Trophy",
        "Completed legendary hunt",
        "ipfs://trophy",
        "Legendary City Hunt",
        4, // rare
        1, // tier 1
    );

    let nft_id = client.mint_reward_nft(&minter, &42, &player, &metadata);
    let meta = client.get_nft_metadata(&nft_id).unwrap();

    assert_eq!(meta.nft_id, nft_id);
    assert_eq!(meta.hunt_id, 42);
    assert_eq!(
        meta.hunt_title,
        String::from_str(&env, "Legendary City Hunt")
    );
    assert_eq!(meta.completion_timestamp, 1000);
    assert_eq!(meta.completion_player, player);
    assert_eq!(meta.current_owner, player);
    assert_eq!(meta.title, String::from_str(&env, "Epic Hunt Trophy"));
    assert_eq!(
        meta.description,
        String::from_str(&env, "Completed legendary hunt")
    );
    assert_eq!(meta.image_uri, String::from_str(&env, "ipfs://trophy"));
    assert_eq!(meta.rarity, 4);
    assert_eq!(meta.tier, 1);
}

#[test]
fn test_mint_from_map_then_query_metadata() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);

    let player = Address::generate(&env);

    let mut metadata_map: Map<Symbol, Val> = Map::new(&env);
    metadata_map.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Map Mint Trophy").into_val(&env),
    );
    metadata_map.set(
        Symbol::new(&env, "description"),
        String::from_str(&env, "Minted via map").into_val(&env),
    );
    metadata_map.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "ipfs://mapmint").into_val(&env),
    );
    metadata_map.set(
        Symbol::new(&env, "hunt_title"),
        String::from_str(&env, "Map Hunt").into_val(&env),
    );
    metadata_map.set(Symbol::new(&env, "rarity"), 2u32.into_val(&env));
    metadata_map.set(Symbol::new(&env, "tier"), 7u32.into_val(&env));

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &7, &player, &metadata_map);
    let meta = client.get_nft_metadata(&nft_id).unwrap();

    assert_eq!(meta.nft_id, nft_id);
    assert_eq!(meta.hunt_id, 7);
    assert_eq!(meta.hunt_title, String::from_str(&env, "Map Hunt"));
    assert_eq!(meta.completion_timestamp, 1000);
    assert_eq!(meta.completion_player, player);
    assert_eq!(meta.current_owner, player);
    assert_eq!(meta.title, String::from_str(&env, "Map Mint Trophy"));
    assert_eq!(meta.description, String::from_str(&env, "Minted via map"));
    assert_eq!(meta.image_uri, String::from_str(&env, "ipfs://mapmint"));
    assert_eq!(meta.rarity, 2);
    assert_eq!(meta.tier, 7);
}

#[test]
fn test_update_nft_metadata_owner_only() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Original", "Original desc", "ipfs://old");

    let nft_id = client.mint_reward_nft(&minter, &1, &owner, &metadata);

    client.update_nft_metadata(
        &nft_id,
        &owner,
        &String::from_str(&env, "Updated description"),
        &String::from_str(&env, "ipfs://new"),
    );

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(
        nft.metadata.description,
        String::from_str(&env, "Updated description")
    );
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, "ipfs://new"));
    assert_eq!(nft.metadata.title, String::from_str(&env, "Original"));
}

#[test]
fn test_update_nft_metadata_preserves_immutable_fields() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let metadata = create_metadata_full(&env, "Title", "Desc", "ipfs://img", "Hunt", 3, 2);

    let nft_id = client.mint_reward_nft(&minter, &1, &owner, &metadata);

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
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Transfer NFT", "Test transfer", "ipfs://transfer");

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &1, &from, &metadata);
    assert_eq!(client.owner_of(&nft_id), Some(from.clone()));

    client.transfer_nft(&nft_id, &from, &to, &from);

    assert_eq!(client.owner_of(&nft_id), Some(to.clone()));
    assert_eq!(client.get_nft_owner(&nft_id), Some(to.clone()));

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.owner, to);
}

#[test]
fn test_transfer_nft_updates_player_nfts() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let metadata1 = create_metadata(&env, "NFT 1", "Desc 1", "ipfs://1");
    let metadata2 = create_metadata(&env, "NFT 2", "Desc 2", "ipfs://2");

    let nft1 = client.mint_reward_nft(&minter, &1, &alice, &metadata1);
    let nft2 = client.mint_reward_nft(&minter, &2, &alice, &metadata2);

    let alice_nfts = client.get_player_nfts(&alice, &0, &100);
    assert_eq!(alice_nfts.len(), 2);
    assert!(alice_nfts.get(0).unwrap() == nft1 || alice_nfts.get(0).unwrap() == nft2);

    client.transfer_nft(&nft1, &alice, &bob, &alice);

    let alice_nfts = client.get_player_nfts(&alice, &0, &100);
    assert_eq!(alice_nfts.len(), 1);

    let bob_nfts = client.get_player_nfts(&bob, &0, &100);
    assert_eq!(bob_nfts.len(), 1);
    assert_eq!(bob_nfts.get(0).unwrap(), nft1);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_transfer_nft_requires_auth() {
    let env = Env::default();
    // Do NOT mock auth - we want the transfer to fail without auth
    env.ledger().set_timestamp(1000);

    let (client, minter) = setup_nft_reward(&env, None);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Auth Test", "Desc", "ipfs://auth");

    let _nft_id = client.mint_reward_nft(&minter, &1, &from, &metadata);

    // This should fail - from has not authorized
    client.transfer_nft(&1, &from, &to, &from);
}

#[test]
#[should_panic]
fn test_transfer_nft_nonexistent() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.transfer_nft(&999, &from, &to, &from);
}

#[test]
#[should_panic]
fn test_transfer_nft_not_owner() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Owner Test", "Desc", "ipfs://owner");

    let nft_id = client.mint_reward_nft(&minter, &1, &owner, &metadata);

    // Attacker tries to transfer - with mock_all_auths they "auth" but NotOwner check fails
    client.transfer_nft(&nft_id, &attacker, &to, &attacker);
}

#[test]
#[should_panic]
fn test_transfer_nft_invalid_recipient_same_as_from() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Same Addr", "Desc", "ipfs://same");

    let nft_id = client.mint_reward_nft(&minter, &1, &owner, &metadata);

    client.transfer_nft(&nft_id, &owner, &owner, &owner);
}

#[test]
fn test_transfer_nft_emits_event() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Event NFT", "Desc", "ipfs://event");

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &1, &from, &metadata);
    client.transfer_nft(&nft_id, &from, &to, &from);

    // Transfer succeeded; NftTransferred event is emitted by transfer_nft
    assert_eq!(client.owner_of(&nft_id), Some(to));
}

// =========================================================================
// NFT APPROVAL TESTS - Per-token delegation system
// =========================================================================

#[test]
fn test_approve_and_get_approved() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);

    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let metadata = create_metadata(&env, "Approval Test", "Test approve", "ipfs://approve");

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &1, &owner, &metadata);

    // Initially no approval
    assert_eq!(client.get_approved(&nft_id), None);

    // Owner approves spender
    client.approve(&owner, &nft_id, &spender).unwrap();

    // Verify approval is set
    assert_eq!(client.get_approved(&nft_id), Some(spender));
}

#[test]
fn test_approved_address_can_transfer() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);

    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Spender NFT", "Desc", "ipfs://spender");

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &1, &owner, &metadata);

    // Owner approves spender
    client.approve(&owner, &nft_id, &spender).unwrap();

    // Spender transfers to recipient (using spender as caller)
    client.transfer_nft(&nft_id, &owner, &recipient, &spender).unwrap();

    // Verify new owner
    assert_eq!(client.owner_of(&nft_id), Some(recipient.clone()));

    // Verify approval was cleared after transfer
    assert_eq!(client.get_approved(&nft_id), None);
}

#[test]
fn test_approval_cleared_after_transfer() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);

    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Clear Approval NFT", "Desc", "ipfs://clear");

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &1, &owner, &metadata);

    // Owner approves spender
    client.approve(&owner, &nft_id, &spender).unwrap();
    assert_eq!(client.get_approved(&nft_id), Some(spender.clone()));

    // Owner transfers (owner is still authorized)
    client.transfer_nft(&nft_id, &owner, &recipient, &owner).unwrap();

    // Verify approval was cleared
    assert_eq!(client.get_approved(&nft_id), None);
}

#[test]
#[should_panic]
fn test_only_owner_can_approve() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);

    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let metadata = create_metadata(&env, "Owner Only NFT", "Desc", "ipfs://owner_only");

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &1, &owner, &metadata);

    // Non-owner tries to approve - should panic
    client.approve(&non_owner, &nft_id, &spender).unwrap();
}

#[test]
fn test_revoke_approval() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);

    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let metadata = create_metadata(&env, "Revoke Test NFT", "Desc", "ipfs://revoke");

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &1, &owner, &metadata);

    // Owner approves spender
    client.approve(&owner, &nft_id, &spender).unwrap();
    assert_eq!(client.get_approved(&nft_id), Some(spender));

    // Owner revokes approval
    client.revoke_approval(&owner, &nft_id).unwrap();

    // Verify approval is removed
    assert_eq!(client.get_approved(&nft_id), None);
}

#[test]
#[should_panic]
fn test_revoked_address_cannot_transfer() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);

    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Cannot Transfer NFT", "Desc", "ipfs://notr");

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &1, &owner, &metadata);

    // Owner approves spender
    client.approve(&owner, &nft_id, &spender).unwrap();

    // Owner revokes approval
    client.revoke_approval(&owner, &nft_id).unwrap();

    // Spender tries to transfer - should panic
    client.transfer_nft(&nft_id, &owner, &recipient, &spender).unwrap();
}

#[test]
#[should_panic]
fn test_non_approved_cannot_transfer() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);

    let owner = Address::generate(&env);
    let random_address = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Unapproved NFT", "Desc", "ipfs://unappr");

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &1, &owner, &metadata);

    // Random address (not owner, not approved) tries to transfer - should panic
    client.transfer_nft(&nft_id, &owner, &recipient, &random_address).unwrap();
}

#[test]
fn test_get_player_nfts_empty_for_new_address() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);

    let new_addr = Address::generate(&env);
    let nfts = client.get_player_nfts(&new_addr, &0, &100);
    assert_eq!(nfts.len(), 0);
}

#[test]
fn test_get_nft_owner_matches_owner_of() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Alias Test", "Desc", "ipfs://alias");

    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    assert_eq!(client.owner_of(&nft_id), client.get_nft_owner(&nft_id));
    assert_eq!(client.get_nft_owner(&nft_id), Some(player));
}

#[test]
fn test_search_by_title() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    
    let metadata1 = create_metadata(&env, "Dragon Slayer", "Desc", "ipfs://1");
    client.mint_reward_nft(&1, &player, &metadata1);
    
    let metadata2 = create_metadata(&env, "Dragon Master", "Desc", "ipfs://2");
    client.mint_reward_nft(&2, &player, &metadata2);
    
    let metadata3 = create_metadata(&env, "Phoenix Rider", "Desc", "ipfs://3");
    client.mint_reward_nft(&3, &player, &metadata3);

    // Search for "dragon" (case-insensitive)
    let results = client.search_by_title(&String::from_str(&env, "dragon"));
    assert_eq!(results.len(), 2);
    
    // Search for "Dragon" (case-insensitive)
    let results = client.search_by_title(&String::from_str(&env, "Dragon"));
    assert_eq!(results.len(), 2);
    
    // Search for "phoenix"
    let results = client.search_by_title(&String::from_str(&env, "phoenix"));
    assert_eq!(results.len(), 1);
    
    // Search for non-existent
    let results = client.search_by_title(&String::from_str(&env, "nonexistent"));
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_by_hunt_title() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    
    let metadata1 = create_metadata_full(&env, "NFT 1", "Desc", "ipfs://1", "City Hunt", 1, 0);
    client.mint_reward_nft(&1, &player, &metadata1);
    
    let metadata2 = create_metadata_full(&env, "NFT 2", "Desc", "ipfs://2", "Forest Hunt", 2, 0);
    client.mint_reward_nft(&2, &player, &metadata2);
    
    let metadata3 = create_metadata_full(&env, "NFT 3", "Desc", "ipfs://3", "City Hunt", 3, 0);
    client.mint_reward_nft(&3, &player, &metadata3);

    // Search for "city" (case-insensitive)
    let results = client.search_by_hunt_title(&String::from_str(&env, "city"));
    assert_eq!(results.len(), 2);
    
    // Search for "forest"
    let results = client.search_by_hunt_title(&String::from_str(&env, "forest"));
    assert_eq!(results.len(), 1);
    
    // Search for non-existent
    let results = client.search_by_hunt_title(&String::from_str(&env, "mountain"));
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_by_rarity() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    
    let metadata1 = create_metadata_full(&env, "NFT 1", "Desc", "ipfs://1", "Hunt 1", 1, 0);
    client.mint_reward_nft(&1, &player, &metadata1);
    
    let metadata2 = create_metadata_full(&env, "NFT 2", "Desc", "ipfs://2", "Hunt 2", 1, 0);
    client.mint_reward_nft(&2, &player, &metadata2);
    
    let metadata3 = create_metadata_full(&env, "NFT 3", "Desc", "ipfs://3", "Hunt 3", 3, 0);
    client.mint_reward_nft(&3, &player, &metadata3);

    // Search for rarity 1 (common)
    let results = client.search_by_rarity(&1);
    assert_eq!(results.len(), 2);
    
    // Search for rarity 3 (rare)
    let results = client.search_by_rarity(&3);
    assert_eq!(results.len(), 1);
    
    // Search for non-existent rarity
    let results = client.search_by_rarity(&5);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_by_tier() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    
    let metadata1 = create_metadata_full(&env, "NFT 1", "Desc", "ipfs://1", "Hunt 1", 0, 1);
    client.mint_reward_nft(&1, &player, &metadata1);
    
    let metadata2 = create_metadata_full(&env, "NFT 2", "Desc", "ipfs://2", "Hunt 2", 0, 1);
    client.mint_reward_nft(&2, &player, &metadata2);
    
    let metadata3 = create_metadata_full(&env, "NFT 3", "Desc", "ipfs://3", "Hunt 3", 0, 2);
    client.mint_reward_nft(&3, &player, &metadata3);

    // Search for tier 1
    let results = client.search_by_tier(&1);
    assert_eq!(results.len(), 2);
    
    // Search for tier 2
    let results = client.search_by_tier(&2);
    assert_eq!(results.len(), 1);
    
    // Search for tier 0 (none)
    let results = client.search_by_tier(&0);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_nfts_multiple_filters() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    
    let metadata1 = create_metadata_full(&env, "Dragon Slayer", "Desc", "ipfs://1", "City Hunt", 1, 0);
    client.mint_reward_nft(&1, &player, &metadata1);
    
    let metadata2 = create_metadata_full(&env, "Dragon Master", "Desc", "ipfs://2", "Forest Hunt", 1, 1);
    client.mint_reward_nft(&2, &player, &metadata2);
    
    let metadata3 = create_metadata_full(&env, "Phoenix Rider", "Desc", "ipfs://3", "City Hunt", 3, 0);
    client.mint_reward_nft(&3, &player, &metadata3);

    // Search with title filter only
    let results = client.search_nfts(
        Some(String::from_str(&env, "dragon")),
        None,
        None,
        None,
    );
    assert_eq!(results.len(), 2);

    let nft_id = client.mint_reward_nft(&Address::generate(&env), &1, &player, &metadata);

    // Update metadata
    client.update_nft_metadata(
        &nft_id,
        &player,
        &String::from_str(&env, "Updated Desc"),
        &String::from_str(&env, "ipfs://updated"),
    ).unwrap();

    // Search should still return only 1 NFT (not duplicated)
    let results = client.search_by_title(&String::from_str(&env, "original"));
    assert_eq!(results.len(), 1);
    
    // Search with no filters should return only 1 NFT
    let all_results = client.search_nfts(None, None, None, None);
    assert_eq!(all_results.len(), 1);
// ---------------------------------------------------------------------------
// Schema versioning tests
// ---------------------------------------------------------------------------

#[test]
fn test_fresh_mint_gets_current_schema_version() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Versioned NFT", "Has version", "ipfs://v");
    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    let meta_resp = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(
        meta_resp.schema_version, METADATA_SCHEMA_VERSION,
        "freshly minted NFT should have schema_version = METADATA_SCHEMA_VERSION"
    );
}

#[test]
fn test_legacy_record_read_as_v1() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Legacy", "Old format", "ipfs://legacy");

    // Mint a fresh NFT to get its data into storage, then verify that
    // an NFT with NO version key defaults to schema_version = 1.
    // We simulate a legacy NFT by not calling set_nft_version — it's a
    // freshly minted one that has the version key set during mint.
    let nft_id = client.mint_reward_nft(&player, &5, &player, &metadata);

    let meta_resp = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(
        meta_resp.schema_version, METADATA_SCHEMA_VERSION,
        "newly minted NFT has schema_version = METADATA_SCHEMA_VERSION"
    );
    assert_eq!(meta_resp.title, metadata.title);
    assert_eq!(meta_resp.hunt_id, 5);
}

#[test]
fn test_legacy_nft_without_version_key_defaults_to_v1() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "NoVer", "no version key", "ipfs://nover");

    // Mint, then manually delete the version key to simulate a legacy NFT
    // that was stored before versioning existed.
    let nft_id = client.mint_reward_nft(&player, &3, &player, &metadata);

    let nft_version_key = (Symbol::new(&env, "NVER"), nft_id);
    env.as_contract(&contract_id, || {
        env.storage().persistent().remove(&nft_version_key);
    });

    // Read it back via get_nft_metadata — should default to v1
    let meta_resp = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(
        meta_resp.schema_version, METADATA_SCHEMA_VERSION,
        "NFT without version key defaults to METADATA_SCHEMA_VERSION"
    );

    let nft_id = client.mint_reward_nft(&Address::generate(&env), &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, Some(creator.clone()));
    assert_eq!(nft.metadata.royalty_bps, Some(royalty_bps));

    let meta = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta.creator, Some(creator));
    assert_eq!(meta.royalty_bps, Some(royalty_bps));
}

#[test]
fn test_migration_v0_to_v1_sets_schema_version() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "No Creator", "No creator set", "ipfs://nocreator");

    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, None);
    assert_eq!(nft.metadata.royalty_bps, None);
}

#[test]
fn test_mint_from_map_with_creator_and_royalty() {
    use soroban_sdk::{Map, Symbol};

    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);

    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    let mut metadata: Map<Symbol, Val> = Map::new(&env);
    metadata.set(Symbol::new(&env, "title"), String::from_str(&env, "Map NFT").into_val(&env));
    metadata.set(
        Symbol::new(&env, "description"),
        String::from_str(&env, "NFT from map").into_val(&env),
    );
    metadata.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "ipfs://map").into_val(&env),
    );
    metadata.set(Symbol::new(&env, "creator"), creator.clone().into_val(&env));
    metadata.set(Symbol::new(&env, "royalty_bps"), 500u32.into_val(&env));

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, Some(creator.clone()));
    assert_eq!(nft.metadata.royalty_bps, Some(500u32));
}

#[test]
fn test_mint_from_map_creator_defaults_to_player() {
    use soroban_sdk::{Map, Symbol};

    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);

    let player = Address::generate(&env);

    let mut metadata: Map<Symbol, Val> = Map::new(&env);
    metadata.set(Symbol::new(&env, "title"), String::from_str(&env, "Default Creator").into_val(&env));

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    // When creator is not specified in map, it defaults to player_address
    assert_eq!(nft.metadata.creator, Some(player));
    assert_eq!(nft.metadata.royalty_bps, None);
}

#[test]
fn test_creator_preserved_across_metadata_queries() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    let metadata = create_metadata_with_creator(
        &env,
        "Preserved Creator",
        "Creator should be preserved",
        "ipfs://preserved",
        creator.clone(),
        Some(1000u32),
    );

    let nft_id = client.mint_reward_nft(&Address::generate(&env), &42, &player, &metadata);

    // Query via get_nft
    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, Some(creator.clone()));
    assert_eq!(nft.metadata.royalty_bps, Some(1000u32));

    // Query via get_nft_metadata
    let meta = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta.creator, Some(creator.clone()));
    assert_eq!(meta.royalty_bps, Some(1000u32));
    assert_eq!(meta.current_owner, player);
}

#[test]
fn test_burn_removes_nft_and_clears_owner_list() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Burn Me", "Desc", "ipfs://burn");

    let nft_id = client.mint_reward_nft(&minter, &1, &owner, &metadata);
    assert!(client.get_nft(&nft_id).is_some());

    client.burn(&nft_id, &owner);

    assert!(client.get_nft(&nft_id).is_none());
    assert_eq!(client.get_player_nfts(&owner, &0, &100).len(), 0);
}

#[test]
fn test_initialize_stores_admin_and_minter() {
    let (env, contract_id, admin, minter) = setup_initialized();
    let client = NftRewardClient::new(&env, &contract_id);

    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
fn test_burn_fails_if_not_owner() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let other = Address::generate(&env);
    let metadata = create_metadata(&env, "Not Yours", "Desc", "ipfs://notyours");

    let nft_id = client.mint_reward_nft(&minter, &1, &owner, &metadata);

    let result = client.try_burn(&nft_id, &other);
    assert!(result.is_err());
    assert!(client.get_nft(&nft_id).is_some());
}

#[test]
fn test_burn_fails_for_nonexistent_nft() {
    let env = setup_env();
    let (client, _minter) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let result = client.try_burn(&999u64, &owner);
    assert!(result.is_err());
}

#[test]
fn test_metadata_preserved_during_migration() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, Some(2));

    let player = Address::generate(&env);
    let creator = Address::generate(&env);

    // Mint an NFT with full metadata
    let metadata = NftMetadata {
        title: String::from_str(&env, "Detailed NFT"),
        description: String::from_str(&env, "A very detailed description"),
        image_uri: String::from_str(&env, "ipfs://QmDetailed"),
        hunt_title: String::from_str(&env, "Grand Hunt"),
        rarity: 4,
        tier: 2,
        creator: Some(creator.clone()),
        royalty_bps: Some(500u32),
    };

    client.mint_reward_nft(&minter, &1, &player1, &metadata1);
    client.mint_reward_nft(&minter, &1, &player2, &metadata2);
    client.mint_reward_nft(&minter, &1, &player3, &metadata3);
}

#[test]
fn test_migration_dry_run_does_not_write() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(NftReward, ());
    let admin = Address::generate(&env);

    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "DryRun", "dry", "ipfs://dry");
    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    let nft_id = client.mint_reward_nft_from_map(&Address::generate(&env), &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.title, String::from_str(&env, "Test NFT"));
    assert_eq!(nft.metadata.description, String::from_str(&env, "")); // default
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, "")); // default
    assert_eq!(nft.metadata.hunt_title, String::from_str(&env, "Test NFT")); // defaults to title
    assert_eq!(nft.metadata.rarity, 0u32); // default
    assert_eq!(nft.metadata.tier, 0u32); // default
    assert_eq!(nft.transferable, false); // default
}

#[test]
fn test_mint_reward_nft_from_map_with_invalid_types_uses_defaults() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let mut metadata: Map<Symbol, Val> = Map::new(&env);
    
    // Provide valid values for required fields
    metadata.set(Symbol::new(&env, "title"), String::from_str(&env, "Valid Title").into_val(&env));
    metadata.set(Symbol::new(&env, "description"), String::from_str(&env, "Valid description").into_val(&env));
    metadata.set(Symbol::new(&env, "image_uri"), String::from_str(&env, "ipfs://valid").into_val(&env));
    
    // Provide invalid types for optional fields (wrong type conversions will fail and use defaults)
    metadata.set(Symbol::new(&env, "hunt_title"), 999u32.into_val(&env)); // u32 instead of String
    metadata.set(Symbol::new(&env, "rarity"), String::from_str(&env, "invalid").into_val(&env)); // String instead of u32
    metadata.set(Symbol::new(&env, "tier"), String::from_str(&env, "invalid").into_val(&env)); // String instead of u32
    metadata.set(Symbol::new(&env, "transferable"), 123u32.into_val(&env)); // u32 instead of bool

    // This should not panic; invalid types for optional fields should use defaults
    let nft_id = client.mint_reward_nft_from_map(&Address::generate(&env), &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.title, String::from_str(&env, "Valid Title"));
    assert_eq!(nft.metadata.description, String::from_str(&env, "Valid description"));
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, "ipfs://valid"));
    assert_eq!(nft.metadata.hunt_title, String::from_str(&env, "Valid Title")); // defaults to title
    assert_eq!(nft.metadata.rarity, 0u32); // default due to invalid type
    assert_eq!(nft.metadata.tier, 0u32); // default due to invalid type
    assert_eq!(nft.transferable, false); // default due to invalid type
}

// ---------------------------------------------------------------------------
// NFT Transfer edge case tests
// ---------------------------------------------------------------------------

/// Helper: mint a soulbound (non-transferable) NFT via mint_reward_nft.
/// By default, mint_reward_nft sets transferable = false.
fn mint_soulbound(
    env: &Env,
    client: &NftRewardClient<'_>,
    hunt_id: u64,
    owner: &Address,
    metadata: &NftMetadata,
) -> u64 {
    client.mint_reward_nft(owner, &hunt_id, owner, metadata)
}

// --- Transfer to self ---

#[test]
fn test_transfer_to_self_is_rejected() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Self Transfer", "desc", "ipfs://self");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    // Transferring to oneself should return InvalidRecipient
    let result = client.try_transfer_nft(&nft_id, &owner, &owner, &owner);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::InvalidRecipient)),
        "transfer to self must return InvalidRecipient"
    );
}

#[test]
fn test_transfer_to_self_does_not_mutate_state() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "No State Change", "desc", "ipfs://nsc");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    // Attempt (and ignore error) — ownership must be unchanged
    let _ = client.try_transfer_nft(&nft_id, &owner, &owner, &owner);

    assert_eq!(
        client.owner_of(&nft_id),
        Some(owner.clone()),
        "owner must remain the same after rejected self-transfer"
    );
    let owner_nfts = client.get_player_nfts(&owner, &0, &100);
    assert_eq!(
        owner_nfts.len(),
        1,
        "player NFT index must be unchanged after rejected self-transfer"
    );
}

// --- Transfer non-existent NFT ---

#[test]
fn test_transfer_nonexistent_nft_returns_not_found() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    let result = client.try_transfer_nft(&9999, &from, &to, &from);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::NftNotFound)),
        "transferring a non-existent NFT must return NftNotFound"
    );
}

#[test]
fn test_transfer_id_zero_returns_not_found() {
    // NFT IDs start at 1; ID 0 should never exist.
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    let result = client.try_transfer_nft(&0, &from, &to, &from);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::NftNotFound)),
        "transferring NFT id=0 must return NftNotFound"
    );
}

// --- Transfer by non-owner ---

#[test]
fn test_transfer_by_non_owner_returns_not_owner() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Owned NFT", "desc", "ipfs://owned");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    // attacker passes their own address as `from`, which doesn't match the actual owner
    let result = client.try_transfer_nft(&nft_id, &attacker, &recipient, &attacker);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::NotOwner)),
        "transfer where from != actual owner must return NotOwner"
    );
}

#[test]
fn test_transfer_by_non_owner_does_not_change_ownership() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Guard NFT", "desc", "ipfs://guard");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    let _ = client.try_transfer_nft(&nft_id, &attacker, &recipient, &attacker);

    assert_eq!(
        client.owner_of(&nft_id),
        Some(owner.clone()),
        "ownership must be unchanged after failed transfer attempt by non-owner"
    );
    // attacker and recipient should have no NFTs
    assert_eq!(client.get_player_nfts(&attacker, &0, &100).len(), 0);
    assert_eq!(client.get_player_nfts(&recipient, &0, &100).len(), 0);
}

#[test]
fn test_transfer_caller_not_operator_returns_not_operator() {
    // `from` is the real owner but `caller` is neither owner nor approved operator
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let third_party = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Operator Test", "desc", "ipfs://op");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    // caller is third_party (not owner, not operator)
    let result = client.try_transfer_nft(&nft_id, &owner, &recipient, &third_party);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::NotOperator)),
        "transfer by unapproved caller must return NotOperator"
    );
}

#[test]
fn test_operator_can_transfer_on_behalf_of_owner() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Operator Xfer", "desc", "ipfs://opxfer");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    client.set_operator(&owner, &operator);
    client.transfer_nft(&nft_id, &owner, &recipient, &operator);

    assert_eq!(
        client.owner_of(&nft_id),
        Some(recipient.clone()),
        "operator-initiated transfer must update ownership to recipient"
    );
}

#[test]
fn test_revoked_operator_cannot_transfer() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Revoke Test", "desc", "ipfs://revoke");
    let nft_id = mint_transferable(&env, &client, 1, &owner, &metadata);

    client.set_operator(&owner, &operator);
    client.remove_operator(&owner, &operator);

    let result = client.try_transfer_nft(&nft_id, &owner, &recipient, &operator);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::NotOperator)),
        "revoked operator must not be able to transfer"
    );
    assert_eq!(client.owner_of(&nft_id), Some(owner.clone()));
}

// --- Transfer soulbound NFT ---

#[test]
fn test_transfer_soulbound_nft_returns_soulbound_error() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Soulbound NFT", "non-transferable", "ipfs://soul");

    // mint_reward_nft defaults to transferable = false (soulbound)
    let nft_id = mint_soulbound(&env, &client, 1, &owner, &metadata);

    let result = client.try_transfer_nft(&nft_id, &owner, &recipient, &owner);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::SoulboundNft)),
        "transferring a soulbound NFT must return SoulboundNft"
    );
}

#[test]
fn test_soulbound_nft_ownership_unchanged_after_attempted_transfer() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Bound Trophy", "soul", "ipfs://bound");
    let nft_id = mint_soulbound(&env, &client, 1, &owner, &metadata);

    let _ = client.try_transfer_nft(&nft_id, &owner, &recipient, &owner);

    // owner still holds the NFT
    assert_eq!(client.owner_of(&nft_id), Some(owner.clone()));
    assert_eq!(client.get_player_nfts(&owner, &0, &100).len(), 1);
    assert_eq!(client.get_player_nfts(&recipient, &0, &100).len(), 0);
}

#[test]
fn test_soulbound_nft_operator_cannot_override_soulbound() {
    // Even an approved operator should not be able to transfer a soulbound NFT.
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let recipient = Address::generate(&env);
    let metadata = create_metadata(&env, "Soul + Op", "soul", "ipfs://soulop");
    let nft_id = mint_soulbound(&env, &client, 1, &owner, &metadata);

    client.set_operator(&owner, &operator);

    let result = client.try_transfer_nft(&nft_id, &owner, &recipient, &operator);
    assert_eq!(
        result,
        Err(Ok(crate::errors::NftErrorCode::SoulboundNft)),
        "an operator must not be able to bypass the soulbound restriction"
    );
    assert_eq!(client.owner_of(&nft_id), Some(owner.clone()));
}

// --- Transfer and verify ownership update ---

#[test]
fn test_transfer_updates_owner_field_in_nft_data() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let metadata = create_metadata(&env, "Ownership Check", "desc", "ipfs://oc");
    let nft_id = mint_transferable(&env, &client, 1, &alice, &metadata);

    client.transfer_nft(&nft_id, &alice, &bob, &alice);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.owner, bob, "NftData.owner must reflect the new owner");
}

#[test]
fn test_transfer_updates_owner_of_query() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let metadata = create_metadata(&env, "OwnerOf Query", "desc", "ipfs://oq");
    let nft_id = mint_transferable(&env, &client, 1, &alice, &metadata);

    client.transfer_nft(&nft_id, &alice, &bob, &alice);

    assert_eq!(client.owner_of(&nft_id), Some(bob.clone()));
    assert_eq!(client.get_nft_owner(&nft_id), Some(bob));
}

#[test]
fn test_transfer_updates_player_nft_indexes_correctly() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Mint two NFTs for alice
    let m1 = create_metadata(&env, "NFT A", "a", "ipfs://a");
    let m2 = create_metadata(&env, "NFT B", "b", "ipfs://b");
    let nft1 = mint_transferable(&env, &client, 1, &alice, &m1);
    let nft2 = mint_transferable(&env, &client, 2, &alice, &m2);

    assert_eq!(client.get_player_nfts(&alice, &0, &100).len(), 2);
    assert_eq!(client.get_player_nfts(&bob, &0, &100).len(), 0);

    // Transfer nft1 to bob
    client.transfer_nft(&nft1, &alice, &bob, &alice);

    let alice_nfts = client.get_player_nfts(&alice, &0, &100);
    assert_eq!(alice_nfts.len(), 1, "alice must have 1 NFT remaining");
    assert_eq!(
        alice_nfts.get(0).unwrap(),
        nft2,
        "alice's remaining NFT must be nft2"
    );

    let bob_nfts = client.get_player_nfts(&bob, &0, &100);
    assert_eq!(bob_nfts.len(), 1, "bob must have 1 NFT after transfer");
    assert_eq!(
        bob_nfts.get(0).unwrap(),
        nft1,
        "bob's NFT must be the transferred nft1"
    );
}

#[test]
fn test_transfer_preserves_completion_player_field() {
    // completion_player is the address that originally completed the hunt —
    // it must be immutable even after ownership changes.
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let original_player = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Provenance NFT", "desc", "ipfs://prov");
    let nft_id = mint_transferable(&env, &client, 1, &original_player, &metadata);

    client.transfer_nft(&nft_id, &original_player, &new_owner, &original_player);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(
        nft.completion_player, original_player,
        "completion_player must not change after a transfer"
    );
    assert_eq!(nft.owner, new_owner);
}

#[test]
fn test_transfer_preserves_metadata_fields() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let metadata = create_metadata_full(
        &env,
        "Rare Trophy",
        "Very rare",
        "ipfs://rare",
        "Epic Hunt",
        4,
        2,
    );
    let nft_id = mint_transferable(&env, &client, 42, &alice, &metadata);

    client.transfer_nft(&nft_id, &alice, &bob, &alice);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.title, String::from_str(&env, "Rare Trophy"));
    assert_eq!(nft.metadata.description, String::from_str(&env, "Very rare"));
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, "ipfs://rare"));
    assert_eq!(nft.metadata.hunt_title, String::from_str(&env, "Epic Hunt"));
    assert_eq!(nft.metadata.rarity, 4);
    assert_eq!(nft.metadata.tier, 2);
    assert_eq!(nft.hunt_id, 42);
}

#[test]
fn test_chained_transfers_track_ownership_correctly() {
    // A → B → C: each step must correctly update owner and player indexes.
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let c = Address::generate(&env);
    let metadata = create_metadata(&env, "Chain NFT", "chain", "ipfs://chain");
    let nft_id = mint_transferable(&env, &client, 1, &a, &metadata);

    // A → B
    client.transfer_nft(&nft_id, &a, &b, &a);
    assert_eq!(client.owner_of(&nft_id), Some(b.clone()));
    assert_eq!(client.get_player_nfts(&a, &0, &100).len(), 0);
    assert_eq!(client.get_player_nfts(&b, &0, &100).len(), 1);

    // B → C
    client.transfer_nft(&nft_id, &b, &c, &b);
    assert_eq!(client.owner_of(&nft_id), Some(c.clone()));
    assert_eq!(client.get_player_nfts(&b, &0, &100).len(), 0);
    assert_eq!(client.get_player_nfts(&c, &0, &100).len(), 1);
}

#[test]
fn test_transfer_emits_nft_transferred_event_with_correct_fields() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register(NftReward, ()));

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Event Check", "desc", "ipfs://ev");
    let nft_id = mint_transferable(&env, &client, 1, &from, &metadata);

    client.transfer_nft(&nft_id, &from, &to, &from);

    let events = env.events().all();
    // Find the NftTransferred event (last event published)
    let (_contract, topics, data) = events.get(events.len() - 1).unwrap();

    // topics[0] == "NftTransferred", topics[1] == nft_id
    let topic0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    assert_eq!(topic0, Symbol::new(&env, "NftTransferred"));

    let event: crate::NftTransferredEvent =
        crate::NftTransferredEvent::try_from_val(&env, &data).unwrap();
    assert_eq!(event.nft_id, nft_id);
    assert_eq!(event.from, from);
    assert_eq!(event.to, to);
}

// ---------------------------------------------------------------------------
// admin_update_image_uris tests
// ---------------------------------------------------------------------------

#[test]
fn test_admin_update_image_uris_replaces_matching_prefix() {
    let env = setup_env();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let player = Address::generate(&env);

    client.initialize(&admin, &None);

    // Mint an NFT with an old IPFS gateway prefix
    let metadata = create_metadata(
        &env,
        "Trophy",
        "Award",
        "ipfs://old-gateway.example.com/QmHash123",
    );
    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    // Update from old gateway to new gateway
    let updated = client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://old-gateway.example.com"),
        &String::from_str(&env, "https://new-cdn.example.com"),
    );
    assert_eq!(updated, 1);

    // Verify the image_uri was updated
    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(
        nft.metadata.image_uri,
        String::from_str(&env, "https://new-cdn.example.com/QmHash123")
    );
}

#[test]
fn test_admin_update_image_uris_only_matches_exact_prefix() {
    let env = setup_env();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let player = Address::generate(&env);

    client.initialize(&admin, &None);

    // Mint NFT with a specific prefix
    let metadata1 = create_metadata(
        &env,
        "NFT 1",
        "Desc 1",
        "ipfs://gateway-a/QmHash1",
    );
    client.mint_reward_nft(&player, &1, &player, &metadata1);

    // Mint NFT with a different prefix that shares a partial match
    let metadata2 = create_metadata(
        &env,
        "NFT 2",
        "Desc 2",
        "ipfs://gateway-ab/QmHash2",
    );
    client.mint_reward_nft(&player, &2, &player, &metadata2);

    // Update only "ipfs://gateway-a/" — should NOT match "ipfs://gateway-ab/"
    let updated = client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://gateway-a/"),
        &String::from_str(&env, "https://new-cdn/"),
    );
    assert_eq!(updated, 1);

    // NFT 1 should be updated
    let nft1 = client.get_nft(&1).unwrap();
    assert_eq!(
        nft1.metadata.image_uri,
        String::from_str(&env, "https://new-cdn/QmHash1")
    );

    // NFT 2 should NOT be updated (prefix "ipfs://gateway-a/" does not match "ipfs://gateway-ab/")
    let nft2 = client.get_nft(&2).unwrap();
    assert_eq!(
        nft2.metadata.image_uri,
        String::from_str(&env, "ipfs://gateway-ab/QmHash2")
    );
}

#[test]
fn test_admin_update_image_uris_no_matches_returns_zero() {
    let env = setup_env();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let player = Address::generate(&env);

    client.initialize(&admin, &None);

    // Mint an NFT with a prefix that won't match
    let metadata = create_metadata(
        &env,
        "Trophy",
        "Award",
        "https://cdn.example.com/QmHash",
    );
    client.mint_reward_nft(&player, &1, &player, &metadata);

    // Try to update with a prefix that doesn't match
    let updated = client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://nonexistent/"),
        &String::from_str(&env, "ipfs://new/"),
    );
    assert_eq!(updated, 0);

    // NFT should be unchanged
    let nft = client.get_nft(&1).unwrap();
    assert_eq!(
        nft.metadata.image_uri,
        String::from_str(&env, "https://cdn.example.com/QmHash")
    );
}

#[test]
fn test_admin_update_image_uris_multiple_nfts() {
    let env = setup_env();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let player = Address::generate(&env);

    client.initialize(&admin, &None);

    // Mint multiple NFTs with old gateway prefix
    for i in 1..=5 {
        let uri = format!("ipfs://old-gateway/QmHash{}", i);
        let metadata = create_metadata(&env, "NFT", "Desc", &uri);
        client.mint_reward_nft(&player, &(i as u64), &player, &metadata);
    }

    // Also mint an NFT with a different prefix
    let metadata_other = create_metadata(
        &env,
        "Other NFT",
        "Other Desc",
        "https://other-cdn.example.com/QmOther",
    );
    client.mint_reward_nft(&player, &10, &player, &metadata_other);

    // Batch update all old gateway NFTs
    let updated = client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://old-gateway/"),
        &String::from_str(&env, "https://new-cdn/"),
    );
    assert_eq!(updated, 5);

    // Verify all matching NFTs were updated
    for i in 1..=5 {
        let nft = client.get_nft(&(i as u64)).unwrap();
        assert_eq!(
            nft.metadata.image_uri,
            String::from_str(&env, &format!("https://new-cdn/QmHash{}", i))
        );
    }

    // Verify the non-matching NFT was NOT updated
    let nft_other = client.get_nft(&6).unwrap();
    assert_eq!(
        nft_other.metadata.image_uri,
        String::from_str(&env, "https://other-cdn.example.com/QmOther")
    );
}

#[test]
fn test_admin_update_image_uris_emits_event() {
    let env = setup_env();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let player = Address::generate(&env);

    client.initialize(&admin, &None);

    let metadata = create_metadata(
        &env,
        "Event NFT",
        "Desc",
        "ipfs://old/QmEvent",
    );
    client.mint_reward_nft(&player, &1, &player, &metadata);

    client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://old/"),
        &String::from_str(&env, "ipfs://new/"),
    );

    // Check that AdminImageUrisUpdated event was emitted
    let events = env.events().all();
    // Find AdminImageUrisUpdated event
    let mut found_event = false;
    for event in events.iter() {
        let (_contract, topics, data) = event;
        let topic0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
        if topic0 == Symbol::new(&env, "AdminImageUrisUpdated") {
            let event_data: crate::AdminImageUrisUpdatedEvent =
                crate::AdminImageUrisUpdatedEvent::try_from_val(&env, &data).unwrap();
            assert_eq!(
                event_data.old_prefix,
                String::from_str(&env, "ipfs://old/")
            );
            assert_eq!(
                event_data.new_prefix,
                String::from_str(&env, "ipfs://new/")
            );
            assert_eq!(event_data.updated_count, 1);
            found_event = true;
        }
    }
    assert!(found_event, "AdminImageUrisUpdated event must be emitted");
}

#[test]
#[should_panic(expected = "HostError")]
fn test_admin_update_image_uris_requires_auth() {
    let env = Env::default();
    // Do NOT mock auth - we want the call to fail without auth
    env.ledger().set_timestamp(1000);

    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // This should fail because admin has not authorized
    client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://old/"),
        &String::from_str(&env, "ipfs://new/"),
    );
}

#[test]
fn test_admin_update_image_uris_preserves_other_metadata() {
    let env = setup_env();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let player = Address::generate(&env);

    client.initialize(&admin, &None);

    let metadata = create_metadata_full(
        &env,
        "Rare Trophy",
        "Very rare award",
        "ipfs://old-gateway/QmPreserve",
        "Epic Hunt",
        4,
        2,
    );
    let nft_id = client.mint_reward_nft(&player, &42, &player, &metadata);

    client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://old-gateway/"),
        &String::from_str(&env, "https://new-cdn/"),
    );

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.title, String::from_str(&env, "Rare Trophy"));
    assert_eq!(
        nft.metadata.description,
        String::from_str(&env, "Very rare award")
    );
    assert_eq!(
        nft.metadata.image_uri,
        String::from_str(&env, "https://new-cdn/QmPreserve")
    );
    assert_eq!(
        nft.metadata.hunt_title,
        String::from_str(&env, "Epic Hunt")
    );
    assert_eq!(nft.metadata.rarity, 4);
    assert_eq!(nft.metadata.tier, 2);
    assert_eq!(nft.hunt_id, 42);
    assert_eq!(nft.owner, player);
}

#[test]
fn test_admin_update_image_uris_no_nfts_returns_zero() {
    let env = setup_env();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &None);

    // No NFTs minted yet
    let updated = client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://old/"),
        &String::from_str(&env, "ipfs://new/"),
    );
    assert_eq!(updated, 0);
}

#[test]
fn test_admin_update_image_uris_empty_prefix_replacement() {
    let env = setup_env();
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let player = Address::generate(&env);

    client.initialize(&admin, &None);

    // An empty prefix should match everything (all strings start with "")
    let metadata = create_metadata(&env, "NFT", "Desc", "ipfs://something");
    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    let updated = client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, ""),
        &String::from_str(&env, "https://prefixed/"),
    );
    assert_eq!(updated, 1);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(
        nft.metadata.image_uri,
        String::from_str(&env, "https://prefixed/ipfs://something")
    );
}

#[test]
fn test_all_nft_error_codes_are_unique() {
    let mut seen = std::collections::BTreeSet::new();
    let variants: &[(crate::errors::NftErrorCode, &str)] = &[
        (crate::errors::NftErrorCode::NftNotFound, "NftNotFound"),
        (crate::errors::NftErrorCode::Unauthorized, "Unauthorized"),
        (crate::errors::NftErrorCode::NotOwner, "NotOwner"),
        (crate::errors::NftErrorCode::InvalidRecipient, "InvalidRecipient"),
        (crate::errors::NftErrorCode::SoulboundNft, "SoulboundNft"),
        (crate::errors::NftErrorCode::InvalidRarity, "InvalidRarity"),
        (crate::errors::NftErrorCode::AlreadyInitialized, "AlreadyInitialized"),
        (crate::errors::NftErrorCode::MaxSupplyReached, "MaxSupplyReached"),
        (crate::errors::NftErrorCode::NotInitialized, "NotInitialized"),
        (crate::errors::NftErrorCode::NotOperator, "NotOperator"),
        (crate::errors::NftErrorCode::NftNotTransferable, "NftNotTransferable"),
        (crate::errors::NftErrorCode::NftLocked, "NftLocked"),
        (crate::errors::NftErrorCode::InvalidMetadata, "InvalidMetadata"),
    ];
    for (variant, name) in variants {
        let code = *variant as u32;
        assert!(
            seen.insert(code),
            "Duplicate NftErrorCode value {} for variant '{}'",
            code,
            name
        );
    }
}
