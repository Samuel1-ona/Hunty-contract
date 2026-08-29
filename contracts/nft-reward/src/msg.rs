// src/msg.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ... existing imports ...

// ADD THESE TO YOUR EXISTING ExecuteMsg:
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // ... your existing messages ...
    
    // ADD THESE:
    SetNftExtension {
        token_id: String,
        key: String,
        value: String,
    },
    RemoveNftExtension {
        token_id: String,
        key: String,
    },
    ClearNftExtensions {
        token_id: String,
    },
}

// ADD THESE TO YOUR EXISTING QueryMsg:
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // ... your existing queries ...
    
    // ADD THESE:
    GetNftExtension {
        token_id: String,
        key: String,
    },
    GetAllNftExtensions {
        token_id: String,
    },
    GetNftExtensionCount {
        token_id: String,
    },
}

// ADD THESE NEW RESPONSE STRUCTS:
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NftExtensionResponse {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllNftExtensionsResponse {
    pub token_id: String,
    pub extensions: HashMap<String, String>,
    pub count: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NftExtensionCountResponse {
    pub token_id: String,
    pub count: u8,
    pub max_allowed: u8,
}