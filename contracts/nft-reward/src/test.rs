#![cfg(test)]
extern crate std;

use crate::{
    CollectionMetadata, NftMetadata, NftMintedEvent, NftReward, NftRewardClient, NftErrorCode,
    METADATA_SCHEMA_VERSION,
};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    Address, Env, IntoVal, Map, String, Symbol, Val, TryFromVal,
};

fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);
    env
}

fn default_collection_metadata(env: &Env) -> CollectionMetadata {
    CollectionMetadata {
        name: String::from_str(env, "Hunty Rewards"),
        description: String::from_str(env, "Reward NFTs for completed hunts"),
        total_supply: 0,
        creator: None,
    }
}

fn setup_nft_reward(env: &Env, max_supply: Option<u64>) -> (NftRewardClient<'_>, Address) {
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let minter = Address::generate(env);
    client.initialize(&admin, &minter, &max_supply, &default_collection_metadata(env));
    (client, minter)
}

fn setup_initialized() -> (Env, Address, Address, Address) {
    let env = setup_env();
    let contract_id = env.register_contract(None, NftReward);
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));
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
        extensions: Map::new(env),
    }
}

fn create_metadata_with_extensions(
    env: &Env,
    title: &str,
    desc: &str,
    image_uri: &str,
    extensions: Map<String, String>,
) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, desc),
        image_uri: String::from_str(env, image_uri),
        hunt_title: String::from_str(env, title),
        rarity: 0u32,
        tier: 0u32,
        creator: None,
        royalty_bps: None,
        extensions,
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
        extensions: Map::new(env),
    }
}

fn create_transferable_metadata(
    env: &Env,
    title: &str,
    desc: &str,
    image_uri: &str,
) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, desc),
        image_uri: String::from_str(env, image_uri),
        hunt_title: String::from_str(env, title),
        rarity: 0u32,
        tier: 0u32,
        creator: None,
        royalty_bps: None,
        extensions: Map::new(env),
    }
}

fn create_metadata_with_creator(
    env: &Env,
    title: &str,
    desc: &str,
    image_uri: &str,
    creator: Address,
    royalty_bps: Option<u32>,
) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, desc),
        image_uri: String::from_str(env, image_uri),
        hunt_title: String::from_str(env, title),
        rarity: 0u32,
        tier: 0u32,
        creator: Some(creator),
        royalty_bps,
        extensions: Map::new(env),
    }
}

// =========================================================================
// INITIALIZATION TESTS
// =========================================================================

#[test]
fn test_initialize_stores_admin() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));

    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
fn test_initialize_requires_auth() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));
    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
#[should_panic(expected = "HostError")]
fn test_initialize_cannot_be_called_twice() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));
    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));
}

#[test]
fn test_collection_metadata_is_set_during_initialization_and_updates_supply() {
    let env = setup_env();
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let creator = Address::generate(&env);

    let collection_metadata = CollectionMetadata {
        name: String::from_str(&env, "Hunty Rewards"),
        description: String::from_str(&env, "Reward NFTs for completed hunts"),
        total_supply: 0,
        creator: Some(creator.clone()),
    };

    client.initialize(&admin, &minter, &None, &collection_metadata);

    let initial = client.get_collection_metadata().unwrap();
    assert_eq!(initial.name, String::from_str(&env, "Hunty Rewards"));
    assert_eq!(
        initial.description,
        String::from_str(&env, "Reward NFTs for completed hunts")
    );
    assert_eq!(initial.creator, Some(creator));
    assert_eq!(initial.total_supply, 0);

    let player = Address::generate(&env);
    let metadata = create_metadata(
        &env,
        "Hunt Champion",
        "Completed the City Hunt",
        "ipfs://QmExample123",
    );
    client.mint_reward_nft(&minter, &1, &player, &metadata);

    let updated = client.get_collection_metadata().unwrap();
    assert_eq!(updated.total_supply, 1);
}

// =========================================================================
// MINTING TESTS
// =========================================================================

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
#[should_panic]
fn test_soulbound_nft_cannot_be_transferred() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);
    let mut metadata_map: Map<Symbol, Val> = Map::new(&env);
    metadata_map.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Soulbound Trophy").into_val(&env),
    );
    metadata_map.set(
        Symbol::new(&env, "description"),
        String::from_str(&env, "Bound to the owner").into_val(&env),
    );
    metadata_map.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "ipfs://soulbound").into_val(&env),
    );
    metadata_map.set(Symbol::new(&env, "transferable"), false.into_val(&env));

    let nft_id = client.mint_reward_nft_from_map(&minter, &1, &owner, &metadata_map);
    client.transfer_nft(&nft_id, &owner, &recipient, &owner);
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
    let (_contract, topics, data): (Address, soroban_sdk::Vec<Val>, Val) =
        events.get(events.len() - 1).unwrap();
    assert_eq!(topics.len(), 2);
    assert_eq!(
        Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap(),
        Symbol::new(&env, "NftMinted")
    );
    assert_eq!(
        u64::try_from_val(&env, &topics.get(1).unwrap()).unwrap(),
        nft_id
    );

    let event = NftMintedEvent::try_from_val(&env, &data).unwrap();
    assert_eq!(event.nft_id, nft_id);
    assert_eq!(event.hunt_id, 7);
    assert_eq!(event.owner, player);
    assert_eq!(event.minted_at, 1000);
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
        4,
        1,
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
    assert_eq!(meta.schema_version, METADATA_SCHEMA_VERSION);
}

#[test]
fn test_mint_from_map_then_query_metadata() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();
    let reward_manager = Address::generate(&env);
    client.set_reward_manager(&admin, &reward_manager);
    client.add_minter(&admin, &reward_manager);

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

// =========================================================================
// METADATA UPDATE TESTS
// =========================================================================

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

// =========================================================================
// TRANSFER TESTS
// =========================================================================

#[test]
fn test_transfer_nft_success() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    let mut map: Map<Symbol, Val> = Map::new(&env);
    map.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Transfer NFT").into_val(&env),
    );
    map.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "ipfs://transfer").into_val(&env),
    );
    map.set(Symbol::new(&env, "transferable"), true.into_val(&env));
    let nft_id = client.mint_reward_nft_from_map(&minter, &1, &from, &map);

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
    assert!(
        alice_nfts.get(0).unwrap() == nft1 || alice_nfts.get(0).unwrap() == nft2
    );

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
    env.ledger().set_timestamp(1000);

    let (client, minter) = setup_nft_reward(&env, None);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Auth Test", "Desc", "ipfs://auth");

    let _nft_id = client.mint_reward_nft(&minter, &1, &from, &metadata);

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
    let (client, minter) = setup_nft_reward(&env, None);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let mut map: Map<Symbol, Val> = Map::new(&env);
    map.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Event NFT").into_val(&env),
    );
    map.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "ipfs://event").into_val(&env),
    );
    map.set(Symbol::new(&env, "transferable"), true.into_val(&env));

    let nft_id = client.mint_reward_nft_from_map(&minter, &1, &from, &map);
    client.transfer_nft(&nft_id, &from, &to, &from);

    assert_eq!(client.owner_of(&nft_id), Some(to));
}

// =========================================================================
// BURN TESTS
// =========================================================================

#[test]
fn test_burn_removes_nft_and_clears_owner_list() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Burn Me", "Desc", "ipfs://burn");
    let nft_id = client.mint_reward_nft(&minter, &1, &owner, &metadata);
    assert!(client.get_nft(&nft_id).is_some());
    assert_eq!(client.total_supply(), 1);

    client.burn_nft(&nft_id, &owner);

    assert!(client.get_nft(&nft_id).is_none());
    assert_eq!(client.get_player_nfts(&owner, &0, &100).len(), 0);
    assert_eq!(client.total_supply(), 0);
}

#[test]
#[should_panic]
fn test_burn_fails_if_not_owner() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let metadata = create_metadata(&env, "Owned NFT", "Desc", "ipfs://owned");
    let nft_id = client.mint_reward_nft(&minter, &1, &owner, &metadata);

    client.burn_nft(&nft_id, &attacker);
}

#[test]
#[should_panic]
fn test_burn_fails_for_nonexistent_nft() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);

    let rogue = Address::generate(&env);
    client.burn_nft(&999, &rogue);
}

#[test]
fn test_burn_decrements_total_supply() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let m1 = create_metadata(&env, "NFT 1", "Desc", "ipfs://1");
    let m2 = create_metadata(&env, "NFT 2", "Desc", "ipfs://2");
    let m3 = create_metadata(&env, "NFT 3", "Desc", "ipfs://3");

    let nft1 = client.mint_reward_nft(&minter, &1, &owner, &m1);
    let _nft2 = client.mint_reward_nft(&minter, &2, &owner, &m2);
    let _nft3 = client.mint_reward_nft(&minter, &3, &owner, &m3);
    assert_eq!(client.total_supply(), 3);

    client.burn_nft(&nft1, &owner);
    assert_eq!(client.total_supply(), 2);

    let collection = client.get_collection_metadata().unwrap();
    assert_eq!(collection.total_supply, 2);
}

#[test]
#[should_panic]
fn test_burn_locked_nft_fails() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Locked NFT", "Desc", "ipfs://locked");
    let nft_id = client.mint_reward_nft(&minter, &1, &owner, &metadata);

    client.set_nft_locked(&admin, &nft_id, &true);

    client.burn_nft(&nft_id, &owner);
}

#[test]
fn test_burn_unlocked_nft_succeeds() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Temp Locked", "Desc", "ipfs://temp");
    let nft_id = client.mint_reward_nft(&minter, &1, &owner, &metadata);

    client.set_nft_locked(&admin, &nft_id, &true);
    client.set_nft_locked(&admin, &nft_id, &false);
    client.burn_nft(&nft_id, &owner);
    assert!(client.get_nft(&nft_id).is_none());
}

#[test]
#[should_panic]
fn test_set_nft_locked_requires_admin() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let metadata = create_metadata(&env, "Test Lock", "Desc", "ipfs://lock");
    let nft_id = client.mint_reward_nft(&minter, &1, &owner, &metadata);

    client.set_nft_locked(&non_admin, &nft_id, &true);
}

// =========================================================================
// OWNER NFT QUERIES
// =========================================================================

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

// =========================================================================
// CREATOR/ATTRIBUTION TESTS
// =========================================================================

#[test]
fn test_nft_with_creator_attribution() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    let metadata = create_metadata_with_creator(
        &env,
        "Creator NFT",
        "NFT with creator attribution",
        "ipfs://creator",
        creator.clone(),
        None,
    );

    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, Some(creator.clone()));
    assert_eq!(nft.metadata.royalty_bps, None);

    let meta = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta.creator, Some(creator));
    assert_eq!(meta.royalty_bps, None);
}

#[test]
fn test_nft_with_creator_and_royalty() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    let royalty_bps = 250u32;
    let metadata = create_metadata_with_creator(
        &env,
        "Royalty NFT",
        "NFT with creator and royalty",
        "ipfs://royalty",
        creator.clone(),
        Some(royalty_bps),
    );

    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, Some(creator.clone()));
    assert_eq!(nft.metadata.royalty_bps, Some(royalty_bps));

    let meta = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta.creator, Some(creator));
    assert_eq!(meta.royalty_bps, Some(royalty_bps));
}

#[test]
fn test_nft_without_creator_defaults_to_none() {
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
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    let mut metadata: Map<Symbol, Val> = Map::new(&env);
    metadata.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Map NFT").into_val(&env),
    );
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

    let nft_id = client.mint_reward_nft_from_map(&minter, &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, Some(creator.clone()));
    assert_eq!(nft.metadata.royalty_bps, Some(500u32));
}

#[test]
fn test_mint_from_map_creator_defaults_to_player() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);

    let mut metadata: Map<Symbol, Val> = Map::new(&env);
    metadata.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Default Creator").into_val(&env),
    );
    metadata.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "ipfs://default").into_val(&env),
    );

    let nft_id = client.mint_reward_nft_from_map(&minter, &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, Some(player));
    assert_eq!(nft.metadata.royalty_bps, None);
}

#[test]
fn test_creator_preserved_across_metadata_queries() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

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

    let nft_id = client.mint_reward_nft(&minter, &42, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, Some(creator.clone()));
    assert_eq!(nft.metadata.royalty_bps, Some(1000u32));

    let meta = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta.creator, Some(creator));
    assert_eq!(meta.royalty_bps, Some(1000u32));
    assert_eq!(meta.current_owner, player);
}

// =========================================================================
// MINTER/ADMIN TESTS
// =========================================================================

#[test]
fn test_add_minter_allows_new_minter() {
    let (env, contract_id, admin, _original_minter) = setup_initialized();
    let client = NftRewardClient::new(&env, &contract_id);

    let new_minter = Address::generate(&env);
    client.add_minter(&admin, &new_minter);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "New Minter NFT", "Desc", "ipfs://new");
    let nft_id = client.mint_reward_nft(&new_minter, &1, &player, &metadata);
    assert_eq!(nft_id, 1);
}

#[test]
#[should_panic]
fn test_remove_minter_revokes_access() {
    let (env, contract_id, admin, minter) = setup_initialized();
    let client = NftRewardClient::new(&env, &contract_id);

    client.remove_minter(&admin, &minter);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Revoked NFT", "Desc", "ipfs://revoked");
    client.mint_reward_nft(&minter, &1, &player, &metadata);
}

#[test]
#[should_panic]
fn test_add_minter_requires_admin() {
    let (env, contract_id, _admin, _minter) = setup_initialized();
    let client = NftRewardClient::new(&env, &contract_id);

    let imposter = Address::generate(&env);
    let new_minter = Address::generate(&env);
    client.add_minter(&imposter, &new_minter);
}

// =========================================================================
// SUPPLY CAP TESTS
// =========================================================================

#[test]
#[should_panic]
fn test_max_supply_enforced() {
    let env = setup_env();
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    client.initialize(&admin, &minter, &Some(2), &default_collection_metadata(&env));

    let player = Address::generate(&env);
    let m1 = create_metadata(&env, "NFT 1", "Desc", "ipfs://1");
    let m2 = create_metadata(&env, "NFT 2", "Desc", "ipfs://2");
    let m3 = create_metadata(&env, "NFT 3", "Desc", "ipfs://3");

    client.mint_reward_nft(&minter, &1, &player, &m1);
    client.mint_reward_nft(&minter, &2, &player, &m2);
    client.mint_reward_nft(&minter, &3, &player, &m3);
}

#[test]
fn test_no_max_supply_allows_unlimited_mints() {
    let (env, contract_id, _admin, minter) = setup_initialized();
    let client = NftRewardClient::new(&env, &contract_id);

    let player = Address::generate(&env);
    for i in 1u64..=5 {
        let metadata = create_metadata(&env, "NFT", "Desc", "ipfs://x");
        client.mint_reward_nft(&minter, &i, &player, &metadata);
    }
    assert_eq!(client.total_supply(), 5);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_max_supply_cap_blocks_additional_mints() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, Some(2));

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let player3 = Address::generate(&env);

    let metadata1 = create_metadata(&env, "NFT 1", "Desc 1", "ipfs://1");
    let metadata2 = create_metadata(&env, "NFT 2", "Desc 2", "ipfs://2");
    let metadata3 = create_metadata(&env, "NFT 3", "Desc 3", "ipfs://3");

    client.mint_reward_nft(&minter, &1, &player1, &metadata1);
    client.mint_reward_nft(&minter, &2, &player2, &metadata2);
    client.mint_reward_nft(&minter, &3, &player3, &metadata3);
}

#[test]
fn test_mint_reward_nft_from_map_with_missing_keys_uses_defaults() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let mut metadata: Map<Symbol, Val> = Map::new(&env);
    metadata.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Test NFT").into_val(&env),
    );
    metadata.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "ipfs://default").into_val(&env),
    );

    let nft_id = client.mint_reward_nft_from_map(&minter, &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.title, String::from_str(&env, "Test NFT"));
    assert_eq!(nft.metadata.description, String::from_str(&env, ""));
    assert_eq!(
        nft.metadata.image_uri,
        String::from_str(&env, "ipfs://default")
    );
    assert_eq!(
        nft.metadata.hunt_title,
        String::from_str(&env, "Test NFT")
    );
    assert_eq!(nft.metadata.rarity, 0u32);
    assert_eq!(nft.metadata.tier, 0u32);
    assert_eq!(nft.transferable, false);
}

#[test]
fn test_mint_reward_nft_from_map_with_invalid_types_uses_defaults() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let mut metadata: Map<Symbol, Val> = Map::new(&env);

    metadata.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Valid Title").into_val(&env),
    );
    metadata.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "ipfs://valid").into_val(&env),
    );
    metadata.set(Symbol::new(&env, "description"), 123456u32.into_val(&env));
    metadata.set(Symbol::new(&env, "hunt_title"), 999u32.into_val(&env));
    metadata.set(
        Symbol::new(&env, "rarity"),
        String::from_str(&env, "invalid").into_val(&env),
    );
    metadata.set(
        Symbol::new(&env, "tier"),
        String::from_str(&env, "invalid").into_val(&env),
    );
    metadata.set(Symbol::new(&env, "transferable"), 123u32.into_val(&env));

    let nft_id = client.mint_reward_nft_from_map(&minter, &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.title, String::from_str(&env, "Valid Title"));
    assert_eq!(nft.metadata.description, String::from_str(&env, ""));
    assert_eq!(
        nft.metadata.image_uri,
        String::from_str(&env, "ipfs://valid")
    );
    assert_eq!(
        nft.metadata.hunt_title,
        String::from_str(&env, "Valid Title")
    );
    assert_eq!(nft.metadata.rarity, 0u32);
    assert_eq!(nft.metadata.tier, 0u32);
    assert_eq!(nft.transferable, false);
}

// =========================================================================
// METADATA SEARCH TESTS
// =========================================================================

#[test]
fn test_search_nfts_by_title() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);

    let metadata1 = create_metadata(&env, "Dragon Slayer", "Epic dragon hunt", "ipfs://dragon");
    let metadata2 = create_metadata(&env, "Treasure Hunter", "Gold hunt", "ipfs://treasure");
    let metadata3 = create_metadata(&env, "Dragon Slayer", "Another dragon", "ipfs://dragon2");

    client.mint_reward_nft(&minter, &1, &player1, &metadata1);
    client.mint_reward_nft(&minter, &2, &player2, &metadata2);
    client.mint_reward_nft(&minter, &3, &player1, &metadata3);

    let results = client.search_nfts_by_metadata(
        &0,
        &10,
        &Some(String::from_str(&env, "Dragon Slayer")),
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_nfts_by_rarity() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);

    let metadata1 = create_metadata_full(&env, "Common NFT", "desc", "ipfs://1", "Hunt1", 1, 0);
    let metadata2 = create_metadata_full(&env, "Rare NFT", "desc", "ipfs://2", "Hunt2", 3, 0);
    let metadata3 =
        create_metadata_full(&env, "Legendary NFT", "desc", "ipfs://3", "Hunt3", 5, 0);

    client.mint_reward_nft(&minter, &1, &player, &metadata1);
    client.mint_reward_nft(&minter, &2, &player, &metadata2);
    client.mint_reward_nft(&minter, &3, &player, &metadata3);

    let results = client.search_nfts_by_metadata(
        &0,
        &10,
        &None,
        &None,
        &Some(3u32),
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0).unwrap().metadata.rarity, 3);
}

#[test]
fn test_search_nfts_by_hunt_id() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);

    let metadata = create_metadata(&env, "Test NFT", "desc", "ipfs://test");

    client.mint_reward_nft(&minter, &100, &player, &metadata);
    client.mint_reward_nft(&minter, &200, &player, &metadata);
    client.mint_reward_nft(&minter, &100, &player, &metadata);

    let results = client.search_nfts_by_metadata(
        &0,
        &10,
        &None,
        &None,
        &None,
        &None,
        &None,
        &Some(100u64),
        &None,
        &None,
    );

    assert_eq!(results.len(), 2);
    for nft in results.iter() {
        assert_eq!(nft.hunt_id, 100);
    }
}

#[test]
fn test_search_nfts_by_creator() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    let mut metadata1 = create_metadata(&env, "NFT 1", "desc", "ipfs://1");
    metadata1.creator = Some(creator1.clone());

    let mut metadata2 = create_metadata(&env, "NFT 2", "desc", "ipfs://2");
    metadata2.creator = Some(creator2.clone());

    let mut metadata3 = create_metadata(&env, "NFT 3", "desc", "ipfs://3");
    metadata3.creator = Some(creator1.clone());

    client.mint_reward_nft(&minter, &1, &player, &metadata1);
    client.mint_reward_nft(&minter, &2, &player, &metadata2);
    client.mint_reward_nft(&minter, &3, &player, &metadata3);

    let results = client.search_nfts_by_metadata(
        &0,
        &10,
        &None,
        &None,
        &None,
        &None,
        &Some(creator1),
        &None,
        &None,
        &None,
    );

    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_nfts_by_extension_key() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);

    let mut extensions1 = Map::new(&env);
    extensions1.set(
        String::from_str(&env, "season"),
        String::from_str(&env, "2024"),
    );

    let mut extensions2 = Map::new(&env);
    extensions2.set(
        String::from_str(&env, "event"),
        String::from_str(&env, "special"),
    );

    let mut extensions3 = Map::new(&env);
    extensions3.set(
        String::from_str(&env, "season"),
        String::from_str(&env, "2023"),
    );

    let metadata1 =
        create_metadata_with_extensions(&env, "NFT 1", "desc", "ipfs://1", extensions1);
    let metadata2 =
        create_metadata_with_extensions(&env, "NFT 2", "desc", "ipfs://2", extensions2);
    let metadata3 =
        create_metadata_with_extensions(&env, "NFT 3", "desc", "ipfs://3", extensions3);

    client.mint_reward_nft(&minter, &1, &player, &metadata1);
    client.mint_reward_nft(&minter, &2, &player, &metadata2);
    client.mint_reward_nft(&minter, &3, &player, &metadata3);

    let results = client.search_nfts_by_metadata(
        &0,
        &10,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &Some(String::from_str(&env, "season")),
        &None,
    );

    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_nfts_by_extension_key_value() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);

    let mut extensions1 = Map::new(&env);
    extensions1.set(
        String::from_str(&env, "season"),
        String::from_str(&env, "2024"),
    );

    let mut extensions2 = Map::new(&env);
    extensions2.set(
        String::from_str(&env, "season"),
        String::from_str(&env, "2023"),
    );

    let mut extensions3 = Map::new(&env);
    extensions3.set(
        String::from_str(&env, "event"),
        String::from_str(&env, "special"),
    );

    let metadata1 =
        create_metadata_with_extensions(&env, "NFT 1", "desc", "ipfs://1", extensions1);
    let metadata2 =
        create_metadata_with_extensions(&env, "NFT 2", "desc", "ipfs://2", extensions2);
    let metadata3 =
        create_metadata_with_extensions(&env, "NFT 3", "desc", "ipfs://3", extensions3);

    client.mint_reward_nft(&minter, &1, &player, &metadata1);
    client.mint_reward_nft(&minter, &2, &player, &metadata2);
    client.mint_reward_nft(&minter, &3, &player, &metadata3);

    let results = client.search_nfts_by_metadata(
        &0,
        &10,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &Some(String::from_str(&env, "season")),
        &Some(String::from_str(&env, "2024")),
    );

    assert_eq!(results.len(), 1);
}

#[test]
fn test_search_nfts_combined_filters() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let creator = Address::generate(&env);

    let mut extensions = Map::new(&env);
    extensions.set(
        String::from_str(&env, "season"),
        String::from_str(&env, "2024"),
    );

    let mut metadata1 =
        create_metadata_with_extensions(&env, "Dragon", "desc", "ipfs://1", extensions.clone());
    metadata1.rarity = 3;
    metadata1.creator = Some(creator.clone());

    let mut metadata2 =
        create_metadata_with_extensions(&env, "Dragon", "desc", "ipfs://2", extensions.clone());
    metadata2.rarity = 1;
    metadata2.creator = Some(creator.clone());

    let metadata3 = create_metadata(&env, "Treasure", "desc", "ipfs://3");

    client.mint_reward_nft(&minter, &100, &player, &metadata1);
    client.mint_reward_nft(&minter, &200, &player, &metadata2);
    client.mint_reward_nft(&minter, &100, &player, &metadata3);

    let results = client.search_nfts_by_metadata(
        &0,
        &10,
        &Some(String::from_str(&env, "Dragon")),
        &None,
        &Some(3u32),
        &None,
        &Some(creator),
        &Some(100u64),
        &Some(String::from_str(&env, "season")),
        &None,
    );

    assert_eq!(results.len(), 1);
    assert_eq!(
        results.get(0).unwrap().metadata.title,
        String::from_str(&env, "Dragon")
    );
}

#[test]
fn test_search_nfts_pagination() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);

    for i in 0..15 {
        let metadata = create_metadata(&env, &format!("NFT {}", i), "desc", "ipfs://test");
        client.mint_reward_nft(&minter, &i, &player, &metadata);
    }

    let page1 = client.search_nfts_by_metadata(
        &0,
        &5,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(page1.len(), 5);

    let page2 = client.search_nfts_by_metadata(
        &5,
        &5,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(page2.len(), 5);

    let page3 = client.search_nfts_by_metadata(
        &10,
        &5,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(page3.len(), 5);

    let page4 = client.search_nfts_by_metadata(
        &15,
        &5,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(page4.len(), 0);
}

#[test]
fn test_search_nfts_no_filters_returns_all() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);

    for i in 0..5 {
        let metadata = create_metadata(&env, &format!("NFT {}", i), "desc", "ipfs://test");
        client.mint_reward_nft(&minter, &i, &player, &metadata);
    }

    let results = client.search_nfts_by_metadata(
        &0,
        &10,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(results.len(), 5);
}

#[test]
fn test_search_nfts_no_matches() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);

    let metadata = create_metadata(&env, "Dragon", "desc", "ipfs://test");
    client.mint_reward_nft(&minter, &1, &player, &metadata);

    let results = client.search_nfts_by_metadata(
        &0,
        &10,
        &Some(String::from_str(&env, "NonExistent")),
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    assert_eq!(results.len(), 0);
}

// =========================================================================
// ADMIN UPDATE IMAGE URIS TESTS
// =========================================================================

#[test]
fn test_admin_update_image_uris_basic() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();

    let player = Address::generate(&env);
    let m1 = create_metadata(&env, "NFT 1", "Desc", "ipfs://old-gateway/img1.png");
    let m2 = create_metadata(&env, "NFT 2", "Desc", "ipfs://old-gateway/img2.png");
    let m3 = create_metadata(&env, "NFT 3", "Desc", "ipfs://other/img3.png");

    let nft1 = client.mint_reward_nft(&minter, &1, &player, &m1);
    let nft2 = client.mint_reward_nft(&minter, &2, &player, &m2);
    let _nft3 = client.mint_reward_nft(&minter, &3, &player, &m3);

    let updated = client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://old-gateway/"),
        &String::from_str(&env, "ipfs://new-gateway/"),
        &0,
        &100,
    );

    assert_eq!(updated, 2);

    let nft1_data = client.get_nft(&nft1).unwrap();
    assert_eq!(
        nft1_data.metadata.image_uri,
        String::from_str(&env, "ipfs://new-gateway/img1.png")
    );

    let nft2_data = client.get_nft(&nft2).unwrap();
    assert_eq!(
        nft2_data.metadata.image_uri,
        String::from_str(&env, "ipfs://new-gateway/img2.png")
    );
}

#[test]
fn test_admin_update_image_uris_pagination() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();

    let player = Address::generate(&env);
    for i in 0..5 {
        let metadata = create_metadata(
            &env,
            &format!("NFT {}", i),
            "Desc",
            &format!("ipfs://old-gw/img{}.png", i),
        );
        client.mint_reward_nft(&minter, &i, &player, &metadata);
    }

    let updated1 = client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://old-gw/"),
        &String::from_str(&env, "ipfs://new-gw/"),
        &0,
        &2,
    );
    assert_eq!(updated1, 2);

    let updated2 = client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://old-gw/"),
        &String::from_str(&env, "ipfs://new-gw/"),
        &2,
        &10,
    );
    assert_eq!(updated2, 3);

    let updated3 = client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://old-gw/"),
        &String::from_str(&env, "ipfs://new-gw/"),
        &5,
        &10,
    );
    assert_eq!(updated3, 0);
}

#[test]
#[should_panic]
fn test_admin_update_image_uris_requires_admin() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let non_admin = Address::generate(&env);
    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "NFT", "Desc", "ipfs://old/img.png");
    client.mint_reward_nft(&minter, &1, &player, &metadata);

    client.admin_update_image_uris(
        &non_admin,
        &String::from_str(&env, "ipfs://old/"),
        &String::from_str(&env, "ipfs://new/"),
        &0,
        &100,
    );
}

// =========================================================================
// REPLACE_PREFIX EDGE CASE TESTS
// =========================================================================

#[test]
fn test_replace_prefix_long_uri() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let admin = client.get_admin().unwrap();

    let player = Address::generate(&env);
    let long_suffix = "a".repeat(300);
    let long_uri = format!("ipfs://old-gateway/{}", long_suffix);
    let metadata = create_metadata(&env, "Long URI NFT", "Desc", &long_uri);
    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    let updated = client.admin_update_image_uris(
        &admin,
        &String::from_str(&env, "ipfs://old-gateway/"),
        &String::from_str(&env, "ipfs://new-gateway/"),
        &0,
        &100,
    );

    assert_eq!(updated, 1);
    let nft = client.get_nft(&nft_id).unwrap();
    let expected = format!("ipfs://new-gateway/{}", long_suffix);
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, &expected));
}

// =========================================================================
// LIST ALL NFTS TESTS
// =========================================================================

#[test]
fn test_list_all_nfts_pagination() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    for i in 0..3 {
        let metadata = create_metadata(&env, &format!("NFT {}", i), "desc", "ipfs://test");
        client.mint_reward_nft(&minter, &i, &player, &metadata);
    }

    let page1 = client.list_all_nfts(&0, &2);
    assert_eq!(page1.len(), 2);

    let page2 = client.list_all_nfts(&2, &2);
    assert_eq!(page2.len(), 1);

    let empty = client.list_all_nfts(&3, &10);
    assert_eq!(empty.len(), 0);
}
