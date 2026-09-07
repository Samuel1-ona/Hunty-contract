use soroban_sdk::contracterror;

// NAMESPACE: reward-manager error codes occupy the range 2001–2999.
// This mirror in reward-interface must stay in sync with
// contracts/reward-manager/src/errors.rs so XDR discriminants match exactly.
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
    PoolAlreadyExists = 2008,
    PoolNotFound = 2009,
    Unauthorized = 2010,
    BelowMinimumAmount = 2011,
    AlreadyInitialized = 2012,
    HuntNotFound = 2013,
    ReentrancyDetected = 2014,
    PoolBalanceDivergence = 2015,
    ReplayDetected = 2016,
    PoolBalanceOverflow = 2017,
    BelowMinimumFunding = 2018,
    ExceedsMaximumFunding = 2019,
    DailyCapExceeded = 2020,
    GlobalDailyCapExceeded = 2021,
    ContractPaused = 2022,
    NftMintPendingNotFound = 2023,
    DistributionNotFound = 2024,
    SourcePoolNotEligible = 2025,
    DestinationPoolNotFound = 2026,
    InvalidMigration = 2027,
    PoolFrozen = 2028,
    DistributionRateLimited = 2029,
    BatchTooLarge = 2030,
    InvalidScore = 2031,
    InvalidTokenContract = 2032,
    VestingNotStarted = 2033,
    VestingAlreadyClaimed = 2034,
    NothingToVest = 2035,
    VestingNotConfigured = 2036,
    FundingPaused = 2037,
    DistributionPaused = 2038,
    TooManyFunders = 2039,
    InvalidHuntStatus = 2040,
}
