//! Soft make-only smoke (no live node/keys required).
//! Live NCTL/KMS suites: `live_local`, `live_kms`.

#![cfg(all(feature = "tx-return", feature = "cep18"))]

use actix_web::test;
use ceps_rust_api::server::create_app;

#[actix_web::test]
async fn make_only_cep18_install_tolerates_offline() {
    let app = test::init_service(create_app(ceps_rust_api::state::AppState::new(
        ceps_rust_api::config::Config::default(),
    )))
    .await;
    let initiator = format!("01{}", "11".repeat(32));
    let uri = format!(
        "/v1/cep18/install?submit=return&signer={initiator}&payment_amount=500000000000&name=SoftTok&symbol=STK&decimals=9&total_supply=1000000000000&wasm=cep18"
    );
    let resp = test::call_service(&app, test::TestRequest::post().uri(&uri).to_request()).await;
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
