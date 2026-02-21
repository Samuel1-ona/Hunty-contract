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
}
