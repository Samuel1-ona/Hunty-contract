#![no_std]

pub mod errors;
pub mod types;

pub use errors::RewardErrorCode;
pub use types::{
    rank_tiers_are_strictly_ascending, resolve_rank_tier_amount, resolve_tier_amount,
    tiers_are_strictly_ascending, DistributionMode, RankBasedRewardTier, RankRewardTier,
    RewardConfig, RewardPoolConfig, TierError, TimeBasedRewardTier,
};
