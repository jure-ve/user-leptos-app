//! Repositorio de usuarios: acceso a datos puro, sin autenticación ni
//! autorización. Esas decisiones se toman en `crate::api`, antes de llamar aquí.

use crate::domain::error::AppError;
use crate::domain::user::{CreateUserRequest, UpdateUserRequest, User, UserRole};
use sqlx::SqlitePool;
use std::sync::LazyLock;

type UserRow = (i64, String, String, String);

fn to_user((id, username, email, role): UserRow) -> User {
    User {
        id,
        username,
        email,
        role: UserRole::from_str(&role),
    }
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<User>, AppError> {
    let rows: Vec<UserRow> =
        sqlx::query_as("SELECT id, username, email, role FROM users ORDER BY id")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(to_user).collect())
}

pub async fn find(pool: &SqlitePool, id: i64) -> Result<User, AppError> {
    let row: Option<UserRow> =
        sqlx::query_as("SELECT id, username, email, role FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    row.map(to_user)
        .ok_or_else(|| AppError::NotFound("Usuario no encontrado".into()))
}

/// Hash usado cuando el usuario no existe, para que `verify_credentials`
/// tarde lo mismo exista o no el usuario (evita enumerar usuarios midiendo
/// el tiempo de respuesta).
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    bcrypt::hash("dummy-password", bcrypt::DEFAULT_COST).expect("bcrypt hash")
});

/// Verifica usuario y contraseña. Devuelve el mismo error en ambos casos de fallo.
pub async fn verify_credentials(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<User, AppError> {
    let row: Option<(i64, String, String, String, String)> = sqlx::query_as(
        "SELECT id, username, email, password_hash, role FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    let hash = row
        .as_ref()
        .map(|r| r.3.clone())
        .unwrap_or_else(|| DUMMY_HASH.clone());
    let valid = bcrypt::verify(password, &hash).unwrap_or(false);

    match row {
        Some((id, username, email, _, role)) if valid => Ok(to_user((id, username, email, role))),
        _ => Err(AppError::Validation(
            "Usuario o contraseña incorrectos".into(),
        )),
    }
}

pub async fn create(pool: &SqlitePool, request: CreateUserRequest) -> Result<User, AppError> {
    let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)
        .map_err(|_| AppError::Internal)?;

    // La restricción UNIQUE de la tabla es la fuente de verdad: si choca,
    // `From<sqlx::Error>` lo convierte en `AppError::Conflict`.
    let result =
        sqlx::query("INSERT INTO users (username, email, password_hash, role) VALUES (?, ?, ?, ?)")
            .bind(&request.username)
            .bind(&request.email)
            .bind(&password_hash)
            .bind(UserRole::User.as_str())
            .execute(pool)
            .await?;

    Ok(User {
        id: result.last_insert_rowid(),
        username: request.username,
        email: request.email,
        role: UserRole::User,
    })
}

/// Actualiza un usuario. `role: None` conserva el rol actual (antes lo
/// degradaba silenciosamente a `User`).
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    request: UpdateUserRequest,
) -> Result<User, AppError> {
    let current = find(pool, id).await?;
    let role = request.role.unwrap_or(current.role);

    match &request.password {
        Some(password) => {
            let password_hash =
                bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|_| AppError::Internal)?;
            sqlx::query(
                "UPDATE users SET username = ?, email = ?, password_hash = ?, role = ? WHERE id = ?",
            )
            .bind(&request.username)
            .bind(&request.email)
            .bind(&password_hash)
            .bind(role.as_str())
            .bind(id)
            .execute(pool)
            .await?;
        }
        None => {
            sqlx::query("UPDATE users SET username = ?, email = ?, role = ? WHERE id = ?")
                .bind(&request.username)
                .bind(&request.email)
                .bind(role.as_str())
                .bind(id)
                .execute(pool)
                .await?;
        }
    }

    Ok(User {
        id,
        username: request.username,
        email: request.email,
        role,
    })
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Usuario no encontrado".into()));
    }
    Ok(())
}
