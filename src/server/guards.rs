//! Guards de autenticación para usar al inicio de cada server function.
//!
//! ```ignore
//! #[server(prefix = "/api", endpoint = "delete_user")]
//! pub async fn delete_user(id: i64) -> Result<(), AppError> {
//!     let actor = require_user().await?;       // 401 si no hay sesión
//!     policy::can_delete_user(&actor, id)?;    // 403 si no tiene permiso
//!     ...
//! }
//! ```

use crate::domain::error::AppError;
use crate::domain::policy;
use crate::domain::user::User;
use crate::server::{db, session};

/// Usuario de la sesión actual, si la hay. Si la cookie apunta a una sesión
/// caducada o inexistente, la borra del navegador.
pub async fn current_user() -> Result<Option<User>, AppError> {
    let Some(session_id) = session::read_cookie().await else {
        return Ok(None);
    };

    let pool = db::pool()?;
    let user = session::find_user(&pool, &session_id).await?;
    if user.is_none() {
        session::clear_cookie();
    }
    Ok(user)
}

/// Exige una sesión válida. Devuelve `AppError::Unauthorized` (HTTP 401) si no la hay.
pub async fn require_user() -> Result<User, AppError> {
    current_user().await?.ok_or(AppError::Unauthorized)
}

/// Exige una sesión válida de administrador (401 sin sesión, 403 sin rol).
pub async fn require_admin() -> Result<User, AppError> {
    let user = require_user().await?;
    policy::can_manage_users(&user)?;
    Ok(user)
}

