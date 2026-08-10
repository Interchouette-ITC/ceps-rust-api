//! HTTP client for kms-secp256k1-api (sign-kms feature).

use crate::error::ApiError;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct KmsClient {
    base: String,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyResponse {
    pub public_key: String,
    #[serde(default)]
    pub address: Option<String>,
}

impl KmsClient {
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base: base_url.into().trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn create_key(&self) -> Result<CreateKeyResponse, ApiError> {
        let url = format!("{}/createKey", self.base);
        let resp = self
            .http
            .post(&url)
            .send()
            .await
            .map_err(|e| ApiError::Kms(e.to_string()))?;
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::Kms(format!("createKey failed: {body}")));
        }
        resp.json()
            .await
            .map_err(|e| ApiError::Kms(format!("createKey json: {e}")))
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

    pub async fn list_keys(&self) -> Result<Value, ApiError> {
        let url = format!("{}/listKeys", self.base);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::Kms(e.to_string()))?;
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::Kms(format!("listKeys failed: {body}")));
        }
        resp.json()
            .await
            .map_err(|e| ApiError::Kms(format!("listKeys json: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn create_key_parses_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/createKey"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "public_key": "0202aabb",
                "address": "account-hash-dead"
            })))
            .mount(&server)
            .await;
        let client = KmsClient::new(server.uri());
        let key = client.create_key().await.unwrap();
        assert_eq!(key.public_key, "0202aabb");
    }
}
