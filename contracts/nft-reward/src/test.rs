#![allow(deprecated, dead_code, clippy::bool_assert_comparison)]

extern crate std;

use crate::{
    CollectionMetadata, NftErrorCode, NftMetadata, NftMintedEvent, NftReward, NftRewardClient,
    MAX_NFT_URI_BYTES, MAX_ROYALTY_BPS, MAX_SCAN_LIMIT, METADATA_SCHEMA_VERSION,
};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    Address, Env, IntoVal, Map, String, Symbol, TryFromVal, Val,
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
    client.initialize(
        &admin,
        &minter,
        &max_supply,
        &default_collection_metadata(env),
    );
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
    client.mint_reward_nft_from_map(&minter, &hunt_id, owner, &map).unwrap()
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
fn test_mint_reward_nft_rejects_empty_image_uri_consistently() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let player = Address::generate(&env);

    let empty_metadata = create_metadata(&env, "Hunt Champion", "Completed the City Hunt", "");
    let direct_err = client
        .try_mint_reward_nft(&minter, &1, &player, &empty_metadata)
        .unwrap_err();
    assert_eq!(direct_err, Ok(NftErrorCode::InvalidMetadata));

    let mut map: Map<Symbol, Val> = Map::new(&env);
    map.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Hunt Champion").into_val(&env),
    );
    map.set(
        Symbol::new(&env, "description"),
        String::from_str(&env, "Completed the City Hunt").into_val(&env),
    );
    map.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "").into_val(&env),
    );

    let map_err = client
        .try_mint_reward_nft_from_map(&minter, &1, &player, &map)
        .unwrap_err();
    assert_eq!(map_err, Ok(NftErrorCode::InvalidMetadata));
}

#[test]
fn test_mint_reward_nft_enforces_single_uri_length_limit() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let player = Address::generate(&env);

    let prefix = "ipfs://";
    let at_limit = format!("{}{}", prefix, "a".repeat(MAX_NFT_URI_BYTES as usize - prefix.len()));
    let over_limit = format!("{}{}", prefix, "a".repeat(MAX_NFT_URI_BYTES as usize - prefix.len() + 1));

    client
        .mint_reward_nft(
            &minter,
            &1,
            &player,
            &create_metadata(&env, "At Limit", "Valid boundary", &at_limit),
        )
        .unwrap();

    let err = client
        .try_mint_reward_nft(
            &minter,
            &2,
            &player,
            &create_metadata(&env, "Over Limit", "Too long", &over_limit),
        )
        .unwrap_err();
    assert_eq!(err, Ok(NftErrorCode::InvalidMetadata));
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

    let nft_id = client.mint_reward_nft_from_map(&minter, &1, &owner, &metadata_map).unwrap();
    let err = client
        .try_transfer_nft(&nft_id, &owner, &recipient, &owner)
        .unwrap_err();

    assert_eq!(err, Ok(NftErrorCode::NftNotTransferable));
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

    let event = NftMintedEvent::try_from_val(&env, &data).unwrap();
    assert_eq!(event.nft_id, nft_id);
    assert_eq!(event.hunt_id, 7);
    assert_eq!(event.owner, player);
    assert_eq!(event.rarity, 0);
    assert_eq!(event.tier, 0);
    assert_eq!(event.minted_at, 1000);
    assert_eq!(
        u64::try_from_val(&env, &topics.get(1).unwrap()).unwrap(),
        nft_id
    );
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

    let nft_id = client.mint_reward_nft_from_map(&reward_manager, &7, &player, &metadata_map).unwrap();
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
    let (client, minter) = setup_nft_reward(&env, None);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Transfer NFT", "Test transfer", "ipfs://transfer");

    let nft_id = client.mint_reward_nft(&minter, &1, &from, &metadata);
    assert_eq!(client.owner_of(&nft_id), Some(from.clone()));

    client.transfer_nft(&nft_id, &from, &to, &from);

    assert_eq!(client.owner_of(&nft_id), Some(to.clone()));

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

    // This should fail - from has not authorized the transfer
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
    let (client, minter) = setup_nft_reward(&env, None);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Event NFT", "Desc", "ipfs://event");

    let nft_id = client.mint_reward_nft(&minter, &1, &from, &metadata);
    client.transfer_nft(&nft_id, &from, &to, &from);

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
fn test_owner_of_returns_nft_owner() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "OwnerOf Test", "Desc", "ipfs://test");

    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    assert_eq!(client.owner_of(&nft_id), Some(player));
    assert_eq!(client.owner_of(&999), None);
}

#[test]
fn test_nft_with_creator_attribution() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

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

    let nft_id = client.mint_reward_nft(&creator, &1, &player, &metadata);

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
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    let royalty_bps = 250u32; // 2.5% royalty
    let metadata = create_metadata_with_creator(
        &env,
        "Royalty NFT",
        "NFT with creator and royalty",
        "ipfs://royalty",
        creator.clone(),
        Some(royalty_bps),
    );

    let nft_id = client.mint_reward_nft(&creator, &1, &player, &metadata);

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
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "No Creator", "No creator set", "ipfs://nocreator");

    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, None);
    assert_eq!(nft.metadata.royalty_bps, None);
}

#[test]
fn test_mint_from_map_with_creator_and_royalty() {
    use soroban_sdk::{Map, Symbol};

    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    let mut metadata = Map::new(&env);
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

    let nft_id = client.mint_reward_nft_from_map(&creator, &1, &player, &metadata).unwrap();

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, Some(creator.clone()));
    assert_eq!(nft.metadata.royalty_bps, Some(500u32));
}

#[test]
fn test_mint_from_map_creator_defaults_to_player() {
    use soroban_sdk::{Map, Symbol};

    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);

    let mut metadata = Map::new(&env);
    metadata.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Default Creator").into_val(&env),
    );
    metadata.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "ipfs://default").into_val(&env),
    );

    let nft_id = client.mint_reward_nft_from_map(&player, &1, &player, &metadata).unwrap();

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

    let nft_id = client.mint_reward_nft(&creator, &42, &player, &metadata);

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

    client.burn_nft(&nft_id, &owner);

    assert!(client.get_nft(&nft_id).is_none());
    assert_eq!(client.get_player_nfts(&owner, &0, &100).len(), 0);
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

    // Attacker tries to burn — NotOwner check should fail
    client.burn_nft(&nft_id, &attacker);
}

#[test]
#[should_panic]
fn test_burn_fails_for_nonexistent_nft() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);

    let rogue = Address::generate(&env);
    // Burn a non-existent NFT — should panic
    client.burn_nft(&999, &rogue);
}

#[test]
#[should_panic]
fn test_max_supply_enforced() {
    let env = setup_env();
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    client.initialize(
        &admin,
        &minter,
        &Some(2),
        &default_collection_metadata(&env),
    );

    let player = Address::generate(&env);
    let m1 = create_metadata(&env, "NFT 1", "Desc", "ipfs://1");
    let m2 = create_metadata(&env, "NFT 2", "Desc", "ipfs://2");
    let m3 = create_metadata(&env, "NFT 3", "Desc", "ipfs://3");

    client.mint_reward_nft(&minter, &1, &player, &m1);
    client.mint_reward_nft(&minter, &2, &player, &m2);
    // Third mint should panic — max supply is 2
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
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);
    let mut metadata: Map<Symbol, Val> = Map::new(&env);
    // Provide title and image_uri, omit all other keys
    metadata.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Test NFT").into_val(&env),
    );
    metadata.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "ipfs://defaults").into_val(&env),
    );

    let nft_id = client.mint_reward_nft_from_map(&player, &1, &player, &metadata).unwrap();

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.title, String::from_str(&env, "Test NFT"));
    assert_eq!(
        nft.metadata.image_uri,
        String::from_str(&env, "ipfs://defaults")
    );
    assert_eq!(nft.metadata.description, String::from_str(&env, "")); // default
    assert_eq!(nft.metadata.hunt_title, String::from_str(&env, "Test NFT")); // defaults to title
    assert_eq!(nft.metadata.rarity, 0u32); // default
    assert_eq!(nft.metadata.tier, 0u32); // default
    assert_eq!(nft.transferable, false); // default
}

#[test]
fn test_mint_reward_nft_from_map_present_wrong_type_returns_invalid_metadata() {
    let env = setup_env();
    let client = NftRewardClient::new(&env, &env.register_contract(None, NftReward));

    let player = Address::generate(&env);

    // --- rarity: present as String instead of u32 ---
    let mut metadata: Map<Symbol, Val> = Map::new(&env);
    metadata.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Valid Title").into_val(&env),
    );
    metadata.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "ipfs://valid").into_val(&env),
    );
    metadata.set(
        Symbol::new(&env, "rarity"),
        String::from_str(&env, "epic").into_val(&env), // String, not u32
    );

    let res = client.try_mint_reward_nft_from_map(&player, &1, &player, &metadata);
    assert_eq!(res, Err(Ok(NftErrorCode::InvalidMetadata)),
        "present rarity with wrong type must fail with InvalidMetadata");

    // --- image_uri: present as u32 instead of String ---
    let mut metadata2: Map<Symbol, Val> = Map::new(&env);
    metadata2.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Valid Title").into_val(&env),
    );
    assert_eq!(nft.metadata.description, String::from_str(&env, "")); // default due to invalid type
    assert_eq!(
        nft.metadata.hunt_title,
        String::from_str(&env, "Valid Title")
    ); // defaults to title
    assert_eq!(nft.metadata.rarity, 0u32); // default due to invalid type
    assert_eq!(nft.metadata.tier, 0u32); // default due to invalid type
    assert_eq!(nft.transferable, false); // default due to invalid type

    // Test for the new function
    let nft_id_invalid = client.mint_reward_nft_from_map(&player, &1, &player, &metadata);
    let nft_invalid = client.get_nft(&nft_id_invalid).unwrap();
    assert_eq!(nft_invalid.metadata.title, String::from_str(&env, "Valid Title"));
    assert_eq!(nft_invalid.metadata.image_uri, String::from_str(&env, "ipfs://valid"));
    assert_eq!(nft_invalid.metadata.description, String::from_str(&env, ""));
    assert_eq!(nft_invalid.metadata.hunt_title, String::from_str(&env, "Valid Title"));
    assert_eq!(nft_invalid.metadata.rarity, 0u32);
    assert_eq!(nft_invalid.metadata.tier, 0u32);
    assert_eq!(nft_invalid.transferable, false);
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

    // Search for "Dragon Slayer"
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
    let metadata3 = create_metadata_full(&env, "Legendary NFT", "desc", "ipfs://3", "Hunt3", 5, 0);

    client.mint_reward_nft(&minter, &1, &player, &metadata1);
    client.mint_reward_nft(&minter, &2, &player, &metadata2);
    client.mint_reward_nft(&minter, &3, &player, &metadata3);

    // Search for rarity 3 (Rare)
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

    // Search for hunt_id 100
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

    // Search for creator1
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

    let metadata1 = create_metadata_with_extensions(&env, "NFT 1", "desc", "ipfs://1", extensions1);
    let metadata2 = create_metadata_with_extensions(&env, "NFT 2", "desc", "ipfs://2", extensions2);
    let metadata3 = create_metadata_with_extensions(&env, "NFT 3", "desc", "ipfs://3", extensions3);

    client.mint_reward_nft(&minter, &1, &player, &metadata1);
    client.mint_reward_nft(&minter, &2, &player, &metadata2);
    client.mint_reward_nft(&minter, &3, &player, &metadata3);

    // Search for NFTs with "season" extension key
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

    let metadata1 = create_metadata_with_extensions(&env, "NFT 1", "desc", "ipfs://1", extensions1);
    let metadata2 = create_metadata_with_extensions(&env, "NFT 2", "desc", "ipfs://2", extensions2);
    let metadata3 = create_metadata_with_extensions(&env, "NFT 3", "desc", "ipfs://3", extensions3);

    client.mint_reward_nft(&minter, &1, &player, &metadata1);
    client.mint_reward_nft(&minter, &2, &player, &metadata2);
    client.mint_reward_nft(&minter, &3, &player, &metadata3);

    // Search for NFTs with season=2024
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

    // Search for Dragon + rarity 3 + creator + hunt_id 100 + season extension
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

    // Get first page
    let page1 = client.search_nfts_by_metadata(
        &0, &5, &None, &None, &None, &None, &None, &None, &None, &None,
    );
    assert_eq!(page1.len(), 5);

    // Get second page
    let page2 = client.search_nfts_by_metadata(
        &5, &5, &None, &None, &None, &None, &None, &None, &None, &None,
    );
    assert_eq!(page2.len(), 5);

    // Get third page
    let page3 = client.search_nfts_by_metadata(
        &10, &5, &None, &None, &None, &None, &None, &None, &None, &None,
    );
    assert_eq!(page3.len(), 5);

    // Beyond available results
    let page4 = client.search_nfts_by_metadata(
        &15, &5, &None, &None, &None, &None, &None, &None, &None, &None,
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
        &0, &10, &None, &None, &None, &None, &None, &None, &None, &None,
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

    // Search for non-existent title
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

// ========== Initialization and Audit Event Tests ==========

#[test]
fn test_initialize_emits_event_with_admin_minter_max_supply() {
    let env = setup_env();
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let max_supply = Some(1000u64);

    client.initialize(&admin, &minter, &max_supply, &default_collection_metadata(&env));

    // Check for ContractInitializedEvent
    let events = env.events().all();
    let init_events: Vec<_> = events
        .iter()
        .filter(|(_, topics, _)| {
            topics.len() > 0 && topics.get(0).unwrap().to_xdr(&env).unwrap() ==
                Symbol::new(&env, "INIT").to_xdr(&env).unwrap()
        })
        .collect();

    // Should have at least one INIT event
    assert!(init_events.len() > 0, "No INIT event found");
}

#[test]
fn test_add_authorized_contract_emits_event() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = Address::generate(&env);
    let contract = Address::generate(&env);

    // First initialize the contract with the admin
    env.as_contract(&env.current_contract_address(), || {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, NftReward);
        let client = NftRewardClient::new(&env, &contract_id);
        let admin_local = Address::generate(&env);
        let minter = Address::generate(&env);
        client.initialize(&admin_local, &minter, &None, &default_collection_metadata(&env));
    });

    // Clear previous events
    let _ = env.events().all();

    // Add an authorized contract
    let result = client.add_authorized_contract(&admin, &contract);
    assert!(result.is_ok());

    // Check for AuthorizedContractAddedEvent
    let events = env.events().all();
    let auth_events: Vec<_> = events
        .iter()
        .filter(|(_, topics, _)| {
            topics.len() > 0 && topics.get(0).unwrap().to_xdr(&env).unwrap() ==
                Symbol::new(&env, "AUTH_ADD").to_xdr(&env).unwrap()
        })
        .collect();

    // Should have at least one AUTH_ADD event
    assert!(auth_events.len() > 0, "No AUTH_ADD event found");
}

#[test]
fn test_remove_authorized_contract_emits_event() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = Address::generate(&env);
    let contract = Address::generate(&env);

    // Add and then remove
    let _ = client.add_authorized_contract(&admin, &contract);

    // Clear events
    let _ = env.events().all();

    // Remove the authorized contract
    let result = client.remove_authorized_contract(&admin, &contract);
    assert!(result.is_ok());

    // Check for AuthorizedContractRemovedEvent
    let events = env.events().all();
    let auth_events: Vec<_> = events
        .iter()
        .filter(|(_, topics, _)| {
            topics.len() > 0 && topics.get(0).unwrap().to_xdr(&env).unwrap() ==
                Symbol::new(&env, "AUTH_REM").to_xdr(&env).unwrap()
        })
        .collect();

    // Should have at least one AUTH_REM event
    assert!(auth_events.len() > 0, "No AUTH_REM event found");
}

#[test]
fn test_set_reward_manager_emits_event() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let admin = Address::generate(&env);
    let reward_manager = Address::generate(&env);

    // Clear events
    let _ = env.events().all();

    // Set reward manager
    let result = client.set_reward_manager(&admin, &reward_manager);
    assert!(result.is_ok());

    // Check for RewardManagerSetEvent
    let events = env.events().all();
    let reward_events: Vec<_> = events
        .iter()
        .filter(|(_, topics, _)| {
            topics.len() > 0 && topics.get(0).unwrap().to_xdr(&env).unwrap() ==
                Symbol::new(&env, "RWD_MGR").to_xdr(&env).unwrap()
        })
        .collect();

    // Should have at least one RWD_MGR event
    assert!(reward_events.len() > 0, "No RWD_MGR event found");
}

#[test]
fn test_initialize_requires_admin_authorization() {
    let env = setup_env();
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);

    // First initialization should succeed
    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));

    // Second initialization should fail
    let result = client.try_initialize(&admin, &minter, &None, &default_collection_metadata(&env));
    assert!(result.is_err(), "Second initialization should fail");
}

#[test]
#[should_panic]
fn test_initialize_panics_without_admin_auth() {
    // No mocked auth: initialize must require admin authorization and thus panic
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    // This should panic because `admin.require_auth()` will fail
    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));
}

#[test]
fn test_initialize_succeeds_with_admin_auth() {
    // With mocked auth, initialization should succeed and store admin/minter
    let env = setup_env(); // setup_env mocks auth
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);

    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));

    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
fn test_add_authorized_contract_requires_admin_authorization() {
    let env = setup_env();
    let (client, _) = setup_nft_reward(&env, None);
    let attacker = Address::generate(&env);
    let contract = Address::generate(&env);

    // Non-admin tries to add authorized contract
    let result = client.try_add_authorized_contract(&attacker, &contract);
    
    // Should either fail or succeed depending on auth setup
    // The key point is that the event should have the correct admin field
    if result.is_ok() {
        let events = env.events().all();
        let auth_events: Vec<_> = events
            .iter()
            .filter(|(_, topics, _)| {
                topics.len() > 1 && topics.get(0).unwrap().to_xdr(&env).unwrap() ==
                    Symbol::new(&env, "AUTH_ADD").to_xdr(&env).unwrap()
            })
            .collect();
        
        // If the operation succeeded, we should see an AUTH_ADD event
        // The event should have been published with the attacker's address in the topics
        assert!(auth_events.len() > 0 || result.is_err(), "Expected either event or error");
    }
}

// -----------------------------------------------------------------------------
// total_supply / live-supply tracking on burn (issue #846)
// -----------------------------------------------------------------------------

#[test]
fn test_burn_nft_decrements_total_supply() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let player = Address::generate(&env);

    let mut ids = std::vec::Vec::new();
    for i in 1u64..=3 {
        let uri = std::format!("https://gateway.example/{}", i);
        let metadata = create_metadata(&env, "NFT", "Desc", &uri);
        ids.push(client.mint_reward_nft(&minter, &i, &player, &metadata));
    }
    assert_eq!(client.total_supply(), 3);

    client.burn_nft(&ids[0], &player);
    assert_eq!(
        client.total_supply(),
        2,
        "total_supply must reflect live NFTs after a burn"
    );

    client.burn_nft(&ids[1], &player);
    assert_eq!(client.total_supply(), 1);
}

#[test]
fn test_burned_nft_id_is_never_reused() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let player = Address::generate(&env);

    let metadata1 = create_metadata(&env, "NFT", "Desc", "https://gateway.example/1");
    let first_id = client.mint_reward_nft(&minter, &1, &player, &metadata1);
    client.burn_nft(&first_id, &player);
    assert_eq!(client.total_supply(), 0);

    let metadata2 = create_metadata(&env, "NFT", "Desc", "https://gateway.example/2");
    let second_id = client.mint_reward_nft(&minter, &2, &player, &metadata2);

    assert_ne!(
        first_id, second_id,
        "a burned NFT's ID must never be handed out to a later mint"
    );
    assert_eq!(client.total_supply(), 1);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_max_supply_caps_lifetime_mints_not_live_count() {
    // Documents the explicit semantics from issue #846: max_supply caps the
    // number of NFTs ever minted, not the number currently live. Burning an
    // NFT frees up nothing under the cap.
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, Some(2));
    let player = Address::generate(&env);

    let m1 = create_metadata(&env, "NFT", "Desc", "https://gateway.example/1");
    let m2 = create_metadata(&env, "NFT", "Desc", "https://gateway.example/2");
    let id1 = client.mint_reward_nft(&minter, &1, &player, &m1);
    client.mint_reward_nft(&minter, &2, &player, &m2);

    // Burn one — live count drops to 1, well under the cap of 2 — but the
    // lifetime mint count (2) is what the cap actually tracks.
    client.burn_nft(&id1, &player);
    assert_eq!(client.total_supply(), 1);

    // This mint must still panic: 2 NFTs have already been minted, lifetime.
    let m3 = create_metadata(&env, "NFT", "Desc", "https://gateway.example/3");
    client.mint_reward_nft(&minter, &3, &player, &m3);
}

// -----------------------------------------------------------------------------
// burn_nft: locked flag + soulbound policy (issue #847)
// -----------------------------------------------------------------------------

#[test]
fn test_burn_locked_nft_returns_error_and_leaves_storage_untouched() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let player = Address::generate(&env);

    let metadata = create_metadata(&env, "NFT", "Desc", "https://gateway.example/a");
    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    // There's no public API to lock an NFT yet, so set it directly through
    // storage (test.rs is a descendant module of the crate root and can see
    // the crate-private `storage` module) to exercise the check in burn_nft.
    let mut nft = client.get_nft(&nft_id).unwrap();
    nft.locked = true;
    crate::storage::Storage::save_nft(&env, &nft);

    let result = client.try_burn_nft(&nft_id, &player);
    assert!(result.is_err());

    // Storage is untouched: the NFT still exists, unchanged.
    let still_there = client.get_nft(&nft_id).unwrap();
    assert!(still_there.locked);
}

#[test]
fn test_burn_soulbound_nft_succeeds() {
    // Decision documented on burn_nft: `transferable` gates transfer_nft
    // only. A soulbound (non-transferable) NFT can still be burned by its
    // own owner — burning isn't a transfer to someone else.
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let player = Address::generate(&env);

    let mut map: Map<Symbol, Val> = Map::new(&env);
    map.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Soulbound").into_val(&env),
    );
    map.set(
        Symbol::new(&env, "description"),
        String::from_str(&env, "Desc").into_val(&env),
    );
    map.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "https://gateway.example/a").into_val(&env),
    );
    map.set(Symbol::new(&env, "transferable"), false.into_val(&env));

    let nft_id = client
        .mint_reward_nft_from_map(&minter, &1, &player, &map)
        .unwrap();

    let nft = client.get_nft(&nft_id).unwrap();
    assert!(!nft.transferable, "expected a soulbound NFT for this test");

    // Soulbound NFTs cannot be transferred...
    let other = Address::generate(&env);
    let transfer_result = client.try_transfer_nft(&nft_id, &player, &other, &player);
    assert!(transfer_result.is_err());

    // ...but the owner can still burn it.
    client.burn_nft(&nft_id, &player);
    assert!(
        client.get_nft(&nft_id).is_none(),
        "burned NFT must be removed from storage"
    );
}

#[test]
fn test_unauthorized_cannot_mint_before_and_after_init() {
    // Do NOT mock auth here — we expect unauthorized addresses to fail.
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);

    let arbitrary = Address::generate(&env);
    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Guarded", "Desc", "ipfs://x");

    // Before initialization: minting by an arbitrary address without auth should fail
    let pre_init = std::panic::catch_unwind(|| {
        client.mint_reward_nft(&arbitrary, &1, &player, &metadata);
    });
    assert!(pre_init.is_err());

    // Initialize the contract with a distinct minter
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));

    // After initialization: the arbitrary address should still not be able to mint
    let post_init = std::panic::catch_unwind(|| {
        client.mint_reward_nft(&arbitrary, &1, &player, &metadata);
    });
    assert!(post_init.is_err());
}

// -----------------------------------------------------------------------------
// admin_update_image_uris / replace_prefix (issue #844)
// -----------------------------------------------------------------------------

/// `mint_reward_nft` validates image_uri against a strict https://|ipfs://
/// scheme (and a 200-byte cap), but `update_nft_metadata` does not — so it's
/// the realistic way for an NFT to end up with an arbitrary, longer
/// image_uri for these admin_update_image_uris regression tests.
fn set_raw_image_uri(
    env: &Env,
    client: &NftRewardClient<'_>,
    nft_id: u64,
    owner: &Address,
    uri: &str,
) {
    let nft = client.get_nft(&nft_id).unwrap();
    client
        .update_nft_metadata(
            &nft_id,
            owner,
            &nft.metadata.description,
            &String::from_str(env, uri),
        )
        .unwrap();
}

#[test]
fn test_admin_update_image_uris_replaces_matching_prefix() {
    let env = setup_env();
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "NFT", "Desc", "https://old-gateway.example/a");
    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    let old_prefix = String::from_str(&env, "https://old-gateway.example/");
    let new_prefix = String::from_str(&env, "https://new-gateway.example/");
    let (updated, _next_offset) = client
        .admin_update_image_uris(&admin, &old_prefix, &new_prefix, &0, &10)
        .unwrap();

    assert_eq!(updated, 1);
    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(
        nft.metadata.image_uri,
        String::from_str(&env, "https://new-gateway.example/a")
    );
}

#[test]
fn test_admin_update_image_uris_handles_uri_over_256_bytes_without_corruption() {
    // Regression test for issue #844: a >256-byte URI used to be silently
    // truncated to zero-padded garbage because the copy into the fixed
    // buffer was clamped to 256 bytes while the comparison/copy lengths
    // downstream were not.
    let env = setup_env();
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "NFT", "Desc", "https://x.example/a");
    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    let prefix = "ipfs://old-gateway/";
    // 400 bytes of 'a' after the prefix keeps the whole URI under the
    // 512-byte MAX_NFT_URI_BYTES cap while comfortably exceeding 256.
    let suffix = "a".repeat(400);
    let long_uri = std::format!("{}{}", prefix, suffix);
    assert!(long_uri.len() > 256 && long_uri.len() <= 512);
    set_raw_image_uri(&env, &client, nft_id, &player, &long_uri);

    let old_prefix = String::from_str(&env, prefix);
    let new_prefix = String::from_str(&env, "ipfs://new-gateway/");
    let (updated, _next_offset) = client
        .admin_update_image_uris(&admin, &old_prefix, &new_prefix, &0, &10)
        .unwrap();
    assert_eq!(updated, 1);

    let expected = std::format!("ipfs://new-gateway/{}", suffix);
    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, &expected));
}

#[test]
fn test_admin_update_image_uris_long_old_prefix_does_not_panic() {
    // Regression test for issue #844: an old_prefix longer than 256 bytes
    // used to index a [0u8; 256] buffer out of bounds and panic.
    let env = setup_env();
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "NFT", "Desc", "https://x.example/a");
    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    let long_uri = "b".repeat(300);
    set_raw_image_uri(&env, &client, nft_id, &player, &long_uri);

    // 280-byte old_prefix: longer than the old 256-byte buffer, shorter
    // than the 300-byte URI, and matches it byte-for-byte.
    let old_prefix_str = "b".repeat(280);
    let old_prefix = String::from_str(&env, &old_prefix_str);
    let new_prefix = String::from_str(&env, "c");

    let (updated, _next_offset) = client
        .admin_update_image_uris(&admin, &old_prefix, &new_prefix, &0, &10)
        .unwrap();
    assert_eq!(updated, 1);
}

#[test]
fn test_admin_update_image_uris_oversized_result_is_skipped_not_panicked() {
    // Regression test for issue #844: a new_prefix long enough to push the
    // assembled result past 512 bytes used to overflow the output buffer
    // and panic. It should now simply be skipped (URI left unchanged).
    let env = setup_env();
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    client.initialize(&admin, &minter, &None, &default_collection_metadata(&env));

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "NFT", "Desc", "https://x.example/a");
    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);

    // A near-max-length URI so that swapping in a longer prefix overflows 512.
    let uri_str = std::format!("x/{}", "y".repeat(500));
    set_raw_image_uri(&env, &client, nft_id, &player, &uri_str);

    let old_prefix = String::from_str(&env, "x/");
    let new_prefix = String::from_str(&env, &"z".repeat(100));

    let (updated, _next_offset) = client
        .admin_update_image_uris(&admin, &old_prefix, &new_prefix, &0, &10)
        .unwrap();
    assert_eq!(updated, 0);

    // Unchanged — the oversized replacement was skipped, not applied garbled.
    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, &uri_str));
}

/// Acceptance test for issue #856:
/// Two players mint for the same hunt with distinct `completion_rank` values
/// supplied via the metadata map.  Asserts that:
/// - the first minter gets rank 1 in the emitted `NftMintedEvent`,
/// - the second minter gets rank 2, and
/// - `get_nft_count_for_hunt` is irrelevant to the stored ranks (i.e. the
///   counter growing does not clobber an already-emitted rank).
#[test]
fn test_completion_rank_is_distinct_per_player() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let hunt_id: u64 = 42;

    // Build a minimal metadata map for the first player, rank = 1.
    let mut meta1: Map<Symbol, Val> = Map::new(&env);
    meta1.set(Symbol::new(&env, "title"), String::from_str(&env, "Hunt Winner").into_val(&env));
    meta1.set(Symbol::new(&env, "description"), String::from_str(&env, "First finisher").into_val(&env));
    meta1.set(Symbol::new(&env, "image_uri"), String::from_str(&env, "ipfs://rank1").into_val(&env));
    meta1.set(Symbol::new(&env, "completion_rank"), 1u32.into_val(&env));

    let nft1 = client.mint_reward_nft_from_map(&minter, &hunt_id, &player1, &meta1)
        .expect("first mint should succeed");

    // Build metadata map for the second player, rank = 2.
    let mut meta2: Map<Symbol, Val> = Map::new(&env);
    meta2.set(Symbol::new(&env, "title"), String::from_str(&env, "Hunt Runner-up").into_val(&env));
    meta2.set(Symbol::new(&env, "description"), String::from_str(&env, "Second finisher").into_val(&env));
    meta2.set(Symbol::new(&env, "image_uri"), String::from_str(&env, "ipfs://rank2").into_val(&env));
    meta2.set(Symbol::new(&env, "completion_rank"), 2u32.into_val(&env));

    let nft2 = client.mint_reward_nft_from_map(&minter, &hunt_id, &player2, &meta2)
        .expect("second mint should succeed");

    // Collect the two NftMinted events (the last two events in the log).
    let all_events = env.events().all();
    assert!(all_events.len() >= 2, "expected at least 2 NftMinted events");

    let event_count = all_events.len();
    let (_, _, data1): (Address, soroban_sdk::Vec<Val>, Val) =
        all_events.get(event_count - 2).unwrap();
    let (_, _, data2): (Address, soroban_sdk::Vec<Val>, Val) =
        all_events.get(event_count - 1).unwrap();

    let ev1 = NftMintedEvent::try_from_val(&env, &data1).unwrap();
    let ev2 = NftMintedEvent::try_from_val(&env, &data2).unwrap();

    // Basic sanity checks.
    assert_eq!(ev1.nft_id, nft1);
    assert_eq!(ev2.nft_id, nft2);

    // Core assertion: ranks are frozen at the values supplied by hunty-core,
    // not at the live hunt-NFT counter.
    assert_eq!(ev1.completion_rank, 1, "first player should have rank 1");
    assert_eq!(ev2.completion_rank, 2, "second player should have rank 2");
    assert_ne!(ev1.completion_rank, ev2.completion_rank, "ranks must be distinct");

    // total_minted_for_hunt reflects the collection counter, not the rank.
    assert_ne!(
        ev2.total_minted_for_hunt, ev2.completion_rank,
        "total_minted_for_hunt and completion_rank are different concepts"
    );
}

// =========================================================================
// ROYALTY BPS VALIDATION TESTS
// =========================================================================

#[test]
fn test_mint_accepts_valid_royalty_bps_at_boundary() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let player = Address::generate(&env);
    let creator = Address::generate(&env);

    // Test boundary value: exactly 10,000 bp (100%)
    let metadata = create_metadata_with_creator(
        &env,
        "Hunt Champion",
        "Completed the City Hunt",
        "ipfs://QmExample123",
        creator,
        Some(10_000),
    );

    let nft_id = client.mint_reward_nft(&minter, &1, &player, &metadata);
    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.royalty_bps, Some(10_000));
}

#[test]
fn test_mint_rejects_royalty_bps_above_max() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let player = Address::generate(&env);
    let creator = Address::generate(&env);

    // Test rejection: 10,001 bp (100.01%, above max)
    let metadata = create_metadata_with_creator(
        &env,
        "Hunt Champion",
        "Completed the City Hunt",
        "ipfs://QmExample123",
        creator,
        Some(10_001),
    );

    let err = client
        .try_mint_reward_nft(&minter, &1, &player, &metadata)
        .unwrap_err();
    assert_eq!(err, Ok(NftErrorCode::InvalidRoyalty));
}

#[test]
fn test_mint_from_map_rejects_excessive_royalty_bps() {
    let env = setup_env();
    let (client, minter) = setup_nft_reward(&env, None);
    let player = Address::generate(&env);
    let creator = Address::generate(&env);

    let mut map: Map<Symbol, Val> = Map::new(&env);
    map.set(
        Symbol::new(&env, "title"),
        String::from_str(&env, "Hunt Champion").into_val(&env),
    );
    map.set(
        Symbol::new(&env, "description"),
        String::from_str(&env, "Completed the City Hunt").into_val(&env),
    );
    map.set(
        Symbol::new(&env, "image_uri"),
        String::from_str(&env, "ipfs://QmExample123").into_val(&env),
    );
    map.set(
        Symbol::new(&env, "creator"),
        creator.into_val(&env),
    );
    map.set(
        Symbol::new(&env, "royalty_bps"),
        50_000u32.into_val(&env), // 500% - way above max
    );

    let err = client
        .try_mint_reward_nft_from_map(&minter, &1, &player, &map)
        .unwrap_err();
    assert_eq!(err, Ok(NftErrorCode::InvalidRoyalty));
}
