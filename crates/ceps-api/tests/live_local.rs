//! Live local integration (NCTL + `LOCAL_KEYS_JSON`).
//!
//! Skip unless env is prepared (ops: `scripts/export-nctl-local-keys.sh` + reachable RPC).
//! Assertions live in Rust; funding/key export stay in ops scripts.

#![cfg(all(feature = "tx-return", feature = "sign-local", feature = "cep18"))]

#[path = "integration/harness.rs"]
mod harness;

use actix_web::test;
use ceps_rust_api::routes::hello::HelloResult;
use ceps_rust_api::server::create_app;
use harness::{rpc_reachable, state_from_env};

#[actix_web::test]
async fn hello_reports_version_and_local_backend() {
    let state = state_from_env();
    if state.keyring.is_empty() {
        eprintln!("skip live_local: set LOCAL_KEYS_JSON (scripts/export-nctl-local-keys.sh)");
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
    assert_eq!(body.sign_backend, "local");
    assert!(body.local_keys_loaded >= 1);
    assert!(!body.rpc_url.is_empty());
    assert!(!body.chain_name.is_empty());
}

#[actix_web::test]
async fn cep18_install_make_only_with_local_signer() {
    let state = state_from_env();
    if state.keyring.is_empty() {
        eprintln!("skip live_local: set LOCAL_KEYS_JSON");
        return;
    }
    if !rpc_reachable(&state.config.rpc_url).await {
        eprintln!("skip live_local: RPC unreachable");
        return;
    }

    let pk = state
        .keyring
        .public_keys()
        .into_iter()
        .next()
        .expect("keyring non-empty");
    let app = test::init_service(create_app(state)).await;
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/cep18/install")
            .set_json(serde_json::json!({
                "submit": "return",
                "signer": {"public_key": pk},
                "payment_amount": "500000000000",
                "name": "LiveLocal",
                "symbol": "LL",
                "decimals": 9,
                "total_supply": "1000000000000",
                "wasm": "cep18"
            }))
            .to_request(),
    )
    .await;
    assert!(
        resp.status().is_success(),
        "make-only install failed: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body.get("transaction").is_some() || body.get("transaction_hash").is_some(),
        "unexpected body: {body}"
    );
}
