//! Actix application factory and server.

use crate::config::Config;
use crate::middleware::cors::demo_cors;
use crate::openapi::ApiDoc;
use crate::routes::{health_handler, hello_handler};
use crate::state::AppState;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tracing::info;
use utoipa::OpenApi;

#[cfg(feature = "swagger-ui")]
use utoipa_swagger_ui::SwaggerUi;

async fn redirect_docs_absolute() -> impl Responder {
    HttpResponse::TemporaryRedirect()
        .append_header(("Location", "/docs/"))
        .finish()
}

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
    let openapi = ApiDoc::openapi();

    let mut app = App::new()
        .app_data(web::Data::new(state))
        .wrap(demo_cors())
        .service(hello_handler)
        .service(health_handler)
        .route("/docs", web::get().to(redirect_docs_absolute));

    app = svc!(
        app,
        crate::routes::chain::chain_balance,
        crate::routes::chain::chain_account,
        crate::routes::chain::chain_transaction,
        crate::routes::instances::list_instances,
        crate::routes::instances::register_instance,
        crate::routes::instances::get_instance,
        crate::routes::instances::delete_instance,
        crate::routes::wasm::list_wasm,
    );

    #[cfg(feature = "chain-put")]
    {
        app = app.service(crate::routes::chain::chain_put_transaction);
    }
    #[cfg(feature = "custody")]
    {
        app = app.service(crate::routes::chain::chain_fund);
        app = app.service(crate::routes::keys::keys_create);
    }
    #[cfg(feature = "sign-local")]
    {
        app = app.service(crate::routes::keys::keys_list);
    }
    #[cfg(feature = "sign-kms")]
    {
        app = app.service(crate::routes::keys::kms_create_key);
        app = app.service(crate::routes::keys::kms_list_keys);
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
            crate::routes::cep78::cep78_collection_name,
            crate::routes::cep78::cep78_collection_symbol,
            crate::routes::cep78::cep78_total_token_supply,
            crate::routes::cep78::cep78_number_of_minted_tokens,
            crate::routes::cep78::cep78_events_mode,
            crate::routes::cep78::cep78_owner_of,
            crate::routes::cep78::cep78_balance_of,
            crate::routes::cep78::cep78_is_approved_for_all,
            crate::routes::cep78::cep78_get_approved,
            crate::routes::cep78::cep78_metadata,
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
            crate::routes::cep85::cep85_collection_name,
            crate::routes::cep85::cep85_collection_uri,
            crate::routes::cep85::cep85_balance_of,
            crate::routes::cep85::cep85_supply_of,
            crate::routes::cep85::cep85_total_supply_of,
            crate::routes::cep85::cep85_uri,
            crate::routes::cep85::cep85_is_non_fungible,
            crate::routes::cep85::cep85_is_approved_for_all,
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
            crate::routes::cep95::cep95_name,
            crate::routes::cep95::cep95_symbol,
            crate::routes::cep95::cep95_total_supply,
            crate::routes::cep95::cep95_owner_of,
            crate::routes::cep95::cep95_balance_of,
            crate::routes::cep95::cep95_get_approved,
            crate::routes::cep95::cep95_is_approved_for_all,
            crate::routes::cep95::cep95_token_metadata,
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
    info!(%bind, "ceps-rust-api listening");
    info!(
        sign_backend = %state.config.sign_backend.as_str(),
        kms_url_configured = state.config.kms_url_configured(),
        "sign config"
    );
    info!(ceps = ?state.config.enabled_ceps(), "enabled CEP features");

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
            .uri("/v1/cep18/transfer")
            .set_json(serde_json::json!({
                "submit": "put",
                "signer": {"public_key": "01aa"},
                "payment_amount": "1",
                "contract_hash": "aa",
                "recipient": "account-hash-bb",
                "amount": "1"
            }))
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
        let req = test::TestRequest::post()
            .uri("/v1/cep18/transfer")
            .set_json(serde_json::json!({
                "submit": "return",
                "signer": {"public_key": initiator},
                "payment_amount": "1000000000",
                "contract_hash": "cfa781f5eb69c3eee952c2944ce9670a049f88c5e46b83fb5881ebe13fb98e6d",
                "recipient": "account-hash-b485c074cef7ccaccd0302949d2043ab7133abdb14cfa87e8392945c0bd80a5f",
                "amount": "1"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "status {}", resp.status());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("transaction").is_some());
    }
}
