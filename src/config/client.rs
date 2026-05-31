use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::Config;
use crate::response::AppError;
use utility_types::Omit;

// ── Auth request / response types ──

#[derive(Debug, Clone, Serialize, Deserialize, Omit)]
#[omit(arg(ident=SignInRequest, fields(name), derive(Debug, Clone, Serialize, Deserialize)))]
pub struct SignUpRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub expires_at: Option<u64>,
}

// ── Neon client ──

#[derive(Debug)]
pub struct NeonClient {
    http: Client,
    auth_url: String,
    data_api_url: String,
    jwt_token: Option<String>,
}

impl NeonClient {
    pub fn new(config: &Config) -> Self {
        Self {
            http: Client::new(),
            auth_url: config.auth_url.clone(),
            data_api_url: config.data_api_url.clone(),
            jwt_token: None,
        }
    }

    pub fn with_token(config: &Config, token: String) -> Self {
        Self {
            http: Client::new(),
            auth_url: config.auth_url.clone(),
            data_api_url: config.data_api_url.clone(),
            jwt_token: Some(token),
        }
    }

    #[allow(dead_code)]
    pub fn token(&self) -> Option<&str> {
        self.jwt_token.as_deref()
    }

    // ── Auth ──

    pub async fn sign_up(
        &mut self,
        email: String,
        name: String,
        password: String,
    ) -> Result<String, reqwest::Error> {
        let origin = origin_from_url(&self.auth_url);
        let response = self
            .http
            .post(format!("{}/sign-up/email", self.auth_url))
            .header("Origin", origin)
            .json(&SignUpRequest {
                email,
                name,
                password,
            })
            .send()
            .await?;
        let jwt = extract_jwt_from_response(&response);
        let status = response.status();
        let text = response.text().await.map_err(reqwest::Error::from)?;
        if !status.is_success() {
            tracing::warn!("sign_up status={} body={:?}", status, text);
        }
        self.jwt_token = jwt;
        self.get_session().await?;
        Ok(self.jwt_token.clone().unwrap_or_default())
    }

    pub async fn sign_in(
        &mut self,
        email: String,
        password: String,
    ) -> Result<String, reqwest::Error> {
        let origin = origin_from_url(&self.auth_url);
        let response = self
            .http
            .post(format!("{}/sign-in/email", self.auth_url))
            .header("Origin", origin)
            .json(&SignInRequest { email, password })
            .send()
            .await?;
        let jwt = extract_jwt_from_response(&response);
        let status = response.status();
        let text = response.text().await.map_err(reqwest::Error::from)?;
        if !status.is_success() {
            tracing::warn!("sign_in status={} body={:?}", status, text);
        }
        self.jwt_token = jwt;
        self.get_session().await?;
        Ok(self.jwt_token.clone().unwrap_or_default())
    }

    pub async fn get_session(&mut self) -> Result<Option<Session>, reqwest::Error> {
        let token = match &self.jwt_token {
            Some(t) => t.clone(),
            None => return Ok(None),
        };
        let response = self
            .http
            .get(format!("{}/get-session", self.auth_url))
            .header(
                "Cookie",
                format!("__Secure-neon-auth.session_token={}", token),
            )
            .send()
            .await?;
        // Extract the JWT from the set-auth-jwt response header
        let jwt = response
            .headers()
            .get("set-auth-jwt")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let status = response.status();
        let text = response.text().await.map_err(reqwest::Error::from)?;
        if !status.is_success() {
            tracing::warn!("get_session status={} body={:?}", status, text);
        }
        if let Some(jwt) = jwt {
            self.jwt_token = Some(jwt);
        }
        let session = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| {
                v.get("session")
                    .or_else(|| v.pointer("/data/session"))
                    .cloned()
            })
            .and_then(|s| serde_json::from_value::<Session>(s).ok());
        Ok(session)
    }

    pub async fn sign_out(&mut self) -> Result<(), reqwest::Error> {
        let token = match &self.jwt_token {
            Some(t) => t.clone(),
            None => return Ok(()),
        };
        self.http
            .post(format!("{}/sign-out", self.auth_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        self.jwt_token = None;
        Ok(())
    }

    // ── Generic Data API CRUD ──

    pub async fn create<T: serde::de::DeserializeOwned>(
        &self,
        resource: &str,
        body: impl Serialize,
    ) -> Result<Vec<T>, anyhow::Error> {
        let url = format!("{}/{}", self.data_api_url, resource);
        Ok(self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.bearer_token()?))
            .header("Prefer", "return=representation")
            .json(&body)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_all<T: serde::de::DeserializeOwned>(
        &self,
        resource: &str,
    ) -> Result<Vec<T>, anyhow::Error> {
        let url = format!("{}/{}", self.data_api_url, resource);
        let response = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.bearer_token()?))
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            tracing::warn!("get_all({}) status={} body={:?}", resource, status, text);
        }
        Ok(serde_json::from_str(&text)?)
    }

    pub async fn get_one<T: serde::de::DeserializeOwned>(
        &self,
        resource: &str,
        id: i32,
    ) -> Result<Option<T>, anyhow::Error> {
        let url = format!("{}/{}?id=eq.{}", self.data_api_url, resource, id);
        let response = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.bearer_token()?))
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            tracing::warn!("get_one({}) status={} body={:?}", resource, status, text);
        }
        let mut records: Vec<T> = serde_json::from_str(&text)?;
        Ok(records.pop())
    }

    pub async fn update<T: serde::de::DeserializeOwned>(
        &self,
        resource: &str,
        id: i32,
        body: impl Serialize,
    ) -> Result<Vec<T>, anyhow::Error> {
        let url = format!("{}/{}?id=eq.{}", self.data_api_url, resource, id);
        Ok(self
            .http
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.bearer_token()?))
            .header("Prefer", "return=representation")
            .json(&body)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn delete(&self, resource: &str, id: i32) -> Result<(), anyhow::Error> {
        let url = format!("{}/{}?id=eq.{}", self.data_api_url, resource, id);
        self.http
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.bearer_token()?))
            .send()
            .await?;
        Ok(())
    }

    fn bearer_token(&self) -> Result<&str, anyhow::Error> {
        self.jwt_token
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("not authenticated"))
    }
}

/// Extract the origin (scheme + authority) from a URL string.
fn origin_from_url(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        format!("{}://{}", parsed.scheme(), parsed.authority())
    } else {
        String::new()
    }
}

/// Extract the full JWT from the `Set-Cookie` header of a sign-in/up response.
///
/// The cookie `__Secure-neon-auth.session_token` contains the real JWT
/// (`session_id.signature`), while the body `token` is just the session ID
/// without the signature and is not accepted by the Data API.
fn extract_jwt_from_response(response: &reqwest::Response) -> Option<String> {
    let cookie = response.headers().get("Set-Cookie")?.to_str().ok()?;
    // Find the cookie value for __Secure-neon-auth.session_token
    let value = cookie
        .split(';')
        .next()?
        .strip_prefix("__Secure-neon-auth.session_token=")?;
    // URL-decode the value (the signature part may be URL-encoded)
    let decoded = urlencoding::decode(value).ok()?;
    Some(decoded.into_owned())
}

// ── Axum extractor ──

impl<S> FromRequestParts<S> for NeonClient
where
    S: Send + Sync,
    Arc<Config>: FromRef<S>,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let config = Arc::from_ref(state);
        let result = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::Unauthorized("missing or invalid Authorization header".into()))
            .map(|token| NeonClient::with_token(&config, token));
        async move { result }
    }
}
