use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum NftErrorCode {
    NftNotFound = 1,
    Unauthorized = 2,
    NotOwner = 3,
    InvalidRecipient = 4,
    SoulboundNft = 5,
    InvalidRarity = 6,
    AlreadyInitialized = 7,
    MaxSupplyReached = 8,
    NotInitialized = 9,
    NotOperator = 10,
    NftNotTransferable = 11,
    NftLocked = 12,
    InvalidMetadata = 13,
    InvalidTitle = 14,
    InvalidDescription = 15,
    InvalidImageUri = 16,
    InvalidHuntTitle = 17,
}
