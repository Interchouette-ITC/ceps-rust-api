//! HTTP client for kms-secp256k1-api (sign-kms feature).
//!
//! Used only for signing puts. Key create/list stay on the KMS peer HTTP API.

use crate::error::ApiError;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct KmsClient {
    base: String,
    http: reqwest::Client,
}

impl KmsClient {
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base: base_url.into().trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn sign_transaction(
        &self,
        public_key: &str,
        transaction: &Value,
    ) -> Result<Value, ApiError> {
        let url = format!("{}/signTransaction?keys={public_key}", self.base);
        let resp = self
            .http
            .post(&url)
            .json(transaction)
            .send()
            .await
            .map_err(|e| ApiError::Kms(e.to_string()))?;
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::Kms(format!("signTransaction failed: {body}")));
        }
        resp.json()
            .await
            .map_err(|e| ApiError::Kms(format!("signTransaction json: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn sign_transaction_parses_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/signTransaction"))
            .and(query_param("keys", "0202aabb"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "transaction": {"signed": true}
            })))
            .mount(&server)
            .await;
        let client = KmsClient::new(server.uri());
        let signed = client
            .sign_transaction("0202aabb", &serde_json::json!({"tx": 1}))
            .await
            .unwrap();
        assert_eq!(signed["transaction"]["signed"], true);
    }
}
