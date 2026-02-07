use super::user::User;
use serde::{Deserialize, Serialize};

/// Authentication state for the application
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum AuthState {
    /// Initial loading state while checking session
    #[default]
    Loading,
    /// User is not authenticated
    Unauthenticated,
    /// User is authenticated with their data
    Authenticated(User),
}

impl AuthState {
    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        matches!(self, AuthState::Authenticated(_))
    }

    /// Get the authenticated user if present
    pub fn user(&self) -> Option<&User> {
        match self {
            AuthState::Authenticated(user) => Some(user),
            _ => None,
        }
    }
}

/// Generic form submission state
#[derive(Clone, Debug, PartialEq, Default)]
pub enum FormState<T> {
    #[default]
    Idle,
    Submitting,
    Success(T),
    Error(String),
}

impl<T> FormState<T> {
    pub fn is_submitting(&self) -> bool {
        matches!(self, FormState::Submitting)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, FormState::Error(_))
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            FormState::Error(msg) => Some(msg),
            _ => None,
        }
    }
}
