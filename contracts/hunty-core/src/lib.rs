#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Address, String, Symbol};
use crate::types::{Hunt, HuntStatus, RewardConfig, HuntCreatedEvent};
use crate::storage::Storage;
use crate::errors::HuntErrorCode;

#[contract]
pub struct HuntyCore;

#[contractimpl]
impl HuntyCore {
    /// Creates a new scavenger hunt with the provided metadata.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `creator` - The address of the hunt creator (typically use env.invoker() from the caller)
    /// * `title` - The title of the hunt (max 200 characters)
    /// * `description` - The description of the hunt (max 2000 characters)
    /// * `start_time` - Optional start timestamp (0 means no start time restriction)
    /// * `end_time` - Optional end timestamp (0 means no end time restriction)
    /// 
    /// # Returns
    /// The unique hunt ID of the newly created hunt
    /// 
    /// # Errors
    /// * `InvalidTitle` - If title is empty or exceeds maximum length
    /// * `InvalidDescription` - If description exceeds maximum length
    /// * `InvalidAddress` - If creator address is invalid
    pub fn create_hunt(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        _start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> Result<u64, HuntErrorCode> {
        // Validate creator address - in Soroban, Address is always valid if constructed,
        // but we ensure it's not a zero/null address pattern if needed
        // For now, we accept any valid Address type
        
        // Validate title
        let title_len = title.len();
        if title_len == 0 {
            return Err(HuntErrorCode::InvalidTitle);
        }
        const MAX_TITLE_LENGTH: u32 = 200;
        if title_len > MAX_TITLE_LENGTH {
            return Err(HuntErrorCode::InvalidTitle);
        }
        
        // Validate description
        const MAX_DESCRIPTION_LENGTH: u32 = 2000;
        if description.len() > MAX_DESCRIPTION_LENGTH {
            return Err(HuntErrorCode::InvalidDescription);
        }
        
        // Get current timestamp
        let current_time = env.ledger().timestamp();
        
        // Generate unique hunt ID
        let hunt_id = Storage::next_hunt_id(&env);
        
        // Initialize reward config with zero pool
        let reward_config = RewardConfig::new(
            0,              // xlm_pool: zero initially
            false,          // nft_enabled: false initially
            None,           // nft_contract: None initially
            0,              // max_winners: 0 initially
        );
        
        // Create the hunt with Draft status
        let hunt = Hunt {
            hunt_id,
            creator: creator.clone(),
            title: title.clone(),
            description: description.clone(),
            status: HuntStatus::Draft,
            created_at: current_time,
            activated_at: 0,  // Will be set when hunt is activated
            end_time: end_time.unwrap_or(0),
            reward_config,
            total_clues: 0,  // Empty clue list initially
            required_clues: 0,
        };
        
        // Store the hunt
        Storage::save_hunt(&env, &hunt);
        
        // Emit HuntCreated event
        let event = HuntCreatedEvent {
            hunt_id,
            creator: creator.clone(),
            title: title.clone(),
        };
        env.events().publish(
            (Symbol::new(&env, "HuntCreated"), hunt_id),
            event,
        );
        
        Ok(hunt_id)
    }
}

mod types;
mod storage;
mod errors;

#[cfg(test)]
mod test;

