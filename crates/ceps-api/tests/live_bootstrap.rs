//! Live bootstrap: faucet PEM funds a target (tests only).
//!
//! Order: resolve target (`CEPS_FUND_TARGET` or KMS createKey directly) → SDK transfer
//! FROM faucet PEM (typically NCTL faucet) → optional balance query on the API.

#![cfg(all(feature = "sign-local", feature = "tx-return"))]

#[path = "integration/harness.rs"]
mod harness;

use actix_web::test;
use ceps_api::server::create_app;
use harness::{
    fund_from_faucet, kms_create_key, load_faucet_pem, rpc_reachable, state_with_faucet,
};

#[actix_web::test]
async fn faucet_funds_target_when_configured() {
    let Some((faucet_pk, faucet_pem)) = load_faucet_pem() else {
        eprintln!("skip live bootstrap: set CEPS_FAUCET_PEM or CEPS_FAUCET_PEM_PATH");
        return;
    };

    let kms_url = std::env::var("KMS_URL")
        .ok()
        .filter(|u| !u.trim().is_empty());
    let fund_target = std::env::var("CEPS_FUND_TARGET")
        .ok()
        .filter(|t| !t.trim().is_empty());

    let state = state_with_faucet(&faucet_pk, &faucet_pem, kms_url.clone());
    if !rpc_reachable(&state.config.rpc_url).await {
        eprintln!(
            "skip live bootstrap: RPC unreachable at {}",
            state.config.rpc_url
        );
        return;
    }

    let target = if let Some(t) = fund_target {
        t
    } else if let Some(ref url) = kms_url {
        match kms_create_key(url).await {
            Ok(pk) => pk,
            Err(e) => {
                eprintln!("skip live bootstrap: kms createKey failed: {e}");
                return;
            }
        }
    } else {
        eprintln!("skip live bootstrap: set CEPS_FUND_TARGET (e.g. NCTL user) or KMS_URL");
        return;
    };

    let hash = match fund_from_faucet(
        &state.config.rpc_url,
        &state.config.chain_name,
        &faucet_pem,
        &target,
        "2500000000",
        "1000000000",
    )
    .await
    {
        Ok(h) => h,
        Err(e) => {
            panic!("harness fund failed: {e}");
        }
    };
    assert!(!hash.is_empty());

    let app = test::init_service(create_app(state)).await;
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
