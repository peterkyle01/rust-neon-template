use serde::{Deserialize, Serialize};

/// Represents a user record from the Neon Data API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    // Add more fields as needed
}
