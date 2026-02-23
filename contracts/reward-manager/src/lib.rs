#![cfg_attr(not(test), no_std)]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

pub use crate::errors::RewardErrorCode;
pub use crate::types::{DistributionRecord, DistributionStatus, RewardConfig};
use crate::storage::Storage;
use crate::xlm_handler::XlmHandler;
use crate::nft_handler::NftHandler;

#[contract]
pub struct RewardManager;

/// Event emitted when rewards are successfully distributed.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardsDistributedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub xlm_amount: i128,
    pub nft_id: Option<u64>,
}

#[contractimpl]
impl RewardManager {
    /// Initializes the RewardManager with the XLM token contract address (SAC).
    /// Must be called once before any reward distribution.
    pub fn initialize(env: Env, xlm_token: Address) {
        Storage::set_xlm_token(&env, &xlm_token);
    }

    /// Sets the default NftReward contract address used for NFT distributions
    /// when a per-call NFT contract is not provided.
    pub fn set_nft_reward_contract(env: Env, nft_contract: Address) {
        Storage::set_nft_contract(&env, &nft_contract);
    }

    /// Funds the reward pool for a specific hunt.
    /// Transfers XLM from the funder to this contract and records the pool balance.
    ///
    /// # Arguments
    /// * `funder` - The address funding the pool (must authorize)
    /// * `hunt_id` - The hunt to fund
    /// * `amount` - XLM amount to add to the pool
    pub fn fund_reward_pool(
        env: Env,
        funder: Address,
        hunt_id: u64,
        amount: i128,
    ) -> Result<(), RewardErrorCode> {
        funder.require_auth();

        if amount <= 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }

        let xlm_token = Storage::get_xlm_token(&env)
            .ok_or(RewardErrorCode::NotInitialized)?;

        // Transfer XLM from funder to this contract
        let contract_addr = env.current_contract_address();
        let client = soroban_sdk::token::Client::new(&env, &xlm_token);
        client.transfer(&funder, &contract_addr, &amount);

        // Update pool balance
        let current = Storage::get_pool_balance(&env, hunt_id);
        Storage::set_pool_balance(&env, hunt_id, current + amount);

        env.events().publish(
            (Symbol::new(&env, "PoolFunded"), hunt_id),
            (funder, amount),
        );

        Ok(())
    }

    /// Main entry point for reward distribution. Determines reward type from configuration,
    /// routes to XLM and/or NFT handlers, and ensures atomic all-or-nothing execution.
    ///
    /// # Arguments
    /// * `hunt_id` - The hunt being rewarded
    /// * `player_address` - The player receiving rewards
    /// * `reward_config` - Configuration specifying XLM amount and/or NFT metadata
    ///
    /// # Returns
    /// `Ok(())` on success
    ///
    /// # Errors
    /// * `InvalidConfig` - No reward type configured or invalid values
    /// * `NotInitialized` - XLM token not set (when XLM rewards requested)
    /// * `AlreadyDistributed` - Rewards already distributed for this hunt/player
    /// * `InsufficientPool` - Pool has insufficient XLM for requested amount
    /// * `InvalidAmount` - XLM amount <= 0 (when XLM requested)
    /// * `NftMintFailed` - NFT minting failed (when NFT requested)
    pub fn distribute_rewards(
        env: Env,
        hunt_id: u64,
        player_address: Address,
        reward_config: RewardConfig,
    ) -> Result<(), RewardErrorCode> {
        // Validate configuration
        if !reward_config.is_valid() {
            return Err(RewardErrorCode::InvalidConfig);
        }

        // Prevent double distribution
        if Storage::is_distributed(&env, hunt_id, &player_address) {
            return Err(RewardErrorCode::AlreadyDistributed);
        }

        let mut xlm_amount = 0i128;
        let mut nft_id: Option<u64> = None;

        // Route to XLM handler if configured
        if reward_config.has_xlm() {
            let amount = reward_config.xlm_amount.unwrap();
            if amount <= 0 {
                return Err(RewardErrorCode::InvalidAmount);
            }

            let xlm_token = Storage::get_xlm_token(&env)
                .ok_or(RewardErrorCode::NotInitialized)?;

            let pool_balance = Storage::get_pool_balance(&env, hunt_id);
            if pool_balance < amount {
                return Err(RewardErrorCode::InsufficientPool);
            }

            let contract_addr = env.current_contract_address();
            XlmHandler::distribute_xlm(
                &env,
                &xlm_token,
                &contract_addr,
                &player_address,
                amount,
            );
            xlm_amount = amount;
            Storage::set_pool_balance(&env, hunt_id, pool_balance - amount);
        }

        // Route to NFT handler if configured
        if reward_config.has_nft() {
            let nft_contract = reward_config
                .nft_contract
                .as_ref()
                .cloned()
                .or_else(|| Storage::get_nft_contract(&env))
                .ok_or(RewardErrorCode::InvalidConfig)?;

            nft_id = Some(NftHandler::distribute_nft(
                &env,
                &nft_contract,
                hunt_id,
                &player_address,
                reward_config.nft_title.clone(),
                reward_config.nft_description.clone(),
                reward_config.nft_image_uri.clone(),
                reward_config.nft_hunt_title.clone(),
                reward_config.nft_rarity,
                reward_config.nft_tier,
            ));
        }

        // All operations succeeded — update state atomically
        Storage::set_distributed(&env, hunt_id, &player_address);
        Storage::set_distribution_record(
            &env,
            hunt_id,
            &player_address,
            &DistributionRecord {
                xlm_amount,
                nft_id,
            },
        );

        // Emit RewardsDistributed event
        let event = RewardsDistributedEvent {
            hunt_id,
            player: player_address.clone(),
            xlm_amount,
            nft_id,
        };
        env.events()
            .publish((Symbol::new(&env, "RewardsDistributed"), hunt_id), event);

        Ok(())
    }

    /// Legacy entry point for XLM-only or XLM + NFT (placeholder) distribution.
    /// Kept for backward compatibility with HuntyCore. For full config support use distribute_rewards.
    ///
    /// Note: When nft_enabled is true, NFT distribution is NOT performed by this legacy path
    /// (metadata/contract not available). Use distribute_rewards with RewardConfig for NFT support.
    pub fn distribute_rewards_legacy(
        env: Env,
        player: Address,
        hunt_id: u64,
        xlm_amount: i128,
        _nft_enabled: bool,
    ) -> bool {
        let config = RewardConfig {
            xlm_amount: if xlm_amount > 0 {
                Some(xlm_amount)
            } else {
                None
            },
            nft_contract: None,
            nft_title: soroban_sdk::String::from_str(&env, ""),
            nft_description: soroban_sdk::String::from_str(&env, ""),
            nft_image_uri: soroban_sdk::String::from_str(&env, ""),
            nft_hunt_title: soroban_sdk::String::from_str(&env, ""),
            nft_rarity: 0,
            nft_tier: 0,
        };
        Self::distribute_rewards(env, hunt_id, player, config).is_ok()
    }

    /// Returns the distribution status for a hunt/player pair.
    pub fn get_distribution_status(
        env: Env,
        hunt_id: u64,
        player: Address,
    ) -> DistributionStatus {
        let distributed = Storage::is_distributed(&env, hunt_id, &player);
        let record = Storage::get_distribution_record(&env, hunt_id, &player);

        match record {
            Some(r) => DistributionStatus {
                distributed,
                xlm_amount: r.xlm_amount,
                nft_id: r.nft_id,
            },
            None => DistributionStatus {
                distributed,
                xlm_amount: 0,
                nft_id: None,
            },
        }
    }

    /// Returns the current reward pool balance for a hunt.
    pub fn get_pool_balance(env: Env, hunt_id: u64) -> i128 {
        Storage::get_pool_balance(&env, hunt_id)
    }

    /// Returns whether a reward has been distributed to a player for a hunt.
    pub fn is_reward_distributed(env: Env, hunt_id: u64, player: Address) -> bool {
        Storage::is_distributed(&env, hunt_id, &player)
    }
}

pub mod errors;
mod nft_handler;
mod storage;
mod types;
mod xlm_handler;

#[cfg(test)]
mod test;
