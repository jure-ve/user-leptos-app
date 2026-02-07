use crate::domain::user::{CreateUserRequest, UpdateUserRequest, User, UserRole};
use crate::domain::validation::validate;
use crate::server::db::get_pool;
use leptos::prelude::*;

/// Server function to list all users
#[server(ListUsers, "/api")]
pub async fn list_users() -> Result<Vec<User>, ServerFnError> {
    let pool = get_pool();

    let rows: Vec<(i64, String, String, String)> = sqlx::query_as(
        "SELECT id, username, email, role FROM users ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let users = rows
        .into_iter()
        .map(|(id, username, email, role)| User {
            id,
            username,
            email,
            role: UserRole::from_str(&role),
        })
        .collect();

    Ok(users)
}

/// Server function to get a single user by ID
#[server(GetUser, "/api")]
pub async fn get_user(id: i64) -> Result<User, ServerFnError> {
    let pool = get_pool();

    let row: Option<(i64, String, String, String)> = sqlx::query_as(
        "SELECT id, username, email, role FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let (id, username, email, role) =
        row.ok_or_else(|| ServerFnError::new("Usuario no encontrado"))?;

    Ok(User {
        id,
        username,
        email,
        role: UserRole::from_str(&role),
    })
}

/// Server function to create a new user
#[server(CreateUser, "/api")]
pub async fn create_user(request: CreateUserRequest) -> Result<User, ServerFnError> {
    // Validate request
    validate(&request).map_err(|e| ServerFnError::new(e))?;

    let pool = get_pool();

    // Check if username already exists
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM users WHERE username = ? OR email = ?")
            .bind(&request.username)
            .bind(&request.email)
            .fetch_optional(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    if exists.is_some() {
        return Err(ServerFnError::new("El usuario o email ya existe"));
    }

    // Hash password
    let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)
        .map_err(|_| ServerFnError::new("Error procesando contraseña"))?;

    // Insert user
    let result = sqlx::query(
        "INSERT INTO users (username, email, password_hash, role) VALUES (?, ?, ?, ?)",
    )
    .bind(&request.username)
    .bind(&request.email)
    .bind(&password_hash)
    .bind(UserRole::User.as_str())
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Error creando usuario: {}", e)))?;

    Ok(User {
        id: result.last_insert_rowid(),
        username: request.username,
        email: request.email,
        role: UserRole::User,
    })
}

/// Server function to update an existing user
#[server(UpdateUser, "/api")]
pub async fn update_user(id: i64, request: UpdateUserRequest) -> Result<User, ServerFnError> {
    // Validate request
    validate(&request).map_err(|e| ServerFnError::new(e))?;

    let pool = get_pool();

    // Check if user exists
    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    if exists.is_none() {
        return Err(ServerFnError::new("Usuario no encontrado"));
    }

    // Check username/email uniqueness (excluding current user)
    let conflict: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM users WHERE id != ? AND (username = ? OR email = ?)",
    )
    .bind(id)
    .bind(&request.username)
    .bind(&request.email)
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    if conflict.is_some() {
        return Err(ServerFnError::new("El usuario o email ya existe"));
    }

    // Build update query
    let role = request.role.unwrap_or(UserRole::User);
    
    if let Some(password) = &request.password {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|_| ServerFnError::new("Error procesando contraseña"))?;

        sqlx::query("UPDATE users SET username = ?, email = ?, password_hash = ?, role = ? WHERE id = ?")
            .bind(&request.username)
            .bind(&request.email)
            .bind(&password_hash)
            .bind(role.as_str())
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Error actualizando usuario: {}", e)))?;
    } else {
        sqlx::query("UPDATE users SET username = ?, email = ?, role = ? WHERE id = ?")
            .bind(&request.username)
            .bind(&request.email)
            .bind(role.as_str())
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Error actualizando usuario: {}", e)))?;
    }

    Ok(User {
        id,
        username: request.username,
        email: request.email,
        role,
    })
}

/// Server function to delete a user
#[server(DeleteUser, "/api")]
pub async fn delete_user(id: i64) -> Result<(), ServerFnError> {
    let pool = get_pool();

    // Delete sessions first
    sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(id)
        .execute(pool)
        .await
        .ok();

    // Delete user
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Error eliminando usuario: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Usuario no encontrado"));
    }

    Ok(())
}
