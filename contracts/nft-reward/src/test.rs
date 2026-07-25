#![cfg(test)]
extern crate std;

use crate::{
    CollectionMetadata, NftMetadata, NftMintedEvent, NftReward, NftRewardClient, NftErrorCode,
    METADATA_SCHEMA_VERSION,
};
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

// =========================================================================
// EXISTING TESTS (preserved from original)
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
#[should_panic(expected = "HostError")]
fn test_initialize_requires_auth() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(&env, &contract_id);

    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));
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
    let err = client.transfer_nft(&nft_id, &owner, &recipient, &owner).unwrap_err();

    assert_eq!(err, NftErrorCode::NftNotTransferable);
    assert_eq!(client.owner_of(&nft_id).unwrap(), owner);
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
    assert_eq!(event.metadata.hunt_title, metadata.hunt_title);
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
    assert_eq!(meta.schema_version, METADATA_SCHEMA_VERSION);
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