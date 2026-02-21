use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum NftErrorCode {
    NftNotFound = 1,
    Unauthorized = 2,
    NotOwner = 3,
    InvalidRecipient = 4,
}
