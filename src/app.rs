use crate::frontend::auth::LoginPage;
use crate::frontend::layouts::MainLayout;
use crate::frontend::users::UsersListPage;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="es">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/user-leptos-app.css"/>
        <Title text="User App - Leptos"/>

        <Router>
            <MainLayout>
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("login") view=LoginPage/>
                    <Route path=StaticSegment("users") view=UsersListPage/>
                </Routes>
            </MainLayout>
        </Router>
    }
}

/// Home page component
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="home-page">
            <div class="hero">
                <h1>"Bienvenido a User App"</h1>
                <p>"Sistema de gestión de usuarios con autenticación"</p>
                <div class="hero-actions">
                    <a href="/login" class="btn btn-primary">"Iniciar Sesión"</a>
                    <a href="/users" class="btn btn-secondary">"Ver Usuarios"</a>
                </div>
            </div>
            <div class="features">
                <div class="feature">
                    <h3>"🔐 Autenticación Segura"</h3>
                    <p>"Login con cookies de sesión y contraseñas hasheadas con bcrypt"</p>
                </div>
                <div class="feature">
                    <h3>"👥 CRUD de Usuarios"</h3>
                    <p>"Crear, leer, actualizar y eliminar usuarios fácilmente"</p>
                </div>
                <div class="feature">
                    <h3>"⚡ Full-Stack Reactivo"</h3>
                    <p>"Leptos + Axum con SSR e hidratación automática"</p>
                </div>
            </div>
        </div>
    }
}

/// 404 Not found page
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="not-found-page">
            <h1>"404"</h1>
            <p>"Página no encontrada"</p>
            <a href="/" class="btn btn-primary">"Volver al inicio"</a>
        </div>
    }
}
