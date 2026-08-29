// src/state.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, Map};
use cosmwasm_std::Addr;
use std::collections::HashMap;

// ... existing state definitions ...

// Update your TokenInfo or NftMetadata struct
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NftMetadata {
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    // ... other existing fields ...
    
    // ADD THIS LINE:
    pub extensions: HashMap<String, String>,
}

// ADD THIS NEW STORAGE:
// Track extension count per NFT (max 10)
pub const NFT_EXTENSION_COUNT: Map<&str, u8> = Map::new("nft_extension_count");

// ... rest of your existing state ...