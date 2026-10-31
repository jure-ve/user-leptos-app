use serde::{Deserialize, Serialize};
use validator::Validate;

/// Represents a user in the system
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: UserRole,
}

/// User roles for authorization
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum UserRole {
    Admin,
    #[default]
    User,
}

impl UserRole {
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            _ => UserRole::User,
        }
    }
}

/// Credentials for user login
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct LoginCredentials {
    #[validate(length(min = 3, message = "Usuario debe tener al menos 3 caracteres"))]
    pub username: String,
    #[validate(length(min = 6, message = "Contraseña debe tener al menos 6 caracteres"))]
    pub password: String,
}

/// Request payload for creating a new user
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, message = "Usuario debe tener al menos 3 caracteres"))]
    pub username: String,
    #[validate(email(message = "Email inválido"))]
    pub email: String,
    #[validate(length(min = 6, message = "Contraseña debe tener al menos 6 caracteres"))]
    pub password: String,
}

/// Request payload for updating an existing user
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, message = "Usuario debe tener al menos 3 caracteres"))]
    pub username: String,
    #[validate(email(message = "Email inválido"))]
    pub email: String,
    /// Optional: only update password if provided
    #[validate(length(min = 6, message = "Contraseña debe tener al menos 6 caracteres"))]
    pub password: Option<String>,
    pub role: Option<UserRole>,
}
