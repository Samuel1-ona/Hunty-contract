use soroban_sdk::{contracttype, Address, String, Vec};

/// Configuration for distributing rewards across the HuntyCore ↔ RewardManager boundary.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardConfig {
    pub xlm_amount: Option<i128>,
    pub nft_contract: Option<Address>,
    pub nft_title: String,
    pub nft_description: String,
    pub nft_image_uri: String,
    pub nft_hunt_title: String,
    pub nft_rarity: u32,
    pub nft_tier: u32,
    /// The player's finishing position for this hunt, set by hunty-core at
    /// completion time and threaded here so the NFT can record an immutable rank.
    pub completion_rank: u32,
}

impl RewardConfig {
    pub fn has_xlm(&self) -> bool {
        self.xlm_amount.map(|a| a > 0).unwrap_or(false)
    }

    pub fn has_nft(&self) -> bool {
        self.nft_contract.is_some()
    }

    pub fn is_valid(&self) -> bool {
        self.has_xlm() || self.has_nft()
    }
}

/// Mirror of the RewardManager's per-hunt pool configuration. Callers such as
/// HuntyCore use this to deserialize `get_pool_config` cross-contract results.
/// Field names and order must stay in sync with the RewardManager's struct so
/// the XDR encodings match.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardPoolConfig {
    /// Address of the hunt creator who owns this pool.
    pub creator: Address,
    /// Addresses allowed to distribute rewards for this pool.
    pub delegates: Vec<Address>,
    /// Minimum XLM amount per distribution. 0 means no minimum enforced.
    pub min_distribution_amount: i128,
    /// Optional time-based reward tiers. When empty, the per-winner amount
    /// is computed from `xlm_pool / max_winners`.
    pub time_based_tiers: Vec<TimeBasedRewardTier>,
    /// Whether distributions from this pool are temporarily frozen.
    pub frozen: bool,
    /// Token address for the reward pool (e.g., XLM, USDC, or other SAC tokens).
    pub token_address: Address,
    /// Optional NFT contract address for NFT-only or mixed reward pools.
    pub nft_contract: Option<Address>,
    /// Target funding amount for progress tracking (0 = disabled).
    pub target_amount: i128,
    /// Minimum seconds between distributions (0 = disabled).
    pub min_distribution_interval_secs: u64,
    /// Distribution mode (Fixed or Proportional).
    pub distribution_mode: DistributionMode,
    /// Optional vesting period in seconds. When > 0, XLM rewards are not
    /// transferred immediately at distribution time. Instead, a `VestingRecord`
    /// is created and the player must call `claim_vested` to receive tokens
    /// proportionally as time elapses. 0 means vesting is disabled (instant payout).
    pub vesting_period_secs: u64,
    /// Unix timestamp after which claims are no longer allowed (0 = disabled).
    pub claim_deadline: u64,
    /// Creator royalty basis points (0-10000) for NFT secondary market sales.
    pub nft_royalty_bps: u32,
    /// Whether reward NFTs minted from this pool are transferable.
    pub nft_transferable: bool,
}

/// How rewards are calculated from the pool at distribution time.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum DistributionMode {
    /// Fixed amount supplied by the caller.
    Fixed = 0,
    /// Share of the pool based on player score.
    Proportional = 1,
}

/// One tier of a time-based reward schedule configured on a reward pool.
///
/// A tier defines an XLM amount that is granted to a player who completes the
/// hunt within `max_completion_secs` of registering. Tiers must be stored in
/// ascending order by `max_completion_secs` — i.e. a "faster" tier must
/// appear before a "slower" tier. The first tier for which
/// `max_completion_secs >= elapsed` is selected at distribution time; if the
/// elapsed time exceeds every configured tier, the last (slowest) tier's
/// amount is used as a fallback so the player still receives a reward.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeBasedRewardTier {
    /// Inclusive upper bound on elapsed time (completion_time - registration_time)
    /// in seconds. Must be strictly increasing across the tier list.
    pub max_completion_secs: u64,
    /// XLM amount awarded to a player who qualifies for this tier.
    pub xlm_amount: i128,
}

impl TimeBasedRewardTier {
    /// Creates a new tier. Returns `Err(reason)` if `xlm_amount` is not strictly
    /// positive. (Sort-order validation is performed against the full tier
    /// list, not on individual tiers.)
    pub fn new(max_completion_secs: u64, xlm_amount: i128) -> Result<Self, TierError> {
        if xlm_amount <= 0 {
            return Err(TierError::NonPositiveAmount);
        }
        Ok(Self {
            max_completion_secs,
            xlm_amount,
        })
    }
}

/// Reason a tier or tier list failed validation.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TierError {
    /// A tier's `xlm_amount` was zero or negative.
    NonPositiveAmount,
    /// Two adjacent tiers share the same `max_completion_secs` or appear out of
    /// ascending order. Tiers must be strictly increasing in time bound.
    NotStrictlyAscending,
    /// The tier list is empty when at least one tier was required.
    Empty,
}

/// Returns the XLM amount from the first tier whose `max_completion_secs` is
/// greater than or equal to `elapsed_secs`. If the elapsed time exceeds the
/// largest configured tier, the last tier's amount is returned. Returns
/// `None` if `tiers` is empty.
///
/// Tier list MUST be sorted ascending by `max_completion_secs` (a precondition
/// enforced by `tiers_are_strictly_ascending`). Behavior is undefined if the
/// caller passes an unsorted list — do not rely on it.
pub fn resolve_tier_amount(tiers: &Vec<TimeBasedRewardTier>, elapsed_secs: u64) -> Option<i128> {
    let len = tiers.len();
    if len == 0 {
        return None;
    }

    // First-fit scan: first tier with max_completion_secs >= elapsed.
    for i in 0..len {
        let tier = tiers.get(i).unwrap();
        if elapsed_secs <= tier.max_completion_secs {
            return Some(tier.xlm_amount);
        }
    }

    // Otherwise, fall back to the slowest (last) tier so the player still
    // receives a reward rather than nothing.
    Some(tiers.get(len - 1).unwrap().xlm_amount)
}

/// Validates that a tier list is non-empty, has strictly positive amounts,
/// and is sorted in strictly ascending order by `max_completion_secs`.
pub fn tiers_are_strictly_ascending(tiers: &Vec<TimeBasedRewardTier>) -> Result<(), TierError> {
    if tiers.is_empty() {
        return Err(TierError::Empty);
    }

    let mut prev: Option<u64> = None;
    for i in 0..tiers.len() {
        let tier = tiers.get(i).unwrap();
        if tier.xlm_amount <= 0 {
            return Err(TierError::NonPositiveAmount);
        }
        match prev {
            None => prev = Some(tier.max_completion_secs),
            Some(p) => {
                if tier.max_completion_secs <= p {
                    return Err(TierError::NotStrictlyAscending);
                }
                prev = Some(tier.max_completion_secs);
            }
        }
    }

    Ok(())
}
