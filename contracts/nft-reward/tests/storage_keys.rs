use nft_reward::{
    CollectionMetadata, NftMetadata, NftReward, NftRewardClient, METADATA_SCHEMA_VERSION,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env, IntoVal, Map, String, Symbol, Val,
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

fn setup_client(env: &Env) -> (NftRewardClient<'_>, Address, Address) {
    let contract_id = env.register(NftReward, ());
    let client = NftRewardClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let minter = Address::generate(env);
    client.initialize(&admin, &minter, &None, &default_collection_metadata(env));
    (client, minter, contract_id)
}

fn sample_metadata(env: &Env, title: &str) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, "desc"),
        image_uri: String::from_str(env, "ipfs://test"),
        hunt_title: String::from_str(env, title),
        rarity: 0,
        tier: 0,
        creator: None,
        royalty_bps: None,
        extensions: Map::new(env),
    }
}

fn mint_transferable(
    env: &Env,
    client: &NftRewardClient<'_>,
    minter: &Address,
    hunt_id: u64,
    owner: &Address,
    title: &str,
) -> u64 {
    let metadata = sample_metadata(env, title);
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
    client.mint_reward_nft_from_map(minter, &hunt_id, owner, &map).unwrap()
}

#[test]
fn test_nft_version_key_distinct_from_contract_version_key() {
    let env = setup_env();
    let (client, minter, _contract_id) = setup_client(&env);
    let owner = Address::generate(&env);

    let nft_id = mint_transferable(&env, &client, &minter, 1, &owner, "Version Test");
    let meta = client.get_nft_metadata(&nft_id).unwrap();

    assert_eq!(meta.schema_version, METADATA_SCHEMA_VERSION);
    assert_ne!(meta.schema_version, 0);
}

#[test]
fn test_hunt_nft_count_stays_in_sync_with_mints() {
    let env = setup_env();
    let (client, minter, _contract_id) = setup_client(&env);
    let player = Address::generate(&env);

    assert_eq!(client.get_hunt_nft_count(&1), 0);

    mint_transferable(&env, &client, &minter, 1, &player, "Hunt 1 A");
    mint_transferable(&env, &client, &minter, 1, &player, "Hunt 1 B");
    mint_transferable(&env, &client, &minter, 2, &player, "Hunt 2 A");

    assert_eq!(client.get_hunt_nft_count(&1), 2);
    assert_eq!(client.get_hunt_nft_count(&2), 1);

    let hunt_one_nfts = client.get_nfts_by_hunt(&1, &0, &10);
    assert_eq!(hunt_one_nfts.len(), 2);
}

#[test]
fn test_hunt_nft_count_decrements_after_burn() {
    let env = setup_env();
    let (client, minter, _contract_id) = setup_client(&env);
    let owner = Address::generate(&env);

    let nft_id = mint_transferable(&env, &client, &minter, 7, &owner, "Burn Me");
    assert_eq!(client.get_hunt_nft_count(&7), 1);

    client.burn_nft(&nft_id, &owner);

    assert_eq!(client.get_hunt_nft_count(&7), 0);
}

#[test]
fn test_operator_approve_revoke_is_approved() {
    let env = setup_env();
    let (client, _minter, _contract_id) = setup_client(&env);
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let other = Address::generate(&env);

    assert!(!client.is_operator(&owner, &operator));

    client.set_operator(&owner, &operator);
    assert!(client.is_operator(&owner, &operator));
    assert!(!client.is_operator(&owner, &other));

    client.remove_operator(&owner, &operator);
    assert!(!client.is_operator(&owner, &operator));
}

#[test]
fn test_contract_operator_approve_revoke_and_transfer() {
    let env = setup_env();
    let (client, minter, _contract_id) = setup_client(&env);
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let recipient = Address::generate(&env);

    let nft_id = mint_transferable(&env, &client, &minter, 1, &owner, "Operator NFT");

    assert!(!client.is_operator(&owner, &operator));
    client.set_operator(&owner, &operator);
    assert!(client.is_operator(&owner, &operator));

    client.transfer_nft(&nft_id, &owner, &recipient, &operator);
    assert_eq!(client.owner_of(&nft_id), Some(recipient.clone()));

    client.remove_operator(&owner, &operator);
    assert!(!client.is_operator(&owner, &operator));
}

#[test]
fn test_owner_hunt_index_lifecycle_mint_transfer_burn() {
    let env = setup_env();
    let (client, minter, _contract_id) = setup_client(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    assert!(!client.has_hunt_nft(&alice, &42));
    assert!(!client.has_hunt_nft(&bob, &42));

    // Mint 2 NFTs for hunt 42 to Alice
    let nft1 = mint_transferable(&env, &client, &minter, 42, &alice, "Hunt 42 A");
    let nft2 = mint_transferable(&env, &client, &minter, 42, &alice, "Hunt 42 B");

    assert!(client.has_hunt_nft(&alice, &42));
    assert!(!client.has_hunt_nft(&bob, &42));

    // Transfer 1 NFT to Bob
    client.transfer_nft(&nft1, &alice, &bob, &alice);
    assert!(client.has_hunt_nft(&alice, &42));
    assert!(client.has_hunt_nft(&bob, &42));

    // Burn Alice's remaining NFT for hunt 42
    client.burn_nft(&nft2, &alice);
    assert!(!client.has_hunt_nft(&alice, &42));
    assert!(client.has_hunt_nft(&bob, &42));

    // Burn Bob's NFT
    client.burn_nft(&nft1, &bob);
    assert!(!client.has_hunt_nft(&bob, &42));
}

#[test]
fn test_get_player_nfts_bounded_by_max_scan_limit() {
    let env = setup_env();
    let (client, minter, _contract_id) = setup_client(&env);
    let owner = Address::generate(&env);

    for i in 1..=10 {
        mint_transferable(&env, &client, &minter, i, &owner, "NFT");
    }

    let nfts = client.get_player_nfts(&owner, &0, &u32::MAX);
    assert_eq!(nfts.len(), 10);

    let paged = client.get_player_nfts(&owner, &2, &3);
    assert_eq!(paged.len(), 3);
}
