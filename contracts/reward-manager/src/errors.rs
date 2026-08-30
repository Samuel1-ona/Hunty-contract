use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RewardErrorCode {
    NotInitialized = 1,
    InsufficientPool = 2,
    AlreadyDistributed = 3,

    TransferFailed = 4,
    InvalidAmount = 5,
    InvalidConfig = 6,
    NftMintFailed = 7,

    /// Attempted to create a pool that already exists for this hunt_id.
    PoolAlreadyExists = 8,

    /// Pool has not been created yet via create_reward_pool().
    PoolNotFound = 9,

    /// Caller is not the pool creator and is not authorized to fund this pool.
    Unauthorized = 10,

    /// Distribution amount is below the pool's minimum distribution threshold.
    BelowMinimumAmount = 11,

    /// Contract initialization can only happen once.
    AlreadyInitialized = 12,

    /// hunt_id does not exist in HuntyCore (validated via cross-contract call).
    HuntNotFound = 13,

    /// A recursive distribution attempt was detected during an external XLM or NFT call.
    ReentrancyDetected = 14,

    /// The tracked pool balance diverged from the actual XLM token balance.
    PoolBalanceDivergence = 15,

    /// Replay attack detected: distribution nonce state inconsistency.
    ReplayDetected = 16,

    /// Pool balance would exceed maximum allowed limit.
    PoolBalanceOverflow = 17,

    /// Funding amount is below the minimum threshold (dust attack prevention).
    BelowMinimumFunding = 18,

    /// Single funding amount exceeds the maximum allowed.
    ExceedsMaximumFunding = 19,

    /// Daily distribution cap for a specific pool has been exceeded.
    DailyCapExceeded = 20,

    /// Global daily distribution cap across all pools has been exceeded.
    GlobalDailyCapExceeded = 21,

    /// Contract is paused and cannot perform this operation.
    ContractPaused = 22,

    /// No pending failed NFT mint found for retry.
    NftMintPendingNotFound = 23,

    /// No distribution record exists for the given hunt/player.
    DistributionNotFound = 24,

    /// The source pool is not eligible for migration: its hunt is neither
    /// expired nor cancelled.
    SourcePoolNotEligible = 25,

    /// The destination pool does not exist (must be created first).
    DestinationPoolNotFound = 26,

    /// Source and destination refer to the same hunt, or there is no balance
    /// to migrate.
    InvalidMigration = 27,

    /// Pool is frozen and distributions have been temporarily disabled.
    PoolFrozen = 28,

    /// Distribution rate limit not yet elapsed (cooldown period active).
    DistributionRateLimited = 29,

    /// Batch size exceeds maximum allowed limit.
    BatchTooLarge = 30,

    /// Invalid score value provided.
    InvalidScore = 31,

    /// Token contract validation failed.
    InvalidTokenContract = 32,

    /// No vesting record exists for the given hunt/player pair.
    VestingNotStarted = 33,

    /// Player has already claimed the full vested amount.
    VestingAlreadyClaimed = 34,

    /// Nothing has vested yet (elapsed time is zero or vesting just started).
    NothingToVest = 35,

    /// The pool does not have vesting configured (vesting_period_secs == 0).
    VestingNotConfigured = 36,

    /// Pool funding is paused (issue #628). Distribution may still be running.
    FundingPaused = 37,

    /// Reward distribution is paused (issue #628). Funding may still be open.
    DistributionPaused = 38,

    /// The pool already has the maximum number of distinct tracked funders;
    /// a new sponsor cannot be added until the pool is refunded.
    TooManyFunders = 39,

    /// The hunt is not in a terminal state (cancelled or ended), so its pool
    /// cannot be refunded yet.
    InvalidHuntStatus = 40,
}
