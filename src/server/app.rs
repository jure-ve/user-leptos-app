use crate::app::{shell, App};
use axum::Router;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use sqlx::SqlitePool;

/// Construye el router de Axum con las rutas de Leptos y las server functions.
///
/// El pool se inyecta en el contexto de **cada** petición (tanto de SSR como
/// de server functions) con `leptos_routes_with_context`. Vive aquí, y no en
/// `main.rs`, para que los tests de integración usen exactamente el mismo router.
pub fn build_router(leptos_options: LeptosOptions, pool: SqlitePool) -> Router {
    let routes = generate_route_list(App);

    Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || provide_context(pool.clone()),
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
}
