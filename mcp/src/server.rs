//! MCP server (`rmcp`) for `ceps-rust-api-mcp` (stdio or Streamable HTTP).

#![allow(clippy::unused_async)]

use std::sync::Arc;

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};

use crate::tool_args::*;
use crate::{client, ops};

/// MCP server handle exposing ceps-rust-api Make/Docker + HTTP API tools.
#[derive(Clone, Default)]
pub struct CepsApiMcp;

/// Default HTTP bind address for Streamable MCP.
pub const DEFAULT_HTTP_LISTEN: &str = crate::paths::DEFAULT_HTTP_LISTEN;

fn text_ok(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(text.into())])
}

#[tool_router]
impl CepsApiMcp {
    #[tool(description = "make help + MCP Make↔tool parity map")]
    async fn ceps_api_help(&self) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::help()))
    }

    #[tool(description = "make build [FEATURES=…]")]
    async fn ceps_api_build(
        &self,
        Parameters(FeaturesArgs { features }): Parameters<FeaturesArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::build(features.as_deref())))
    }

    #[tool(description = "make build-release [FEATURES=…]")]
    async fn ceps_api_build_release(
        &self,
        Parameters(FeaturesArgs { features }): Parameters<FeaturesArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::build_release(features.as_deref())))
    }

    #[tool(description = "make check [FEATURES=…]")]
    async fn ceps_api_check(
        &self,
        Parameters(FeaturesArgs { features }): Parameters<FeaturesArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::check(features.as_deref())))
    }

    #[tool(description = "make lint [FEATURES=…] - fmt check + clippy")]
    async fn ceps_api_lint(
        &self,
        Parameters(FeaturesArgs { features }): Parameters<FeaturesArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::lint(features.as_deref())))
    }

    #[tool(description = "make test [FEATURES=…]")]
    async fn ceps_api_test(
        &self,
        Parameters(FeaturesArgs { features }): Parameters<FeaturesArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::test_suite(features.as_deref())))
    }

    #[tool(description = "make verify [FEATURES=…] - lint + tests")]
    async fn ceps_api_verify(
        &self,
        Parameters(FeaturesArgs { features }): Parameters<FeaturesArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::verify(features.as_deref())))
    }

    #[tool(description = "make docker-build - API image tags")]
    async fn ceps_api_docker_build(&self) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::docker_build()))
    }

    #[tool(description = "make docker-run - compose up -d (port 8080)")]
    async fn ceps_api_docker_run(&self) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::docker_run()))
    }

    #[tool(description = "make docker-run-kms - compose with kms profile")]
    async fn ceps_api_docker_run_kms(&self) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::docker_run_kms()))
    }

    #[tool(description = "make docker-stop")]
    async fn ceps_api_docker_stop(&self) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::docker_stop()))
    }

    #[tool(description = "make version-show")]
    async fn ceps_api_version_show(&self) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::version_show()))
    }

    #[tool(description = "Host cargo API start (background)")]
    async fn ceps_api_start(&self) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::api_start()))
    }

    #[tool(description = "Stop host cargo API")]
    async fn ceps_api_stop(&self) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::api_stop()))
    }

    #[tool(description = "Status: containers + API /health probe")]
    async fn ceps_api_status(&self) -> Result<CallToolResult, McpError> {
        Ok(text_ok(ops::status()))
    }

    #[tool(description = "GET / - hello")]
    async fn ceps_api_hello(&self) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::hello().await))
    }

    #[tool(description = "GET /health")]
    async fn ceps_api_health(&self) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::health().await))
    }

    #[tool(description = "GET /docs/ceps-openapi.json")]
    async fn ceps_api_openapi(&self) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::openapi().await))
    }

    #[tool(description = "POST CEP-18 install - query params as JSON object")]
    async fn ceps_api_cep18_install(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_install(&body).await))
    }

    #[tool(description = "POST CEP-18 upgrade - query params as JSON object")]
    async fn ceps_api_cep18_upgrade(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_upgrade(&body).await))
    }

    #[tool(description = "POST CEP-18 transfer - query params as JSON object")]
    async fn ceps_api_cep18_transfer(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_transfer(&body).await))
    }

    #[tool(description = "POST CEP-18 transfer-from - query params as JSON object")]
    async fn ceps_api_cep18_transfer_from(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_transfer_from(&body).await))
    }

    #[tool(description = "POST CEP-18 approve - query params as JSON object")]
    async fn ceps_api_cep18_approve(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_approve(&body).await))
    }

    #[tool(description = "POST CEP-18 increase-allowance - query params as JSON object")]
    async fn ceps_api_cep18_increase_allowance(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_increase_allowance(&body).await))
    }

    #[tool(description = "POST CEP-18 decrease-allowance - query params as JSON object")]
    async fn ceps_api_cep18_decrease_allowance(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_decrease_allowance(&body).await))
    }

    #[tool(description = "POST CEP-18 mint - query params as JSON object")]
    async fn ceps_api_cep18_mint(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_mint(&body).await))
    }

    #[tool(description = "POST CEP-18 burn - query params as JSON object")]
    async fn ceps_api_cep18_burn(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_burn(&body).await))
    }

    #[tool(description = "POST CEP-18 change-events-mode - query params as JSON object")]
    async fn ceps_api_cep18_change_events_mode(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_change_events_mode(&body).await))
    }

    #[tool(description = "POST CEP-18 change-security - query params as JSON object")]
    async fn ceps_api_cep18_change_security(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_change_security(&body).await))
    }

    #[tool(description = "POST CEP-78 install - query params as JSON object")]
    async fn ceps_api_cep78_install(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_install(&body).await))
    }

    #[tool(description = "POST CEP-78 upgrade - query params as JSON object")]
    async fn ceps_api_cep78_upgrade(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_upgrade(&body).await))
    }

    #[tool(description = "POST CEP-78 mint - query params as JSON object")]
    async fn ceps_api_cep78_mint(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_mint(&body).await))
    }

    #[tool(description = "POST CEP-78 transfer - query params as JSON object")]
    async fn ceps_api_cep78_transfer(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_transfer(&body).await))
    }

    #[tool(description = "POST CEP-78 burn - query params as JSON object")]
    async fn ceps_api_cep78_burn(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_burn(&body).await))
    }

    #[tool(description = "POST CEP-78 register-owner - query params as JSON object")]
    async fn ceps_api_cep78_register_owner(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_register_owner(&body).await))
    }

    #[tool(description = "POST CEP-78 approve - query params as JSON object")]
    async fn ceps_api_cep78_approve(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_approve(&body).await))
    }

    #[tool(description = "POST CEP-78 revoke - query params as JSON object")]
    async fn ceps_api_cep78_revoke(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_revoke(&body).await))
    }

    #[tool(description = "POST CEP-78 set-approval-for-all - query params as JSON object")]
    async fn ceps_api_cep78_set_approval_for_all(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_set_approval_for_all(&body).await))
    }

    #[tool(description = "POST CEP-78 set-token-metadata - query params as JSON object")]
    async fn ceps_api_cep78_set_token_metadata(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_set_token_metadata(&body).await))
    }

    #[tool(description = "POST CEP-78 set-variables - query params as JSON object")]
    async fn ceps_api_cep78_set_variables(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_set_variables(&body).await))
    }

    #[tool(description = "POST CEP-78 mint-session - query params as JSON object")]
    async fn ceps_api_cep78_mint_session(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_mint_session(&body).await))
    }

    #[tool(description = "POST CEP-78 transfer-session - query params as JSON object")]
    async fn ceps_api_cep78_transfer_session(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_transfer_session(&body).await))
    }

    #[tool(description = "POST CEP-78 updated-receipts - query params as JSON object")]
    async fn ceps_api_cep78_updated_receipts(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_updated_receipts(&body).await))
    }

    #[tool(description = "POST CEP-78 owner-of-session - query params as JSON object")]
    async fn ceps_api_cep78_owner_of_session(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_owner_of_session(&body).await))
    }

    #[tool(description = "POST CEP-78 balance-of-session - query params as JSON object")]
    async fn ceps_api_cep78_balance_of_session(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_balance_of_session(&body).await))
    }

    #[tool(description = "POST CEP-78 get-approved-session - query params as JSON object")]
    async fn ceps_api_cep78_get_approved_session(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_get_approved_session(&body).await))
    }

    #[tool(description = "POST CEP-78 is-approved-for-all-session - query params as JSON object")]
    async fn ceps_api_cep78_is_approved_for_all_session(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_is_approved_for_all_session(&body).await,
        ))
    }

    #[tool(description = "POST CEP-85 install - query params as JSON object")]
    async fn ceps_api_cep85_install(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_install(&body).await))
    }

    #[tool(description = "POST CEP-85 upgrade - query params as JSON object")]
    async fn ceps_api_cep85_upgrade(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_upgrade(&body).await))
    }

    #[tool(description = "POST CEP-85 mint - query params as JSON object")]
    async fn ceps_api_cep85_mint(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_mint(&body).await))
    }

    #[tool(description = "POST CEP-85 batch-mint - query params as JSON object")]
    async fn ceps_api_cep85_batch_mint(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_batch_mint(&body).await))
    }

    #[tool(description = "POST CEP-85 transfer - query params as JSON object")]
    async fn ceps_api_cep85_transfer(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_transfer(&body).await))
    }

    #[tool(description = "POST CEP-85 batch-transfer - query params as JSON object")]
    async fn ceps_api_cep85_batch_transfer(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_batch_transfer(&body).await))
    }

    #[tool(description = "POST CEP-85 burn - query params as JSON object")]
    async fn ceps_api_cep85_burn(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_burn(&body).await))
    }

    #[tool(description = "POST CEP-85 batch-burn - query params as JSON object")]
    async fn ceps_api_cep85_batch_burn(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_batch_burn(&body).await))
    }

    #[tool(description = "POST CEP-85 set-approval-for-all - query params as JSON object")]
    async fn ceps_api_cep85_set_approval_for_all(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_set_approval_for_all(&body).await))
    }

    #[tool(description = "POST CEP-85 set-uri - query params as JSON object")]
    async fn ceps_api_cep85_set_uri(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_set_uri(&body).await))
    }

    #[tool(description = "POST CEP-85 set-total-supply-of - query params as JSON object")]
    async fn ceps_api_cep85_set_total_supply_of(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_set_total_supply_of(&body).await))
    }

    #[tool(description = "POST CEP-85 set-total-supply-of-batch - query params as JSON object")]
    async fn ceps_api_cep85_set_total_supply_of_batch(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep85_set_total_supply_of_batch(&body).await,
        ))
    }

    #[tool(description = "POST CEP-85 change-security - query params as JSON object")]
    async fn ceps_api_cep85_change_security(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_change_security(&body).await))
    }

    #[tool(description = "POST CEP-85 set-modalities - query params as JSON object")]
    async fn ceps_api_cep85_set_modalities(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_set_modalities(&body).await))
    }

    #[tool(description = "POST CEP-85 balance-of-batch - query params as JSON object")]
    async fn ceps_api_cep85_balance_of_batch(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_balance_of_batch(&body).await))
    }

    #[tool(description = "POST CEP-85 supply-of-batch - query params as JSON object")]
    async fn ceps_api_cep85_supply_of_batch(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_supply_of_batch(&body).await))
    }

    #[tool(description = "POST CEP-85 total-supply-of-batch - query params as JSON object")]
    async fn ceps_api_cep85_total_supply_of_batch(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_total_supply_of_batch(&body).await))
    }

    #[tool(description = "POST CEP-95 install - query params as JSON object")]
    async fn ceps_api_cep95_install(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_install(&body).await))
    }

    #[tool(description = "POST CEP-95 transfer-from - query params as JSON object")]
    async fn ceps_api_cep95_transfer_from(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_transfer_from(&body).await))
    }

    #[tool(description = "POST CEP-95 safe-transfer-from - query params as JSON object")]
    async fn ceps_api_cep95_safe_transfer_from(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_safe_transfer_from(&body).await))
    }

    #[tool(description = "POST CEP-95 approve - query params as JSON object")]
    async fn ceps_api_cep95_approve(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_approve(&body).await))
    }

    #[tool(description = "POST CEP-95 revoke-approval - query params as JSON object")]
    async fn ceps_api_cep95_revoke_approval(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_revoke_approval(&body).await))
    }

    #[tool(description = "POST CEP-95 approve-for-all - query params as JSON object")]
    async fn ceps_api_cep95_approve_for_all(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_approve_for_all(&body).await))
    }

    #[tool(description = "POST CEP-95 revoke-approval-for-all - query params as JSON object")]
    async fn ceps_api_cep95_revoke_approval_for_all(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_revoke_approval_for_all(&body).await))
    }

    #[tool(description = "POST CEP-95 mint - query params as JSON object")]
    async fn ceps_api_cep95_mint(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_mint(&body).await))
    }

    #[tool(description = "POST CEP-95 burn - query params as JSON object")]
    async fn ceps_api_cep95_burn(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_burn(&body).await))
    }

    #[tool(description = "POST CEP-95 transfer-ownership - query params as JSON object")]
    async fn ceps_api_cep95_transfer_ownership(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_transfer_ownership(&body).await))
    }

    #[tool(description = "POST CEP-95 bind-odra-install - query params as JSON object")]
    async fn ceps_api_cep95_bind_odra_install(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_bind_odra_install(&body).await))
    }

    #[tool(description = "POST /v1/chain/put-transaction - query params as JSON object")]
    async fn ceps_api_put_transaction(
        &self,
        Parameters(BodyArgs { body }): Parameters<BodyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::put_transaction(&body).await))
    }

    #[tool(description = "GET CEP-18 name")]
    async fn ceps_api_cep18_name(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_name(&contract_hash).await))
    }

    #[tool(description = "GET CEP-18 symbol")]
    async fn ceps_api_cep18_symbol(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_symbol(&contract_hash).await))
    }

    #[tool(description = "GET CEP-18 decimals")]
    async fn ceps_api_cep18_decimals(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_decimals(&contract_hash).await))
    }

    #[tool(description = "GET CEP-18 total-supply")]
    async fn ceps_api_cep18_total_supply(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_total_supply(&contract_hash).await))
    }

    #[tool(description = "GET CEP-18 events-mode")]
    async fn ceps_api_cep18_events_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep18_events_mode(&contract_hash).await))
    }

    #[tool(description = "GET CEP-18 is-mint-and-burn-enabled")]
    async fn ceps_api_cep18_is_mint_and_burn_enabled(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep18_is_mint_and_burn_enabled(&contract_hash).await,
        ))
    }

    #[tool(description = "GET CEP-18 balance-of")]
    async fn ceps_api_cep18_balance_of(
        &self,
        Parameters(ContractOwnerArgs {
            contract_hash,
            owner,
        }): Parameters<ContractOwnerArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep18_balance_of(&contract_hash, &owner).await,
        ))
    }

    #[tool(description = "GET CEP-18 allowances")]
    async fn ceps_api_cep18_allowances(
        &self,
        Parameters(ContractOwnerSpenderArgs {
            contract_hash,
            owner,
            spender,
        }): Parameters<ContractOwnerSpenderArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep18_allowances(&contract_hash, &owner, &spender).await,
        ))
    }

    #[tool(description = "GET CEP-18 security-badge")]
    async fn ceps_api_cep18_security_badge(
        &self,
        Parameters(ContractAccountArgs {
            contract_hash,
            account,
        }): Parameters<ContractAccountArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep18_security_badge(&contract_hash, &account).await,
        ))
    }

    #[tool(description = "GET CEP-78 collection-name")]
    async fn ceps_api_cep78_collection_name(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_collection_name(&contract_hash).await))
    }

    #[tool(description = "GET CEP-78 collection-symbol")]
    async fn ceps_api_cep78_collection_symbol(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_collection_symbol(&contract_hash).await,
        ))
    }

    #[tool(description = "GET CEP-78 total-token-supply")]
    async fn ceps_api_cep78_total_token_supply(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_total_token_supply(&contract_hash).await,
        ))
    }

    #[tool(description = "GET CEP-78 number-of-minted-tokens")]
    async fn ceps_api_cep78_number_of_minted_tokens(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_number_of_minted_tokens(&contract_hash).await,
        ))
    }

    #[tool(description = "GET CEP-78 events-mode")]
    async fn ceps_api_cep78_events_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_events_mode(&contract_hash).await))
    }

    #[tool(description = "GET CEP-78 owner-of")]
    async fn ceps_api_cep78_owner_of(
        &self,
        Parameters(ContractTokenArgs {
            contract_hash,
            token,
        }): Parameters<ContractTokenArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_owner_of(&contract_hash, &token).await,
        ))
    }

    #[tool(description = "GET CEP-78 balance-of")]
    async fn ceps_api_cep78_balance_of(
        &self,
        Parameters(ContractOwnerArgs {
            contract_hash,
            owner,
        }): Parameters<ContractOwnerArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_balance_of(&contract_hash, &owner).await,
        ))
    }

    #[tool(description = "GET CEP-78 is-approved-for-all")]
    async fn ceps_api_cep78_is_approved_for_all(
        &self,
        Parameters(ContractOwnerOperatorArgs {
            contract_hash,
            owner,
            operator,
        }): Parameters<ContractOwnerOperatorArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_is_approved_for_all(&contract_hash, &owner, &operator).await,
        ))
    }

    #[tool(description = "GET CEP-78 get-approved")]
    async fn ceps_api_cep78_get_approved(
        &self,
        Parameters(ContractTokenArgs {
            contract_hash,
            token,
        }): Parameters<ContractTokenArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_get_approved(&contract_hash, &token).await,
        ))
    }

    #[tool(description = "GET CEP-78 metadata")]
    async fn ceps_api_cep78_metadata(
        &self,
        Parameters(ContractTokenArgs {
            contract_hash,
            token,
        }): Parameters<ContractTokenArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_metadata(&contract_hash, &token).await,
        ))
    }

    #[tool(description = "GET CEP-78 allow-minting")]
    async fn ceps_api_cep78_allow_minting(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_allow_minting(&contract_hash).await))
    }

    #[tool(description = "GET CEP-78 operator-burn-mode")]
    async fn ceps_api_cep78_operator_burn_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_operator_burn_mode(&contract_hash).await,
        ))
    }

    #[tool(description = "GET CEP-78 package-operator-mode")]
    async fn ceps_api_cep78_package_operator_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_package_operator_mode(&contract_hash).await,
        ))
    }

    #[tool(description = "GET CEP-78 acl-package-mode")]
    async fn ceps_api_cep78_acl_package_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_acl_package_mode(&contract_hash).await,
        ))
    }

    #[tool(description = "GET CEP-78 json-schema")]
    async fn ceps_api_cep78_json_schema(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_json_schema(&contract_hash).await))
    }

    #[tool(description = "GET CEP-78 minting-mode")]
    async fn ceps_api_cep78_minting_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_minting_mode(&contract_hash).await))
    }

    #[tool(description = "GET CEP-78 whitelist-mode")]
    async fn ceps_api_cep78_whitelist_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_whitelist_mode(&contract_hash).await))
    }

    #[tool(description = "GET CEP-78 reporting-mode")]
    async fn ceps_api_cep78_reporting_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_reporting_mode(&contract_hash).await))
    }

    #[tool(description = "GET CEP-78 burn-mode")]
    async fn ceps_api_cep78_burn_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_burn_mode(&contract_hash).await))
    }

    #[tool(description = "GET CEP-78 holder-mode")]
    async fn ceps_api_cep78_holder_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_holder_mode(&contract_hash).await))
    }

    #[tool(description = "GET CEP-78 identifier-mode")]
    async fn ceps_api_cep78_identifier_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_identifier_mode(&contract_hash).await))
    }

    #[tool(description = "GET CEP-78 metadata-mutability")]
    async fn ceps_api_cep78_metadata_mutability(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_metadata_mutability(&contract_hash).await,
        ))
    }

    #[tool(description = "GET CEP-78 nft-kind")]
    async fn ceps_api_cep78_nft_kind(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_nft_kind(&contract_hash).await))
    }

    #[tool(description = "GET CEP-78 nft-metadata-kind")]
    async fn ceps_api_cep78_nft_metadata_kind(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_nft_metadata_kind(&contract_hash).await,
        ))
    }

    #[tool(description = "GET CEP-78 ownership-mode")]
    async fn ceps_api_cep78_ownership_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep78_ownership_mode(&contract_hash).await))
    }

    #[tool(description = "GET CEP-78 is-acl-whitelisted")]
    async fn ceps_api_cep78_is_acl_whitelisted(
        &self,
        Parameters(ContractEntityArgs {
            contract_hash,
            entity,
        }): Parameters<ContractEntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep78_is_acl_whitelisted(&contract_hash, &entity).await,
        ))
    }

    #[tool(description = "GET CEP-85 collection-name")]
    async fn ceps_api_cep85_collection_name(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_collection_name(&contract_hash).await))
    }

    #[tool(description = "GET CEP-85 collection-uri")]
    async fn ceps_api_cep85_collection_uri(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_collection_uri(&contract_hash).await))
    }

    #[tool(description = "GET CEP-85 balance-of")]
    async fn ceps_api_cep85_balance_of(
        &self,
        Parameters(ContractOwnerIdArgs {
            contract_hash,
            owner,
            id,
        }): Parameters<ContractOwnerIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep85_balance_of(&contract_hash, &owner, &id).await,
        ))
    }

    #[tool(description = "GET CEP-85 supply-of")]
    async fn ceps_api_cep85_supply_of(
        &self,
        Parameters(ContractIdArgs { contract_hash, id }): Parameters<ContractIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_supply_of(&contract_hash, &id).await))
    }

    #[tool(description = "GET CEP-85 total-supply-of")]
    async fn ceps_api_cep85_total_supply_of(
        &self,
        Parameters(ContractIdArgs { contract_hash, id }): Parameters<ContractIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep85_total_supply_of(&contract_hash, &id).await,
        ))
    }

    #[tool(description = "GET CEP-85 uri")]
    async fn ceps_api_cep85_uri(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_uri(&contract_hash).await))
    }

    #[tool(description = "GET CEP-85 uri with token id query")]
    async fn ceps_api_cep85_uri_with_id(
        &self,
        Parameters(ContractIdArgs { contract_hash, id }): Parameters<ContractIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep85_uri_with_id(&contract_hash, &id).await,
        ))
    }

    #[tool(description = "GET CEP-85 total-fungible-supply")]
    async fn ceps_api_cep85_total_fungible_supply(
        &self,
        Parameters(ContractIdArgs { contract_hash, id }): Parameters<ContractIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep85_total_fungible_supply(&contract_hash, &id).await,
        ))
    }

    #[tool(description = "GET CEP-85 enable-burn")]
    async fn ceps_api_cep85_enable_burn(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_enable_burn(&contract_hash).await))
    }

    #[tool(description = "GET CEP-85 events-mode")]
    async fn ceps_api_cep85_events_mode(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep85_events_mode(&contract_hash).await))
    }

    #[tool(description = "GET CEP-85 number-of-minted-tokens")]
    async fn ceps_api_cep85_number_of_minted_tokens(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep85_number_of_minted_tokens(&contract_hash).await,
        ))
    }

    #[tool(description = "GET CEP-85 transfer-filter-contract")]
    async fn ceps_api_cep85_transfer_filter_contract(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep85_transfer_filter_contract(&contract_hash).await,
        ))
    }

    #[tool(description = "GET CEP-85 transfer-filter-method")]
    async fn ceps_api_cep85_transfer_filter_method(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep85_transfer_filter_method(&contract_hash).await,
        ))
    }

    #[tool(description = "GET CEP-85 security-badge")]
    async fn ceps_api_cep85_security_badge(
        &self,
        Parameters(ContractEntityArgs {
            contract_hash,
            entity,
        }): Parameters<ContractEntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep85_security_badge(&contract_hash, &entity).await,
        ))
    }

    #[tool(description = "GET CEP-85 is-non-fungible")]
    async fn ceps_api_cep85_is_non_fungible(
        &self,
        Parameters(ContractIdArgs { contract_hash, id }): Parameters<ContractIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep85_is_non_fungible(&contract_hash, &id).await,
        ))
    }

    #[tool(description = "GET CEP-85 is-approved-for-all")]
    async fn ceps_api_cep85_is_approved_for_all(
        &self,
        Parameters(ContractOwnerOperatorArgs {
            contract_hash,
            owner,
            operator,
        }): Parameters<ContractOwnerOperatorArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep85_is_approved_for_all(&contract_hash, &owner, &operator).await,
        ))
    }

    #[tool(description = "GET CEP-95 name")]
    async fn ceps_api_cep95_name(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_name(&contract_hash).await))
    }

    #[tool(description = "GET CEP-95 symbol")]
    async fn ceps_api_cep95_symbol(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_symbol(&contract_hash).await))
    }

    #[tool(description = "GET CEP-95 total-supply")]
    async fn ceps_api_cep95_total_supply(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_total_supply(&contract_hash).await))
    }

    #[tool(description = "GET CEP-95 owner-of")]
    async fn ceps_api_cep95_owner_of(
        &self,
        Parameters(ContractTokenIdArgs {
            contract_hash,
            token_id,
        }): Parameters<ContractTokenIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep95_owner_of(&contract_hash, &token_id).await,
        ))
    }

    #[tool(description = "GET CEP-95 balance-of")]
    async fn ceps_api_cep95_balance_of(
        &self,
        Parameters(ContractOwnerArgs {
            contract_hash,
            owner,
        }): Parameters<ContractOwnerArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep95_balance_of(&contract_hash, &owner).await,
        ))
    }

    #[tool(description = "GET CEP-95 get-approved")]
    async fn ceps_api_cep95_get_approved(
        &self,
        Parameters(ContractTokenIdArgs {
            contract_hash,
            token_id,
        }): Parameters<ContractTokenIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep95_get_approved(&contract_hash, &token_id).await,
        ))
    }

    #[tool(description = "GET CEP-95 is-approved-for-all")]
    async fn ceps_api_cep95_is_approved_for_all(
        &self,
        Parameters(ContractOwnerOperatorArgs {
            contract_hash,
            owner,
            operator,
        }): Parameters<ContractOwnerOperatorArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep95_is_approved_for_all(&contract_hash, &owner, &operator).await,
        ))
    }

    #[tool(description = "GET CEP-95 token-metadata")]
    async fn ceps_api_cep95_token_metadata(
        &self,
        Parameters(ContractTokenIdArgs {
            contract_hash,
            token_id,
        }): Parameters<ContractTokenIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(
            client::cep95_token_metadata(&contract_hash, &token_id).await,
        ))
    }

    #[tool(description = "GET CEP-95 get-owner")]
    async fn ceps_api_cep95_get_owner(
        &self,
        Parameters(ContractHashArgs { contract_hash }): Parameters<ContractHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(text_ok(client::cep95_get_owner(&contract_hash).await))
    }
}

/// Serves MCP over stdio until the client disconnects.
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = CepsApiMcp;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

/// Serves MCP over Streamable HTTP until the process is stopped.
pub async fn run_http(addr: &str) -> std::io::Result<()> {
    let config =
        rmcp::transport::streamable_http_server::tower::StreamableHttpServerConfig::default();
    let service = rmcp::transport::streamable_http_server::tower::StreamableHttpService::new(
        || Ok(CepsApiMcp),
        Arc::new(
            rmcp::transport::streamable_http_server::session::local::LocalSessionManager::default(),
        ),
        config,
    );
    let method_router = axum::routing::any_service(service);
    let app = axum::Router::new()
        .route("/mcp", method_router.clone())
        .route("/mcp/", method_router);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "ceps-rust-api-mcp HTTP listening");
    axum::serve(listener, app).await?;
    Ok(())
}

#[tool_handler]
impl ServerHandler for CepsApiMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(rmcp::model::Implementation::new(
                "ceps-rust-api",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "MCP tools for ceps-rust-api: Make/Docker lifecycle and HTTP CEP API calls. Set CEPS_API_ROOT / CEPS_API_URL as needed.",
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_server_version_matches_crate() {
        let info = CepsApiMcp.get_info();
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.server_info.name.as_str(), "ceps-rust-api");
    }
}
