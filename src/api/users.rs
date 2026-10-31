use crate::domain::error::AppError;
use crate::domain::user::{CreateUserRequest, UpdateUserRequest, User};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::{
    domain::{policy, validation::validate},
    server::{db, guards, session, users},
};

/// Lista de usuarios. Requiere sesión.
#[server(prefix = "/api", endpoint = "list_users")]
pub async fn list_users() -> Result<Vec<User>, AppError> {
    let actor = guards::require_user().await?;
    policy::can_view_users(&actor)?;

    users::list(&db::pool()?).await
}

/// Crea un usuario. Solo administradores.
#[server(prefix = "/api", endpoint = "create_user")]
pub async fn create_user(request: CreateUserRequest) -> Result<User, AppError> {
    guards::require_admin().await?;
    validate(&request).map_err(AppError::Validation)?;

    users::create(&db::pool()?, request).await
}

/// Actualiza un usuario. Solo administradores, y nadie puede quitarse a sí
/// mismo el rol de administrador.
#[server(prefix = "/api", endpoint = "update_user")]
pub async fn update_user(id: i64, request: UpdateUserRequest) -> Result<User, AppError> {
    let actor = guards::require_admin().await?;
    if let Some(role) = &request.role {
        policy::can_set_role(&actor, id, role)?;
    }
    validate(&request).map_err(AppError::Validation)?;

    users::update(&db::pool()?, id, request).await
}

/// Elimina un usuario y sus sesiones. Solo administradores, y no a sí mismos.
#[server(prefix = "/api", endpoint = "delete_user")]
pub async fn delete_user(id: i64) -> Result<(), AppError> {
    let actor = guards::require_admin().await?;
    policy::can_delete_user(&actor, id)?;

    let pool = db::pool()?;
    // Cerramos sus sesiones de inmediato (ON DELETE CASCADE también lo haría).
    session::delete_for_user(&pool, id).await?;
    users::delete(&pool, id).await
}

