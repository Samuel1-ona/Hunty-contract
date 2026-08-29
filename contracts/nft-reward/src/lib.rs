#![cfg_attr(not(test), no_std)]
#![allow(clippy::too_many_arguments)]
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, Address, Env, Map, String, Symbol, Val,
    Vec,
};
use hunty_common::audit::{
    emit_audit_event, detail, ACTION_ADMIN_ADDED, ACTION_ADMIN_REMOVED, TOPIC_AUDIT,
};

#[allow(dead_code)]
const MAX_URI_LEN: usize = 512;
const MAX_NFT_TITLE_BYTES: u32 = 128;
const MAX_NFT_DESCRIPTION_BYTES: u32 = 1024;
const MAX_NFT_URI_BYTES: u32 = 512;
const MAX_EXTENSION_FIELDS: u32 = 10;
const MAX_EXTENSION_KEY_BYTES: u32 = 64;
const MAX_EXTENSION_VALUE_BYTES: u32 = 512;
/// Maximum NFTs returned by a single scan/list query.
///
/// Matches hunty-core's `MAX_LEADERBOARD_SCAN_SIZE` / `MAX_HUNT_SEARCH_SCAN_SIZE`
/// (200). `MAX_BATCH_SIZE` (50) is the tighter write-batch cap used elsewhere;
/// 200 is the read-scan cap so listing stays inside a similar gas budget.
const MAX_SCAN_LIMIT: u32 = 200;

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
    /// Arbitrary key-value metadata extensions beyond the core fields.
    /// Max 10 extension fields per NFT.
    pub extensions: Map<String, String>,
}

/// Collection-level metadata stored at initialization and exposed via a query.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionMetadata {
    pub name: String,
    pub description: String,
    pub total_supply: u64,
    pub creator: Option<Address>,
}

/// Collection-level statistics included in mint events for indexers.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftCollectionStats {
    pub total_supply: u64,
    pub total_hunts: u64,
    pub total_owners: u64,
}

fn image_uri_is_valid(uri: &String) -> bool {
    // Accept non-empty URIs that start with https:// or ipfs://
    // soroban_sdk::String has no as_str(); compare via UTF-8 text when possible.
    let len = uri.len();
    if len == 0 || len > 200 {
        return false;
    }
    let mut buf = [0u8; 200];
    uri.copy_into_slice(&mut buf[..len as usize]);
    // SAFETY: `buf` is populated from a soroban_sdk::String via
    // copy_into_slice, so the bytes are guaranteed to be valid UTF-8.
    let text = unsafe { core::str::from_utf8_unchecked(&buf[..len as usize]) };

    if text.starts_with("https://") {
        // Require at least one non-whitespace character after the scheme.
        let authority = &text[8..];
        return !authority.is_empty() && !authority.bytes().all(|b| b == b' ');
    }
    if text.starts_with("ipfs://") {
        // Require CID of at least 46 chars (IPFS v0 base58) after "ipfs://".
        let cid = &text[7..];
        return cid.len() >= 46;
    }
    false
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
    /// Arbitrary key-value metadata extensions.
    pub extensions: Map<String, String>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftCore {
    pub nft_id: u64,
    pub hunt_id: u64,
    pub owner: Address,
    pub completion_player: Address,
    pub transferable: bool,
    pub minted_at: u64,
    pub locked: bool,
}

/// NFT data structure stored on-chain.
/// NOTE: Do NOT add new fields here without a migration step — the Soroban
/// host rejects stored structs whose field count differs from the stored
/// ScVal map. Use per-NFT auxiliary keys for new metadata instead.
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
    pub minted_at: u64,
    pub hunt_title: String,
    pub total_minted_for_hunt: u32,
    pub completion_rank: u32,
    pub collection_stats: String,
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

/// Event emitted when an NFT extension is set.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftExtensionSetEvent {
    pub nft_id: u64,
    pub key: String,
    pub updater: Address,
}

/// Event emitted when an NFT extension is removed.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftExtensionRemovedEvent {
    pub nft_id: u64,
    pub key: String,
    pub updater: Address,
}

/// Event emitted when an NFT is burned (permanently destroyed).
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftBurnedEvent {
    pub nft_id: u64,
    pub hunt_id: u64,
    pub owner: Address,
}

/// Event emitted on contract initialization with admin, minter, and max supply details.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractInitializedEvent {
    pub admin: Address,
    pub minter: Address,
    pub max_supply: Option<u64>,
    pub timestamp: u64,
}

/// Event emitted when an authorized contract is added.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizedContractAddedEvent {
    pub admin: Address,
    pub contract: Address,
    pub timestamp: u64,
}

/// Event emitted when an authorized contract is removed.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizedContractRemovedEvent {
    pub admin: Address,
    pub contract: Address,
    pub timestamp: u64,
}

/// Event emitted when the reward manager contract is set.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardManagerSetEvent {
    pub admin: Address,
    pub reward_manager: Address,
    pub timestamp: u64,
}

mod errors;
pub use errors::NftErrorCode;
mod migration;
mod sanitization;
pub mod storage;
#[cfg(test)]
mod test;
use storage::Storage;

#[contract]
pub struct NftReward;

/// Current metadata schema version (bump when adding/changing NftMetadata shape).
pub const METADATA_SCHEMA_VERSION: u32 = 2;

/// Contract version constant.
pub const CONTRACT_VERSION: u32 = 3;

#[contractimpl]
impl NftReward {
    /// Initializes the NFT reward contract with an admin, minter, and optional max supply cap.
    pub fn initialize(
        env: Env,
        admin: Address,
        minter: Address,
        max_supply: Option<u64>,
        collection_metadata: CollectionMetadata,
    ) -> Result<(), crate::errors::NftErrorCode> {
        // Require the admin to authorize initialization to prevent an attacker from
        // becoming admin by racing the first transaction after deployment.
        admin.require_auth();

        if Storage::is_initialized(&env) {
            return Err(crate::errors::NftErrorCode::AlreadyInitialized);
        }

        if let Some(0) = max_supply {
            return Err(crate::errors::NftErrorCode::InvalidMaxSupply);
        }

        Storage::save_admin(&env, &admin);
        Storage::add_minter(&env, &minter);
        // Ensure the initial minter is also enrolled as an authorized contract
        Storage::add_authorized_contract(&env, &minter);
        Storage::set_max_supply(&env, max_supply);
        Storage::save_collection_metadata(&env, &collection_metadata);
        Storage::mark_initialized(&env);
        Storage::set_contract_version(&env, CONTRACT_VERSION);

        // Emit initialization event
        let timestamp = env.ledger().timestamp();
        env.events().publish(
            (symbol_short!("INIT"), admin.clone()),
            ContractInitializedEvent {
                admin: admin.clone(),
                minter: minter.clone(),
                max_supply,
                timestamp,
            },
        );

        Ok(())
    }

    fn require_admin(env: &Env, admin: &Address) -> Result<(), crate::errors::NftErrorCode> {
        admin.require_auth();
        let stored_admin =
            Storage::get_admin(env).ok_or(crate::errors::NftErrorCode::NotInitialized)?;
            if stored_admin != *admin {
            return Err(crate::errors::NftErrorCode::Unauthorized);
        }
        Ok(())
    }

    fn require_authorized_caller(env: &Env, caller: &Address) {
        // Check the stored reward_manager address first.
        if let Some(reward_manager) = Storage::get_reward_manager(env) {
            if reward_manager == *caller {
                caller.require_auth();
                return;
            }
        }
        // Always require the caller to authorize the operation.
        caller.require_auth();
        // Fail-closed: caller must be explicitly authorized.
        if !Storage::is_authorized_contract(env, caller) {
            panic_with_error!(env, crate::errors::NftErrorCode::Unauthorized);
        }
    }

    /// Mints a unique NFT as a reward for hunt completion.
    ///
    /// `minter` must be an authorized minter (and must sign the transaction) when the
    /// contract has been initialized. Before initialization the check is skipped so
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
        minter: Address,
        hunt_id: u64,
        player_address: Address,
        metadata: NftMetadata,
    ) -> u64 {
        Self::require_authorized_caller(&env, &minter);
        Self::mint_reward_nft_impl(env, hunt_id, player_address, metadata, true)
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
    /// - "extensions": Map<String, String> (optional, arbitrary key-value metadata)
    ///
    /// # Errors
    /// Returns `NftErrorCode::InvalidMetadata` when a key is **present** but holds
    /// a value of the wrong type. An **absent** key silently takes its documented default.
    pub fn mint_reward_nft_from_map(
        env: Env,
        minter: Address,
        hunt_id: u64,
        player_address: Address,
        metadata: Map<Symbol, Val>,
    ) -> Result<u64, crate::errors::NftErrorCode> {
        Self::require_authorized_caller(&env, &minter);
        use soroban_sdk::TryFromVal;

        macro_rules! extract_field {
            ($key:expr, $ty:ty, $default:expr) => {
                match metadata.get(Symbol::new(&env, $key)) {
                    None => $default,
                    Some(v) => <$ty>::try_from_val(&env, &v)
                        .map_err(|_| crate::errors::NftErrorCode::InvalidMetadata)?,
                }
            };
        }

        macro_rules! extract_optional_field {
            ($key:expr, $ty:ty) => {
                match metadata.get(Symbol::new(&env, $key)) {
                    None => None,
                    Some(v) => Some(
                        <$ty>::try_from_val(&env, &v)
                            .map_err(|_| crate::errors::NftErrorCode::InvalidMetadata)?,
                    ),
                }
            };
        }

        let title = extract_field!("title", String, String::from_str(&env, ""));
        let description = extract_field!("description", String, String::from_str(&env, ""));
        let image_uri = extract_field!("image_uri", String, String::from_str(&env, ""));

        let hunt_title = metadata
            .get(Symbol::new(&env, "hunt_title"))
            .and_then(|v| String::try_from_val(&env, &v).ok())
            .unwrap_or_else(|| title.clone());

        let rarity = metadata
            .get(Symbol::new(&env, "rarity"))
            .and_then(|v| u32::try_from_val(&env, &v).ok())
            .unwrap_or(0u32);

        let tier = metadata
            .get(Symbol::new(&env, "tier"))
            .and_then(|v| u32::try_from_val(&env, &v).ok())
            .unwrap_or(0u32);

        let creator = metadata
            .get(Symbol::new(&env, "creator"))
            .and_then(|v| Address::try_from_val(&env, &v).ok())
            .or_else(|| Some(player_address.clone()));

        let creator = match metadata.get(Symbol::new(&env, "creator")) {
            None => Some(player_address.clone()),
            Some(v) => Some(
                Address::try_from_val(&env, &v)
                    .map_err(|_| crate::errors::NftErrorCode::InvalidMetadata)?,
            ),
        };

        let royalty_bps: Option<u32> = extract_optional_field!("royalty_bps", u32);
        let transferable = extract_field!("transferable", bool, false);

        // Parse extensions from metadata map
        let extensions: Map<String, String> =
            extract_field!("extensions", Map<String, String>, Map::new(&env));

        let meta = NftMetadata {
            title,
            description,
            image_uri,
            hunt_title,
            rarity,
            tier,
            creator,
            royalty_bps,
            extensions,
        };
        Ok(Self::mint_reward_nft_impl(env, hunt_id, player_address, meta, transferable))
    }

    fn validate_image_uri(env: &Env, value: &String) -> Result<(), NftErrorCode> {
        if !image_uri_is_valid(value) {
            return Err(NftErrorCode::InvalidMetadata);
        }
        Ok(())
    }

    fn sanitize_metadata_field(
        env: &Env,
        value: &String,
        max_bytes: u32,
        allow_empty: bool,
    ) -> String {
        match sanitization::StringSanitizer::sanitize(env, value, max_bytes, allow_empty) {
            Ok(s) => s,
            Err(_) => panic_with_error!(env, crate::errors::NftErrorCode::InvalidMetadata),
        }
    }

    fn validate_extensions(
        _env: &Env,
        extensions: &Map<String, String>,
    ) -> Result<(), NftErrorCode> {
        let count = extensions.len();
        if count > MAX_EXTENSION_FIELDS {
            return Err(NftErrorCode::TooManyExtensions);
        }
        for (key, value) in extensions.iter() {
            if key.len() > MAX_EXTENSION_KEY_BYTES {
                return Err(NftErrorCode::InvalidExtensionKey);
            }
            if value.len() > MAX_EXTENSION_VALUE_BYTES {
                return Err(NftErrorCode::InvalidExtensionValue);
            }
        }
        Ok(())
    }

    fn compute_completion_rank(
        env: &Env,
        hunt_id: u64,
    ) -> u32 {
        let players = Storage::get_hunt_players(env, hunt_id);
        let mut completed: u32 = 0;
        for i in 0..players.len() {
            let progress = players.get(i).unwrap();
            if progress.is_completed {
                completed += 1;
            }
        }
        completed.saturating_add(1)
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
        if let Err(e) = Self::validate_image_uri(&env, &metadata.image_uri) {
            panic_with_error!(&env, e);
        }

        // Validate extensions
        if let Err(e) = Self::validate_extensions(&env, &metadata.extensions) {
            panic_with_error!(&env, e);
        }

        let mut metadata = metadata;
        metadata.title =
            Self::sanitize_metadata_field(&env, &metadata.title, MAX_NFT_TITLE_BYTES, false);
        metadata.description = Self::sanitize_metadata_field(
            &env,
            &metadata.description,
            MAX_NFT_DESCRIPTION_BYTES,
            true,
        );
        metadata.image_uri =
            Self::sanitize_metadata_field(&env, &metadata.image_uri, MAX_NFT_URI_BYTES, true);
        metadata.hunt_title =
            Self::sanitize_metadata_field(&env, &metadata.hunt_title, MAX_NFT_TITLE_BYTES, true);

        if let Some(max_supply) = Storage::get_max_supply(&env) {
            let current_supply = Storage::get_nft_counter(&env);
            if current_supply >= max_supply {
                panic_with_error!(&env, crate::errors::NftErrorCode::MaxSupplyReached);
            }
        }

        let minted_at = env.ledger().timestamp();
        let nft_id = Storage::next_nft_id(&env);

        let event = NftMintedEvent {
            nft_id,
            hunt_id,
            owner: player_address.clone(),
            rarity: metadata.rarity,
            tier: metadata.tier,
            minted_at,
            metadata: metadata.clone(),
        };

        let nft_data = NftData {
            nft_id,
            hunt_id,
            owner: player_address.clone(),
            completion_player: player_address.clone(),
            metadata,
            transferable,
            minted_at,
            locked: false,
        };

        Storage::save_nft(&env, &nft_data);
        Storage::set_nft_version(&env, nft_id, METADATA_SCHEMA_VERSION);
        Storage::add_nft_to_owner(&env, &player_address, nft_id);
        Storage::increment_owner_hunt_count(&env, &player_address, hunt_id);
        Storage::add_nft_to_hunt(&env, hunt_id, nft_id);
        Storage::mark_hunt_minted(&env, hunt_id);
        Storage::update_collection_metadata_total_supply(&env, Storage::get_nft_counter(&env));

        let event = NftMintedEvent {
            nft_id,
            hunt_id,
            owner: player_address,
            rarity: nft_data.metadata.rarity,
            tier: nft_data.metadata.tier,
            minted_at,
            hunt_title: metadata.hunt_title.clone(),
            total_minted_for_hunt: Storage::get_nft_counter(&env) as u32,
            completion_rank: Self::compute_completion_rank(env, hunt_id),
            collection_stats: format!(
                "total_supply={},total_hunts={},total_owners={}",
                Storage::get_nft_counter(&env),
                0u64, // total_hunts would need tracking
                0u64  // total_owners would need tracking
            ),
        };
        env.events()
            .publish((Symbol::new(&env, "NftMinted"), nft_id), event);

        nft_id
    }

    /// Retrieves NFT data by ID.
    pub fn get_nft(env: Env, nft_id: u64) -> Option<NftData> {
        Storage::get_nft(&env, nft_id)
    }

    /// Returns the collection-level metadata configured at initialization.
    pub fn get_collection_metadata(env: Env) -> Option<CollectionMetadata> {
        Storage::get_collection_metadata(&env)
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
            extensions: nft.metadata.extensions.clone(),
        })
    }

    /// Sets an extension field on an NFT. Only the NFT owner can call this.
    /// Max 10 extension fields per NFT. If the key already exists, it is updated.
    /// If the maximum is reached and the key is new, it returns an error.
    ///
    /// # Arguments
    /// * `nft_id` - The NFT to extend
    /// * `owner` - The current owner (must authorize)
    /// * `key` - The extension key (max 64 bytes)
    /// * `value` - The extension value (max 512 bytes)
    pub fn set_nft_extension(
        env: Env,
        nft_id: u64,
        owner: Address,
        key: String,
        value: String,
    ) -> Result<(), crate::errors::NftErrorCode> {
        owner.require_auth();

        let mut nft =
            Storage::get_nft(&env, nft_id).ok_or(crate::errors::NftErrorCode::NftNotFound)?;

        if nft.owner != owner {
            return Err(crate::errors::NftErrorCode::NotOwner);
        }

        // Validate key and value lengths
        if key.len() > MAX_EXTENSION_KEY_BYTES {
            return Err(crate::errors::NftErrorCode::InvalidExtensionKey);
        }
        if value.len() > MAX_EXTENSION_VALUE_BYTES {
            return Err(crate::errors::NftErrorCode::InvalidExtensionValue);
        }

        // Check if key already exists
        let key_exists = nft.metadata.extensions.contains_key(key.clone());

        if !key_exists && nft.metadata.extensions.len() >= MAX_EXTENSION_FIELDS {
            return Err(crate::errors::NftErrorCode::TooManyExtensions);
        }

        nft.metadata.extensions.set(key.clone(), value);
        Storage::save_nft(&env, &nft);

        env.events().publish(
            (Symbol::new(&env, "NftExtensionSet"), nft_id),
            NftExtensionSetEvent {
                nft_id,
                key,
                updater: owner,
            },
        );

        Ok(())
    }

    /// Gets the value of a specific extension field for an NFT.
    ///
    /// # Arguments
    /// * `nft_id` - The NFT to query
    /// * `key` - The extension key to look up
    ///
    /// # Returns
    /// The extension value if found, None otherwise.
    pub fn get_nft_extension(env: Env, nft_id: u64, key: String) -> Option<String> {
        let nft = Storage::get_nft(&env, nft_id)?;
        nft.metadata.extensions.get(key)
    }

    /// Gets all extension fields for an NFT.
    ///
    /// # Arguments
    /// * `nft_id` - The NFT to query
    ///
    /// # Returns
    /// Map of all extension key-value pairs.
    pub fn get_nft_extensions(env: Env, nft_id: u64) -> Option<Map<String, String>> {
        let nft = Storage::get_nft(&env, nft_id)?;
        Some(nft.metadata.extensions)
    }

    /// Removes an extension field from an NFT. Only the NFT owner can call this.
    ///
    /// # Arguments
    /// * `nft_id` - The NFT to modify
    /// * `owner` - The current owner (must authorize)
    /// * `key` - The extension key to remove
    pub fn remove_nft_extension(
        env: Env,
        nft_id: u64,
        owner: Address,
        key: String,
    ) -> Result<(), crate::errors::NftErrorCode> {
        owner.require_auth();

        let mut nft =
            Storage::get_nft(&env, nft_id).ok_or(crate::errors::NftErrorCode::NftNotFound)?;

        if nft.owner != owner {
            return Err(crate::errors::NftErrorCode::NotOwner);
        }

        if !nft.metadata.extensions.contains_key(key.clone()) {
            return Err(crate::errors::NftErrorCode::ExtensionNotFound);
        }

        nft.metadata.extensions.remove(key.clone());
        Storage::save_nft(&env, &nft);

        env.events().publish(
            (Symbol::new(&env, "NftExtensionRemoved"), nft_id),
            NftExtensionRemovedEvent {
                nft_id,
                key,
                updater: owner,
            },
        );

        Ok(())
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
        Storage::save_reward_manager(&env, &reward_manager);

        // Emit event for audit trail
        let timestamp = env.ledger().timestamp();
        env.events().publish(
            (symbol_short!("RWD_MGR"), admin.clone()),
            RewardManagerSetEvent {
                admin: admin.clone(),
                reward_manager: reward_manager.clone(),
                timestamp,
            },
        );

        Ok(())
    }

    /// Adds a contract to the authorized callers list. Only the admin can call this.
    pub fn add_authorized_contract(
        env: Env,
        admin: Address,
        contract: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::add_authorized_contract(&env, &contract);

        // Emit event for audit trail
        let timestamp = env.ledger().timestamp();
        env.events().publish(
            (symbol_short!("AUTH_ADD"), admin.clone()),
            AuthorizedContractAddedEvent {
                admin: admin.clone(),
                contract: contract.clone(),
                timestamp,
            },
        );

        Ok(())
    }

    /// Removes a contract from the authorized callers list. Only the admin can call this.
    pub fn remove_authorized_contract(
        env: Env,
        admin: Address,
        contract: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::remove_authorized_contract(&env, &contract);

        // Emit event for audit trail
        let timestamp = env.ledger().timestamp();
        env.events().publish(
            (symbol_short!("AUTH_REM"), admin.clone()),
            AuthorizedContractRemovedEvent {
                admin: admin.clone(),
                contract: contract.clone(),
                timestamp,
            },
        );

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
                if let Some(new_uri) =
                    Self::replace_prefix(&env, &nft.metadata.image_uri, &old_prefix, &new_prefix)
                {
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

    /// Replaces a matching `old_prefix` at the start of `uri` with `new_prefix`.
    ///
    /// Returns `None` — meaning "leave the URI untouched" — whenever the
    /// operation cannot be performed *exactly*:
    /// - `uri` does not start with `old_prefix`, or
    /// - any of `uri` / `old_prefix` / `new_prefix` exceeds `MAX_NFT_URI_BYTES`
    ///   (all three are bounded by that constant elsewhere in the contract;
    ///   this defends against callers that bypass those checks), or
    /// - the resulting URI would exceed `MAX_NFT_URI_BYTES`.
    ///
    /// Every early return above is an explicit, checked rejection. Unlike the
    /// previous implementation, nothing here is silently truncated (the old
    /// code copied at most 256 bytes into a fixed buffer but kept comparing
    /// against the untruncated length) and nothing can index out of bounds
    /// (the old code panicked when `old_prefix` exceeded 256 bytes, or when
    /// `new_prefix` was long enough to overflow the 512-byte output buffer).
    fn replace_prefix(
        env: &Env,
        uri: &String,
        old_prefix: &String,
        new_prefix: &String,
    ) -> Option<String> {
        const MAX: usize = MAX_NFT_URI_BYTES as usize;

        let uri_len = uri.len() as usize;
        let old_len = old_prefix.len() as usize;
        let new_len = new_prefix.len() as usize;

        // Reject anything that wouldn't fit our fixed-size buffers instead of
        // truncating the copy (old bug #1) or indexing past it (old bug #2).
        if uri_len > MAX || old_len > MAX || new_len > MAX {
            return None;
        }
        if uri_len < old_len {
            return None;
        }

        let mut buf_uri = [0u8; MAX];
        let mut buf_old = [0u8; MAX];

        uri.copy_into_slice(&mut buf_uri[..uri_len]);
        old_prefix.copy_into_slice(&mut buf_old[..old_len]);

        if buf_uri[..old_len] != buf_old[..old_len] {
            return None;
        }

        let suffix_len = uri_len - old_len;
        let total_len = new_len + suffix_len;
        // Reject instead of overflowing the output buffer (old bug #3) when a
        // longer `new_prefix` would push the result past the max URI length.
        if total_len > MAX {
            return None;
        }

        let mut buf_new = [0u8; MAX];
        new_prefix.copy_into_slice(&mut buf_new[..new_len]);

        let mut final_buf = [0u8; MAX];
        final_buf[..new_len].copy_from_slice(&buf_new[..new_len]);
        final_buf[new_len..total_len].copy_from_slice(&buf_uri[old_len..uri_len]);

        // SAFETY: `final_buf` is assembled entirely from bytes copied out of
        // soroban_sdk::String values, so the slice is valid UTF-8. The split
        // point `old_len` is a byte-for-byte match between `uri` and
        // `old_prefix`, both independently valid UTF-8, so it falls on a
        // codepoint boundary in both; concatenating `new_prefix` (itself
        // valid UTF-8) with that boundary-aligned suffix cannot produce an
        // invalid sequence.
        let text = unsafe { core::str::from_utf8_unchecked(&final_buf[..total_len]) };
        Some(String::from_str(env, text))
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

        let new_description =
            Self::sanitize_metadata_field(&env, &new_description, MAX_NFT_DESCRIPTION_BYTES, true);
        let new_image_uri =
            Self::sanitize_metadata_field(&env, &new_image_uri, MAX_NFT_URI_BYTES, true);

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

    /// Returns the configured maximum total supply of NFTs.
    ///
    /// - `None`  → no cap was set (unlimited minting)
    /// - `Some(n)` → at most `n` NFTs may ever be minted
    pub fn get_max_supply(env: Env) -> Option<u64> {
        Storage::get_max_supply(&env)
    }

    /// Updates the maximum total supply cap. Admin only.
    ///
    /// - Pass `None` to remove the cap (unlimited).
    /// - Pass `Some(n)` where `n > 0` and `n >= current total_supply` to set a new cap.
    ///   Attempting to set a cap of 0 or lower than the already-minted count is
    ///   rejected with `InvalidMaxSupply` to prevent bricking the contract.
    ///
    /// # Errors
    /// * `NotInitialized` - Contract has not been initialized yet
    /// * `Unauthorized`   - Caller is not the admin
    /// * `InvalidMaxSupply` - Attempting to set cap to Some(0) or below already-minted supply
    pub fn set_max_supply(
        env: Env,
        admin: Address,
        new_max: Option<u64>,
    ) -> Result<(), crate::errors::NftErrorCode> {
        Self::require_admin(&env, &admin)?;

        // Guard: never allow setting a cap of 0 or below what's already minted.
        if let Some(cap) = new_max {
            if cap == 0 {
                return Err(crate::errors::NftErrorCode::InvalidMaxSupply);
            }
            let minted = Storage::get_nft_counter(&env);
            if cap < minted {
                return Err(crate::errors::NftErrorCode::InvalidMaxSupply);
            }
        }

        Storage::set_max_supply(&env, new_max);
        Ok(())
    }

    /// Returns the number of NFTs that can still be minted.
    ///
    /// - `None`  → unlimited (no cap configured)
    /// - `Some(n)` → exactly `n` more NFTs may be minted before the cap is hit
    ///
    /// Once the cap is reached this returns `Some(0)`, and any subsequent mint
    /// will panic with `MaxSupplyReached`.
    pub fn get_remaining_supply(env: Env) -> Option<u64> {
        match Storage::get_max_supply(&env) {
            None => None,
            Some(max) => {
                let minted = Storage::get_nft_counter(&env);
                Some(max.saturating_sub(minted))
            }
        }
    }

    /// Lists all NFTs minted by the contract with pagination support.
    ///
    /// Returns a vector of NftData structs, paginated by offset and limit.
    /// The limit is bounded to MAX_SCAN_LIMIT (200) to prevent excessive gas consumption.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `offset` - The starting index for pagination (0-based)
    /// * `limit` - The maximum number of NFTs to return (capped at MAX_SCAN_LIMIT)
    ///
    /// # Returns
    /// Vec<NftData> - A vector of NFT data structures, bounded by limit or remaining NFTs
    pub fn list_all_nfts(env: Env, offset: u32, limit: u32) -> Vec<NftData> {
        let all_nft_ids = Storage::get_all_nft_ids(&env);
        let total_count = all_nft_ids.len();

        if offset >= total_count {
            return Vec::new(&env);
        }

        // Apply bounded scan limit to prevent excessive gas consumption
        let bounded_limit = limit.min(MAX_SCAN_LIMIT);
        let end = offset.saturating_add(bounded_limit).min(total_count);

        let mut result = Vec::new(&env);
        for i in offset..end {
            if let Some(nft_id) = all_nft_ids.get(i) {
                if let Some(nft_data) = Storage::get_nft(&env, nft_id) {
                    result.push_back(nft_data);
                }
            }
        }

        result
    }

    /// Searches NFTs by metadata fields with pagination support.
    ///
    /// Allows filtering NFTs by various metadata fields. All filter parameters are optional -
    /// only provided filters are applied. Returns matching NFTs with pagination.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `offset` - The starting index for pagination (0-based)
    /// * `limit` - The maximum number of NFTs to return (capped at MAX_SCAN_LIMIT)
    /// * `title_filter` - Optional filter for NFT title (exact match)
    /// * `hunt_title_filter` - Optional filter for hunt title (exact match)
    /// * `rarity_filter` - Optional filter for rarity tier (0-5)
    /// * `tier_filter` - Optional filter for custom tier
    /// * `creator_filter` - Optional filter for creator address
    /// * `hunt_id_filter` - Optional filter for hunt ID
    /// * `extension_key` - Optional extension key to search for
    /// * `extension_value` - Optional extension value to match (requires extension_key)
    ///
    /// # Returns
    /// Vec<NftData> - A vector of matching NFT data structures, paginated by offset and limit
    pub fn search_nfts_by_metadata(
        env: Env,
        offset: u32,
        limit: u32,
        title_filter: Option<String>,
        hunt_title_filter: Option<String>,
        rarity_filter: Option<u32>,
        tier_filter: Option<u32>,
        creator_filter: Option<Address>,
        hunt_id_filter: Option<u64>,
        extension_key: Option<String>,
        extension_value: Option<String>,
    ) -> Vec<NftData> {
        let all_nft_ids = Storage::get_all_nft_ids(&env);
        let mut matches = Vec::new(&env);

        // Collect all matching NFTs first
        for nft_id in all_nft_ids.iter() {
            if let Some(nft) = Storage::get_nft(&env, nft_id) {
                let mut is_match = true;

                // Apply title filter
                if let Some(ref filter) = title_filter {
                    if nft.metadata.title != *filter {
                        is_match = false;
                    }
                }

                // Apply hunt title filter
                if is_match {
                    if let Some(ref filter) = hunt_title_filter {
                        if nft.metadata.hunt_title != *filter {
                            is_match = false;
                        }
                    }
                }

                // Apply rarity filter
                if is_match {
                    if let Some(filter) = rarity_filter {
                        if nft.metadata.rarity != filter {
                            is_match = false;
                        }
                    }
                }

                // Apply tier filter
                if is_match {
                    if let Some(filter) = tier_filter {
                        if nft.metadata.tier != filter {
                            is_match = false;
                        }
                    }
                }

                // Apply creator filter
                if is_match {
                    if let Some(ref filter) = creator_filter {
                        if nft.metadata.creator != Some(filter.clone()) {
                            is_match = false;
                        }
                    }
                }

                // Apply hunt ID filter
                if is_match {
                    if let Some(filter) = hunt_id_filter {
                        if nft.hunt_id != filter {
                            is_match = false;
                        }
                    }
                }

                // Apply extension filter
                if is_match {
                    if let Some(ref key) = extension_key {
                        if let Some(ref value) = extension_value {
                            // Both key and value provided - exact match
                            if let Some(stored_value) = nft.metadata.extensions.get(key.clone()) {
                                if stored_value != *value {
                                    is_match = false;
                                }
                            } else {
                                is_match = false;
                            }
                        } else {
                            // Only key provided - check if extension exists
                            if !nft.metadata.extensions.contains_key(key.clone()) {
                                is_match = false;
                            }
                        }
                    }
                }

                if is_match {
                    matches.push_back(nft);
                }
            }
        }

        // Apply pagination to results
        let total_matches = matches.len();
        if offset >= total_matches {
            return Vec::new(&env);
        }

        let bounded_limit = limit.min(MAX_SCAN_LIMIT);
        let end = offset.saturating_add(bounded_limit).min(total_matches);

        let mut result = Vec::new(&env);
        for i in offset..end {
            if let Some(nft) = matches.get(i) {
                result.push_back(nft.clone());
            }
        }

        result
    }

    /// Transfers an NFT to a new owner when the NFT is transferable.
    /// Non-transferable (soulbound) NFTs remain bound to the minting recipient.
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

        if nft.locked {
            return Err(crate::errors::NftErrorCode::NftLocked);
        }
        if !nft.transferable {
            return Err(crate::errors::NftErrorCode::NftNotTransferable);
        }
        if nft.owner != from_address {
            return Err(crate::errors::NftErrorCode::NotOwner);
        }
        if to_address == from_address {
            return Err(crate::errors::NftErrorCode::InvalidRecipient);
        }
        if caller != from_address && !Storage::is_operator(&env, &from_address, &caller) {
            return Err(crate::errors::NftErrorCode::NotOperator);
        }

        let hunt_id = nft.hunt_id;
        Storage::remove_nft_from_owner(&env, &from_address, nft_id);
        Storage::decrement_owner_hunt_count(&env, &from_address, hunt_id);
        nft.owner = to_address.clone();
        Storage::save_nft(&env, &nft);
        Storage::add_nft_to_owner(&env, &to_address, nft_id);
        Storage::increment_owner_hunt_count(&env, &to_address, hunt_id);

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

    /// Returns the owner of an NFT.
    pub fn owner_of(env: Env, nft_id: u64) -> Option<Address> {
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
    /// Performs an O(1) indexed lookup via the stored (owner, hunt_id) count mapping.
    pub fn has_hunt_nft(env: Env, address: Address, hunt_id: u64) -> bool {
        Storage::has_hunt_nft(&env, &address, hunt_id)
    }

    /// Returns paginated NFT IDs owned by an address.
    /// The limit is bounded to MAX_SCAN_LIMIT (1000) to prevent excessive gas consumption.
    pub fn get_player_nfts(env: Env, owner: Address, offset: u32, limit: u32) -> Vec<u64> {
        let nfts = Storage::get_owner_nfts(&env, &owner);
        let len = nfts.len();
        if offset >= len {
            return Vec::new(&env);
        }
        let bounded_limit = limit.min(MAX_SCAN_LIMIT);
        let end = offset.saturating_add(bounded_limit).min(len);
        nfts.slice(offset..end)
    }

    /// Returns paginated NFT IDs minted for a hunt.
    /// The limit is bounded to MAX_SCAN_LIMIT (1000) to prevent excessive gas consumption.
    pub fn get_nfts_by_hunt(env: Env, hunt_id: u64, offset: u32, limit: u32) -> Vec<u64> {
        Storage::get_hunt_nfts(&env, hunt_id, offset, limit.min(MAX_SCAN_LIMIT))
    }

    /// Returns the total number of NFTs minted for a hunt.
    pub fn get_hunt_nft_count(env: Env, hunt_id: u64) -> u32 {
        Storage::get_hunt_nft_count(&env, hunt_id)
    }

    /// Grants `operator` the ability to manage all NFTs owned by `owner`.
    ///
    /// # Authorization
    /// `owner` must authorize this call.
    pub fn set_operator(env: Env, owner: Address, operator: Address) {
        owner.require_auth();
        Storage::set_operator(&env, &owner, &operator);
        env.events().publish(
            (Symbol::new(&env, "OperatorChanged"),),
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
            (Symbol::new(&env, "OperatorChanged"),),
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

    /// Burns (permanently destroys) an NFT, removing it from storage and the owner's list.
    ///
    /// # Authorization
    /// The `owner` must authorize this call and be the current owner of the NFT.
    ///
    /// # Errors
    /// Returns `NftNotFound` if the NFT does not exist.
    /// Returns `NotOwner` if the caller is not the current owner.
    /// Returns `NftLocked` if the NFT is locked (e.g., staked elsewhere).
    pub fn burn_nft(
        env: Env,
        nft_id: u64,
        owner: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        owner.require_auth();

        let nft = Storage::get_nft(&env, nft_id).ok_or(crate::errors::NftErrorCode::NftNotFound)?;

        if nft.owner != owner {
            return Err(crate::errors::NftErrorCode::NotOwner);
        }

        if nft.locked {
            return Err(crate::errors::NftErrorCode::NftLocked);
        }

        let hunt_id = nft.hunt_id;
        Storage::remove_nft(&env, nft_id);
        Storage::remove_nft_from_hunt(&env, hunt_id, nft_id);
        Storage::remove_nft_from_owner(&env, &owner, nft_id);
        Storage::decrement_owner_hunt_count(&env, &owner, hunt_id);

        env.events().publish(
            (Symbol::new(&env, "NftBurned"), nft_id),
            NftBurnedEvent {
                nft_id,
                hunt_id,
                owner,
            },
        );

        Ok(())
    }

    // -----------------------------------------------------------------------------
    // Schema Migration
    // -----------------------------------------------------------------------------

    pub fn get_schema_version(env: Env) -> u32 {
        migration::NftRewardMigration::get_schema_version(&env)
    }

    pub fn initialize_schema(env: Env, admin: Address) {
        admin.require_auth();
        migration::NftRewardMigration::initialize_schema(&env, &admin);
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
        migration::NftRewardMigration::get_upgrade_history(&env, offset, limit.min(MAX_SCAN_LIMIT))
    }

    pub fn run_migration(
        env: Env,
        admin: Address,
        target_version: u32,
        dry_run: bool,
    ) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError> {
        let from_version = migration::NftRewardMigration::get_schema_version(&env);
        let report = migration::NftRewardMigration::run_migration(
            &env,
            &admin,
            target_version,
            dry_run,
        )?;
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
}
