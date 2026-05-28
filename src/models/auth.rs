use serde::{Deserialize, Serialize};

// ── Request types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignUpRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

// ── Response types ──

#[derive(Debug, Clone, Deserialize)]
pub struct AuthResponse {
    pub session: Session,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    // Add other session fields as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session: Option<Session>,
}
