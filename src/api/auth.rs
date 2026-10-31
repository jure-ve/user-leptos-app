use crate::domain::error::AppError;
use crate::domain::user::{LoginCredentials, User};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::{
    domain::validation::validate,
    server::{db, guards, session, users},
};

/// Inicia sesión y deja la cookie de sesión en la respuesta.
#[server(prefix = "/api", endpoint = "login")]
pub async fn login(credentials: LoginCredentials) -> Result<User, AppError> {
    validate(&credentials).map_err(AppError::Validation)?;

    let pool = db::pool()?;
    let user =
        users::verify_credentials(&pool, &credentials.username, &credentials.password).await?;

    let session_id = session::create(&pool, user.id).await?;
    session::set_cookie(session_id);

    Ok(user)
}

/// Cierra la sesión actual (si existe) y borra la cookie.
#[server(prefix = "/api", endpoint = "logout")]
pub async fn logout() -> Result<(), AppError> {
    if let Some(session_id) = session::read_cookie().await {
        let pool = db::pool()?;
        session::delete(&pool, &session_id).await?;
    }
    session::clear_cookie();
    Ok(())
}

/// Usuario autenticado actual, o `None`. No exige sesión: la usa la UI para
/// saber si mostrar "Iniciar sesión" o el menú de usuario.
#[server(prefix = "/api", endpoint = "current_user")]
pub async fn get_current_user() -> Result<Option<User>, AppError> {
    guards::current_user().await
}

