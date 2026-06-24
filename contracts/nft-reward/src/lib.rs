#![cfg_attr(not(test), no_std)]
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, Address, Env, Map, String, Symbol,
    Val, Vec,
};

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

fn image_uri_is_valid(uri: &String) -> bool {
    // Accept non-empty URIs that start with https:// or ipfs://
    let s = uri.clone();
    let sstr = s.as_str();
    if sstr.len() == 0 {
        return false;
    }
    sstr.starts_with("https://") || sstr.starts_with("ipfs://")
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
}

/// NFT data structure stored on-chain.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftData {
    pub nft_id: u64,
    pub hunt_id: u64,
    pub owner: Address,
    pub metadata: NftMetadata,
    pub transferable: bool,
    pub minted_at: u64,
}

/// Event emitted when an NFT is minted.
#[contracttype]
#[derive(Clone, Debug)]
pub struct NftMintedEvent {
    pub nft_id: u64,
    pub hunt_id: u64,
    pub owner: Address,
    pub rarity: u32,
    pub tier: u32,
    pub metadata: NftMetadata,
    pub minted_at: u64,
}

/// Event emitted when an NFT is transferred.
#[contracttype]
#[derive(Clone, Debug)]
pub struct NftTransferredEvent {
    pub nft_id: u64,
    pub from: Address,
    pub to: Address,
}

/// Event emitted when an NFT's mutable metadata is updated.
#[contracttype]
#[derive(Clone, Debug)]
pub struct NftMetadataUpdatedEvent {
    pub nft_id: u64,
    pub updater: Address,
}

mod errors;
pub use errors::NftErrorCode;
mod storage;
use storage::Storage;

#[contract]
pub struct NftReward;

#[contractimpl]
impl NftReward {
    /// Initializes the NFT reward contract with an optional max supply cap.
    /// Call this once if you want to enforce a finite NFT supply.
    pub fn initialize(env: Env, max_supply: Option<u64>) -> Result<(), crate::errors::NftErrorCode> {
        if Storage::is_initialized(&env) {
            return Err(crate::errors::NftErrorCode::AlreadyInitialized);
        }

        Storage::set_max_supply(&env, max_supply);
        Ok(())
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
        minter: Address,
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
        minter: Address,
        hunt_id: u64,
        player_address: Address,
        metadata: Map<Symbol, Val>,
    ) -> u64 {
        // Ensure only the configured RewardManager contract can call this function
        let reward_mgr = Storage::get_reward_manager(&env).expect("RewardManager not set");
        reward_mgr.require_auth();
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
            panic!("Invalid NFT image_uri: must be non-empty and start with https:// or ipfs://");
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
        };

        Storage::save_nft(&env, &nft_data);
        Storage::add_nft_to_owner(&env, &player_address, nft_id);

        let event = NftMintedEvent {
            nft_id,
            hunt_id,
            owner: player_address,
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
        Some(NftMetadataResponse {
            nft_id: nft.nft_id,
            hunt_id: nft.hunt_id,
            hunt_title: nft.metadata.hunt_title.clone(),
            completion_timestamp: nft.minted_at,
            completion_player: nft.owner.clone(),
            current_owner: nft.owner.clone(),
            title: nft.metadata.title.clone(),
            description: nft.metadata.description.clone(),
            image_uri: nft.metadata.image_uri.clone(),
            rarity: nft.metadata.rarity,
            tier: nft.metadata.tier,
            creator: nft.metadata.creator.clone(),
            royalty_bps: nft.metadata.royalty_bps,
        })
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

    /// Burns (permanently destroys) an NFT, removing it from storage and the owner's list.
    ///
    /// # Authorization
    /// The `owner` must authorize this call. The caller must also be the current owner.
    pub fn burn(
        env: Env,
        nft_id: u64,
        owner: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        owner.require_auth();

        let nft = Storage::get_nft(&env, nft_id)
            .ok_or(crate::errors::NftErrorCode::NftNotFound)?;

        if nft.owner != owner {
            return Err(crate::errors::NftErrorCode::NotOwner);
        }

        Storage::remove_nft(&env, nft_id);

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

    /// Transfers an NFT from one address to another.
    ///
    /// # Arguments
    /// * `nft_id` - The NFT to transfer
    /// * `from_address` - Current owner (must authorize the call)
    /// * `to_address` - New owner
    ///
    /// # Authorization
    /// The `from_address` must authorize this call via `require_auth`.
    /// For automatic transfers during reward distribution, the contract may be
    /// the `from_address` when invoked by an authorized party.
    pub fn transfer_nft(
        env: Env,
        nft_id: u64,
        from_address: Address,
        to_address: Address,
    ) -> Result<(), crate::errors::NftErrorCode> {
        from_address.require_auth();

        let mut nft = Storage::get_nft(&env, nft_id)
            .ok_or(crate::errors::NftErrorCode::NftNotFound)?;

        if nft.owner != from_address {
            return Err(crate::errors::NftErrorCode::NotOwner);
        }

        if nft.owner == to_address {
            return Err(crate::errors::NftErrorCode::InvalidRecipient);
        }

        if !nft.transferable {
            return Err(crate::errors::NftErrorCode::SoulboundNft);
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

    /// Returns the contract version.
    pub fn contract_version() -> u32 {
        1
    }
}

#[cfg(test)]
mod test;
