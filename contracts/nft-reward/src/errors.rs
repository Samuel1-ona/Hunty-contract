use soroban_sdk::contracterror;

// NAMESPACE: nft-reward error codes occupy the range 3001–3999.
//   hunty-core      uses 1001–1999 (see contracts/hunty-core/src/errors.rs).
//   reward-manager  uses 2001–2999 (see contracts/reward-manager/src/errors.rs).
// Keeping ranges disjoint means a numeric code in a transaction envelope is
// unambiguous regardless of which contract frame produced it.
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
    MetadataFrozen = 14,
    TooManyExtensions = 15,
    InvalidExtensionKey = 16,
    InvalidExtensionValue = 17,
    ExtensionNotFound = 18,
    InvalidMaxSupply = 19,
    InvalidRoyalty = 20,
}
