//! Tests de integración de autorización.
//!
//! Atacan el router real de Axum (`build_router`) con peticiones HTTP, igual
//! que lo haría `curl`, sin pasar por la UI. Así comprobamos que la seguridad
//! vive en el servidor y no en los botones que oculta el frontend.
//!
//! Ejecutar con: `cargo test --features ssr`

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use leptos::prelude::LeptosOptions;
use sqlx::SqlitePool;
use tower::ServiceExt;
use user_leptos_app::server::{app::build_router, db};

const FORM: &str = "application/x-www-form-urlencoded";

struct TestApp {
    router: Router,
    pool: SqlitePool,
}

impl TestApp {
    /// App con una BD en memoria que ya contiene `admin` (id 1) y un
    /// usuario normal `alice` (id 2).
    async fn new() -> Self {
        let pool = db::connect("sqlite::memory:").await.expect("db");
        let hash = bcrypt::hash("alice123", 4).unwrap();
        sqlx::query("INSERT INTO users (username, email, password_hash, role) VALUES ('alice', 'alice@example.com', ?, 'user')")
            .bind(hash)
            .execute(&pool)
            .await
            .unwrap();

        let options = LeptosOptions::builder()
            .output_name("user-leptos-app")
            .build();
        let router = build_router(options, pool.clone());
        Self { router, pool }
    }

    /// `POST /api/<endpoint>` con un cuerpo form-urlencoded y cookie opcional.
    async fn post(&self, endpoint: &str, body: &str, cookie: Option<&str>) -> (StatusCode, String, Option<String>) {
        let mut req = Request::post(format!("/api/{endpoint}")).header(header::CONTENT_TYPE, FORM);
        if let Some(cookie) = cookie {
            req = req.header(header::COOKIE, cookie);
        }
        let res = self
            .router
            .clone()
            .oneshot(req.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap();

        let status = res.status();
        let set_cookie = res
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find(|v| v.starts_with("session_id=") && !v.starts_with("session_id=;"))
            .map(|v| v.split(';').next().unwrap().to_string());
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8_lossy(&bytes).into_owned(), set_cookie)
    }

    /// Hace login y devuelve la cookie `session_id=...`.
    async fn login(&self, username: &str, password: &str) -> String {
        let body = format!("credentials[username]={username}&credentials[password]={password}");
        let (status, body, cookie) = self.post("login", &body, None).await;
        assert_eq!(status, StatusCode::OK, "login falló: {body}");
        cookie.expect("el login debe devolver la cookie de sesión")
    }

    async fn user_count(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }
}

// --- Sin sesión: 401 --------------------------------------------------------

#[tokio::test]
async fn anonymous_cannot_list_users() {
    let app = TestApp::new().await;
    let (status, body, _) = app.post("list_users", "", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body.contains("Unauthorized"), "body: {body}");
}

#[tokio::test]
async fn anonymous_cannot_delete_users() {
    let app = TestApp::new().await;
    let (status, _, _) = app.post("delete_user", "id=2", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(app.user_count().await, 2, "no se debe haber borrado nada");
}

#[tokio::test]
async fn anonymous_cannot_create_users() {
    let app = TestApp::new().await;
    let body = "request[username]=mallory&request[email]=m@example.com&request[password]=hacked1";
    let (status, _, _) = app.post("create_user", body, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(app.user_count().await, 2);
}

#[tokio::test]
async fn forged_session_cookie_is_rejected() {
    let app = TestApp::new().await;
    let (status, _, _) = app
        .post("list_users", "", Some("session_id=00000000-0000-0000-0000-000000000000"))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn expired_session_is_rejected() {
    let app = TestApp::new().await;
    let cookie = app.login("alice", "alice123").await;
    sqlx::query("UPDATE sessions SET expires_at = datetime('now', '-1 minute')")
        .execute(&app.pool)
        .await
        .unwrap();

    let (status, _, _) = app.post("list_users", "", Some(&cookie)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// --- Usuario normal: puede leer, no gestionar (403) -------------------------

#[tokio::test]
async fn regular_user_can_list_users() {
    let app = TestApp::new().await;
    let cookie = app.login("alice", "alice123").await;
    let (status, body, _) = app.post("list_users", "", Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("admin@example.com"));
}

#[tokio::test]
async fn regular_user_cannot_delete_users() {
    let app = TestApp::new().await;
    let cookie = app.login("alice", "alice123").await;
    let (status, body, _) = app.post("delete_user", "id=1", Some(&cookie)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body.contains("Forbidden"), "body: {body}");
    assert_eq!(app.user_count().await, 2);
}

#[tokio::test]
async fn regular_user_cannot_promote_themselves() {
    let app = TestApp::new().await;
    let cookie = app.login("alice", "alice123").await;
    let body = "id=2&request[username]=alice&request[email]=alice@example.com&request[role]=Admin";
    let (status, _, _) = app.post("update_user", body, Some(&cookie)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let role: String = sqlx::query_scalar("SELECT role FROM users WHERE id = 2")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(role, "user");
}

// --- Administrador ----------------------------------------------------------

#[tokio::test]
async fn admin_can_delete_other_users() {
    let app = TestApp::new().await;
    let cookie = app.login("admin", "admin123").await;
    let (status, body, _) = app.post("delete_user", "id=2", Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(app.user_count().await, 1);
}

#[tokio::test]
async fn deleting_a_user_revokes_their_sessions() {
    let app = TestApp::new().await;
    let alice = app.login("alice", "alice123").await;
    let admin = app.login("admin", "admin123").await;

    let (status, _, _) = app.post("delete_user", "id=2", Some(&admin)).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _, _) = app.post("list_users", "", Some(&alice)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_cannot_delete_themselves() {
    let app = TestApp::new().await;
    let cookie = app.login("admin", "admin123").await;
    let (status, _, _) = app.post("delete_user", "id=1", Some(&cookie)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_cannot_demote_themselves() {
    let app = TestApp::new().await;
    let cookie = app.login("admin", "admin123").await;
    let body = "id=1&request[username]=admin&request[email]=admin@example.com&request[role]=User";
    let (status, _, _) = app.post("update_user", body, Some(&cookie)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn duplicate_user_returns_conflict() {
    let app = TestApp::new().await;
    let cookie = app.login("admin", "admin123").await;
    let body = "request[username]=alice&request[email]=otra@example.com&request[password]=secret1";
    let (status, _, _) = app.post("create_user", body, Some(&cookie)).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

// --- Login ------------------------------------------------------------------

#[tokio::test]
async fn wrong_password_and_unknown_user_return_the_same_error() {
    let app = TestApp::new().await;
    let (s1, b1, c1) = app
        .post("login", "credentials[username]=admin&credentials[password]=incorrecta", None)
        .await;
    let (s2, b2, c2) = app
        .post("login", "credentials[username]=nadie&credentials[password]=incorrecta", None)
        .await;
    assert_eq!(s1, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!((s1, &b1), (s2, &b2), "no debe revelar si el usuario existe");
    assert!(c1.is_none() && c2.is_none());
}

#[tokio::test]
async fn logout_invalidates_the_session() {
    let app = TestApp::new().await;
    let cookie = app.login("alice", "alice123").await;
    let (status, _, _) = app.post("logout", "", Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _, _) = app.post("list_users", "", Some(&cookie)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

