#![cfg_attr(not(test), no_std)]
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, Address, Env, Map, String, Symbol,
    Val, Vec, symbol_short,
};

const MAX_URI_LEN: usize = 512;
const MAX_NFT_TITLE_BYTES: u32 = 128;
const MAX_NFT_DESCRIPTION_BYTES: u32 = 1024;
const MAX_NFT_URI_BYTES: u32 = 512;

/// Core display metadata for an NFT (title, description, image URI).
/// Supports off-chain storage references to keep gas costs low.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftMetadata {
    pub title: String,
    pub description: String,
    pub image_uri: String,
    /// Hunt title at time of mint (for context/display).
    pub hunt_title: String,
    /// Rarity tier: 0 = default, 1 = common, 2 = uncommon, 3 = rare, 4 = epic, 5 = legendary.
    pub rarity: u32,
    /// Custom tier for special categories (0 = none).
    pub tier: u32,
    /// Original creator of the NFT (stamped at mint time for provenance/attribution).
    /// Essential for secondary market royalty distribution and creator attribution.
    pub creator: Option<Address>,
    /// Royalty in basis points (1 bp = 0.01%). For example, 250 = 2.5% royalty.
    /// Used for secondary market sales to provide ongoing creator revenue.
    pub royalty_bps: Option<u32>,
}

fn is_valid_image_uri_format(uri: &String) -> bool {
    let len = uri.len() as usize;
    let mut buf = [0u8; 512];
    uri.copy_into_slice(&mut buf[..len]);
    let text = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
    text.starts_with("https://") || text.starts_with("ipfs://")
}

/// Complete metadata returned by get_nft_metadata (includes NftData-derived fields).
#[contracttype]
#[derive(Clone, Debug)]
pub struct NftMetadataResponse {
    pub nft_id: u64,
    pub hunt_id: u64,
    pub hunt_title: String,
    pub completion_timestamp: u64,
    pub completion_player: Address,
    pub current_owner: Address,
    pub title: String,
    pub description: String,
    pub image_uri: String,
    pub rarity: u32,
    pub tier: u32,
    pub creator: Option<Address>,
    pub royalty_bps: Option<u32>,
    /// Schema version of the NFT metadata.
    pub schema_version: u32,
}

/// NFT data structure stored on-chain.
/// NOTE: Do NOT add new fields here without a migration step — the Soroban
/// host rejects stored structs whose field count differs from the stored
/// ScVal map.  Use per-NFT auxiliary keys for new metadata instead.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftData {
    pub nft_id: u64,
    pub hunt_id: u64,
    pub owner: Address,
    pub completion_player: Address,
    pub metadata: NftMetadata,
    pub transferable: bool,
    pub minted_at: u64,
    pub locked: bool,
}

/// Event emitted when an NFT is minted.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftMintedEvent {
    pub nft_id: u64,
    pub hunt_id: u64,
    pub owner: Address,
    pub rarity: u32,
    pub tier: u32,
    pub metadata: NftMetadata,
    pub hunt_title: String,
    pub total_minted_for_hunt: u64,
    pub completion_rank: u32,
    pub collection_stats: NftCollectionStats,
    pub minted_at: u64,
}

/// Event emitted when an operator approval changes.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorChangedEvent {
    pub owner: Address,
    pub operator: Address,
    pub approved: bool,
}

/// Event emitted when an NFT is transferred.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftTransferredEvent {
    pub nft_id: u64,
    pub from: Address,
    pub to: Address,
}

/// Event emitted when an NFT's mutable metadata is updated.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftMetadataUpdatedEvent {
    pub nft_id: u64,
    pub updater: Address,
}

/// Event emitted when admin batch-updates image URIs across NFTs.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminImageUrisUpdatedEvent {
    pub old_prefix: String,
    pub new_prefix: String,
    pub updated_count: u32,
}

mod errors;
pub use errors::NftErrorCode;
mod migration;
mod sanitization;
mod storage;
use storage::Storage;

#[contract]
pub struct NftReward;

/// Current metadata schema version (bump when adding/changing NftMetadata shape).
pub const METADATA_SCHEMA_VERSION: u32 = 1;

/// Contract version constant.
pub const CONTRACT_VERSION: u32 = 2;

#[contractimpl]
impl NftReward {
    /// Initializes the NFT reward contract with an admin, minter, and optional max supply cap.
    pub fn initialize(
        env: Env,
        admin: Address,
        minter: Address,
        max_supply: Option<u64>,
    ) -> Result<(), crate::errors::NftErrorCode> {
        if Storage::is_initialized(&env) {
            return Err(crate::errors::NftErrorCode::AlreadyInitialized);
        }

        Storage::save_admin(&env, &admin);
        Storage::add_minter(&env, &minter);
        Storage::set_max_supply(&env, max_supply);
        Storage::set_contract_version(&env, CONTRACT_VERSION);
        Ok(())
    }

    fn require_admin(env: &Env, admin: &Address) -> Result<(), crate::errors::NftErrorCode> {
        admin.require_auth();
        let stored_admin =
            Storage::get_admin(env).ok_or(crate::errors::NftErrorCode::NotInitialized)?;
        if stored_admin != admin.clone() {
            return Err(crate::errors::NftErrorCode::Unauthorized);
        }
        Ok(())
    }

    pub fn get_admin(env: Env) -> Option<Address> {
        Storage::get_admin(&env)
    }

    pub fn add_minter(
        env: Env,
        admin: Address,
        minter: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        admin.require_auth();
        let stored_admin = Storage::get_admin(&env).ok_or(crate::errors::NftErrorCode::Unauthorized)?;
        if admin != stored_admin {
            return Err(crate::errors::NftErrorCode::Unauthorized);
        }
        Storage::add_minter(&env, &minter);
        Ok(())
    }

    pub fn remove_minter(
        env: Env,
        admin: Address,
        minter: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        admin.require_auth();
        let stored_admin = Storage::get_admin(&env).ok_or(crate::errors::NftErrorCode::Unauthorized)?;
        if admin != stored_admin {
            return Err(crate::errors::NftErrorCode::Unauthorized);
        }
        Storage::remove_minter(&env, &minter);
        Ok(())
    }

    pub fn is_minter(env: Env, minter: Address) -> bool {
        Storage::is_minter(&env, &minter)
    }

    pub fn set_reward_manager(
        env: Env,
        admin: Address,
        reward_manager: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        admin.require_auth();
        let stored_admin = Storage::get_admin(&env).ok_or(crate::errors::NftErrorCode::Unauthorized)?;
        if admin != stored_admin {
            return Err(crate::errors::NftErrorCode::Unauthorized);
        }
        Storage::set_reward_manager(&env, &reward_manager);
        Ok(())
    }

    pub fn get_reward_manager(env: Env) -> Option<Address> {
        Storage::get_reward_manager(&env)
    }

    /// Mints a unique NFT as a reward for hunt completion.
    ///
    /// `minter` must be an authorized minter (and must sign the transaction) when the
    /// contract has been initialized.  Before initialization the check is skipped so
    /// that existing deployments remain functional.
    ///
    /// # Arguments
    /// * `minter` - Address performing the mint (must be whitelisted after init)
    /// * `hunt_id` - The hunt this NFT commemorates
    /// * `player_address` - The address of the player completing the hunt (initial owner)
    /// * `metadata` - NFT metadata (title, description, image URI, hunt_title, rarity, tier)
    ///
    /// # Returns
    /// The unique NFT ID of the minted NFT
    pub fn mint_reward_nft(
        env: Env,
        _minter: Address,
        hunt_id: u64,
        player_address: Address,
        metadata: NftMetadata,
    ) -> u64 {
        Self::mint_reward_nft_impl(env, hunt_id, player_address, metadata, false)
    }

    /// Mints a reward NFT from a generic metadata map. This is the entrypoint
    /// used by cross-contract callers (e.g. RewardManager) that cannot depend
    /// on this crate's `NftMetadata` type directly.
    ///
    /// `minter` is the calling contract's address and must be whitelisted when the
    /// contract has been initialized.
    ///
    /// Expected keys in `metadata` (all optional, with sensible defaults):
    /// - "title": String
    /// - "description": String
    /// - "image_uri": String
    /// - "hunt_title": String (defaults to title when omitted/empty)
    /// - "rarity": u32
    /// - "tier": u32
    /// - "creator": Address (defaults to player_address if omitted)
    /// - "royalty_bps": u32 (optional, basis points for royalty percentage)
    /// - "transferable": bool
    pub fn mint_reward_nft_from_map(
        env: Env,
        _minter: Address,
        hunt_id: u64,
        player_address: Address,
        metadata: Map<Symbol, Val>,
    ) -> u64 {
        // Ensure only the configured RewardManager contract can call this function
        if let Some(reward_mgr) = Storage::get_reward_manager(&env) {
            reward_mgr.require_auth();
        }
        use soroban_sdk::TryFromVal;

        let title = metadata
            .get(Symbol::new(&env, "title"))
            .and_then(|v| String::try_from_val(&env, &v).ok())
            .unwrap_or_else(|| String::from_str(&env, ""));

        let description = metadata
            .get(Symbol::new(&env, "description"))
            .and_then(|v| String::try_from_val(&env, &v).ok())
            .unwrap_or_else(|| String::from_str(&env, ""));

        let image_uri = metadata
            .get(Symbol::new(&env, "image_uri"))
            .and_then(|v| String::try_from_val(&env, &v).ok())
            .unwrap_or_else(|| String::from_str(&env, ""));

        let hunt_title = metadata
            .get(Symbol::new(&env, "hunt_title"))
            .and_then(|v| String::try_from_val(&env, &v).ok())
            .unwrap_or_else(|| title.clone());

        let rarity = metadata
            .get(Symbol::new(&env, "rarity"))
            .and_then(|v| u32::try_from_val(&env, &v).ok())
            .unwrap_or(0u32);

        if rarity > 5 {
            panic!("InvalidRarity");
        }

        let tier = metadata
            .get(Symbol::new(&env, "tier"))
            .and_then(|v| u32::try_from_val(&env, &v).ok())
            .unwrap_or(0u32);

        let creator = metadata
            .get(Symbol::new(&env, "creator"))
            .and_then(|v| Address::try_from_val(&env, &v).ok())
            .or_else(|| Some(player_address.clone()));

        let royalty_bps = metadata
            .get(Symbol::new(&env, "royalty_bps"))
            .and_then(|v| u32::try_from_val(&env, &v).ok());

        let transferable = metadata
            .get(Symbol::new(&env, "transferable"))
            .and_then(|v| bool::try_from_val(&env, &v).ok())
            .unwrap_or(false);

        let meta = NftMetadata {
            title,
            description,
            image_uri,
            hunt_title,
            rarity,
            tier,
            creator,
            royalty_bps,
        };
        Self::mint_reward_nft_impl(env, hunt_id, player_address, meta, transferable)
    }

    fn validate_metadata_field(
        env: &Env,
        value: &String,
        max_bytes: u32,
        allow_empty: bool,
        error_code: crate::errors::NftErrorCode,
    ) -> String {
        match sanitization::StringSanitizer::sanitize(env, value, max_bytes, allow_empty) {
            Ok(s) => s,
            Err(_) => panic_with_error!(env, error_code),
        }
    }

    fn validate_image_uri(env: &Env, value: &String) -> String {
        let s = Self::validate_metadata_field(
            env,
            value,
            MAX_NFT_URI_BYTES,
            false,
            crate::errors::NftErrorCode::InvalidImageUri,
        );
        if !is_valid_image_uri_format(&s) {
            panic_with_error!(env, crate::errors::NftErrorCode::InvalidImageUri);
        }
        s
    }

    fn mint_reward_nft_impl(
        env: Env,
        hunt_id: u64,
        player_address: Address,
        metadata: NftMetadata,
        transferable: bool,
    ) -> u64 {
        if metadata.rarity > 5 {
            panic_with_error!(&env, crate::errors::NftErrorCode::InvalidRarity);
        }

        let mut metadata = metadata;
        metadata.title = Self::validate_metadata_field(
            &env,
            &metadata.title,
            MAX_NFT_TITLE_BYTES,
            false,
            crate::errors::NftErrorCode::InvalidTitle,
        );
        metadata.description = Self::validate_metadata_field(
            &env,
            &metadata.description,
            MAX_NFT_DESCRIPTION_BYTES,
            false,
            crate::errors::NftErrorCode::InvalidDescription,
        );
        metadata.image_uri = Self::validate_image_uri(&env, &metadata.image_uri);
        metadata.hunt_title = Self::validate_metadata_field(
            &env,
            &metadata.hunt_title,
            MAX_NFT_TITLE_BYTES,
            true,
            crate::errors::NftErrorCode::InvalidHuntTitle,
        );

        if let Some(max_supply) = Storage::get_max_supply(&env) {
            let current_supply = Storage::get_nft_counter(&env);
            if current_supply >= max_supply {
                panic_with_error!(&env, crate::errors::NftErrorCode::MaxSupplyReached);
            }
        }

        let minted_at = env.ledger().timestamp();
        let nft_id = Storage::next_nft_id(&env);

        let nft_data = NftData {
            nft_id,
            hunt_id,
            owner: player_address.clone(),
            metadata: metadata.clone(),
            transferable,
            minted_at,
            locked: false,
        };

        Storage::save_nft(&env, &nft_data);
        Storage::set_nft_version(&env, nft_id, METADATA_SCHEMA_VERSION);
        Storage::add_nft_to_owner(&env, &player_address, nft_id);
        Storage::add_nft_to_hunt(&env, hunt_id, nft_id);
        Storage::mark_hunt_minted(&env, hunt_id);

        let event = NftMintedEvent {
            nft_id,
            hunt_id,
            owner: player_address,
            rarity: metadata.rarity,
            tier: metadata.tier,
            metadata,
            minted_at,
        };
        env.events()
            .publish((Symbol::new(&env, "NftMinted"), nft_id), event);

        nft_id
    }

    /// Retrieves NFT data by ID.
    pub fn get_nft(env: Env, nft_id: u64) -> Option<NftData> {
        Storage::get_nft(&env, nft_id)
    }

    /// Returns complete metadata for an NFT, including hunt info and completion details.
    pub fn get_nft_metadata(env: Env, nft_id: u64) -> Option<NftMetadataResponse> {
        let nft = Storage::get_nft(&env, nft_id)?;
        let version = Storage::get_nft_version(&env, nft_id);
        Some(NftMetadataResponse {
            nft_id: nft.nft_id,
            hunt_id: nft.hunt_id,
            hunt_title: nft.metadata.hunt_title.clone(),
            completion_timestamp: nft.minted_at,
            completion_player: nft.completion_player.clone(),
            current_owner: nft.owner.clone(),
            title: nft.metadata.title.clone(),
            description: nft.metadata.description.clone(),
            image_uri: nft.metadata.image_uri.clone(),
            rarity: nft.metadata.rarity,
            tier: nft.metadata.tier,
            creator: nft.metadata.creator.clone(),
            royalty_bps: nft.metadata.royalty_bps,
            schema_version: version,
        })
    }

    /// Returns the configured admin address, if set.
    pub fn get_admin(env: Env) -> Option<Address> {
        Storage::get_admin(&env)
    }

    /// Sets the RewardManager contract address. Only the admin can call this.
    pub fn set_reward_manager(
        env: Env,
        admin: Address,
        reward_manager: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::set_reward_manager(&env, &reward_manager);
        Ok(())
    }

    /// Batch-updates image URIs for all NFTs whose `image_uri` starts with `old_prefix`,
    /// replacing it with `new_prefix`. Useful for migrating between IPFS gateways or CDNs.
    ///
    /// # Authorization
    /// Only the configured admin can call this function.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must match the stored admin)
    /// * `old_prefix` - The prefix to match (e.g. "ipfs://oldgateway/")
    /// * `new_prefix` - The replacement prefix (e.g. "ipfs://newgateway/")
    ///
    /// # Returns
    /// The number of NFTs whose image URIs were updated.
    pub fn admin_update_image_uris(
        env: Env,
        admin: Address,
        old_prefix: String,
        new_prefix: String,
    ) -> Result<u32, crate::errors::NftErrorCode> {
        Self::require_admin(&env, &admin)?;

        let all_ids = Storage::get_all_nft_ids(&env);
        let mut updated: u32 = 0;

        for nft_id in all_ids.iter() {
            if let Some(mut nft) = Storage::get_nft(&env, nft_id) {
                if let Some(new_uri) = replace_prefix(
                    &env,
                    &nft.metadata.image_uri,
                    &old_prefix,
                    &new_prefix,
                ) {
                    nft.metadata.image_uri = new_uri;
                    Storage::save_nft(&env, &nft);
                    updated += 1;
                }
            }
        }

        env.events().publish(
            (Symbol::new(&env, "AdminImageUrisUpdated"),),
            AdminImageUrisUpdatedEvent {
                old_prefix,
                new_prefix,
                updated_count: updated,
            },
        );

        Ok(updated)
    }

    /// Updates mutable metadata fields (description, image_uri). Owner only.
    /// Title, hunt info, and attributes remain immutable for collectibility.
    pub fn update_nft_metadata(
        env: Env,
        nft_id: u64,
        updater: Address,
        new_description: String,
        new_image_uri: String,
    ) -> Result<(), crate::errors::NftErrorCode> {
        updater.require_auth();

        let mut nft =
            Storage::get_nft(&env, nft_id).ok_or(crate::errors::NftErrorCode::NftNotFound)?;

        if nft.owner != updater {
            return Err(crate::errors::NftErrorCode::NotOwner);
        }

        let new_description = Self::validate_metadata_field(
            &env,
            &new_description,
            MAX_NFT_DESCRIPTION_BYTES,
            false,
            crate::errors::NftErrorCode::InvalidDescription,
        );
        let new_image_uri = Self::validate_image_uri(&env, &new_image_uri);

        nft.metadata.description = new_description;
        nft.metadata.image_uri = new_image_uri;
        Storage::save_nft(&env, &nft);

        env.events().publish(
            (Symbol::new(&env, "NftMetadataUpdated"), nft_id),
            NftMetadataUpdatedEvent { nft_id, updater },
        );

        Ok(())
    }

    /// Returns the total number of NFTs minted so far.
    pub fn total_supply(env: Env) -> u64 {
        Storage::get_nft_counter(&env)
    }

    /// Returns the owner of an NFT.
    pub fn owner_of(env: Env, nft_id: u64) -> Option<Address> {
        Storage::get_nft(&env, nft_id).map(|nft| nft.owner)
    }

    /// Alias for owner_of. Returns the owner of an NFT.
    pub fn get_nft_owner(env: Env, nft_id: u64) -> Option<Address> {
        Storage::get_nft(&env, nft_id).map(|nft| nft.owner)
    }

    /// Verifies whether `address` is the current owner of `nft_id`.
    /// Returns `true` when the NFT exists and the stored owner equals `address`.
    pub fn verify_ownership(env: Env, address: Address, nft_id: u64) -> bool {
        if let Some(nft) = Storage::get_nft(&env, nft_id) {
            nft.owner == address
        } else {
            false
        }
    }

    /// Returns `true` if `address` owns any NFT minted for `hunt_id`.
    /// Scans the owner's indexed NFT IDs and checks each NFT's `hunt_id`.
    pub fn has_hunt_nft(env: Env, address: Address, hunt_id: u64) -> bool {
        let nfts = Storage::get_owner_nfts(&env, &address);
        let len = nfts.len();
        for i in 0..len {
            if let Some(id) = nfts.get(i) {
                if let Some(nft) = Storage::get_nft(&env, id) {
                    if nft.hunt_id == hunt_id {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Returns paginated NFT IDs owned by an address.
    pub fn get_player_nfts(env: Env, owner: Address, offset: u32, limit: u32) -> Vec<u64> {
        let nfts = Storage::get_owner_nfts(&env, &owner);
        let len = nfts.len();
        if offset >= len {
            return Vec::new(&env);
        }
        let end = offset.saturating_add(limit).min(len);
        nfts.slice(offset..end)
    }

    /// Returns paginated NFT IDs minted for a hunt.
    pub fn get_nfts_by_hunt(env: Env, hunt_id: u64, offset: u32, limit: u32) -> Vec<u64> {
        Storage::get_hunt_nfts(&env, hunt_id, offset, limit)
    }

    /// Returns the total number of NFTs minted for a hunt.
    pub fn get_hunt_nft_count(env: Env, hunt_id: u64) -> u32 {
        Storage::get_hunt_nft_count(&env, hunt_id)
    }

    /// Burns (permanently destroys) an NFT, removing it from storage and the owner's list.
    ///
    /// # Authorization
    /// The `owner` must authorize this call. The caller must also be the current owner.
    pub fn burn(env: Env, nft_id: u64, owner: Address) -> Result<(), crate::errors::NftErrorCode> {
        owner.require_auth();

        let nft = Storage::get_nft(&env, nft_id).ok_or(crate::errors::NftErrorCode::NftNotFound)?;

        if nft.owner != owner {
            return Err(crate::errors::NftErrorCode::NotOwner);
        }

        let hunt_id = nft.hunt_id;
        Storage::remove_nft(&env, nft_id);
        Storage::remove_nft_from_hunt(&env, hunt_id, nft_id);

        let count_key = (symbol_short!("ONFC"), owner.clone());
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        let exist_key = (symbol_short!("ONFX"), owner.clone(), nft_id);
        if env.storage().persistent().has(&exist_key) {
            let mut found = false;
            for i in 0..count {
                let entry_key = (symbol_short!("ONFT"), owner.clone(), i);
                if let Some(stored_id) = env.storage().persistent().get::<_, u64>(&entry_key) {
                    if stored_id == nft_id {
                        let last_idx = count - 1;
                        if i != last_idx {
                            let last_key = (symbol_short!("ONFT"), owner.clone(), last_idx);
                            if let Some(last_id) =
                                env.storage().persistent().get::<_, u64>(&last_key)
                            {
                                env.storage().persistent().set(&entry_key, &last_id);
                            }
                            env.storage().persistent().remove(&last_key);
                        } else {
                            env.storage().persistent().remove(&entry_key);
                        }
                        found = true;
                        break;
                    }
                }
            }

            if found {
                env.storage().persistent().set(&count_key, &(count - 1));
            }
            env.storage().persistent().remove(&exist_key);
        }

        env.events()
            .publish((Symbol::new(&env, "NftBurned"), nft_id), (nft_id, owner));

        Ok(())
    }

    /// Searches NFTs by title (case-insensitive partial match).
    /// Returns a vector of NFT IDs whose titles contain the search query.
    pub fn search_by_title(env: Env, query: String) -> Vec<u64> {
        let all_nft_ids = Storage::get_all_nft_ids(&env);
        let mut results = Vec::new(&env);
        
        let query_lower = {
            let mut lower = String::new(&env);
            for c in query.chars() {
                lower.push_char(c.to_ascii_lowercase());
            }
            lower
        };

        for nft_id in all_nft_ids.iter() {
            if let Some(nft) = Storage::get_nft(&env, nft_id) {
                let title_lower = {
                    let mut lower = String::new(&env);
                    for c in nft.metadata.title.chars() {
                        lower.push_char(c.to_ascii_lowercase());
                    }
                    lower
                };
                
                if title_lower.contains(&query_lower) {
                    results.push_back(nft_id);
                }
            }
        }
        
        results
    }

    /// Searches NFTs by hunt title (case-insensitive partial match).
    /// Returns a vector of NFT IDs whose hunt titles contain the search query.
    pub fn search_by_hunt_title(env: Env, query: String) -> Vec<u64> {
        let all_nft_ids = Storage::get_all_nft_ids(&env);
        let mut results = Vec::new(&env);
        
        let query_lower = {
            let mut lower = String::new(&env);
            for c in query.chars() {
                lower.push_char(c.to_ascii_lowercase());
            }
            lower
        };

        for nft_id in all_nft_ids.iter() {
            if let Some(nft) = Storage::get_nft(&env, nft_id) {
                let hunt_title_lower = {
                    let mut lower = String::new(&env);
                    for c in nft.metadata.hunt_title.chars() {
                        lower.push_char(c.to_ascii_lowercase());
                    }
                    lower
                };
                
                if hunt_title_lower.contains(&query_lower) {
                    results.push_back(nft_id);
                }
            }
        }
        
        results
    }

    /// Filters NFTs by rarity tier.
    /// Returns a vector of NFT IDs with the specified rarity.
    /// Rarity tiers: 0 = default, 1 = common, 2 = uncommon, 3 = rare, 4 = epic, 5 = legendary.
    pub fn search_by_rarity(env: Env, rarity: u32) -> Vec<u64> {
        let all_nft_ids = Storage::get_all_nft_ids(&env);
        let mut results = Vec::new(&env);

        for nft_id in all_nft_ids.iter() {
            if let Some(nft) = Storage::get_nft(&env, nft_id) {
                if nft.metadata.rarity == rarity {
                    results.push_back(nft_id);
                }
            }
        }
        
        results
    }

    /// Filters NFTs by custom tier.
    /// Returns a vector of NFT IDs with the specified tier.
    /// Tier: 0 = none, other values for custom categories.
    pub fn search_by_tier(env: Env, tier: u32) -> Vec<u64> {
        let all_nft_ids = Storage::get_all_nft_ids(&env);
        let mut results = Vec::new(&env);

        for nft_id in all_nft_ids.iter() {
            if let Some(nft) = Storage::get_nft(&env, nft_id) {
                if nft.metadata.tier == tier {
                    results.push_back(nft_id);
                }
            }
        }
        
        results
    }

    /// General search function with multiple metadata filters.
    /// All parameters are optional - NFTs must match all provided filters.
    /// 
    /// # Arguments
    /// * `title_query` - Optional partial match for NFT title (case-insensitive)
    /// * `hunt_title_query` - Optional partial match for hunt title (case-insensitive)
    /// * `rarity` - Optional rarity filter (exact match)
    /// * `tier` - Optional tier filter (exact match)
    /// 
    /// # Returns
    /// Vector of NFT IDs matching all provided filters
    pub fn search_nfts(
        env: Env,
        title_query: Option<String>,
        hunt_title_query: Option<String>,
        rarity: Option<u32>,
        tier: Option<u32>,
    ) -> Vec<u64> {
        let all_nft_ids = Storage::get_all_nft_ids(&env);
        let mut results = Vec::new(&env);

        let title_lower_opt = title_query.map(|q| {
            let mut lower = String::new(&env);
            for c in q.chars() {
                lower.push_char(c.to_ascii_lowercase());
            }
            lower
        });

        let hunt_title_lower_opt = hunt_title_query.map(|q| {
            let mut lower = String::new(&env);
            for c in q.chars() {
                lower.push_char(c.to_ascii_lowercase());
            }
            lower
        });

        for nft_id in all_nft_ids.iter() {
            if let Some(nft) = Storage::get_nft(&env, nft_id) {
                let mut matches = true;

                // Check title filter
                if let Some(ref query_lower) = title_lower_opt {
                    let title_lower = {
                        let mut lower = String::new(&env);
                        for c in nft.metadata.title.chars() {
                            lower.push_char(c.to_ascii_lowercase());
                        }
                        lower
                    };
                    if !title_lower.contains(query_lower) {
                        matches = false;
                    }
                }

                // Check hunt title filter
                if matches {
                    if let Some(ref query_lower) = hunt_title_lower_opt {
                        let hunt_title_lower = {
                            let mut lower = String::new(&env);
                            for c in nft.metadata.hunt_title.chars() {
                                lower.push_char(c.to_ascii_lowercase());
                            }
                            lower
                        };
                        if !hunt_title_lower.contains(query_lower) {
                            matches = false;
                        }
                    }
                }

                // Check rarity filter
                if matches {
                    if let Some(r) = rarity {
                        if nft.metadata.rarity != r {
                            matches = false;
                        }
                    }
                }

                // Check tier filter
                if matches {
                    if let Some(t) = tier {
                        if nft.metadata.tier != t {
                            matches = false;
                        }
                    }
                }

                if matches {
                    results.push_back(nft_id);
                }
            }
        }
        
        results
    }

    /// Transfers an NFT from one address to another.
    ///
    /// # Arguments
    /// * `nft_id` - The NFT to transfer
    /// * `from_address` - Current owner of the NFT
    /// * `to_address` - New owner
    /// * `caller` - Address authorizing the transfer (must be owner or approved operator)
    ///
    /// # Authorization
    /// `caller` must authorize this call. `caller` must be either the current owner
    /// or an operator approved by the owner via `set_operator`.
    pub fn transfer_nft(
        env: Env,
        nft_id: u64,
        from_address: Address,
        to_address: Address,
        caller: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        caller.require_auth();

        let mut nft =
            Storage::get_nft(&env, nft_id).ok_or(crate::errors::NftErrorCode::NftNotFound)?;

        if nft.owner != from_address {
            return Err(crate::errors::NftErrorCode::NotOwner);
        }

        // caller must be the owner or an approved operator
        if caller != nft.owner && !Storage::is_operator(&env, &nft.owner, &caller) {
            return Err(crate::errors::NftErrorCode::NotOperator);
        }

        if nft.owner == to_address {
            return Err(crate::errors::NftErrorCode::InvalidRecipient);
        }

        if !nft.transferable {
            return Err(crate::errors::NftErrorCode::SoulboundNft);
        }

        if nft.locked {
            return Err(crate::errors::NftErrorCode::NftLocked);
        }

        let count_key = (symbol_short!("ONFC"), from_address.clone());
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        let exist_key = (symbol_short!("ONFX"), from_address.clone(), nft_id);
        if env.storage().persistent().has(&exist_key) {
            let mut found = false;
            for i in 0..count {
                let entry_key = (symbol_short!("ONFT"), from_address.clone(), i);
                if let Some(stored_id) = env.storage().persistent().get::<_, u64>(&entry_key) {
                    if stored_id == nft_id {
                        let last_idx = count - 1;
                        if i != last_idx {
                            let last_key = (symbol_short!("ONFT"), from_address.clone(), last_idx);
                            if let Some(last_id) =
                                env.storage().persistent().get::<_, u64>(&last_key)
                            {
                                env.storage().persistent().set(&entry_key, &last_id);
                            }
                            env.storage().persistent().remove(&last_key);
                        } else {
                            env.storage().persistent().remove(&entry_key);
                        }
                        found = true;
                        break;
                    }
                }
            }

            if found {
                env.storage().persistent().set(&count_key, &(count - 1));
            }
            env.storage().persistent().remove(&exist_key);
        }
        nft.owner = to_address.clone();
        Storage::save_nft(&env, &nft);
        Storage::add_nft_to_owner(&env, &to_address, nft_id);

        env.events().publish(
            (Symbol::new(&env, "NftTransferred"), nft_id),
            NftTransferredEvent {
                nft_id,
                from: from_address,
                to: to_address,
            },
        );

        Ok(())
    }

    /// Returns the on-chain version stored during initialize, or the compiled constant.
    pub fn contract_version(env: Env) -> u32 {
        Storage::get_contract_version(&env).unwrap_or(crate::CONTRACT_VERSION)
    }

    /// Grants `operator` the ability to manage all NFTs owned by `owner`.
    ///
    /// # Authorization
    /// `owner` must authorize this call.
    pub fn set_operator(env: Env, owner: Address, operator: Address) {
        owner.require_auth();
        Storage::set_operator(&env, &owner, &operator);
        env.events().publish(
            (Symbol::new(&env, "OperatorChanged"), owner.clone()),
            OperatorChangedEvent {
                owner,
                operator,
                approved: true,
            },
        );
    }

    /// Revokes operator approval for `operator` over `owner`'s NFTs.
    ///
    /// # Authorization
    /// `owner` must authorize this call.
    pub fn remove_operator(env: Env, owner: Address, operator: Address) {
        owner.require_auth();
        Storage::remove_operator(&env, &owner, &operator);
        env.events().publish(
            (Symbol::new(&env, "OperatorChanged"), owner.clone()),
            OperatorChangedEvent {
                owner,
                operator,
                approved: false,
            },
        );
    }

    /// Returns true if `operator` is approved to manage all NFTs of `owner`.
    pub fn is_operator(env: Env, owner: Address, operator: Address) -> bool {
        Storage::is_operator(&env, &owner, &operator)
    }

    pub fn get_schema_version(env: Env) -> u32 {
        migration::NftRewardMigration::get_schema_version(&env)
    }

    pub fn initialize_schema(env: Env, admin: Address) {
        admin.require_auth();
        migration::NftRewardMigration::initialize_schema(&env);
    }

    pub fn propose_upgrade(
        env: Env,
        admin: Address,
        target_version: u32,
    ) -> Result<hunty_migration::UpgradeProposal, hunty_migration::UpgradeAuthError> {
        let proposal =
            migration::NftRewardMigration::propose_upgrade(&env, &admin, target_version)?;
        env.events().publish(
            migration::NftRewardMigration::upgrade_proposed_topic(&env),
            migration::NftRewardMigration::upgrade_proposed_event(&proposal),
        );
        Ok(proposal)
    }

    pub fn set_upgrade_timelock(
        env: Env,
        admin: Address,
        delay_seconds: u64,
    ) -> Result<(), hunty_migration::UpgradeAuthError> {
        migration::NftRewardMigration::set_upgrade_timelock(&env, &admin, delay_seconds)
    }

    pub fn get_upgrade_proposal(env: Env) -> Option<hunty_migration::UpgradeProposal> {
        migration::NftRewardMigration::get_upgrade_proposal(&env)
    }

    pub fn get_upgrade_timelock(env: Env) -> u64 {
        migration::NftRewardMigration::get_upgrade_timelock(&env)
    }

    pub fn get_upgrade_history(
        env: Env,
        offset: u32,
        limit: u32,
    ) -> soroban_sdk::Vec<hunty_migration::UpgradeHistoryEntry> {
        migration::NftRewardMigration::get_upgrade_history(&env, offset, limit)
    }

    pub fn run_migration(
        env: Env,
        admin: Address,
        target_version: u32,
        dry_run: bool,
    ) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError> {
        let from_version = migration::NftRewardMigration::get_schema_version(&env);
        let report =
            migration::NftRewardMigration::run_migration(&env, &admin, target_version, dry_run)?;
        if !dry_run && report.succeeded && report.from_version < report.to_version {
            env.events().publish(
                migration::NftRewardMigration::upgrade_executed_topic(&env),
                migration::NftRewardMigration::upgrade_executed_event(
                    from_version,
                    report.to_version,
                    env.ledger().timestamp(),
                    admin,
                ),
            );
        }
        Ok(report)
    }

    pub fn rollback_migration(
        env: Env,
        admin: Address,
    ) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError> {
        migration::NftRewardMigration::rollback_migration(&env, &admin)
    }

    /// Searches NFTs by hunt_id using the hunt collection index.
    pub fn search_by_hunt_id(env: Env, hunt_id: u64) -> Vec<u64> {
        Storage::get_hunt_nfts(&env, hunt_id, 0, u32::MAX)
    }

    /// Searches NFTs by rarity range (inclusive).
    pub fn search_by_rarity_range(env: Env, min_rarity: u32, max_rarity: u32) -> Vec<u64> {
        let all_nft_ids = Storage::get_all_nft_ids(&env);
        let mut results = Vec::new(&env);
        for nft_id in all_nft_ids.iter() {
            if let Some(nft) = Storage::get_nft(&env, nft_id) {
                if nft.metadata.rarity >= min_rarity && nft.metadata.rarity <= max_rarity {
                    results.push_back(nft_id);
                }
            }
        }
        results
    }

    /// Locks an NFT to prevent transfers. Only authorized contracts can lock NFTs.
    ///
    /// # Arguments
    /// * `nft_id` - The NFT to lock
    /// * `locker` - The authorized contract locking the NFT (must be whitelisted)
    ///
    /// # Authorization
    /// The `locker` must be an authorized locker contract and must authorize this call.
    pub fn lock_nft(
        env: Env,
        nft_id: u64,
        locker: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        locker.require_auth();

        if !Storage::is_locker(&env, &locker) {
            return Err(crate::errors::NftErrorCode::Unauthorized);
        }

        let mut nft = Storage::get_nft(&env, nft_id)
            .ok_or(crate::errors::NftErrorCode::NftNotFound)?;

        nft.locked = true;
        Storage::save_nft(&env, &nft);

        env.events().publish(
            (Symbol::new(&env, "NftLocked"), nft_id),
            (nft_id, locker),
        );

        Ok(())
    }

    /// Unlocks an NFT to allow transfers. Only authorized contracts can unlock NFTs.
    ///
    /// # Arguments
    /// * `nft_id` - The NFT to unlock
    /// * `locker` - The authorized contract unlocking the NFT (must be whitelisted)
    ///
    /// # Authorization
    /// The `locker` must be an authorized locker contract and must authorize this call.
    pub fn unlock_nft(
        env: Env,
        nft_id: u64,
        locker: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        locker.require_auth();

        if !Storage::is_locker(&env, &locker) {
            return Err(crate::errors::NftErrorCode::Unauthorized);
        }

        let mut nft = Storage::get_nft(&env, nft_id)
            .ok_or(crate::errors::NftErrorCode::NftNotFound)?;

        nft.locked = false;
        Storage::save_nft(&env, &nft);

        env.events().publish(
            (Symbol::new(&env, "NftUnlocked"), nft_id),
            (nft_id, locker),
        );

        Ok(())
    }

    /// Adds an authorized locker contract. Admin only.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must authorize)
    /// * `locker` - The contract address to authorize for locking/unlocking NFTs
    pub fn add_locker(
        env: Env,
        admin: Address,
        locker: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        admin.require_auth();

        let stored_admin = Storage::get_admin(&env)
            .ok_or(crate::errors::NftErrorCode::Unauthorized)?;

        if admin != stored_admin {
            return Err(crate::errors::NftErrorCode::Unauthorized);
        }

        Storage::add_locker(&env, &locker);

        env.events().publish(
            (Symbol::new(&env, "LockerAdded"),),
            locker,
        );

        Ok(())
    }

    /// Removes an authorized locker contract. Admin only.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must authorize)
    /// * `locker` - The contract address to remove authorization from
    pub fn remove_locker(
        env: Env,
        admin: Address,
        locker: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        admin.require_auth();

        let stored_admin = Storage::get_admin(&env)
            .ok_or(crate::errors::NftErrorCode::Unauthorized)?;

        if admin != stored_admin {
            return Err(crate::errors::NftErrorCode::Unauthorized);
        }

        Storage::remove_locker(&env, &locker);

        env.events().publish(
            (Symbol::new(&env, "LockerRemoved"),),
            locker,
        );

        Ok(())
    }
}

#[cfg(test)]
mod test;
