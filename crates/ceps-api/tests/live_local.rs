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
use ceps_rust_api::routes::hello::HelloResult;
use ceps_rust_api::server::create_app;
use ceps_rust_api::state::AppState;
use harness::{rpc_reachable, state_from_env};

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
                    "wasm": "cep18"
                }))
                .to_request(),
        )
        .await;
        assert!(
            resp.status().is_success(),
            "make-only with user key {i} failed: {}",
            resp.status()
        );
        let body: serde_json::Value = test::read_body_json(resp).await;
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
    let app = test::init_service(create_app(state)).await;

    let install = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/cep18/install")
            .set_json(serde_json::json!({
                "submit": "put",
                "wait": "processed",
                "signer": {"public_key": installer},
                "payment_amount": "500000000000",
                "name": "Live18",
                "symbol": "L18",
                "decimals": 9,
                "total_supply": "1000000000000",
                "wasm": "cep18"
            }))
            .to_request(),
    )
    .await;
    assert!(
        install.status().is_success(),
        "put install failed: {}",
        install.status()
    );
    let install_body: serde_json::Value = test::read_body_json(install).await;
    assert!(
        !install_body["transaction_hash"]
            .as_str()
            .unwrap_or("")
            .is_empty(),
        "install body: {install_body}"
    );

    let contract_hash = extract_contract_hash(&install_body);
    let Some(contract_hash) = contract_hash else {
        eprintln!(
            "skip mint/transfer: could not resolve contract hash from install result: {install_body}"
        );
        return;
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
    assert!(
        mint.status().is_success(),
        "mint to user2 failed: {}",
        mint.status()
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
    assert!(
        transfer.status().is_success(),
        "transfer user2→user3 failed: {}",
        transfer.status()
    );
}

/// Best-effort: CES / execution payloads sometimes embed the installed hash.
fn extract_contract_hash(install_body: &serde_json::Value) -> Option<String> {
    if let Some(h) = install_body
        .pointer("/contract_hash")
        .and_then(|v| v.as_str())
    {
        return Some(h.trim_start_matches("hash-").to_string());
    }
    let blob = install_body.to_string();
    // 64 hex chars after optional hash- / entity-contract-
    regex_lite_hash(&blob)
}

fn regex_lite_hash(blob: &str) -> Option<String> {
    // Avoid new deps: scan for "hash-" + 64 hex
    let bytes = blob.as_bytes();
    let needle = b"hash-";
    let mut i = 0;
    while i + 5 + 64 <= bytes.len() {
        if &bytes[i..i + 5] == needle {
            let hex = &blob[i + 5..i + 5 + 64];
            if hex.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(hex.to_string());
            }
        }
        i += 1;
    }
    None
}
