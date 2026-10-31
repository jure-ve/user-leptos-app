#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use leptos::logging::log;
    use leptos::prelude::*;
    use user_leptos_app::server::{app::build_router, db, session};

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data.db?mode=rwc".to_string());
    let pool = db::connect(&database_url)
        .await
        .expect("Failed to initialize database");
    let purged = session::delete_expired(&pool).await.unwrap_or(0);
    log!("Database initialized successfully ({purged} expired sessions purged)");

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let app = build_router(conf.leptos_options, pool);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
