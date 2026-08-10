//! WASM listing under CEPS_WASM_ROOT.

use crate::error::ApiError;
use crate::state::AppState;
use actix_web::{get, web, HttpResponse};
use serde::Serialize;
use std::path::Path;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct WasmEntry {
    pub path: String,
    pub bytes: u64,
}

#[utoipa::path(
    get,
    path = "/v1/wasm",
    responses((status = 200, description = "Available WASM files", body = [WasmEntry])),
    tag = "Instances"
)]
#[get("/v1/wasm")]
pub async fn list_wasm(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let root = &state.config.wasm_root;
    if !root.exists() {
        return Ok(HttpResponse::Ok().json(Vec::<WasmEntry>::new()));
    }
    let mut out = Vec::new();
    collect_wasm(root, root, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(HttpResponse::Ok().json(out))
}

fn collect_wasm(root: &Path, dir: &Path, out: &mut Vec<WasmEntry>) -> Result<(), ApiError> {
    let entries = std::fs::read_dir(dir).map_err(|e| ApiError::Internal(e.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|e| ApiError::Internal(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_wasm(root, &path, out)?;
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("wasm"))
        {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let meta = entry
                .metadata()
                .map_err(|e| ApiError::Internal(e.to_string()))?;
            out.push(WasmEntry {
                path: rel,
                bytes: meta.len(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::server::create_app;
    use actix_web::test;

    #[actix_web::test]
    async fn list_wasm_ok() {
        let app = test::init_service(create_app(AppState::new(Config::default()))).await;
        let req = test::TestRequest::get().uri("/v1/wasm").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
