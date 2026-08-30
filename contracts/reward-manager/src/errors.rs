use soroban_sdk::contracterror;

// NAMESPACE: reward-manager error codes occupy the range 2001–2999.
//   hunty-core  uses 1001–1999 (see contracts/hunty-core/src/errors.rs).
//   nft-reward  uses 3001–3999 (see contracts/nft-reward/src/errors.rs).
// Keeping ranges disjoint means a numeric code in a transaction envelope is
// unambiguous regardless of which contract frame produced it.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RewardErrorCode {
    NotInitialized = 2001,
    InsufficientPool = 2002,
    AlreadyDistributed = 2003,

    TransferFailed = 2004,
    InvalidAmount = 2005,
    InvalidConfig = 2006,
    NftMintFailed = 2007,

    /// Attempted to create a pool that already exists for this hunt_id.
    PoolAlreadyExists = 2008,

    /// Pool has not been created yet via create_reward_pool().
    PoolNotFound = 2009,

    /// Caller is not the pool creator and is not authorized to fund this pool.
    Unauthorized = 2010,

    /// Distribution amount is below the pool's minimum distribution threshold.
    BelowMinimumAmount = 2011,

    /// Contract initialization can only happen once.
    AlreadyInitialized = 2012,

    /// hunt_id does not exist in HuntyCore (validated via cross-contract call).
    HuntNotFound = 2013,

    /// A recursive distribution attempt was detected during an external XLM or NFT call.
    ReentrancyDetected = 2014,

    /// The tracked pool balance diverged from the actual XLM token balance.
    PoolBalanceDivergence = 2015,

    /// Replay attack detected: distribution nonce state inconsistency.
    ReplayDetected = 2016,

    /// Pool balance would exceed maximum allowed limit.
    PoolBalanceOverflow = 2017,

    /// Funding amount is below the minimum threshold (dust attack prevention).
    BelowMinimumFunding = 2018,

    /// Single funding amount exceeds the maximum allowed.
    ExceedsMaximumFunding = 2019,

    /// Daily distribution cap for a specific pool has been exceeded.
    DailyCapExceeded = 2020,

    /// Global daily distribution cap across all pools has been exceeded.
    GlobalDailyCapExceeded = 2021,

    /// Contract is paused and cannot perform this operation.
    ContractPaused = 2022,

    /// No pending failed NFT mint found for retry.
    NftMintPendingNotFound = 2023,

    /// No distribution record exists for the given hunt/player.
    DistributionNotFound = 2024,

    /// The source pool is not eligible for migration: its hunt is neither
    /// expired nor cancelled.
    SourcePoolNotEligible = 2025,

    /// The destination pool does not exist (must be created first).
    DestinationPoolNotFound = 2026,

    /// Source and destination refer to the same hunt, or there is no balance
    /// to migrate.
    InvalidMigration = 2027,

    /// Pool is frozen and distributions have been temporarily disabled.
    PoolFrozen = 2028,

    /// Distribution rate limit not yet elapsed (cooldown period active).
    DistributionRateLimited = 2029,

    /// Batch size exceeds maximum allowed limit.
    BatchTooLarge = 2030,

    /// Invalid score value provided.
    InvalidScore = 2031,

    /// Token contract validation failed.
    InvalidTokenContract = 2032,

    /// No vesting record exists for the given hunt/player pair.
    VestingNotStarted = 2033,

    /// Player has already claimed the full vested amount.
    VestingAlreadyClaimed = 2034,

    /// Nothing has vested yet (elapsed time is zero or vesting just started).
    NothingToVest = 2035,

    /// The pool does not have vesting configured (vesting_period_secs == 0).
    VestingNotConfigured = 2036,

    /// Pool funding is paused (issue #628). Distribution may still be running.
    FundingPaused = 2037,

    /// Reward distribution is paused (issue #628). Funding may still be open.
    DistributionPaused = 2038,

    /// The pool already has the maximum number of distinct tracked funders;
    /// a new sponsor cannot be added until the pool is refunded.
    TooManyFunders = 2039,

    /// The hunt is not in a terminal state (cancelled or ended), so its pool
    /// cannot be refunded yet.
    InvalidHuntStatus = 2040,
}
