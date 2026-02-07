use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::sync::OnceLock;

/// Global database pool
static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

/// Initialize the database pool
/// Should be called once at application startup
pub async fn init_db() -> Result<(), sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:data.db?mode=rwc")
        .await?;

    // Run migrations to create tables
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
    .execute(&pool)
    .await?;

    // Create sessions table for cookie-based auth
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
    .execute(&pool)
    .await?;

    // Insert default admin user if not exists
    let admin_exists: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM users WHERE username = 'admin'")
            .fetch_optional(&pool)
            .await?;

    if admin_exists.is_none() {
        let password_hash = bcrypt::hash("admin123", bcrypt::DEFAULT_COST)
            .expect("Failed to hash password");
        sqlx::query(
            "INSERT INTO users (username, email, password_hash, role) VALUES (?, ?, ?, ?)",
        )
        .bind("admin")
        .bind("admin@example.com")
        .bind(password_hash)
        .bind("admin")
        .execute(&pool)
        .await?;
    }

    DB_POOL.set(pool).expect("Database already initialized");
    Ok(())
}

/// Get the database pool
/// Panics if called before init_db()
pub fn get_pool() -> &'static SqlitePool {
    DB_POOL.get().expect("Database not initialized. Call init_db() first.")
}
