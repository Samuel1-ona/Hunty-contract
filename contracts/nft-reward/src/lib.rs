#![cfg_attr(not(test), no_std)]
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, Address, Env, Map, String, Symbol,
    Val, Vec, symbol_short,
};

const MAX_URI_LEN: usize = 512;
const MAX_NFT_TITLE_BYTES: u32 = 128;
const MAX_NFT_DESCRIPTION_BYTES: u32 = 1024;
const MAX_NFT_URI_BYTES: u32 = 512;
const MAX_EXTENSION_FIELDS: u32 = 10;
const MAX_EXTENSION_KEY_BYTES: u32 = 64;
const MAX_EXTENSION_VALUE_BYTES: u32 = 512;

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

/// Collection-level statistics included in mint events for indexers.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftCollectionStats {
    pub total_supply: u64,
    pub total_hunts: u64,
    pub total_owners: u64,
}

fn image_uri_is_valid(uri: &String) -> bool {
    let len = uri.len();
    if len == 0 || len > 200 {
        return false;
    }
    let mut buf = [0u8; 200];
    uri.copy_into_slice(&mut buf[..len as usize]);
    if let Ok(text) = core::str::from_utf8(&buf[..len as usize]) {
        text.starts_with("https://") || text.starts_with("ipfs://")
    } else {
        false
    }
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
    pub metadata: NftMetadata,
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

mod errors;
pub use errors::NftErrorCode;
mod migration;
mod sanitization;
mod storage;
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

    fn require_authorized_caller(env: &Env, caller: &Address) {
        if Storage::has_authorized_contracts(env) {
            caller.require_auth();
            if !Storage::is_authorized_contract(env, caller) {
                panic_with_error!(env, crate::errors::NftErrorCode::Unauthorized);
            }
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
    /// - "extensions": Map<String, String> (optional, arbitrary key-value metadata)
    pub fn mint_reward_nft_from_map(
        env: Env,
        minter: Address,
        hunt_id: u64,
        player_address: Address,
        metadata: Map<Symbol, Val>,
    ) -> u64 {
        Self::require_authorized_caller(&env, &minter);
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

        if !image_uri_is_valid(&image_uri) {
            panic!("Invalid NFT image_uri: must be non-empty");
        }

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

        // Parse extensions from metadata map
        let extensions = metadata
            .get(Symbol::new(&env, "extensions"))
            .and_then(|v| Map::<String, String>::try_from_val(&env, &v).ok())
            .unwrap_or_else(|| Map::new(&env));

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
        Self::mint_reward_nft_impl(env, hunt_id, player_address, meta, transferable)
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
        env: &Env,
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

        let nft_data = NftData {
            nft_id,
            hunt_id,
            owner: player_address.clone(),
            completion_player: player_address.clone(),
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
            metadata: metadata.clone(),
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

        let mut nft = Storage::get_nft(&env, nft_id)
            .ok_or(crate::errors::NftErrorCode::NftNotFound)?;

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

        let mut nft = Storage::get_nft(&env, nft_id)
            .ok_or(crate::errors::NftErrorCode::NftNotFound)?;

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
                if let Some(new_uri) = Self::replace_prefix(
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

    fn replace_prefix(
        env: &Env,
        uri: &String,
        old_prefix: &String,
        new_prefix: &String,
    ) -> Option<String> {
        let uri_len = uri.len() as usize;
        let old_len = old_prefix.len() as usize;
        let new_len = new_prefix.len() as usize;

        if uri_len < old_len {
            return None;
        }

        let mut buf_uri = [0u8; 256];
        let mut buf_old = [0u8; 256];
        let mut buf_new = [0u8; 256];

        uri.copy_into_slice(&mut buf_uri[..uri_len.min(256)]);
        old_prefix.copy_into_slice(&mut buf_old[..old_len.min(256)]);
        new_prefix.copy_into_slice(&mut buf_new[..new_len.min(256)]);

        if buf_uri[..old_len] == buf_old[..old_len] {
            let mut final_buf = [0u8; 512];
            final_buf[..new_len].copy_from_slice(&buf_new[..new_len]);
            let suffix_len = uri_len - old_len;
            final_buf[new_len..new_len + suffix_len].copy_from_slice(&buf_uri[old_len..uri_len]);
            let total_len = new_len + suffix_len;
            if let Ok(text) = core::str::from_utf8(&final_buf[..total_len]) {
                return Some(String::from_str(env, text));
            }
        }
        None
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

        let new_description = Self::sanitize_metadata_field(
            &env,
            &new_description,
            MAX_NFT_DESCRIPTION_BYTES,
            true,
        );
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

    /// Returns the total count of NFTs currently in the contract.
    /// Equivalent to total_supply() but with a dedicated function name for clarity.
    pub fn get_total_nft_count(env: Env) -> u64 {
        Storage::get_nft_counter(&env)
    }

    /// Lists all NFTs minted by the contract with pagination support.
    ///
    /// Returns a vector of NftData structs, paginated by offset and limit.
    /// The limit is bounded to MAX_SCAN_LIMIT (1000) to prevent excessive gas consumption.
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
}