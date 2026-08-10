//! Root hello (service identity + non-sensitive runtime snapshot).

use crate::config::enabled_cep_features;
use crate::features::CompiledFeatures;
use crate::state::AppState;
use crate::VERSION;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct HelloResult {
    /// Product name.
    pub name: String,
    pub version: String,
    pub framework: String,
    pub rpc_url: String,
    pub sse_url: String,
    pub chain_name: String,
    pub enabled_ceps: Vec<String>,
    pub features: CompiledFeatures,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
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
        name: "ceps-rust-api".to_string(),
        version: VERSION.to_string(),
        framework: "actix-web".to_string(),
        rpc_url: state.config.rpc_url.clone(),
        sse_url: state.config.sse_url.clone(),
        chain_name: state.config.chain_name.clone(),
        enabled_ceps,
        features: CompiledFeatures::current(),
        docs: if cfg!(feature = "swagger-ui") {
            Some("/docs/".into())
        } else {
            None
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::server::create_app;
    use actix_web::test;

    #[actix_web::test]
    async fn hello_reports_identity_not_custody() {
        let app = test::init_service(create_app(AppState::new(Config::default()))).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: HelloResult = test::read_body_json(resp).await;
        assert_eq!(body.name, "ceps-rust-api");
        assert_eq!(body.version, VERSION);
        assert!(!body.rpc_url.is_empty());
        assert!(!body.chain_name.is_empty());
        assert_eq!(body.features, CompiledFeatures::current());
        let raw: serde_json::Value = serde_json::to_value(&body).unwrap();
        assert!(raw.get("sign_backend").is_none());
        assert!(raw.get("kms_url_configured").is_none());
        assert!(raw.get("local_keys_loaded").is_none());
    }
}
