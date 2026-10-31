use leptos::server_fn::codec::JsonEncoding;
use leptos::server_fn::error::{FromServerFnError, ServerFnErrorErr};
use http::StatusCode;
use serde::{Deserialize, Serialize};

/// Errores de la aplicación, compartidos entre servidor y cliente.
///
/// Al implementar [`FromServerFnError`], este tipo puede usarse directamente
/// como error de cualquier `#[server]`: viaja serializado como JSON y el
/// cliente recibe la *variante*, no un `String` opaco. Así la UI puede
/// distinguir un 401 de un error de validación.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, thiserror::Error)]
pub enum AppError {
    /// No hay sesión válida (HTTP 401).
    #[error("Debes iniciar sesión para continuar")]
    Unauthorized,
    /// Hay sesión, pero el usuario no tiene permiso (HTTP 403).
    #[error("{0}")]
    Forbidden(String),
    /// El recurso no existe (HTTP 404).
    #[error("{0}")]
    NotFound(String),
    /// Los datos enviados no son válidos (HTTP 422).
    #[error("{0}")]
    Validation(String),
    /// Conflicto con el estado actual, p. ej. usuario duplicado (HTTP 409).
    #[error("{0}")]
    Conflict(String),
    /// Error interno. El detalle se registra en el servidor y nunca se
    /// envía al cliente (HTTP 500).
    #[error("Error interno del servidor")]
    Internal,
    /// Error del transporte de las server functions (red, serialización...).
    #[error("Error de comunicación con el servidor: {0}")]
    ServerFn(String),
}

impl AppError {
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        AppError::ServerFn(value.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal | AppError::ServerFn(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

