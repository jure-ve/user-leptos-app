use crate::domain::state::AuthState;
use crate::frontend::state::use_auth;
use leptos::prelude::*;

/// Navigation component that adapts based on auth state
#[component]
pub fn Nav() -> impl IntoView {
    let auth = use_auth();

    let on_logout = move |_| {
        auth.logout_action.dispatch(());
    };

    view! {
        <nav class="main-nav">
            <div class="nav-brand">
                <a href="/">"User App"</a>
            </div>
            <div class="nav-links">
                {move || match auth.state.get() {
                    AuthState::Authenticated(user) => view! {
                        <span class="nav-user">"Hola, " {user.username}</span>
                        <a href="/users" class="nav-link">"Usuarios"</a>
                        <button class="btn btn-sm btn-secondary nav-btn" on:click=on_logout>
                            "Cerrar Sesión"
                        </button>
                    }.into_any(),
                    AuthState::Unauthenticated => view! {
                        <a href="/login" class="nav-link">"Iniciar Sesión"</a>
                    }.into_any(),
                    AuthState::Loading => view! {
                        <span class="nav-loading">"..."</span>
                    }.into_any(),
                }}
            </div>
        </nav>
    }
}
