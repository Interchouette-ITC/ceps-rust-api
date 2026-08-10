//! Live local integration: NCTL **users** in `LOCAL_KEYS_JSON` (never the faucet).
//!
//! Ops: `NCTL_USERS="1 2 3" scripts/export-nctl-local-keys.sh`
//! Faucet is reserved for KMS funding (`fund-kms-from-nctl.sh`).
//!
//! Coverage today: CEP-18 put install → mint → transfer across three users.
//! Other CEP families / full entrypoint matrices are not covered here yet.

#![cfg(all(feature = "tx-return", feature = "sign-local", feature = "cep18"))]

#[path = "integration/harness.rs"]
mod harness;

use actix_web::test;
use ceps_client::CEP18Client;
use ceps_rust_api::routes::hello::HelloResult;
use ceps_rust_api::server::create_app;
use ceps_rust_api::state::AppState;
use harness::{rpc_reachable, state_from_env};
use std::time::{SystemTime, UNIX_EPOCH};

fn require_user_keys(state: &AppState, n: usize) -> Option<Vec<String>> {
    let mut keys = state.keyring.public_keys();
    keys.sort();
    if keys.len() < n {
        eprintln!(
            "skip live_local: need >= {n} NCTL user keys in LOCAL_KEYS_JSON (got {})",
            keys.len()
        );
        return None;
    }
    Some(keys)
}

fn skip_unless_live(state: &AppState) -> bool {
    if state.keyring.is_empty() {
        eprintln!(
            "skip live_local: export NCTL users only, e.g. NCTL_USERS=\"1 2 3\" scripts/export-nctl-local-keys.sh"
        );
        return true;
    }
    false
}

fn unique_token_name(prefix: &str) -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{prefix}{secs}")
}

async fn resolve_cep18_contract_hash(
    state: &AppState,
    installer_pk: &str,
    name: &str,
) -> Option<String> {
    let client = CEP18Client::new(
        state.config.rpc_url.clone(),
        Some(state.config.sse_url.clone()),
        Some(state.config.chain_name.clone()),
        None,
    )
    .ok()?;
    client
        .get_account_named_key(installer_pk, &format!("cep18_contract_hash_{name}"))
        .await
        .ok()
}

#[actix_web::test]
async fn hello_reports_identity() {
    let state = state_from_env();
    if skip_unless_live(&state) {
        return;
    }
    if !rpc_reachable(&state.config.rpc_url).await {
        eprintln!(
            "skip live_local: RPC unreachable at {}",
            state.config.rpc_url
        );
        return;
    }

    let app = test::init_service(create_app(state)).await;
    let resp = test::call_service(&app, test::TestRequest::get().uri("/").to_request()).await;
    assert!(resp.status().is_success());
    let body: HelloResult = test::read_body_json(resp).await;
    assert_eq!(body.name, "ceps-rust-api");
    assert!(!body.version.is_empty());
}

#[actix_web::test]
async fn cep18_make_only_uses_distinct_user_signers() {
    let state = state_from_env();
    if skip_unless_live(&state) {
        return;
    }
    let Some(keys) = require_user_keys(&state, 2) else {
        return;
    };
    if !rpc_reachable(&state.config.rpc_url).await {
        eprintln!("skip live_local: RPC unreachable");
        return;
    }

    let app = test::init_service(create_app(state)).await;
    for (i, pk) in keys.iter().take(2).enumerate() {
        let resp = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/cep18/install")
                .set_json(serde_json::json!({
                    "submit": "return",
                    "wait": "accepted",
                    "signer": {"public_key": pk},
                    "payment_amount": "500000000000",
                    "name": format!("U{i}"),
                    "symbol": format!("U{i}"),
                    "decimals": 9,
                    "total_supply": "1000000000000",
                    "wasm": "cep18",
                    "enable_mint_and_burn": true
                }))
                .to_request(),
        )
        .await;
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        assert!(
            status.is_success(),
            "make-only with user key {i} failed: {status} {}",
            String::from_utf8_lossy(&body_bytes)
        );
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert!(
            body.get("transaction").is_some(),
            "missing transaction for {pk}"
        );
    }
}

#[actix_web::test]
async fn cep18_put_install_mint_transfer_across_users() {
    let state = state_from_env();
    if skip_unless_live(&state) {
        return;
    }
    let Some(keys) = require_user_keys(&state, 3) else {
        return;
    };
    if !rpc_reachable(&state.config.rpc_url).await {
        eprintln!("skip live_local: RPC unreachable");
        return;
    }

    let installer = keys[0].clone();
    let recipient = keys[1].clone();
    let later = keys[2].clone();
    let name = unique_token_name("Live18");
    let app = test::init_service(create_app(state.clone())).await;

    let install = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/cep18/install")
            .set_json(serde_json::json!({
                "submit": "put",
                "wait": "processed",
                "signer": {"public_key": installer},
                "payment_amount": "500000000000",
                "name": name,
                "symbol": "L18",
                "decimals": 9,
                "total_supply": "1000000000000",
                "wasm": "cep18",
                "enable_mint_and_burn": true
            }))
            .to_request(),
    )
    .await;
    let install_status = install.status();
    let install_bytes = test::read_body(install).await;
    assert!(
        install_status.is_success(),
        "put install failed: {install_status} {}",
        String::from_utf8_lossy(&install_bytes)
    );
    let install_body: serde_json::Value = serde_json::from_slice(&install_bytes).unwrap();
    assert!(
        !install_body["transaction_hash"]
            .as_str()
            .unwrap_or("")
            .is_empty(),
        "install body: {install_body}"
    );

    let Some(contract_hash) = resolve_cep18_contract_hash(&state, &installer, &name).await else {
        panic!(
            "could not resolve cep18_contract_hash_{name} for installer after install: {install_body}"
        );
    };

    let mint = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/cep18/mint")
            .set_json(serde_json::json!({
                "submit": "put",
                "wait": "processed",
                "signer": {"public_key": installer},
                "payment_amount": "100000000000",
                "contract_hash": contract_hash,
                "owner": recipient,
                "amount": "1000"
            }))
            .to_request(),
    )
    .await;
    let mint_status = mint.status();
    let mint_bytes = test::read_body(mint).await;
    assert!(
        mint_status.is_success(),
        "mint to user2 failed: {mint_status} {}",
        String::from_utf8_lossy(&mint_bytes)
    );

    let transfer = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/cep18/transfer")
            .set_json(serde_json::json!({
                "submit": "put",
                "wait": "processed",
                "signer": {"public_key": recipient},
                "payment_amount": "100000000000",
                "contract_hash": contract_hash,
                "recipient": later,
                "amount": "10"
            }))
            .to_request(),
    )
    .await;
    let transfer_status = transfer.status();
    let transfer_bytes = test::read_body(transfer).await;
    assert!(
        transfer_status.is_success(),
        "transfer user2→user3 failed: {transfer_status} {}",
        String::from_utf8_lossy(&transfer_bytes)
    );
}
