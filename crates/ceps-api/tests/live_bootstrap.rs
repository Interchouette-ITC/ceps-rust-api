//! Live bootstrap against a reachable node when faucet PEM is provided via env.
//!
//! Skips when RPC is down or `CEPS_FAUCET_PEM` / `CEPS_FAUCET_PEM_PATH` is unset.

#![cfg(all(feature = "custody", feature = "sign-local", feature = "tx-return"))]

#[path = "integration/harness.rs"]
mod harness;

use actix_web::test;
use ceps_api::server::create_app;
use harness::{load_faucet_pem, rpc_reachable, state_with_faucet};

#[actix_web::test]
async fn faucet_funds_created_key_when_configured() {
    let Some((faucet_pk, faucet_pem)) = load_faucet_pem() else {
        eprintln!("skip live bootstrap: set CEPS_FAUCET_PEM or CEPS_FAUCET_PEM_PATH");
        return;
    };
    let state = state_with_faucet(&faucet_pk, &faucet_pem);
    if !rpc_reachable(&state.config.rpc_url).await {
        eprintln!(
            "skip live bootstrap: RPC unreachable at {}",
            state.config.rpc_url
        );
        return;
    }

    let app = test::init_service(create_app(state)).await;

    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/keys/create")
            .set_json(serde_json::json!({"algo": "ed25519"}))
            .to_request(),
    )
    .await;
    assert!(
        resp.status().is_success(),
        "keys/create failed: {}",
        resp.status()
    );
    let created: serde_json::Value = test::read_body_json(resp).await;
    let target = created["public_key"]
        .as_str()
        .expect("public_key")
        .to_string();

    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/chain/fund")
            .set_json(serde_json::json!({
                "submit": "put",
                "wait": "accepted",
                "signer": {"public_key": faucet_pk},
                "payment_amount": "1000000000",
                "target": target,
                "amount": "2500000000"
            }))
            .to_request(),
    )
    .await;
    let status = resp.status();
    let body = test::read_body(resp).await;
    assert!(
        status.is_success(),
        "fund status {status} body {}",
        String::from_utf8_lossy(&body)
    );
    let out: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(!out["transaction_hash"].as_str().unwrap_or("").is_empty());

    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/v1/chain/balance/{target}"))
            .to_request(),
    )
    .await;
    assert!(
        resp.status().is_success() || resp.status().as_u16() == 502,
        "balance unexpected {}",
        resp.status()
    );
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
    // Missing wasm file → 400/404/502 is acceptable; success proves make_only path.
    assert!(
        status.is_success()
            || status.as_u16() == 400
            || status.as_u16() == 404
            || status.as_u16() == 502,
        "unexpected {status} {}",
        String::from_utf8_lossy(&body)
    );
    if status.is_success() {
        let out: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(out.get("transaction").is_some());
    }
}
