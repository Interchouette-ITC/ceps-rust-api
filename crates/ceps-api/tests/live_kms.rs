//! Live KMS integration (`SIGN_BACKEND=kms`, funded key in `CEPS_KMS_PUBLIC_KEY`).
//!
//! Skip unless env is prepared. Ops: KMS createKey + `scripts/fund-kms-from-nctl.sh`.
//! Assertions live in Rust; create/fund stay outside this crate.

#![cfg(all(feature = "tx-return", feature = "sign-kms", feature = "cep18"))]

#[path = "integration/harness.rs"]
mod harness;

use actix_web::test;
use ceps_rust_api::config::SignBackend;
use ceps_rust_api::routes::hello::HelloResult;
use ceps_rust_api::server::create_app;
use harness::{rpc_reachable, state_from_env};
use std::env;

#[actix_web::test]
async fn hello_reports_identity() {
    let state = state_from_env();
    if state.config.sign_backend != SignBackend::Kms || !state.config.kms_url_configured() {
        eprintln!("skip live_kms: set SIGN_BACKEND=kms and KMS_URL");
        return;
    }
    if !rpc_reachable(&state.config.rpc_url).await {
        eprintln!("skip live_kms: RPC unreachable at {}", state.config.rpc_url);
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
async fn cep18_install_make_only_with_kms_public_key() {
    let pk = match env::var("CEPS_KMS_PUBLIC_KEY") {
        Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => {
            eprintln!("skip live_kms: set CEPS_KMS_PUBLIC_KEY (funded KMS key)");
            return;
        }
    };
    let state = state_from_env();
    if state.config.sign_backend != SignBackend::Kms || !state.config.kms_url_configured() {
        eprintln!("skip live_kms: set SIGN_BACKEND=kms and KMS_URL");
        return;
    }
    if !rpc_reachable(&state.config.rpc_url).await {
        eprintln!("skip live_kms: RPC unreachable");
        return;
    }

    let app = test::init_service(create_app(state)).await;
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/cep18/install")
            .set_json(serde_json::json!({
                "submit": "return",
                "signer": {"public_key": pk},
                "payment_amount": "500000000000",
                "name": "LiveKms",
                "symbol": "LK",
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
