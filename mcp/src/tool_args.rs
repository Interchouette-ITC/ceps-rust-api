//! JSON-schema parameter structs for rmcp `Parameters<T>` tool handlers.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct FeaturesArgs {
    #[serde(default)]
    pub features: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BodyArgs {
    pub body: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContractHashArgs {
    pub contract_hash: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContractOwnerArgs {
    pub contract_hash: String,
    pub owner: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContractAccountArgs {
    pub contract_hash: String,
    pub account: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContractTokenArgs {
    pub contract_hash: String,
    pub token: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContractOwnerSpenderArgs {
    pub contract_hash: String,
    pub owner: String,
    pub spender: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContractOwnerOperatorArgs {
    pub contract_hash: String,
    pub owner: String,
    pub operator: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContractEntityArgs {
    pub contract_hash: String,
    pub entity: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContractOwnerIdArgs {
    pub contract_hash: String,
    pub owner: String,
    pub id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContractIdArgs {
    pub contract_hash: String,
    pub id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContractTokenIdArgs {
    pub contract_hash: String,
    pub token_id: String,
}
