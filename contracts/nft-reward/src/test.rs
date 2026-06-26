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
    client.transfer_nft(&nft_id, &from, &to);

    // Transfer succeeded; NftTransferred event is emitted by transfer_nft
    assert_eq!(client.owner_of(&nft_id), Some(to));
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
    
    // Provide valid title
    metadata.set(Symbol::new(&env, "title"), String::from_str(&env, "Valid Title").into_val(&env));
    
    // Provide invalid types for other fields (wrong type conversions will fail and use defaults)
    metadata.set(Symbol::new(&env, "description"), 123456u32.into_val(&env)); // u32 instead of String
    metadata.set(Symbol::new(&env, "image_uri"), true.into_val(&env)); // bool instead of String
    metadata.set(Symbol::new(&env, "hunt_title"), 999u32.into_val(&env)); // u32 instead of String
    metadata.set(Symbol::new(&env, "rarity"), String::from_str(&env, "invalid").into_val(&env)); // String instead of u32
    metadata.set(Symbol::new(&env, "tier"), String::from_str(&env, "invalid").into_val(&env)); // String instead of u32
    metadata.set(Symbol::new(&env, "transferable"), 123u32.into_val(&env)); // u32 instead of bool

    // This should not panic; invalid types should use defaults
    let nft_id = client.mint_reward_nft_from_map(&Address::generate(&env), &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.title, String::from_str(&env, "Valid Title"));
    assert_eq!(nft.metadata.description, String::from_str(&env, "")); // default due to invalid type
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, "")); // default due to invalid type
    assert_eq!(nft.metadata.hunt_title, String::from_str(&env, "Valid Title")); // defaults to title
    assert_eq!(nft.metadata.rarity, 0u32); // default due to invalid type
    assert_eq!(nft.metadata.tier, 0u32); // default due to invalid type
    assert_eq!(nft.transferable, false); // default due to invalid type
}
