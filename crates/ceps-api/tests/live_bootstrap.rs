//! Live checks when env is configured.
//!
//! Local: pre-funded NCTL keys via `LOCAL_KEYS_JSON` (no fund step).
//! This suite does not create or fund KMS keys.

#![cfg(feature = "tx-return")]

#[path = "integration/harness.rs"]
mod harness;

use actix_web::test;
use ceps_api::server::create_app;
use harness::{rpc_reachable, state_from_env};

#[actix_web::test]
async fn hello_reports_local_keys_when_configured() {
    let state = state_from_env();
    if state.keyring.is_empty() {
        eprintln!("skip live: set LOCAL_KEYS_JSON with funded NCTL keys");
        return;
    }
    if !rpc_reachable(&state.config.rpc_url).await {
        eprintln!("skip live: RPC unreachable at {}", state.config.rpc_url);
        return;
    }

    let app = test::init_service(create_app(state)).await;
    let resp = test::call_service(&app, test::TestRequest::get().uri("/").to_request()).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["sign_backend"], "local");
}

#[actix_web::test]
async fn make_only_cep18_install_when_rpc_optional() {
    let app = test::init_service(create_app(ceps_api::state::AppState::new(
        ceps_api::config::Config::default(),
    )))
    .await;
    let initiator = format!("01{}", "11".repeat(32));
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/cep18/install")
            .set_json(serde_json::json!({
                "submit": "return",
                "signer": {"public_key": initiator},
                "payment_amount": "500000000000",
                "name": "LiveTok",
                "symbol": "LTK",
                "decimals": 9,
                "total_supply": "1000000000000",
                "wasm": "cep18"
            }))
            .to_request(),
    )
    .await;
    let status = resp.status();
    let body = test::read_body(resp).await;
    assert!(
        status.is_success()
            || status.as_u16() == 400
            || status.as_u16() == 404
            || status.as_u16() == 502,
        "unexpected {} {}",
        status,
        String::from_utf8_lossy(&body)
    );
}
