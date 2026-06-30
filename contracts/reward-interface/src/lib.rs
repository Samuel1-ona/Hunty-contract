#![no_std]

pub mod errors;
pub mod types;

pub use errors::RewardErrorCode;
pub use types::{
    resolve_tier_amount, tiers_are_strictly_ascending, RewardConfig, TierError,
    TimeBasedRewardTier,
};
