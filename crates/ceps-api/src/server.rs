//! Actix application factory and server.

use crate::config::Config;
use crate::middleware::cors::demo_cors;
use crate::openapi::build_openapi;
use crate::routes::{health_handler, hello_handler};
use crate::state::AppState;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tracing::{info, warn};

#[cfg(feature = "swagger-ui")]
use utoipa_swagger_ui::SwaggerUi;

async fn redirect_docs_absolute() -> impl Responder {
    HttpResponse::TemporaryRedirect()
        .append_header(("Location", "/docs/"))
        .finish()
}

#[cfg(any(
    feature = "cep18",
    feature = "cep78",
    feature = "cep85",
    feature = "cep95"
))]
macro_rules! svc {
    ($app:ident, $($s:expr),+ $(,)?) => {{
        $(
            $app = $app.service($s);
        )+
        $app
    }};
}

/// Build the Actix `App` (shared by server and tests).
#[must_use]
pub fn create_app(
    state: AppState,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let openapi = build_openapi();

    let mut app = App::new()
        .app_data(web::Data::new(state))
        .wrap(Logger::default())
        .wrap(demo_cors())
        .service(hello_handler)
        .service(health_handler)
        .route("/docs", web::get().to(redirect_docs_absolute));

    #[cfg(feature = "chain-put")]
    {
        app = app.service(crate::routes::chain::chain_put_transaction);
    }

    #[cfg(feature = "cep18")]
    {
        app = svc!(
            app,
            crate::routes::cep18::cep18_install,
            crate::routes::cep18::cep18_upgrade,
            crate::routes::cep18::cep18_transfer,
            crate::routes::cep18::cep18_transfer_from,
            crate::routes::cep18::cep18_approve,
            crate::routes::cep18::cep18_increase_allowance,
            crate::routes::cep18::cep18_decrease_allowance,
            crate::routes::cep18::cep18_mint,
            crate::routes::cep18::cep18_burn,
            crate::routes::cep18::cep18_change_events_mode,
            crate::routes::cep18::cep18_change_security,
            crate::routes::cep18::cep18_name,
            crate::routes::cep18::cep18_symbol,
            crate::routes::cep18::cep18_decimals,
            crate::routes::cep18::cep18_total_supply,
            crate::routes::cep18::cep18_events_mode,
            crate::routes::cep18::cep18_is_mint_and_burn_enabled,
            crate::routes::cep18::cep18_balance_of,
            crate::routes::cep18::cep18_allowances,
            crate::routes::cep18::cep18_security_badge,
        );
    }
    #[cfg(feature = "cep78")]
    {
        app = svc!(
            app,
            crate::routes::cep78::cep78_install,
            crate::routes::cep78::cep78_upgrade,
            crate::routes::cep78::cep78_mint,
            crate::routes::cep78::cep78_transfer,
            crate::routes::cep78::cep78_burn,
            crate::routes::cep78::cep78_register_owner,
            crate::routes::cep78::cep78_approve,
            crate::routes::cep78::cep78_revoke,
            crate::routes::cep78::cep78_set_approval_for_all,
            crate::routes::cep78::cep78_set_token_metadata,
            crate::routes::cep78::cep78_set_variables,
            crate::routes::cep78::cep78_mint_session,
            crate::routes::cep78::cep78_transfer_session,
            crate::routes::cep78::cep78_updated_receipts,
            crate::routes::cep78::cep78_owner_of_session,
            crate::routes::cep78::cep78_balance_of_session,
            crate::routes::cep78::cep78_get_approved_session,
            crate::routes::cep78::cep78_is_approved_for_all_session,
            crate::routes::cep78::cep78_collection_name,
            crate::routes::cep78::cep78_collection_symbol,
            crate::routes::cep78::cep78_total_token_supply,
            crate::routes::cep78::cep78_number_of_minted_tokens,
            crate::routes::cep78::cep78_allow_minting,
            crate::routes::cep78::cep78_operator_burn_mode,
            crate::routes::cep78::cep78_package_operator_mode,
            crate::routes::cep78::cep78_acl_package_mode,
            crate::routes::cep78::cep78_json_schema,
            crate::routes::cep78::cep78_minting_mode,
            crate::routes::cep78::cep78_whitelist_mode,
            crate::routes::cep78::cep78_reporting_mode,
            crate::routes::cep78::cep78_burn_mode,
            crate::routes::cep78::cep78_holder_mode,
            crate::routes::cep78::cep78_identifier_mode,
            crate::routes::cep78::cep78_metadata_mutability,
            crate::routes::cep78::cep78_nft_kind,
            crate::routes::cep78::cep78_nft_metadata_kind,
            crate::routes::cep78::cep78_ownership_mode,
            crate::routes::cep78::cep78_events_mode,
            crate::routes::cep78::cep78_owner_of,
            crate::routes::cep78::cep78_balance_of,
            crate::routes::cep78::cep78_is_approved_for_all,
            crate::routes::cep78::cep78_get_approved,
            crate::routes::cep78::cep78_metadata,
            crate::routes::cep78::cep78_is_acl_whitelisted,
        );
    }
    #[cfg(feature = "cep85")]
    {
        app = svc!(
            app,
            crate::routes::cep85::cep85_install,
            crate::routes::cep85::cep85_upgrade,
            crate::routes::cep85::cep85_mint,
            crate::routes::cep85::cep85_batch_mint,
            crate::routes::cep85::cep85_transfer,
            crate::routes::cep85::cep85_batch_transfer,
            crate::routes::cep85::cep85_burn,
            crate::routes::cep85::cep85_batch_burn,
            crate::routes::cep85::cep85_set_approval_for_all,
            crate::routes::cep85::cep85_set_uri,
            crate::routes::cep85::cep85_set_total_supply_of,
            crate::routes::cep85::cep85_set_total_supply_of_batch,
            crate::routes::cep85::cep85_change_security,
            crate::routes::cep85::cep85_set_modalities,
            crate::routes::cep85::cep85_balance_of_batch,
            crate::routes::cep85::cep85_supply_of_batch,
            crate::routes::cep85::cep85_total_supply_of_batch,
            crate::routes::cep85::cep85_collection_name,
            crate::routes::cep85::cep85_collection_uri,
            crate::routes::cep85::cep85_balance_of,
            crate::routes::cep85::cep85_supply_of,
            crate::routes::cep85::cep85_total_supply_of,
            crate::routes::cep85::cep85_total_fungible_supply,
            crate::routes::cep85::cep85_uri,
            crate::routes::cep85::cep85_is_non_fungible,
            crate::routes::cep85::cep85_is_approved_for_all,
            crate::routes::cep85::cep85_enable_burn,
            crate::routes::cep85::cep85_events_mode,
            crate::routes::cep85::cep85_number_of_minted_tokens,
            crate::routes::cep85::cep85_transfer_filter_contract,
            crate::routes::cep85::cep85_transfer_filter_method,
            crate::routes::cep85::cep85_security_badge,
        );
    }
    #[cfg(feature = "cep95")]
    {
        app = svc!(
            app,
            crate::routes::cep95::cep95_install,
            crate::routes::cep95::cep95_transfer_from,
            crate::routes::cep95::cep95_safe_transfer_from,
            crate::routes::cep95::cep95_approve,
            crate::routes::cep95::cep95_revoke_approval,
            crate::routes::cep95::cep95_approve_for_all,
            crate::routes::cep95::cep95_revoke_approval_for_all,
            crate::routes::cep95::cep95_mint,
            crate::routes::cep95::cep95_burn,
            crate::routes::cep95::cep95_transfer_ownership,
            crate::routes::cep95::cep95_name,
            crate::routes::cep95::cep95_symbol,
            crate::routes::cep95::cep95_total_supply,
            crate::routes::cep95::cep95_get_owner,
            crate::routes::cep95::cep95_owner_of,
            crate::routes::cep95::cep95_balance_of,
            crate::routes::cep95::cep95_get_approved,
            crate::routes::cep95::cep95_is_approved_for_all,
            crate::routes::cep95::cep95_token_metadata,
            crate::routes::cep95::cep95_bind_odra_install,
        );
    }

    #[cfg(feature = "swagger-ui")]
    {
        app = app.service(
            SwaggerUi::new("/docs/{_:.*}").url("/docs/ceps-openapi.json", openapi.clone()),
        );
    }
    #[cfg(not(feature = "swagger-ui"))]
    {
        let _ = openapi;
    }

    app
}

/// Bind and run the HTTP server.
pub async fn run_server(config: Config) -> std::io::Result<()> {
    let bind = config.bind_addr();
    let state = AppState::new(config);
    info!(
        version = crate::VERSION,
        %bind,
        rpc = %state.config.rpc_url,
        chain = %state.config.chain_name,
        "listening"
    );
    info!(ceps = ?state.config.enabled_ceps(), "compiled CEP features");
    // Custody mode stays in process logs only (never on GET /).
    info!(
        sign_backend = %state.config.sign_backend.as_str(),
        kms_url_set = state.config.kms_url_configured(),
        local_keyring_len = state.config.local_keys.len(),
        "signing"
    );
    if state.config.sign_backend.signs_for_callers() {
        warn!(
            sign_backend = %state.config.sign_backend.as_str(),
            "signing enabled: keep this listener on a private network (any caller who names a loaded public key can request a put signature)"
        );
    }
    if cfg!(feature = "swagger-ui") {
        info!(docs = %format!("http://{bind}/docs/"), "openapi");
    }

    HttpServer::new(move || create_app(state.clone()))
        .bind(&bind)?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn openapi_json_ok() {
        let app = test::init_service(create_app(AppState::new(Config::default()))).await;
        let req = test::TestRequest::get()
            .uri("/docs/ceps-openapi.json")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["info"]["title"], "ceps-rust-api");
        assert!(body["paths"].get("/health").is_some());
        #[cfg(feature = "chain-put")]
        assert!(body["paths"].get("/v1/chain/put-transaction").is_some());
        #[cfg(feature = "cep18")]
        {
            assert!(body["paths"].get("/v1/cep18/approve").is_some());
            assert!(body["paths"].get("/v1/cep18/mint").is_some());
            assert!(body["paths"]
                .get("/v1/cep18/{contract_hash}/security-badge/{account}")
                .is_some());
        }
        #[cfg(feature = "cep78")]
        {
            assert!(body["paths"]
                .get("/v1/cep78/{contract_hash}/ownership-mode")
                .is_some());
            assert!(body["paths"].get("/v1/cep78/owner-of-session").is_some());
        }
        #[cfg(feature = "cep85")]
        {
            assert!(body["paths"].get("/v1/cep85/balance-of-batch").is_some());
            assert!(body["paths"].get("/v1/cep85/transfer").is_some());
            assert!(body["paths"]
                .get("/v1/cep85/{contract_hash}/enable-burn")
                .is_some());
        }
        #[cfg(feature = "cep95")]
        {
            assert!(body["paths"]
                .get("/v1/cep95/{contract_hash}/get-owner")
                .is_some());
            assert!(body["paths"].get("/v1/cep95/transfer-ownership").is_some());
            assert!(body["paths"].get("/v1/cep95/mint").is_some());
        }
        let n = body["paths"].as_object().map(|m| m.len()).unwrap_or(0);
        // Path count depends on enabled features (CI verify-slices runs thin combos).
        let expected = 2usize // / + /health
            + usize::from(cfg!(feature = "chain-put"))
            + if cfg!(feature = "cep18") { 20 } else { 0 }
            + if cfg!(feature = "cep78") { 44 } else { 0 }
            + if cfg!(feature = "cep85") { 32 } else { 0 }
            + if cfg!(feature = "cep95") { 20 } else { 0 };
        assert_eq!(
            n, expected,
            "OpenAPI path count must match enabled features (got {n}, expected {expected})"
        );
        assert!(body["paths"].get("/v1/instances/{id}").is_none());
        assert!(body["paths"].get("/v1/wasm").is_none());
        assert!(body["paths"]
            .get("/v1/account/{public_key}/named-key/{name}")
            .is_none());
        #[cfg(feature = "cep18")]
        {
            let approve = &body["paths"]["/v1/cep18/approve"]["post"];
            let params = approve
                .get("parameters")
                .and_then(|p| p.as_array())
                .cloned()
                .unwrap_or_default();
            assert!(
                !params.is_empty(),
                "cep18 approve must expose query parameters"
            );
            assert!(
                approve.get("requestBody").is_none() || approve["requestBody"]["required"] == false,
                "cep18 approve must not require a JSON body"
            );
            let schemas = &body["components"]["schemas"];
            assert!(
                schemas.get("MutateQuery").is_some()
                    || schemas
                        .as_object()
                        .map(|m| m.keys().any(|k| k.contains("MutateQuery")))
                        .unwrap_or(false),
                "MutateQuery schema must be published"
            );
            assert!(
                schemas.get("MutateEnvelope").is_some(),
                "MutateEnvelope schema must be published"
            );
        }
    }

    #[actix_web::test]
    async fn docs_redirect_is_absolute() {
        let app = test::init_service(create_app(AppState::new(Config::default()))).await;
        let req = test::TestRequest::get().uri("/docs").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::TEMPORARY_REDIRECT
        );
        let loc = resp
            .headers()
            .get(actix_web::http::header::LOCATION)
            .and_then(|v| v.to_str().ok());
        assert_eq!(loc, Some("/docs/"));
    }

    #[cfg(feature = "cep18")]
    #[actix_web::test]
    async fn put_transfer_without_signer_is_no_signer() {
        let app = test::init_service(create_app(AppState::new(Config::default()))).await;
        let req = test::TestRequest::post()
            .uri("/v1/cep18/transfer?submit=put&signer=01aa&payment_amount=1&contract_hash=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa&recipient=account-hash-bb&amount=1")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["code"], "no_signer");
    }

    #[cfg(all(feature = "cep18", feature = "tx-return"))]
    #[actix_web::test]
    async fn return_transfer_make_only_ok() {
        let app = test::init_service(create_app(AppState::new(Config::default()))).await;
        let initiator = format!("01{}", "11".repeat(32));
        let uri = format!(
            "/v1/cep18/transfer?submit=return&signer={initiator}&payment_amount=1000000000&contract_hash=cfa781f5eb69c3eee952c2944ce9670a049f88c5e46b83fb5881ebe13fb98e6d&recipient=account-hash-b485c074cef7ccaccd0302949d2043ab7133abdb14cfa87e8392945c0bd80a5f&amount=1"
        );
        let req = test::TestRequest::post().uri(&uri).to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "status {}", resp.status());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("transaction").is_some());
    }
}
