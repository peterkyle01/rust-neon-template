use crate::config::Config;
use crate::models::auth::{AuthResponse, SessionResponse, SignInRequest, SignUpRequest};
use reqwest::Client;

/// Client for interacting with the Neon Auth API.
///
/// Manages JWT tokens and provides methods for the standard
/// authentication flow: sign-up, sign-in, session management, and sign-out.
pub struct NeonAuthClient {
    http: Client,
    auth_url: String,
    jwt_token: Option<String>,
}

impl NeonAuthClient {
    /// Create a new client. Loads configuration from the environment.
    pub fn new(config: &Config) -> Self {
        Self {
            http: Client::new(),
            auth_url: config.auth_url.clone(),
            jwt_token: None,
        }
    }

    /// Create a new client with a pre-existing token (e.g. from a request header).
    pub fn with_token(config: &Config, token: String) -> Self {
        Self {
            http: Client::new(),
            auth_url: config.auth_url.clone(),
            jwt_token: Some(token),
        }
    }

    /// The current JWT token, if any.
    pub fn token(&self) -> Option<&str> {
        self.jwt_token.as_deref()
    }

    /// Register a new user with email + password.
    pub async fn sign_up(
        &mut self,
        email: String,
        name: String,
        password: String,
    ) -> Result<AuthResponse, reqwest::Error> {
        let response = self
            .http
            .post(format!("{}/api/auth/sign-up/email", self.auth_url))
            .json(&SignUpRequest {
                email,
                name,
                password,
            })
            .send()
            .await?;

        let auth_response: AuthResponse = response.json().await?;
        self.jwt_token = Some(auth_response.session.token.clone());
        Ok(auth_response)
    }

    /// Sign in an existing user with email + password.
    pub async fn sign_in(
        &mut self,
        email: String,
        password: String,
    ) -> Result<AuthResponse, reqwest::Error> {
        let response = self
            .http
            .post(format!("{}/api/auth/sign-in/email", self.auth_url))
            .json(&SignInRequest { email, password })
            .send()
            .await?;

        let auth_response: AuthResponse = response.json().await?;
        self.jwt_token = Some(auth_response.session.token.clone());
        Ok(auth_response)
    }

    /// Retrieve the current session. Refreshes the stored token if a new one is returned.
    pub async fn get_session(&mut self) -> Result<Option<SessionResponse>, reqwest::Error> {
        let token = match &self.jwt_token {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        let response = self
            .http
            .get(format!("{}/api/auth/get-session", self.auth_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let session_response: SessionResponse = response.json().await?;
        if let Some(session) = &session_response.session {
            self.jwt_token = Some(session.token.clone());
        }
        Ok(Some(session_response))
    }

    /// Sign out the current session and clear the stored token.
    pub async fn sign_out(&mut self) -> Result<(), reqwest::Error> {
        let token = match &self.jwt_token {
            Some(t) => t.clone(),
            None => return Ok(()),
        };

        self.http
            .post(format!("{}/api/auth/sign-out", self.auth_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        self.jwt_token = None;
        Ok(())
    }
}
