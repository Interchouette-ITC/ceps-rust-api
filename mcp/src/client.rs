//! Thin HTTP client for the running ceps-rust-api.

use reqwest::Client;
use serde_json::Value;

use crate::paths::api_base_url;

fn client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| Client::new())
}

fn format_response(status: reqwest::StatusCode, body: &str) -> String {
    format!("HTTP {status}\n{body}")
}

async fn text_or_err(result: Result<reqwest::Response, reqwest::Error>) -> String {
    match result {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|e| e.to_string());
            format_response(status, &body)
        }
        Err(e) => format!("request failed: {e}"),
    }
}

fn base() -> String {
    api_base_url().trim_end_matches('/').to_string()
}

pub async fn get_path(path: &str) -> String {
    let url = format!("{}{}", base(), path);
    text_or_err(client().get(&url).send().await).await
}

pub async fn post_json(path: &str, body: &str) -> String {
    let url = format!("{}{}", base(), path);
    let parsed: Result<Value, _> = serde_json::from_str(body);
    match parsed {
        Ok(v) => text_or_err(client().post(&url).json(&v).send().await).await,
        Err(e) => format!("invalid JSON body: {e}"),
    }
}

pub async fn hello() -> String {
    get_path("/").await
}

pub async fn health() -> String {
    get_path("/health").await
}

pub async fn openapi() -> String {
    get_path("/docs/ceps-openapi.json").await
}

pub async fn cep18_install(body: &str) -> String {
    post_json("/v1/cep18/install", body).await
}

pub async fn cep18_upgrade(body: &str) -> String {
    post_json("/v1/cep18/upgrade", body).await
}

pub async fn cep18_transfer(body: &str) -> String {
    post_json("/v1/cep18/transfer", body).await
}

pub async fn cep18_transfer_from(body: &str) -> String {
    post_json("/v1/cep18/transfer-from", body).await
}

pub async fn cep18_approve(body: &str) -> String {
    post_json("/v1/cep18/approve", body).await
}

pub async fn cep18_increase_allowance(body: &str) -> String {
    post_json("/v1/cep18/increase-allowance", body).await
}

pub async fn cep18_decrease_allowance(body: &str) -> String {
    post_json("/v1/cep18/decrease-allowance", body).await
}

pub async fn cep18_mint(body: &str) -> String {
    post_json("/v1/cep18/mint", body).await
}

pub async fn cep18_burn(body: &str) -> String {
    post_json("/v1/cep18/burn", body).await
}

pub async fn cep18_change_events_mode(body: &str) -> String {
    post_json("/v1/cep18/change-events-mode", body).await
}

pub async fn cep18_change_security(body: &str) -> String {
    post_json("/v1/cep18/change-security", body).await
}

pub async fn cep78_install(body: &str) -> String {
    post_json("/v1/cep78/install", body).await
}

pub async fn cep78_upgrade(body: &str) -> String {
    post_json("/v1/cep78/upgrade", body).await
}

pub async fn cep78_mint(body: &str) -> String {
    post_json("/v1/cep78/mint", body).await
}

pub async fn cep78_transfer(body: &str) -> String {
    post_json("/v1/cep78/transfer", body).await
}

pub async fn cep78_burn(body: &str) -> String {
    post_json("/v1/cep78/burn", body).await
}

pub async fn cep78_register_owner(body: &str) -> String {
    post_json("/v1/cep78/register-owner", body).await
}

pub async fn cep78_approve(body: &str) -> String {
    post_json("/v1/cep78/approve", body).await
}

pub async fn cep78_revoke(body: &str) -> String {
    post_json("/v1/cep78/revoke", body).await
}

pub async fn cep78_set_approval_for_all(body: &str) -> String {
    post_json("/v1/cep78/set-approval-for-all", body).await
}

pub async fn cep78_set_token_metadata(body: &str) -> String {
    post_json("/v1/cep78/set-token-metadata", body).await
}

pub async fn cep78_set_variables(body: &str) -> String {
    post_json("/v1/cep78/set-variables", body).await
}

pub async fn cep78_mint_session(body: &str) -> String {
    post_json("/v1/cep78/mint-session", body).await
}

pub async fn cep78_transfer_session(body: &str) -> String {
    post_json("/v1/cep78/transfer-session", body).await
}

pub async fn cep78_updated_receipts(body: &str) -> String {
    post_json("/v1/cep78/updated-receipts", body).await
}

pub async fn cep85_install(body: &str) -> String {
    post_json("/v1/cep85/install", body).await
}

pub async fn cep85_upgrade(body: &str) -> String {
    post_json("/v1/cep85/upgrade", body).await
}

pub async fn cep85_mint(body: &str) -> String {
    post_json("/v1/cep85/mint", body).await
}

pub async fn cep85_batch_mint(body: &str) -> String {
    post_json("/v1/cep85/batch-mint", body).await
}

pub async fn cep85_transfer(body: &str) -> String {
    post_json("/v1/cep85/transfer", body).await
}

pub async fn cep85_batch_transfer(body: &str) -> String {
    post_json("/v1/cep85/batch-transfer", body).await
}

pub async fn cep85_burn(body: &str) -> String {
    post_json("/v1/cep85/burn", body).await
}

pub async fn cep85_batch_burn(body: &str) -> String {
    post_json("/v1/cep85/batch-burn", body).await
}

pub async fn cep85_set_approval_for_all(body: &str) -> String {
    post_json("/v1/cep85/set-approval-for-all", body).await
}

pub async fn cep85_set_uri(body: &str) -> String {
    post_json("/v1/cep85/set-uri", body).await
}

pub async fn cep85_set_total_supply_of(body: &str) -> String {
    post_json("/v1/cep85/set-total-supply-of", body).await
}

pub async fn cep85_set_total_supply_of_batch(body: &str) -> String {
    post_json("/v1/cep85/set-total-supply-of-batch", body).await
}

pub async fn cep85_change_security(body: &str) -> String {
    post_json("/v1/cep85/change-security", body).await
}

pub async fn cep85_set_modalities(body: &str) -> String {
    post_json("/v1/cep85/set-modalities", body).await
}

pub async fn cep95_install(body: &str) -> String {
    post_json("/v1/cep95/install", body).await
}

pub async fn cep95_transfer_from(body: &str) -> String {
    post_json("/v1/cep95/transfer-from", body).await
}

pub async fn cep95_safe_transfer_from(body: &str) -> String {
    post_json("/v1/cep95/safe-transfer-from", body).await
}

pub async fn cep95_approve(body: &str) -> String {
    post_json("/v1/cep95/approve", body).await
}

pub async fn cep95_revoke_approval(body: &str) -> String {
    post_json("/v1/cep95/revoke-approval", body).await
}

pub async fn cep95_approve_for_all(body: &str) -> String {
    post_json("/v1/cep95/approve-for-all", body).await
}

pub async fn cep95_revoke_approval_for_all(body: &str) -> String {
    post_json("/v1/cep95/revoke-approval-for-all", body).await
}

pub async fn cep95_mint(body: &str) -> String {
    post_json("/v1/cep95/mint", body).await
}

pub async fn cep95_burn(body: &str) -> String {
    post_json("/v1/cep95/burn", body).await
}

pub async fn cep95_bind_odra_install(body: &str) -> String {
    post_json("/v1/cep95/bind-odra-install", body).await
}

pub async fn put_transaction(body: &str) -> String {
    post_json("/v1/chain/put-transaction", body).await
}

pub async fn cep18_name(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep18/{contract_hash}/name");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep18_symbol(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep18/{contract_hash}/symbol");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep18_decimals(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep18/{contract_hash}/decimals");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep18_total_supply(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep18/{contract_hash}/total-supply");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep18_events_mode(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep18/{contract_hash}/events-mode");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep18_is_mint_and_burn_enabled(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep18/{contract_hash}/is-mint-and-burn-enabled");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep18_balance_of(contract_hash: &str, owner: &str) -> String {
    let mut path = String::from("/v1/cep18/{contract_hash}/balance-of/{owner}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{owner}", owner);
    get_path(&path).await
}

pub async fn cep18_allowances(contract_hash: &str, owner: &str, spender: &str) -> String {
    let mut path = String::from("/v1/cep18/{contract_hash}/allowances/{owner}/{spender}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{owner}", owner);
    path = path.replace("{spender}", spender);
    get_path(&path).await
}

pub async fn cep78_collection_name(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep78/{contract_hash}/collection-name");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep78_collection_symbol(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep78/{contract_hash}/collection-symbol");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep78_total_token_supply(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep78/{contract_hash}/total-token-supply");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep78_number_of_minted_tokens(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep78/{contract_hash}/number-of-minted-tokens");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep78_events_mode(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep78/{contract_hash}/events-mode");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep78_owner_of(contract_hash: &str, token: &str) -> String {
    let mut path = String::from("/v1/cep78/{contract_hash}/owner-of/{token}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{token}", token);
    get_path(&path).await
}

pub async fn cep78_balance_of(contract_hash: &str, owner: &str) -> String {
    let mut path = String::from("/v1/cep78/{contract_hash}/balance-of/{owner}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{owner}", owner);
    get_path(&path).await
}

pub async fn cep78_is_approved_for_all(contract_hash: &str, owner: &str, operator: &str) -> String {
    let mut path = String::from("/v1/cep78/{contract_hash}/is-approved-for-all/{owner}/{operator}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{owner}", owner);
    path = path.replace("{operator}", operator);
    get_path(&path).await
}

pub async fn cep78_get_approved(contract_hash: &str, token: &str) -> String {
    let mut path = String::from("/v1/cep78/{contract_hash}/get-approved/{token}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{token}", token);
    get_path(&path).await
}

pub async fn cep78_metadata(contract_hash: &str, token: &str) -> String {
    let mut path = String::from("/v1/cep78/{contract_hash}/metadata/{token}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{token}", token);
    get_path(&path).await
}

pub async fn cep85_collection_name(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep85/{contract_hash}/collection-name");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep85_collection_uri(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep85/{contract_hash}/collection-uri");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep85_balance_of(contract_hash: &str, owner: &str, id: &str) -> String {
    let mut path = String::from("/v1/cep85/{contract_hash}/balance-of/{owner}/{id}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{owner}", owner);
    path = path.replace("{id}", id);
    get_path(&path).await
}

pub async fn cep85_supply_of(contract_hash: &str, id: &str) -> String {
    let mut path = String::from("/v1/cep85/{contract_hash}/supply-of/{id}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{id}", id);
    get_path(&path).await
}

pub async fn cep85_total_supply_of(contract_hash: &str, id: &str) -> String {
    let mut path = String::from("/v1/cep85/{contract_hash}/total-supply-of/{id}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{id}", id);
    get_path(&path).await
}

pub async fn cep85_uri(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep85/{contract_hash}/uri");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep85_is_non_fungible(contract_hash: &str, id: &str) -> String {
    let mut path = String::from("/v1/cep85/{contract_hash}/is-non-fungible/{id}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{id}", id);
    get_path(&path).await
}

pub async fn cep85_is_approved_for_all(contract_hash: &str, owner: &str, operator: &str) -> String {
    let mut path = String::from("/v1/cep85/{contract_hash}/is-approved-for-all/{owner}/{operator}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{owner}", owner);
    path = path.replace("{operator}", operator);
    get_path(&path).await
}

pub async fn cep95_name(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep95/{contract_hash}/name");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep95_symbol(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep95/{contract_hash}/symbol");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep95_total_supply(contract_hash: &str) -> String {
    let mut path = String::from("/v1/cep95/{contract_hash}/total-supply");
    path = path.replace("{contract_hash}", contract_hash);
    get_path(&path).await
}

pub async fn cep95_owner_of(contract_hash: &str, token_id: &str) -> String {
    let mut path = String::from("/v1/cep95/{contract_hash}/owner-of/{token_id}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{token_id}", token_id);
    get_path(&path).await
}

pub async fn cep95_balance_of(contract_hash: &str, owner: &str) -> String {
    let mut path = String::from("/v1/cep95/{contract_hash}/balance-of/{owner}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{owner}", owner);
    get_path(&path).await
}

pub async fn cep95_get_approved(contract_hash: &str, token_id: &str) -> String {
    let mut path = String::from("/v1/cep95/{contract_hash}/get-approved/{token_id}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{token_id}", token_id);
    get_path(&path).await
}

pub async fn cep95_is_approved_for_all(contract_hash: &str, owner: &str, operator: &str) -> String {
    let mut path = String::from("/v1/cep95/{contract_hash}/is-approved-for-all/{owner}/{operator}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{owner}", owner);
    path = path.replace("{operator}", operator);
    get_path(&path).await
}

pub async fn cep95_token_metadata(contract_hash: &str, token_id: &str) -> String {
    let mut path = String::from("/v1/cep95/{contract_hash}/token-metadata/{token_id}");
    path = path.replace("{contract_hash}", contract_hash);
    path = path.replace("{token_id}", token_id);
    get_path(&path).await
}

pub async fn cep18_security_badge(contract_hash: &str, account: &str) -> String {
    get_path(&format!(
        "/v1/cep18/{contract_hash}/security-badge/{account}"
    ))
    .await
}

pub async fn cep78_owner_of_session(body: &str) -> String {
    post_json("/v1/cep78/owner-of-session", body).await
}

pub async fn cep78_balance_of_session(body: &str) -> String {
    post_json("/v1/cep78/balance-of-session", body).await
}

pub async fn cep78_get_approved_session(body: &str) -> String {
    post_json("/v1/cep78/get-approved-session", body).await
}

pub async fn cep78_is_approved_for_all_session(body: &str) -> String {
    post_json("/v1/cep78/is-approved-for-all-session", body).await
}

pub async fn cep78_allow_minting(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/allow-minting")).await
}

pub async fn cep78_operator_burn_mode(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/operator-burn-mode")).await
}

pub async fn cep78_package_operator_mode(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/package-operator-mode")).await
}

pub async fn cep78_acl_package_mode(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/acl-package-mode")).await
}

pub async fn cep78_json_schema(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/json-schema")).await
}

pub async fn cep78_minting_mode(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/minting-mode")).await
}

pub async fn cep78_whitelist_mode(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/whitelist-mode")).await
}

pub async fn cep78_reporting_mode(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/reporting-mode")).await
}

pub async fn cep78_burn_mode(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/burn-mode")).await
}

pub async fn cep78_holder_mode(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/holder-mode")).await
}

pub async fn cep78_identifier_mode(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/identifier-mode")).await
}

pub async fn cep78_metadata_mutability(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/metadata-mutability")).await
}

pub async fn cep78_nft_kind(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/nft-kind")).await
}

pub async fn cep78_nft_metadata_kind(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/nft-metadata-kind")).await
}

pub async fn cep78_ownership_mode(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep78/{contract_hash}/ownership-mode")).await
}

pub async fn cep78_is_acl_whitelisted(contract_hash: &str, entity: &str) -> String {
    get_path(&format!(
        "/v1/cep78/{contract_hash}/is-acl-whitelisted/{entity}"
    ))
    .await
}

pub async fn cep85_balance_of_batch(body: &str) -> String {
    post_json("/v1/cep85/balance-of-batch", body).await
}

pub async fn cep85_supply_of_batch(body: &str) -> String {
    post_json("/v1/cep85/supply-of-batch", body).await
}

pub async fn cep85_total_supply_of_batch(body: &str) -> String {
    post_json("/v1/cep85/total-supply-of-batch", body).await
}

pub async fn cep85_uri_with_id(contract_hash: &str, id: &str) -> String {
    get_path(&format!("/v1/cep85/{contract_hash}/uri?id={id}")).await
}

pub async fn cep85_total_fungible_supply(contract_hash: &str, id: &str) -> String {
    get_path(&format!(
        "/v1/cep85/{contract_hash}/total-fungible-supply/{id}"
    ))
    .await
}

pub async fn cep85_enable_burn(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep85/{contract_hash}/enable-burn")).await
}

pub async fn cep85_events_mode(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep85/{contract_hash}/events-mode")).await
}

pub async fn cep85_number_of_minted_tokens(contract_hash: &str) -> String {
    get_path(&format!(
        "/v1/cep85/{contract_hash}/number-of-minted-tokens"
    ))
    .await
}

pub async fn cep85_transfer_filter_contract(contract_hash: &str) -> String {
    get_path(&format!(
        "/v1/cep85/{contract_hash}/transfer-filter-contract"
    ))
    .await
}

pub async fn cep85_transfer_filter_method(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep85/{contract_hash}/transfer-filter-method")).await
}

pub async fn cep85_security_badge(contract_hash: &str, entity: &str) -> String {
    get_path(&format!(
        "/v1/cep85/{contract_hash}/security-badge/{entity}"
    ))
    .await
}

pub async fn cep95_transfer_ownership(body: &str) -> String {
    post_json("/v1/cep95/transfer-ownership", body).await
}

pub async fn cep95_get_owner(contract_hash: &str) -> String {
    get_path(&format!("/v1/cep95/{contract_hash}/get-owner")).await
}
