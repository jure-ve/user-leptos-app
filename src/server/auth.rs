use crate::domain::user::{LoginCredentials, User, UserRole};
use crate::domain::validation::validate;
use crate::server::db::get_pool;
use axum_extra::extract::cookie::{Cookie, SameSite};
use leptos::prelude::*;
use leptos_axum::extract;
use uuid::Uuid;

const SESSION_COOKIE_NAME: &str = "session_id";
const SESSION_DURATION_DAYS: i64 = 7;

/// Server function to log in a user
#[server(Login, "/api")]
pub async fn login(credentials: LoginCredentials) -> Result<User, ServerFnError> {
    // Validate credentials on server side
    validate(&credentials).map_err(|e| ServerFnError::new(e))?;

    let pool = get_pool();

    // Find user by username
    let row: Option<(i64, String, String, String, String)> = sqlx::query_as(
        "SELECT id, username, email, password_hash, role FROM users WHERE username = ?",
    )
    .bind(&credentials.username)
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let (id, username, email, password_hash, role) = row
        .ok_or_else(|| ServerFnError::new("Usuario o contraseña incorrectos"))?;

    // Verify password
    let valid = bcrypt::verify(&credentials.password, &password_hash)
        .map_err(|_| ServerFnError::new("Error verificando contraseña"))?;

    if !valid {
        return Err(ServerFnError::new("Usuario o contraseña incorrectos"));
    }

    // Create session
    let session_id = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(SESSION_DURATION_DAYS);

    sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&session_id)
        .bind(id)
        .bind(expires_at.to_rfc3339())
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Error creando sesión: {}", e)))?;

    // Set cookie via leptos_axum response options
    let response = expect_context::<leptos_axum::ResponseOptions>();
    
    let cookie = Cookie::build((SESSION_COOKIE_NAME, session_id))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(SESSION_DURATION_DAYS))
        .build();
    
    response.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );

    Ok(User {
        id,
        username,
        email,
        role: UserRole::from_str(&role),
    })
}

/// Server function to log out
#[server(Logout, "/api")]
pub async fn logout() -> Result<(), ServerFnError> {
    // Get session cookie from request
    let cookie_jar: axum_extra::extract::CookieJar = extract().await?;
    
    if let Some(cookie) = cookie_jar.get(SESSION_COOKIE_NAME) {
        let session_id = cookie.value().to_string();
        
        // Delete session from database
        let pool = get_pool();
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(&session_id)
            .execute(pool)
            .await
            .ok();
    }

    // Remove cookie by setting expired
    let response = expect_context::<leptos_axum::ResponseOptions>();
    
    let cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::seconds(0))
        .build();
    
    response.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );

    Ok(())
}

/// Server function to get current authenticated user
#[server(GetCurrentUser, "/api")]
pub async fn get_current_user() -> Result<Option<User>, ServerFnError> {
    // Get session cookie from request
    let cookie_jar: axum_extra::extract::CookieJar = extract().await?;
    
    let session_id = match cookie_jar.get(SESSION_COOKIE_NAME) {
        Some(cookie) => cookie.value().to_string(),
        None => return Ok(None),
    };

    let pool = get_pool();

    // Find valid session
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT user_id FROM sessions WHERE id = ? AND expires_at > datetime('now')",
    )
    .bind(&session_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let user_id = match row {
        Some((id,)) => id,
        None => {
            // Session expired or invalid, clean up cookie
            let response = expect_context::<leptos_axum::ResponseOptions>();
            let cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
                .path("/")
                .http_only(true)
                .max_age(time::Duration::seconds(0))
                .build();
            response.insert_header(
                axum::http::header::SET_COOKIE,
                axum::http::HeaderValue::from_str(&cookie.to_string()).unwrap(),
            );
            return Ok(None);
        }
    };

    // Get user data
    let user_row: Option<(i64, String, String, String)> = sqlx::query_as(
        "SELECT id, username, email, role FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    match user_row {
        Some((id, username, email, role)) => Ok(Some(User {
            id,
            username,
            email,
            role: UserRole::from_str(&role),
        })),
        None => Ok(None),
    }
}
