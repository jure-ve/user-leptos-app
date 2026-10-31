use crate::domain::error::AppError;
use leptos::prelude::*;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

/// Crea el pool de conexiones, aplica el esquema y siembra el admin por defecto.
///
/// El pool ya no vive en un `static OnceLock`: se crea en `main` y se inyecta
/// en cada petición mediante el contexto de Leptos (ver `server::app`). Esto
/// permite que los tests usen su propia base de datos en memoria.
pub async fn connect(url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        // SQLite en memoria crea una BD distinta por conexión; con una sola
        // conexión los tests comparten la misma base de datos.
        .max_connections(if url.contains(":memory:") { 1 } else { 5 })
        .connect(url)
        .await?;

    migrate(&pool).await?;
    seed_admin(&pool).await?;
    Ok(pool)
}

/// Crea las tablas si no existen.
pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Nota: sqlx activa `PRAGMA foreign_keys` por defecto en cada conexión,
    // así que `ON DELETE CASCADE` funciona.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // `expires_at` se guarda en el formato nativo de SQLite
    // ('YYYY-MM-DD HH:MM:SS', UTC) para poder compararlo con datetime('now').
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            expires_at DATETIME NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn seed_admin(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let admin_exists: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM users WHERE username = 'admin'")
            .fetch_optional(pool)
            .await?;

    if admin_exists.is_none() {
        let password_hash =
            bcrypt::hash("admin123", bcrypt::DEFAULT_COST).expect("Failed to hash password");
        sqlx::query("INSERT INTO users (username, email, password_hash, role) VALUES (?, ?, ?, ?)")
            .bind("admin")
            .bind("admin@example.com")
            .bind(password_hash)
            .bind("admin")
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// Obtiene el pool desde el contexto de la petición actual.
pub fn pool() -> Result<SqlitePool, AppError> {
    use_context::<SqlitePool>().ok_or_else(|| {
        leptos::logging::error!("SqlitePool no encontrado en el contexto");
        AppError::Internal
    })
}

/// Los errores de base de datos nunca se envían tal cual al cliente: pueden
/// filtrar detalles del esquema. Se registran y se devuelve un error genérico,
/// salvo las violaciones de unicidad, que sí son un error "de negocio".
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let Some(db_err) = err.as_database_error() {
            if db_err.is_unique_violation() {
                return AppError::Conflict("El usuario o email ya existe".into());
            }
        }
        leptos::logging::error!("Database error: {err}");
        AppError::Internal
    }
}
