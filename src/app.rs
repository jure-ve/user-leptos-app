use crate::frontend::auth::LoginPage;
use crate::frontend::layouts::MainLayout;
use crate::frontend::state::provide_auth;
use crate::frontend::users::UsersListPage;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{ProtectedRoute, Route, Router, Routes},
    SsrMode, StaticSegment,
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
    let auth = provide_auth();

    view! {
        <Stylesheet id="leptos" href="/pkg/user-leptos-app.css"/>
        <Title text="User App - Leptos"/>

        <Router>
            <MainLayout>
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=StaticSegment("") view=HomePage/>
                    // Solo para invitados: con sesión, redirige a /users.
                    <ProtectedRoute
                        path=StaticSegment("login")
                        view=LoginPage
                        condition=move || auth.is_authenticated().map(|logged_in| !logged_in)
                        redirect_path=|| "/users"
                        ssr=SsrMode::Async
                    />
                    // Requiere sesión. Con `SsrMode::Async` el servidor espera a
                    // resolver la sesión antes de enviar cabeceras, así que un
                    // visitante anónimo recibe un 302 real a /login, sin llegar
                    // a ver la página ni depender de que cargue el WASM.
                    //
                    // Ojo: esto es UX. Los datos están protegidos porque cada
                    // server function de `crate::api` exige sesión por su cuenta.
                    <ProtectedRoute
                        path=StaticSegment("users")
                        view=UsersListPage
                        condition=move || auth.is_authenticated()
                        redirect_path=|| "/login"
                        ssr=SsrMode::Async
                    />
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
