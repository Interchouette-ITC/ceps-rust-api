//! MCP server (`mcpkit`) for `ceps-rust-api-mcp` (stdio or Streamable HTTP).

#![allow(clippy::unused_async)]

use mcpkit::prelude::*;
use mcpkit::transport::stdio::StdioTransport;
use mcpkit_axum::McpRouter;

use crate::{client, ops};

/// MCP server handle exposing ceps-rust-api Make/Docker + HTTP API tools.
pub struct CepsApiMcp;

// Keep in sync with Cargo.toml `version`.
#[mcp_server(name = "ceps-rust-api", version = "1.0.0")]
impl CepsApiMcp {
    #[tool(description = "make help + MCP Make↔tool parity map")]
    async fn ceps_api_help(&self) -> ToolOutput {
        ToolOutput::text(ops::help())
    }

    #[tool(description = "make build [FEATURES=…]")]
    async fn ceps_api_build(&self, features: Option<String>) -> ToolOutput {
        ToolOutput::text(ops::build(features.as_deref()))
    }

    #[tool(description = "make build-release [FEATURES=…]")]
    async fn ceps_api_build_release(&self, features: Option<String>) -> ToolOutput {
        ToolOutput::text(ops::build_release(features.as_deref()))
    }

    #[tool(description = "make check [FEATURES=…]")]
    async fn ceps_api_check(&self, features: Option<String>) -> ToolOutput {
        ToolOutput::text(ops::check(features.as_deref()))
    }

    #[tool(description = "make lint [FEATURES=…] — fmt check + clippy")]
    async fn ceps_api_lint(&self, features: Option<String>) -> ToolOutput {
        ToolOutput::text(ops::lint(features.as_deref()))
    }

    #[tool(description = "make test [FEATURES=…]")]
    async fn ceps_api_test(&self, features: Option<String>) -> ToolOutput {
        ToolOutput::text(ops::test_suite(features.as_deref()))
    }

    #[tool(description = "make verify [FEATURES=…] — lint + tests")]
    async fn ceps_api_verify(&self, features: Option<String>) -> ToolOutput {
        ToolOutput::text(ops::verify(features.as_deref()))
    }

    #[tool(description = "make docker-build — API image tags")]
    async fn ceps_api_docker_build(&self) -> ToolOutput {
        ToolOutput::text(ops::docker_build())
    }

    #[tool(description = "make docker-run — compose up -d (port 8080)")]
    async fn ceps_api_docker_run(&self) -> ToolOutput {
        ToolOutput::text(ops::docker_run())
    }

    #[tool(description = "make docker-run-kms — compose with kms profile")]
    async fn ceps_api_docker_run_kms(&self) -> ToolOutput {
        ToolOutput::text(ops::docker_run_kms())
    }

    #[tool(description = "make docker-stop")]
    async fn ceps_api_docker_stop(&self) -> ToolOutput {
        ToolOutput::text(ops::docker_stop())
    }

    #[tool(description = "make version-show")]
    async fn ceps_api_version_show(&self) -> ToolOutput {
        ToolOutput::text(ops::version_show())
    }

    #[tool(description = "Host cargo API start (background)")]
    async fn ceps_api_start(&self) -> ToolOutput {
        ToolOutput::text(ops::api_start())
    }

    #[tool(description = "Stop host cargo API")]
    async fn ceps_api_stop(&self) -> ToolOutput {
        ToolOutput::text(ops::api_stop())
    }

    #[tool(description = "Status: containers + API /health probe")]
    async fn ceps_api_status(&self) -> ToolOutput {
        ToolOutput::text(ops::status())
    }

    #[tool(description = "GET / — hello")]
    async fn ceps_api_hello(&self) -> ToolOutput {
        ToolOutput::text(client::hello().await)
    }

    #[tool(description = "GET /health")]
    async fn ceps_api_health(&self) -> ToolOutput {
        ToolOutput::text(client::health().await)
    }

    #[tool(description = "GET /docs/ceps-openapi.json")]
    async fn ceps_api_openapi(&self) -> ToolOutput {
        ToolOutput::text(client::openapi().await)
    }

    #[tool(description = "POST CEP-18 install — JSON body")]
    async fn ceps_api_cep18_install(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep18_install(&body).await)
    }

    #[tool(description = "POST CEP-18 upgrade — JSON body")]
    async fn ceps_api_cep18_upgrade(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep18_upgrade(&body).await)
    }

    #[tool(description = "POST CEP-18 transfer — JSON body")]
    async fn ceps_api_cep18_transfer(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep18_transfer(&body).await)
    }

    #[tool(description = "POST CEP-18 transfer-from — JSON body")]
    async fn ceps_api_cep18_transfer_from(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep18_transfer_from(&body).await)
    }

    #[tool(description = "POST CEP-18 approve — JSON body")]
    async fn ceps_api_cep18_approve(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep18_approve(&body).await)
    }

    #[tool(description = "POST CEP-18 increase-allowance — JSON body")]
    async fn ceps_api_cep18_increase_allowance(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep18_increase_allowance(&body).await)
    }

    #[tool(description = "POST CEP-18 decrease-allowance — JSON body")]
    async fn ceps_api_cep18_decrease_allowance(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep18_decrease_allowance(&body).await)
    }

    #[tool(description = "POST CEP-18 mint — JSON body")]
    async fn ceps_api_cep18_mint(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep18_mint(&body).await)
    }

    #[tool(description = "POST CEP-18 burn — JSON body")]
    async fn ceps_api_cep18_burn(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep18_burn(&body).await)
    }

    #[tool(description = "POST CEP-18 change-events-mode — JSON body")]
    async fn ceps_api_cep18_change_events_mode(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep18_change_events_mode(&body).await)
    }

    #[tool(description = "POST CEP-18 change-security — JSON body")]
    async fn ceps_api_cep18_change_security(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep18_change_security(&body).await)
    }

    #[tool(description = "POST CEP-78 install — JSON body")]
    async fn ceps_api_cep78_install(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_install(&body).await)
    }

    #[tool(description = "POST CEP-78 upgrade — JSON body")]
    async fn ceps_api_cep78_upgrade(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_upgrade(&body).await)
    }

    #[tool(description = "POST CEP-78 mint — JSON body")]
    async fn ceps_api_cep78_mint(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_mint(&body).await)
    }

    #[tool(description = "POST CEP-78 transfer — JSON body")]
    async fn ceps_api_cep78_transfer(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_transfer(&body).await)
    }

    #[tool(description = "POST CEP-78 burn — JSON body")]
    async fn ceps_api_cep78_burn(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_burn(&body).await)
    }

    #[tool(description = "POST CEP-78 register-owner — JSON body")]
    async fn ceps_api_cep78_register_owner(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_register_owner(&body).await)
    }

    #[tool(description = "POST CEP-78 approve — JSON body")]
    async fn ceps_api_cep78_approve(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_approve(&body).await)
    }

    #[tool(description = "POST CEP-78 revoke — JSON body")]
    async fn ceps_api_cep78_revoke(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_revoke(&body).await)
    }

    #[tool(description = "POST CEP-78 set-approval-for-all — JSON body")]
    async fn ceps_api_cep78_set_approval_for_all(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_set_approval_for_all(&body).await)
    }

    #[tool(description = "POST CEP-78 set-token-metadata — JSON body")]
    async fn ceps_api_cep78_set_token_metadata(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_set_token_metadata(&body).await)
    }

    #[tool(description = "POST CEP-78 set-variables — JSON body")]
    async fn ceps_api_cep78_set_variables(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_set_variables(&body).await)
    }

    #[tool(description = "POST CEP-78 mint-session — JSON body")]
    async fn ceps_api_cep78_mint_session(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_mint_session(&body).await)
    }

    #[tool(description = "POST CEP-78 transfer-session — JSON body")]
    async fn ceps_api_cep78_transfer_session(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_transfer_session(&body).await)
    }

    #[tool(description = "POST CEP-78 updated-receipts — JSON body")]
    async fn ceps_api_cep78_updated_receipts(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep78_updated_receipts(&body).await)
    }

    #[tool(description = "POST CEP-85 install — JSON body")]
    async fn ceps_api_cep85_install(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_install(&body).await)
    }

    #[tool(description = "POST CEP-85 upgrade — JSON body")]
    async fn ceps_api_cep85_upgrade(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_upgrade(&body).await)
    }

    #[tool(description = "POST CEP-85 mint — JSON body")]
    async fn ceps_api_cep85_mint(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_mint(&body).await)
    }

    #[tool(description = "POST CEP-85 batch-mint — JSON body")]
    async fn ceps_api_cep85_batch_mint(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_batch_mint(&body).await)
    }

    #[tool(description = "POST CEP-85 transfer — JSON body")]
    async fn ceps_api_cep85_transfer(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_transfer(&body).await)
    }

    #[tool(description = "POST CEP-85 batch-transfer — JSON body")]
    async fn ceps_api_cep85_batch_transfer(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_batch_transfer(&body).await)
    }

    #[tool(description = "POST CEP-85 burn — JSON body")]
    async fn ceps_api_cep85_burn(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_burn(&body).await)
    }

    #[tool(description = "POST CEP-85 batch-burn — JSON body")]
    async fn ceps_api_cep85_batch_burn(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_batch_burn(&body).await)
    }

    #[tool(description = "POST CEP-85 set-approval-for-all — JSON body")]
    async fn ceps_api_cep85_set_approval_for_all(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_set_approval_for_all(&body).await)
    }

    #[tool(description = "POST CEP-85 set-uri — JSON body")]
    async fn ceps_api_cep85_set_uri(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_set_uri(&body).await)
    }

    #[tool(description = "POST CEP-85 set-total-supply-of — JSON body")]
    async fn ceps_api_cep85_set_total_supply_of(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_set_total_supply_of(&body).await)
    }

    #[tool(description = "POST CEP-85 set-total-supply-of-batch — JSON body")]
    async fn ceps_api_cep85_set_total_supply_of_batch(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_set_total_supply_of_batch(&body).await)
    }

    #[tool(description = "POST CEP-85 change-security — JSON body")]
    async fn ceps_api_cep85_change_security(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_change_security(&body).await)
    }

    #[tool(description = "POST CEP-85 set-modalities — JSON body")]
    async fn ceps_api_cep85_set_modalities(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep85_set_modalities(&body).await)
    }

    #[tool(description = "POST CEP-95 install — JSON body")]
    async fn ceps_api_cep95_install(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep95_install(&body).await)
    }

    #[tool(description = "POST CEP-95 transfer-from — JSON body")]
    async fn ceps_api_cep95_transfer_from(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep95_transfer_from(&body).await)
    }

    #[tool(description = "POST CEP-95 safe-transfer-from — JSON body")]
    async fn ceps_api_cep95_safe_transfer_from(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep95_safe_transfer_from(&body).await)
    }

    #[tool(description = "POST CEP-95 approve — JSON body")]
    async fn ceps_api_cep95_approve(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep95_approve(&body).await)
    }

    #[tool(description = "POST CEP-95 revoke-approval — JSON body")]
    async fn ceps_api_cep95_revoke_approval(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep95_revoke_approval(&body).await)
    }

    #[tool(description = "POST CEP-95 approve-for-all — JSON body")]
    async fn ceps_api_cep95_approve_for_all(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep95_approve_for_all(&body).await)
    }

    #[tool(description = "POST CEP-95 revoke-approval-for-all — JSON body")]
    async fn ceps_api_cep95_revoke_approval_for_all(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep95_revoke_approval_for_all(&body).await)
    }

    #[tool(description = "POST CEP-95 mint — JSON body")]
    async fn ceps_api_cep95_mint(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep95_mint(&body).await)
    }

    #[tool(description = "POST CEP-95 burn — JSON body")]
    async fn ceps_api_cep95_burn(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep95_burn(&body).await)
    }

    #[tool(description = "POST CEP-95 bind-odra-install — JSON body")]
    async fn ceps_api_cep95_bind_odra_install(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::cep95_bind_odra_install(&body).await)
    }

    #[tool(description = "POST /v1/chain/put-transaction — JSON body")]
    async fn ceps_api_put_transaction(&self, body: String) -> ToolOutput {
        ToolOutput::text(client::put_transaction(&body).await)
    }

    #[tool(description = "GET CEP-18 name")]
    async fn ceps_api_cep18_name(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep18_name(&contract_hash).await)
    }

    #[tool(description = "GET CEP-18 symbol")]
    async fn ceps_api_cep18_symbol(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep18_symbol(&contract_hash).await)
    }

    #[tool(description = "GET CEP-18 decimals")]
    async fn ceps_api_cep18_decimals(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep18_decimals(&contract_hash).await)
    }

    #[tool(description = "GET CEP-18 total-supply")]
    async fn ceps_api_cep18_total_supply(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep18_total_supply(&contract_hash).await)
    }

    #[tool(description = "GET CEP-18 events-mode")]
    async fn ceps_api_cep18_events_mode(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep18_events_mode(&contract_hash).await)
    }

    #[tool(description = "GET CEP-18 is-mint-and-burn-enabled")]
    async fn ceps_api_cep18_is_mint_and_burn_enabled(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep18_is_mint_and_burn_enabled(&contract_hash).await)
    }

    #[tool(description = "GET CEP-18 balance-of")]
    async fn ceps_api_cep18_balance_of(&self, contract_hash: String, owner: String) -> ToolOutput {
        ToolOutput::text(client::cep18_balance_of(&contract_hash, &owner).await)
    }

    #[tool(description = "GET CEP-18 allowances")]
    async fn ceps_api_cep18_allowances(&self, contract_hash: String, owner: String, spender: String) -> ToolOutput {
        ToolOutput::text(client::cep18_allowances(&contract_hash, &owner, &spender).await)
    }

    #[tool(description = "GET CEP-78 collection-name")]
    async fn ceps_api_cep78_collection_name(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep78_collection_name(&contract_hash).await)
    }

    #[tool(description = "GET CEP-78 collection-symbol")]
    async fn ceps_api_cep78_collection_symbol(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep78_collection_symbol(&contract_hash).await)
    }

    #[tool(description = "GET CEP-78 total-token-supply")]
    async fn ceps_api_cep78_total_token_supply(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep78_total_token_supply(&contract_hash).await)
    }

    #[tool(description = "GET CEP-78 number-of-minted-tokens")]
    async fn ceps_api_cep78_number_of_minted_tokens(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep78_number_of_minted_tokens(&contract_hash).await)
    }

    #[tool(description = "GET CEP-78 events-mode")]
    async fn ceps_api_cep78_events_mode(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep78_events_mode(&contract_hash).await)
    }

    #[tool(description = "GET CEP-78 owner-of")]
    async fn ceps_api_cep78_owner_of(&self, contract_hash: String, token: String) -> ToolOutput {
        ToolOutput::text(client::cep78_owner_of(&contract_hash, &token).await)
    }

    #[tool(description = "GET CEP-78 balance-of")]
    async fn ceps_api_cep78_balance_of(&self, contract_hash: String, owner: String) -> ToolOutput {
        ToolOutput::text(client::cep78_balance_of(&contract_hash, &owner).await)
    }

    #[tool(description = "GET CEP-78 is-approved-for-all")]
    async fn ceps_api_cep78_is_approved_for_all(&self, contract_hash: String, owner: String, operator: String) -> ToolOutput {
        ToolOutput::text(client::cep78_is_approved_for_all(&contract_hash, &owner, &operator).await)
    }

    #[tool(description = "GET CEP-78 get-approved")]
    async fn ceps_api_cep78_get_approved(&self, contract_hash: String, token: String) -> ToolOutput {
        ToolOutput::text(client::cep78_get_approved(&contract_hash, &token).await)
    }

    #[tool(description = "GET CEP-78 metadata")]
    async fn ceps_api_cep78_metadata(&self, contract_hash: String, token: String) -> ToolOutput {
        ToolOutput::text(client::cep78_metadata(&contract_hash, &token).await)
    }

    #[tool(description = "GET CEP-85 collection-name")]
    async fn ceps_api_cep85_collection_name(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep85_collection_name(&contract_hash).await)
    }

    #[tool(description = "GET CEP-85 collection-uri")]
    async fn ceps_api_cep85_collection_uri(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep85_collection_uri(&contract_hash).await)
    }

    #[tool(description = "GET CEP-85 balance-of")]
    async fn ceps_api_cep85_balance_of(&self, contract_hash: String, owner: String, id: String) -> ToolOutput {
        ToolOutput::text(client::cep85_balance_of(&contract_hash, &owner, &id).await)
    }

    #[tool(description = "GET CEP-85 supply-of")]
    async fn ceps_api_cep85_supply_of(&self, contract_hash: String, id: String) -> ToolOutput {
        ToolOutput::text(client::cep85_supply_of(&contract_hash, &id).await)
    }

    #[tool(description = "GET CEP-85 total-supply-of")]
    async fn ceps_api_cep85_total_supply_of(&self, contract_hash: String, id: String) -> ToolOutput {
        ToolOutput::text(client::cep85_total_supply_of(&contract_hash, &id).await)
    }

    #[tool(description = "GET CEP-85 uri")]
    async fn ceps_api_cep85_uri(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep85_uri(&contract_hash).await)
    }

    #[tool(description = "GET CEP-85 is-non-fungible")]
    async fn ceps_api_cep85_is_non_fungible(&self, contract_hash: String, id: String) -> ToolOutput {
        ToolOutput::text(client::cep85_is_non_fungible(&contract_hash, &id).await)
    }

    #[tool(description = "GET CEP-85 is-approved-for-all")]
    async fn ceps_api_cep85_is_approved_for_all(&self, contract_hash: String, owner: String, operator: String) -> ToolOutput {
        ToolOutput::text(client::cep85_is_approved_for_all(&contract_hash, &owner, &operator).await)
    }

    #[tool(description = "GET CEP-95 name")]
    async fn ceps_api_cep95_name(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep95_name(&contract_hash).await)
    }

    #[tool(description = "GET CEP-95 symbol")]
    async fn ceps_api_cep95_symbol(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep95_symbol(&contract_hash).await)
    }

    #[tool(description = "GET CEP-95 total-supply")]
    async fn ceps_api_cep95_total_supply(&self, contract_hash: String) -> ToolOutput {
        ToolOutput::text(client::cep95_total_supply(&contract_hash).await)
    }

    #[tool(description = "GET CEP-95 owner-of")]
    async fn ceps_api_cep95_owner_of(&self, contract_hash: String, token_id: String) -> ToolOutput {
        ToolOutput::text(client::cep95_owner_of(&contract_hash, &token_id).await)
    }

    #[tool(description = "GET CEP-95 balance-of")]
    async fn ceps_api_cep95_balance_of(&self, contract_hash: String, owner: String) -> ToolOutput {
        ToolOutput::text(client::cep95_balance_of(&contract_hash, &owner).await)
    }

    #[tool(description = "GET CEP-95 get-approved")]
    async fn ceps_api_cep95_get_approved(&self, contract_hash: String, token_id: String) -> ToolOutput {
        ToolOutput::text(client::cep95_get_approved(&contract_hash, &token_id).await)
    }

    #[tool(description = "GET CEP-95 is-approved-for-all")]
    async fn ceps_api_cep95_is_approved_for_all(&self, contract_hash: String, owner: String, operator: String) -> ToolOutput {
        ToolOutput::text(client::cep95_is_approved_for_all(&contract_hash, &owner, &operator).await)
    }

    #[tool(description = "GET CEP-95 token-metadata")]
    async fn ceps_api_cep95_token_metadata(&self, contract_hash: String, token_id: String) -> ToolOutput {
        ToolOutput::text(client::cep95_token_metadata(&contract_hash, &token_id).await)
    }

}

/// Serves MCP over stdio until the client disconnects.
pub async fn run() -> Result<(), McpError> {
    let transport = StdioTransport::new();
    let server = ServerBuilder::new(CepsApiMcp)
        .with_tools(CepsApiMcp)
        .build();
    server.serve(transport).await
}

/// Default HTTP bind address for Streamable MCP.
pub const DEFAULT_HTTP_LISTEN: &str = crate::paths::DEFAULT_HTTP_LISTEN;

/// Serves MCP over Streamable HTTP until the process is stopped.
pub async fn run_http(addr: &str) -> std::io::Result<()> {
    McpRouter::new(CepsApiMcp).serve(addr).await
}

impl ResourceHandler for CepsApiMcp {
    async fn list_resources(&self, _ctx: &Context<'_>) -> Result<Vec<Resource>, McpError> {
        Ok(Vec::new())
    }

    async fn read_resource(
        &self,
        uri: &str,
        _ctx: &Context<'_>,
    ) -> Result<Vec<ResourceContents>, McpError> {
        Err(McpError::invalid_params(
            "resources/read",
            format!("unknown resource: {uri}"),
        ))
    }
}

impl PromptHandler for CepsApiMcp {
    async fn list_prompts(&self, _ctx: &Context<'_>) -> Result<Vec<Prompt>, McpError> {
        Ok(Vec::new())
    }

    async fn get_prompt(
        &self,
        name: &str,
        _args: Option<serde_json::Map<String, serde_json::Value>>,
        _ctx: &Context<'_>,
    ) -> Result<GetPromptResult, McpError> {
        Err(McpError::invalid_params(
            "prompts/get",
            format!("unknown prompt: {name}"),
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mcp_server_version_matches_crate() {
        let src = include_str!("server.rs");
        let needle = format!(
            r#"#[mcp_server(name = "ceps-rust-api", version = "{}")]"#,
            env!("CARGO_PKG_VERSION")
        );
        assert!(
            src.contains(&needle),
            "bump #[mcp_server(version = …)] to {} when changing Cargo.toml version",
            env!("CARGO_PKG_VERSION")
        );
    }
}
