//! Root hello.

use crate::config::enabled_cep_features;
use crate::features::CompiledFeatures;
use crate::state::AppState;
use crate::VERSION;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct HelloResult {
    pub message: String,
    pub version: String,
    pub framework: String,
    pub enabled_ceps: Vec<String>,
    pub features: CompiledFeatures,
    pub sign_backend: String,
    pub kms_url_configured: bool,
}

#[utoipa::path(
    get,
    path = "/",
    responses((status = 200, description = "API hello", body = HelloResult)),
    tag = "Health"
)]
#[get("/")]
pub async fn hello_handler(state: web::Data<AppState>) -> impl Responder {
    let enabled_ceps = enabled_cep_features()
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    HttpResponse::Ok().json(HelloResult {
        message: "ceps-rust-api".to_string(),
        version: VERSION.to_string(),
        framework: "actix-web".to_string(),
        enabled_ceps,
        features: CompiledFeatures::current(),
        sign_backend: state.config.sign_backend.as_str().to_string(),
        kms_url_configured: state.config.kms_url_configured(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::server::create_app;
    use actix_web::test;

    #[actix_web::test]
    async fn hello_defaults_to_sign_backend_none() {
        let app = test::init_service(create_app(AppState::new(Config::default()))).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: HelloResult = test::read_body_json(resp).await;
        assert_eq!(body.sign_backend, "none");
        assert!(!body.kms_url_configured);
        assert_eq!(body.features, CompiledFeatures::current());
    }
}
