#![cfg_attr(not(test), no_std)]
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, String, Symbol,
};

/// Metadata for an NFT (title, description, image URI).
/// Supports off-chain storage references to keep gas costs low.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftMetadata {
    pub title: String,
    pub description: String,
    pub image_uri: String,
}

/// NFT data structure stored on-chain.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NftData {
    pub nft_id: u64,
    pub hunt_id: u64,
    pub owner: Address,
    pub metadata: NftMetadata,
    pub minted_at: u64,
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

mod storage;
use storage::Storage;

#[contract]
pub struct NftReward;

#[contractimpl]
impl NftReward {
    /// Mints a unique NFT as a reward for hunt completion.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt this NFT commemorates
    /// * `player_address` - The address of the player completing the hunt (initial owner)
    /// * `metadata` - NFT metadata (title, description, image URI)
    ///
    /// # Returns
    /// The unique NFT ID of the minted NFT
    pub fn mint_reward_nft(
        env: Env,
        hunt_id: u64,
        player_address: Address,
        metadata: NftMetadata,
    ) -> u64 {
        let minted_at = env.ledger().timestamp();

        // Generate unique NFT ID (sequential counter)
        let nft_id = Storage::next_nft_id(&env);

        let nft_data = NftData {
            nft_id,
            hunt_id,
            owner: player_address.clone(),
            metadata: metadata.clone(),
            minted_at,
        };

        // Store NFT in persistent storage
        Storage::save_nft(&env, &nft_data);

        // Update ownership mapping (owner -> list of NFT IDs)
        Storage::add_nft_to_owner(&env, &player_address, nft_id);

        // Emit NftMinted event with all details
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
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `nft_id` - The unique identifier of the NFT
    ///
    /// # Returns
    /// The full NftData struct, or None if the NFT does not exist
    pub fn get_nft(env: Env, nft_id: u64) -> Option<NftData> {
        Storage::get_nft(&env, nft_id)
    }

    /// Returns the total number of NFTs minted so far.
    pub fn total_supply(env: Env) -> u64 {
        Storage::get_nft_counter(&env)
    }

    /// Returns the owner of an NFT.
    pub fn owner_of(env: Env, nft_id: u64) -> Option<Address> {
        Storage::get_nft(&env, nft_id).map(|nft| nft.owner)
    }
}

#[cfg(test)]
mod test;
