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
    NftNotFound = 3001,
    Unauthorized = 3002,
    NotOwner = 3003,
    InvalidRecipient = 3004,
    SoulboundNft = 3005,
    InvalidRarity = 3006,
    AlreadyInitialized = 3007,
    MaxSupplyReached = 3008,
    NotInitialized = 3009,
    NotOperator = 3010,
    NftNotTransferable = 3011,
    NftLocked = 3012,
    InvalidMetadata = 3013,
    MetadataFrozen = 3014,
    TooManyExtensions = 3015,
    InvalidExtensionKey = 3016,
    InvalidExtensionValue = 3017,
    ExtensionNotFound = 3018,
    InvalidMaxSupply = 3019,
}
