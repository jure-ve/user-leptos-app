//! Sesiones basadas en cookie: creación, búsqueda, borrado y la cookie en sí.

use crate::domain::error::AppError;
use crate::domain::user::{User, UserRole};
use axum_extra::extract::cookie::{Cookie, SameSite};
use leptos::prelude::*;
use sqlx::SqlitePool;
use uuid::Uuid;

pub const SESSION_COOKIE_NAME: &str = "session_id";
pub const SESSION_DURATION_DAYS: i64 = 7;

/// Crea una sesión para el usuario y devuelve su id.
pub async fn create(pool: &SqlitePool, user_id: i64) -> Result<String, AppError> {
    // Aprovechamos cada login para purgar sesiones caducadas.
    delete_expired(pool).await?;

    let session_id = Uuid::new_v4().to_string();
    // La fecha la calcula SQLite para que el formato coincida con datetime('now').
    // (Antes se guardaba en RFC 3339, "2026-02-01T10:00:00+00:00", y la
    // comparación como texto con "2026-02-01 12:00:00" fallaba dentro del
    // mismo día porque 'T' > ' '.)
    sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, datetime('now', ?))")
        .bind(&session_id)
        .bind(user_id)
        .bind(format!("+{SESSION_DURATION_DAYS} days"))
        .execute(pool)
        .await?;
    Ok(session_id)
}

/// Devuelve el usuario dueño de una sesión vigente.
pub async fn find_user(pool: &SqlitePool, session_id: &str) -> Result<Option<User>, AppError> {
    let row: Option<(i64, String, String, String)> = sqlx::query_as(
        r#"
        SELECT u.id, u.username, u.email, u.role
        FROM sessions s
        JOIN users u ON u.id = s.user_id
        WHERE s.id = ? AND s.expires_at > datetime('now')
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, username, email, role)| User {
        id,
        username,
        email,
        role: UserRole::from_str(&role),
    }))
}

pub async fn delete(pool: &SqlitePool, session_id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_for_user(pool: &SqlitePool, user_id: i64) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_expired(pool: &SqlitePool) -> Result<u64, AppError> {
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at <= datetime('now')")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

// --- Cookie -----------------------------------------------------------------

/// Lee el id de sesión de la cookie de la petición actual.
pub async fn read_cookie() -> Option<String> {
    let jar: axum_extra::extract::CookieJar = leptos_axum::extract().await.ok()?;
    jar.get(SESSION_COOKIE_NAME).map(|c| c.value().to_string())
}

/// Escribe la cookie de sesión en la respuesta.
pub fn set_cookie(session_id: String) {
    write_cookie(session_id, time::Duration::days(SESSION_DURATION_DAYS));
}

/// Borra la cookie de sesión del navegador.
pub fn clear_cookie() {
    write_cookie(String::new(), time::Duration::ZERO);
}

fn write_cookie(value: String, max_age: time::Duration) {
    let cookie = Cookie::build((SESSION_COOKIE_NAME, value))
        .max_age(max_age)
        .path("/")
        // Inaccesible desde JavaScript: un XSS no puede robar la sesión.
        .http_only(true)
        .same_site(SameSite::Lax)
        // En release la cookie solo viaja por HTTPS. En desarrollo
        // (http://127.0.0.1) `Secure` impediría que el navegador la guarde.
        .secure(!cfg!(debug_assertions))
        .build();

    if let (Some(response), Ok(value)) = (
        use_context::<leptos_axum::ResponseOptions>(),
        axum::http::HeaderValue::from_str(&cookie.to_string()),
    ) {
        response.append_header(axum::http::header::SET_COOKIE, value);
    }
}
