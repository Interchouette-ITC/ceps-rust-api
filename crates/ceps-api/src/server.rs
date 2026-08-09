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

/// Absolute redirect so `/docs` resolves to `/docs/` (avoids relative `docs/` hack).
async fn redirect_docs_absolute() -> impl Responder {
    HttpResponse::TemporaryRedirect()
        .append_header(("Location", "/docs/"))
        .finish()
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
///
/// # Errors
/// Returns I/O errors if the address cannot be bound or the server fails.
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
        assert!(
            resp.status().is_success(),
            "openapi status {}",
            resp.status()
        );
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["info"]["title"], "ceps-rust-api");
        assert!(body["paths"].get("/health").is_some());
    }

    #[actix_web::test]
    async fn docs_ui_ok() {
        let app = test::init_service(create_app(AppState::new(Config::default()))).await;
        let req = test::TestRequest::get().uri("/docs/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(
            resp.status().is_success(),
            "docs UI status {}",
            resp.status()
        );
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
}
