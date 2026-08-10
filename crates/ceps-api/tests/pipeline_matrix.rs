//! Integration tests: pipeline matrix, feature imply/forbid, live NCTL smoke when reachable.

use actix_web::test;
use ceps_rust_api::config::Config;
use ceps_rust_api::server::create_app;
use ceps_rust_api::state::AppState;

#[cfg(feature = "sign-local")]
use ceps_rust_api::config::SignBackend;
#[cfg(feature = "sign-local")]
use ceps_rust_api::sign::LocalKeyring;

fn app_none() -> AppState {
    AppState::new(Config::default())
}

#[actix_web::test]
async fn health_and_hello_features() {
    let app = test::init_service(create_app(app_none())).await;
    let resp = test::call_service(&app, test::TestRequest::get().uri("/health").to_request()).await;
    assert!(resp.status().is_success());

    let resp = test::call_service(&app, test::TestRequest::get().uri("/").to_request()).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "ceps-rust-api");
    assert!(body
        .get("version")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty()));
    assert!(body.get("sign_backend").is_none());
    assert!(body.get("kms_url_configured").is_none());
    assert!(body.get("local_keys_loaded").is_none());
    assert_eq!(body["features"]["tx_return"], cfg!(feature = "tx-return"));
    assert_eq!(body["features"]["cep18"], cfg!(feature = "cep18"));
}

#[cfg(feature = "cep18")]
#[actix_web::test]
async fn put_requires_signer() {
    let app = test::init_service(create_app(app_none())).await;
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/cep18/transfer")
            .set_json(serde_json::json!({
                "submit": "put",
                "signer": {"public_key": "01aa"},
                "payment_amount": "1",
                "contract_hash": "cfa781f5eb69c3eee952c2944ce9670a049f88c5e46b83fb5881ebe13fb98e6d",
                "recipient": "account-hash-b485c074cef7ccaccd0302949d2043ab7133abdb14cfa87e8392945c0bd80a5f",
                "amount": "1"
            }))
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], "no_signer");
}

#[cfg(all(feature = "cep18", feature = "tx-return"))]
#[actix_web::test]
async fn submit_return_make_only_transfer() {
    let app = test::init_service(create_app(app_none())).await;
    let initiator = format!("01{}", "11".repeat(32));
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/cep18/transfer")
            .set_json(serde_json::json!({
                "submit": "return",
                "signer": {"public_key": initiator},
                "payment_amount": "1000000000",
                "contract_hash": "cfa781f5eb69c3eee952c2944ce9670a049f88c5e46b83fb5881ebe13fb98e6d",
                "recipient": "account-hash-b485c074cef7ccaccd0302949d2043ab7133abdb14cfa87e8392945c0bd80a5f",
                "amount": "1"
            }))
            .to_request(),
    )
    .await;
    let status = resp.status();
    let body_bytes = test::read_body(resp).await;
    assert!(
        status.is_success(),
        "status {status} body {}",
        String::from_utf8_lossy(&body_bytes)
    );
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body.get("transaction").is_some());
    assert!(!body["transaction_hash"].as_str().unwrap().is_empty());
}

#[cfg(feature = "chain-put")]
#[actix_web::test]
async fn chain_put_rejects_invalid_json() {
    let app = test::init_service(create_app(app_none())).await;
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/chain/put-transaction")
            .set_json(serde_json::json!({
                "transaction": {"not": "a-transaction"},
                "wait": "accepted"
            }))
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), 400);
}

#[cfg(all(feature = "cep18", feature = "sign-local"))]
#[actix_web::test]
async fn local_put_missing_key_is_no_signer() {
    let cfg = Config {
        sign_backend: SignBackend::Local,
        local_keys: LocalKeyring::new(),
        ..Default::default()
    };
    let app = test::init_service(create_app(AppState::new(cfg))).await;
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/cep18/transfer")
            .set_json(serde_json::json!({
                "submit": "put",
                "signer": {"public_key": "01dead"},
                "payment_amount": "1",
                "contract_hash": "cfa781f5eb69c3eee952c2944ce9670a049f88c5e46b83fb5881ebe13fb98e6d",
                "recipient": "account-hash-b485c074cef7ccaccd0302949d2043ab7133abdb14cfa87e8392945c0bd80a5f",
                "amount": "1"
            }))
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], "no_signer");
}

#[cfg(feature = "sign-kms")]
#[actix_web::test]
async fn kms_sign_client_via_wiremock() {
    use ceps_rust_api::config::SignBackend;
    use ceps_rust_api::kms::KmsClient;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/signTransaction"))
        .and(query_param(
            "keys",
            "0202ccddeeff00112233445566778899aabbccddeeff00112233445566778899aabb",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "transaction": {"ok": true}
        })))
        .mount(&server)
        .await;

    let cfg = Config {
        sign_backend: SignBackend::Kms,
        kms_url: server.uri(),
        ..Default::default()
    };
    let _state = AppState::new(cfg);
    let client = KmsClient::new(server.uri());
    let signed = client
        .sign_transaction(
            "0202ccddeeff00112233445566778899aabbccddeeff00112233445566778899aabb",
            &serde_json::json!({"tx": 1}),
        )
        .await
        .expect("sign");
    assert_eq!(signed["transaction"]["ok"], true);
}

#[cfg(not(feature = "chain-put"))]
#[actix_web::test]
async fn chain_put_absent_when_feature_off() {
    let app = test::init_service(create_app(app_none())).await;
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/chain/put-transaction")
            .set_json(serde_json::json!({
                "transaction": {"x": 1},
                "wait": "accepted"
            }))
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), 404);
}

#[cfg(all(feature = "cep18", not(feature = "tx-return")))]
#[actix_web::test]
async fn submit_return_feature_disabled() {
    let app = test::init_service(create_app(app_none())).await;
    let initiator = format!("01{}", "11".repeat(32));
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/cep18/transfer")
            .set_json(serde_json::json!({
                "submit": "return",
                "signer": {"public_key": initiator},
                "payment_amount": "1",
                "contract_hash": "cfa781f5eb69c3eee952c2944ce9670a049f88c5e46b83fb5881ebe13fb98e6d",
                "recipient": "account-hash-b485c074cef7ccaccd0302949d2043ab7133abdb14cfa87e8392945c0bd80a5f",
                "amount": "1"
            }))
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], "feature_disabled");
}

#[cfg(feature = "swagger-ui")]
#[actix_web::test]
async fn openapi_lists_platform_and_cep18_when_enabled() {
    let app = test::init_service(create_app(app_none())).await;
    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/docs/ceps-openapi.json")
            .to_request(),
    )
    .await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    let paths = body["paths"].as_object().expect("paths");
    assert!(!paths.contains_key("/v1/instances/{id}"));
    assert!(!paths.contains_key("/v1/wasm"));
    #[cfg(feature = "chain-put")]
    assert!(paths.contains_key("/v1/chain/put-transaction"));
    #[cfg(not(feature = "chain-put"))]
    assert!(!paths.contains_key("/v1/chain/put-transaction"));
    assert!(!paths.contains_key("/v1/chain/balance/{public_key}"));
    assert!(!paths.contains_key("/v1/chain/account/{public_key}"));
    assert!(!paths.contains_key("/v1/chain/transaction/{hash}"));
    #[cfg(feature = "cep18")]
    {
        assert!(paths.contains_key("/v1/cep18/install"));
        assert!(paths.contains_key("/v1/cep18/transfer"));
        assert!(paths.contains_key("/v1/cep18/{contract_hash}/balance-of/{owner}"));
    }
    #[cfg(feature = "cep78")]
    {
        assert!(paths.contains_key("/v1/cep78/install"));
        assert!(paths.contains_key("/v1/cep78/mint"));
        assert!(paths.contains_key("/v1/cep78/{contract_hash}/owner-of/{token}"));
    }
    #[cfg(feature = "cep85")]
    {
        assert!(paths.contains_key("/v1/cep85/install"));
        assert!(paths.contains_key("/v1/cep85/mint"));
        assert!(paths.contains_key("/v1/cep85/{contract_hash}/balance-of/{owner}/{id}"));
    }
    #[cfg(feature = "cep95")]
    {
        assert!(paths.contains_key("/v1/cep95/install"));
        assert!(paths.contains_key("/v1/cep95/transfer-from"));
        assert!(paths.contains_key("/v1/cep95/bind-odra-install"));
    }
}
