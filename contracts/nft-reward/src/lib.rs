#![cfg_attr(not(test), no_std)]
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, BytesN, Env, Map, String, Symbol, Val, Vec,
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
    /// Auto-generated or creator-provided display name: "{HuntTitle} #{CompletionRank}".
    pub display_name: String,
    /// The rank at which the player completed the hunt (1-indexed). 0 if not ranked.
    pub completion_rank: u64,
}

/// NFT data structure stored on-chain.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftData {
    pub nft_id: u64,
    pub hunt_id: u64,
    pub owner: Address,
    /// Player who completed the hunt (preserved after transfers).
    pub completion_player: Address,
    pub metadata: NftMetadata,
    pub transferable: bool,
    pub minted_at: u64,
    /// Completion rank within the hunt (1-indexed). 0 if not tracked.
    pub completion_rank: u64,
}

/// Event emitted when an NFT is minted.
#[contracttype]
#[derive(Clone, Debug)]
pub struct NftMintedEvent {
    pub nft_id: u64,
    pub hunt_id: u64,
    pub owner: Address,
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

/// Event emitted when an NFT is revoked by an admin.
#[contracttype]
#[derive(Clone, Debug)]
pub struct NftRevokedEvent {
    pub nft_id: u64,
    pub admin: Address,
    pub previous_owner: Address,
    pub reason: String,
}

/// Event emitted when the contract is upgraded.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ContractUpgradedEvent {
    pub admin: Address,
    pub old_version: u32,
    pub new_version: u32,
}

mod errors;
mod storage;
use storage::Storage;

#[contract]
pub struct NftReward;

#[contractimpl]
impl NftReward {
    // ============================================================
    //  Admin / Initialization
    // ============================================================

    /// Initializes the contract, setting the admin address and contract version.
    /// Can only be called once; subsequent calls are no-ops if admin is already set.
    ///
    /// # Arguments
    /// * `env`   - The Soroban environment
    /// * `admin` - The address that will have administrative privileges
    pub fn initialize(env: Env, admin: Address) {
        if Storage::get_admin(&env).is_some() {
            // Already initialized; silently return to avoid replay issues.
            return;
        }
        admin.require_auth();
        Storage::set_admin(&env, &admin);
        // Initialize contract version to 1
        Storage::set_version(&env, 1u32);
    }

    /// Returns the current admin address, if set.
    pub fn get_admin(env: Env) -> Option<Address> {
        Storage::get_admin(&env)
    }

    /// Returns the current contract version.
    pub fn get_version(env: Env) -> u32 {
        Storage::get_version(&env)
    }

    // ============================================================
    //  Issue #384: Contract Upgrade Mechanism
    // ============================================================

    /// Upgrades the contract WASM to the provided hash.
    ///
    /// Admin-only. All storage keys are preserved because Soroban's
    /// `update_current_contract_wasm` only replaces the executable, not storage.
    /// The contract version counter is incremented and a `ContractUpgraded` event
    /// is emitted so off-chain observers can detect the upgrade.
    ///
    /// # Arguments
    /// * `env`       - The Soroban environment
    /// * `admin`     - Must be the stored admin address (requires auth)
    /// * `wasm_hash` - SHA-256 hash of the new WASM blob, previously uploaded via
    ///                 `install_contract_code` / `stellar contract install`
    ///
    /// # Errors
    /// * Panics with `Unauthorized` (NftErrorCode::Unauthorized) if caller is not admin
    pub fn upgrade(
        env: Env,
        admin: Address,
        wasm_hash: BytesN<32>,
    ) -> Result<(), crate::errors::NftErrorCode> {
        admin.require_auth();

        // Verify admin
        let stored_admin =
            Storage::get_admin(&env).ok_or(crate::errors::NftErrorCode::Unauthorized)?;
        if admin != stored_admin {
            return Err(crate::errors::NftErrorCode::Unauthorized);
        }

        let old_version = Storage::get_version(&env);
        let new_version = old_version + 1;

        // Perform the upgrade — replaces the contract executable in-place.
        // All existing storage entries (NFTs, ownership index, counter, admin, version)
        // are preserved across the upgrade.
        env.deployer().update_current_contract_wasm(wasm_hash);

        // Persist the incremented version number so it survives into the new WASM.
        Storage::set_version(&env, new_version);

        // Emit upgrade event
        env.events().publish(
            (Symbol::new(&env, "ContractUpgraded"),),
            ContractUpgradedEvent {
                admin,
                old_version,
                new_version,
            },
        );

        Ok(())
    }

    // ============================================================
    //  Issue #388: Admin NFT Revocation
    // ============================================================

    /// Revokes (burns) an NFT as an administrative action.
    ///
    /// Admin-only. The NFT is permanently deleted from storage and removed
    /// from the owner's index. A revocation record (owner + reason) is kept
    /// in persistent storage so the event can be reconstructed off-chain.
    /// Emits a `NftRevoked` event.
    ///
    /// # Arguments
    /// * `env`    - The Soroban environment
    /// * `admin`  - Must be the stored admin address (requires auth)
    /// * `nft_id` - The NFT to revoke
    /// * `reason` - Human-readable explanation (fraud, error, legal request, etc.)
    ///
    /// # Errors
    /// * `Unauthorized` - Caller is not the admin
    /// * `NftNotFound`  - No NFT with the given ID exists
    pub fn admin_revoke_nft(
        env: Env,
        admin: Address,
        nft_id: u64,
        reason: String,
    ) -> Result<(), crate::errors::NftErrorCode> {
        admin.require_auth();

        // Verify admin privileges
        let stored_admin =
            Storage::get_admin(&env).ok_or(crate::errors::NftErrorCode::Unauthorized)?;
        if admin != stored_admin {
            return Err(crate::errors::NftErrorCode::Unauthorized);
        }

        // Fetch the NFT before deletion so we can capture the owner for the event
        let nft =
            Storage::get_nft(&env, nft_id).ok_or(crate::errors::NftErrorCode::NftNotFound)?;

        let previous_owner = nft.owner.clone();

        // Remove NFT from persistent storage and from the owner index
        Storage::remove_nft(&env, nft_id);
        Storage::remove_nft_from_owner(&env, &previous_owner, nft_id);

        // Persist revocation record for auditability
        Storage::save_revocation_record(&env, nft_id, &previous_owner, &reason);

        // Emit NftRevoked event
        env.events().publish(
            (Symbol::new(&env, "NftRevoked"), nft_id),
            NftRevokedEvent {
                nft_id,
                admin,
                previous_owner,
                reason,
            },
        );

        Ok(())
    }

    /// Returns the revocation record for a previously revoked NFT, if any.
    /// Returns `None` if the NFT was never revoked.
    pub fn get_revocation_record(env: Env, nft_id: u64) -> Option<(Address, String)> {
        Storage::get_revocation_record(&env, nft_id)
    }

    // ============================================================
    //  Issue #383: NFT Display Name Generation
    // ============================================================

    /// Generates the display name for an NFT following the
    /// "{HuntTitle} #{CompletionRank}" format.
    ///
    /// This is a pure helper exposed so callers can preview the generated name
    /// without minting. The actual name is stored in `NftData` at mint time.
    ///
    /// # Arguments
    /// * `hunt_title`       - The title of the hunt
    /// * `completion_rank`  - The rank of the player within the hunt (1-indexed)
    pub fn generate_display_name(env: Env, hunt_title: String, completion_rank: u64) -> String {
        Self::build_display_name(&env, &hunt_title, completion_rank)
    }

    /// Internal helper that builds the "{HuntTitle} #{CompletionRank}" string.
    fn build_display_name(env: &Env, hunt_title: &String, completion_rank: u64) -> String {
        // Convert completion_rank to ASCII bytes manually (no std format!)
        let rank_bytes = Self::u64_to_bytes(completion_rank);
        let rank_len = rank_bytes.1;

        // Allocate a buffer large enough for: title + " #" + digits
        // Max title = 100, " #" = 2, max u64 digits = 20  => 122 bytes max
        let title_len = hunt_title.len() as usize;
        let total = title_len + 2 + rank_len; // " #" is 2 chars

        let mut buf = [0u8; 200];
        if total > 200 {
            // Fallback: return hunt_title unchanged (safety guard)
            return hunt_title.clone();
        }

        hunt_title.copy_into_slice(&mut buf[..title_len]);
        buf[title_len] = b' ';
        buf[title_len + 1] = b'#';
        buf[title_len + 2..title_len + 2 + rank_len].copy_from_slice(&rank_bytes.0[..rank_len]);

        String::from_bytes(env, &buf[..total])
    }

    /// Converts a u64 to its ASCII decimal bytes. Returns (buffer, length).
    fn u64_to_bytes(mut n: u64) -> ([u8; 20], usize) {
        if n == 0 {
            let mut buf = [0u8; 20];
            buf[0] = b'0';
            return (buf, 1);
        }
        let mut tmp = [0u8; 20];
        let mut len = 0usize;
        while n > 0 {
            tmp[len] = b'0' + (n % 10) as u8;
            n /= 10;
            len += 1;
        }
        // tmp is in reverse order; reverse in place
        let mut i = 0;
        let mut j = len - 1;
        while i < j {
            tmp.swap(i, j);
            i += 1;
            j -= 1;
        }
        (tmp, len)
    }

    // ============================================================
    //  Core NFT Minting
    // ============================================================

    /// Mints a unique NFT as a reward for hunt completion.
    ///
    /// If the provided `metadata.title` is empty, an auto-generated display name
    /// of the form "{HuntTitle} #{CompletionRank}" is used as the title.
    /// The same display name is always stored in `NftData.metadata.title` for
    /// easy rendering, and is also included in `NftMetadataResponse.display_name`.
    ///
    /// # Arguments
    /// * `env`              - The Soroban environment
    /// * `hunt_id`          - The hunt this NFT commemorates
    /// * `player_address`   - The address of the player completing the hunt (initial owner)
    /// * `metadata`         - NFT metadata (title, description, image URI, hunt_title, rarity, tier)
    /// * `completion_rank`  - Completion rank of the player within the hunt (1-indexed, 0 = unknown)
    ///
    /// # Returns
    /// The unique NFT ID of the minted NFT
    pub fn mint_reward_nft(
        env: Env,
        hunt_id: u64,
        player_address: Address,
        metadata: NftMetadata,
        completion_rank: u64,
    ) -> u64 {
        if metadata.rarity > 5 {
            panic!("InvalidRarity");
        }
        let minted_at = env.ledger().timestamp();

        // Issue #383: Auto-generate title if none provided
        let final_title = if metadata.title.len() == 0 {
            Self::build_display_name(&env, &metadata.hunt_title, completion_rank)
        } else {
            metadata.title.clone()
        };

        // Build the display name regardless (stored separately for easy access)
        let display_name =
            Self::build_display_name(&env, &metadata.hunt_title, completion_rank);

        let final_metadata = NftMetadata {
            title: final_title,
            description: metadata.description.clone(),
            image_uri: metadata.image_uri.clone(),
            hunt_title: metadata.hunt_title.clone(),
            rarity: metadata.rarity,
            tier: metadata.tier,
        };

        let nft_id = Storage::next_nft_id(&env);

        let nft_data = NftData {
            nft_id,
            hunt_id,
            owner: player_address.clone(),
            completion_player: player_address.clone(),
            metadata: final_metadata.clone(),
            transferable: false,
            minted_at,
            completion_rank,
        };

        Storage::save_nft(&env, &nft_data);
        Storage::add_nft_to_owner(&env, &player_address, nft_id);
        // Store the display name for fast retrieval
        Storage::save_display_name(&env, nft_id, &display_name);

        let event = NftMintedEvent {
            nft_id,
            hunt_id,
            owner: player_address,
            metadata: final_metadata,
            minted_at,
        };
        env.events()
            .publish((Symbol::new(&env, "NftMinted"), nft_id), event);

        nft_id
    }

    /// Mints a reward NFT from a generic metadata map. This is the entrypoint
    /// used by cross-contract callers (e.g. RewardManager) that cannot depend
    /// on this crate's `NftMetadata` type directly.
    ///
    /// Expected keys in `metadata` (all optional, with sensible defaults):
    /// - "title": String  — if absent or empty, auto-generated as "{hunt_title} #{completion_rank}"
    /// - "description": String
    /// - "image_uri": String
    /// - "hunt_title": String (defaults to title when omitted/empty)
    /// - "rarity": u32
    /// - "tier": u32
    /// - "transferable": bool
    /// - "completion_rank": u64  — defaults to 0
    pub fn mint_reward_nft_from_map(
        env: Env,
        hunt_id: u64,
        player_address: Address,
        metadata: Map<Symbol, Val>,
    ) -> u64 {
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

        let transferable = metadata
            .get(Symbol::new(&env, "transferable"))
            .and_then(|v| bool::try_from_val(&env, &v).ok())
            .unwrap_or(false);

        let completion_rank = metadata
            .get(Symbol::new(&env, "completion_rank"))
            .and_then(|v| u64::try_from_val(&env, &v).ok())
            .unwrap_or(0u64);

        // Issue #383: auto-generate title if empty
        let final_title = if title.len() == 0 {
            Self::build_display_name(&env, &hunt_title, completion_rank)
        } else {
            title
        };

        let display_name = Self::build_display_name(&env, &hunt_title, completion_rank);

        let meta = NftMetadata {
            title: final_title,
            description,
            image_uri,
            hunt_title,
            rarity,
            tier,
        };

        let minted_at = env.ledger().timestamp();
        let nft_id = Storage::next_nft_id(&env);

        let nft_data = NftData {
            nft_id,
            hunt_id,
            owner: player_address.clone(),
            completion_player: player_address.clone(),
            metadata: meta.clone(),
            transferable,
            minted_at,
            completion_rank,
        };

        Storage::save_nft(&env, &nft_data);
        Storage::add_nft_to_owner(&env, &player_address, nft_id);
        Storage::save_display_name(&env, nft_id, &display_name);

        let event = NftMintedEvent {
            nft_id,
            hunt_id,
            owner: player_address,
            metadata: meta,
            minted_at,
        };
        env.events()
            .publish((Symbol::new(&env, "NftMinted"), nft_id), event);

        nft_id
    }

    // ============================================================
    //  Query Functions
    // ============================================================

    /// Retrieves NFT data by ID.
    pub fn get_nft(env: Env, nft_id: u64) -> Option<NftData> {
        Storage::get_nft(&env, nft_id)
    }

    /// Returns complete metadata for an NFT, including hunt info, completion details,
    /// and the auto-generated display name (issue #383).
    pub fn get_nft_metadata(env: Env, nft_id: u64) -> Option<NftMetadataResponse> {
        let nft = Storage::get_nft(&env, nft_id)?;

        // Retrieve stored display name or re-generate it on the fly
        let display_name = Storage::get_display_name(&env, nft_id).unwrap_or_else(|| {
            Self::build_display_name(&env, &nft.metadata.hunt_title, nft.completion_rank)
        });

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
            display_name,
            completion_rank: nft.completion_rank,
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

    /// Returns all NFT IDs owned by an address.
    pub fn get_player_nfts(env: Env, owner: Address) -> Vec<u64> {
        Storage::get_owner_nfts(&env, &owner)
    }

    // ============================================================
    //  Transfer & Burn
    // ============================================================

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
        Storage::remove_nft_from_owner(&env, &owner, nft_id);

        env.events()
            .publish((Symbol::new(&env, "NftBurned"), nft_id), (nft_id, owner));

        Ok(())
    }

    /// Transfers an NFT from one address to another.
    ///
    /// # Arguments
    /// * `nft_id`        - The NFT to transfer
    /// * `from_address`  - Current owner (must authorize the call)
    /// * `to_address`    - New owner
    ///
    /// # Authorization
    /// The `from_address` must authorize this call via `require_auth`.
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

        Storage::remove_nft_from_owner(&env, &from_address, nft_id);
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
}

#[cfg(test)]
mod test;
