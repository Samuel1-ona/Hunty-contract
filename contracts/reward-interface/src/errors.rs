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
    PoolAlreadyExists = 8,
    PoolNotFound = 9,
    Unauthorized = 10,
    BelowMinimumAmount = 11,
    AlreadyInitialized = 12,
    HuntNotFound = 13,
    /// A recursive distribution attempt was detected during an external XLM or NFT call.
    ReentrancyDetected = 14,
    /// The tracked pool balance diverged from the actual XLM token balance.
    PoolBalanceDivergence = 15,
}
