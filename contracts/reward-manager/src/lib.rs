#![cfg_attr(not(test), no_std)]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

pub use crate::errors::RewardErrorCode;
pub use crate::types::{
    DistributionRecord, DistributionStatus, RewardConfig, RewardPoolConfig, RewardPoolStatus,
    ValidationResult,
};
use crate::storage::Storage;
use crate::xlm_handler::XlmHandler;
use crate::nft_handler::NftHandler;

#[contract]
pub struct RewardManager;

/// Event emitted when a reward pool is created for a hunt.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardPoolCreatedEvent {
    pub hunt_id: u64,
    pub creator: Address,
    pub min_distribution_amount: i128,
}

/// Event emitted when a reward pool is funded.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardPoolFundedEvent {
    pub hunt_id: u64,
    pub funder: Address,
    pub amount: i128,
    pub new_balance: i128,
}

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

    /// Creates a reward pool for a specific hunt.
    ///
    /// Must be called before `fund_reward_pool`. Only the creator is authorized
    /// to fund the pool after creation.
    ///
    /// # Arguments
    /// * `creator` - The hunt creator who will own and fund the pool
    /// * `hunt_id` - The hunt this pool is for
    /// * `min_distribution_amount` - Minimum XLM per distribution (0 = no minimum)
    ///
    /// # Errors
    /// * `PoolAlreadyExists` - A pool already exists for this hunt_id
    /// * `InvalidAmount` - min_distribution_amount is negative
    pub fn create_reward_pool(
        env: Env,
        creator: Address,
        hunt_id: u64,
        min_distribution_amount: i128,
    ) -> Result<(), RewardErrorCode> {
        creator.require_auth();

        if min_distribution_amount < 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }

        if Storage::get_pool_config(&env, hunt_id).is_some() {
            return Err(RewardErrorCode::PoolAlreadyExists);
        }

        let config = RewardPoolConfig {
            creator: creator.clone(),
            min_distribution_amount,
        };
        Storage::set_pool_config(&env, hunt_id, &config);

        env.events().publish(
            (Symbol::new(&env, "RewardPoolCreated"), hunt_id),
            RewardPoolCreatedEvent {
                hunt_id,
                creator,
                min_distribution_amount,
            },
        );

        Ok(())
    }

    /// Funds the reward pool for a specific hunt.
    ///
    /// The pool must have been created via `create_reward_pool` first.
    /// Only the original pool creator is authorized to fund it.
    /// Transfers XLM from the funder to this contract and records the balance.
    ///
    /// # Arguments
    /// * `funder` - The address funding the pool (must be the pool creator)
    /// * `hunt_id` - The hunt to fund
    /// * `amount` - XLM amount to add to the pool (must be > 0)
    ///
    /// # Errors
    /// * `PoolNotFound` - Pool has not been created yet
    /// * `Unauthorized` - Funder is not the pool creator
    /// * `InvalidAmount` - Amount is <= 0
    /// * `NotInitialized` - XLM token address not set
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

        let pool_config = Storage::get_pool_config(&env, hunt_id)
            .ok_or(RewardErrorCode::PoolNotFound)?;

        if funder != pool_config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        let xlm_token = Storage::get_xlm_token(&env)
            .ok_or(RewardErrorCode::NotInitialized)?;

        // Transfer XLM from funder to this contract
        let contract_addr = env.current_contract_address();
        let client = soroban_sdk::token::Client::new(&env, &xlm_token);
        client.transfer(&funder, &contract_addr, &amount);

        // Update pool balance and cumulative deposit total
        let current = Storage::get_pool_balance(&env, hunt_id);
        let new_balance = current + amount;
        Storage::set_pool_balance(&env, hunt_id, new_balance);

        let total_deposited = Storage::get_pool_total_deposited(&env, hunt_id) + amount;
        Storage::set_pool_total_deposited(&env, hunt_id, total_deposited);

        env.events().publish(
            (Symbol::new(&env, "RewardPoolFunded"), hunt_id),
            RewardPoolFundedEvent {
                hunt_id,
                funder,
                amount,
                new_balance,
            },
        );

        Ok(())
    }

    /// Returns the full status of a reward pool, including balance, totals, and configuration.
    /// Returns None if no pool has been created for the given hunt_id.
    pub fn get_reward_pool(env: Env, hunt_id: u64) -> Option<RewardPoolStatus> {
        let config = Storage::get_pool_config(&env, hunt_id)?;
        let balance = Storage::get_pool_balance(&env, hunt_id);
        let total_deposited = Storage::get_pool_total_deposited(&env, hunt_id);
        let total_distributed = Storage::get_pool_total_distributed(&env, hunt_id);

        Some(RewardPoolStatus {
            balance,
            total_deposited,
            total_distributed,
            creator: config.creator,
            min_distribution_amount: config.min_distribution_amount,
        })
    }

    /// Validates whether a pool can cover a given distribution amount.
    ///
    /// Checks that:
    /// - The pool exists (was created via create_reward_pool)
    /// - The required_amount is positive
    /// - The pool balance >= required_amount
    /// - The required_amount meets the pool's minimum distribution threshold (if set)
    ///
    /// Returns a `ValidationResult` with balance details regardless of validity,
    /// so callers can diagnose shortfalls without a separate query.
    pub fn validate_pool(env: Env, hunt_id: u64, required_amount: i128) -> ValidationResult {
        let balance = Storage::get_pool_balance(&env, hunt_id);
        let pool_config = Storage::get_pool_config(&env, hunt_id);

        let is_valid = if let Some(ref config) = pool_config {
            let meets_balance = required_amount > 0 && balance >= required_amount;
            let meets_minimum = config.min_distribution_amount == 0
                || required_amount >= config.min_distribution_amount;
            meets_balance && meets_minimum
        } else {
            false
        };

        ValidationResult {
            is_valid,
            balance,
            required: required_amount,
        }
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
    /// * `BelowMinimumAmount` - XLM amount is below the pool's minimum distribution threshold
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

            // Enforce pool minimum distribution amount if a pool config exists
            if let Some(pool_config) = Storage::get_pool_config(&env, hunt_id) {
                if pool_config.min_distribution_amount > 0
                    && amount < pool_config.min_distribution_amount
                {
                    return Err(RewardErrorCode::BelowMinimumAmount);
                }
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

            // Track cumulative distributed amount
            let total_distributed = Storage::get_pool_total_distributed(&env, hunt_id) + amount;
            Storage::set_pool_total_distributed(&env, hunt_id, total_distributed);
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
