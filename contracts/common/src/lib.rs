#![no_std]

// The crate is `no_std`, but its unit tests use `std` collections to decode
// recorded events. Link `std` for test builds only; the wasm artifact never
// includes it.
#[cfg(test)]
extern crate std;

pub mod audit;
pub mod audit_emitter;
pub mod monitoring;

#[cfg(test)]
mod test_audit;
pub mod uri;
