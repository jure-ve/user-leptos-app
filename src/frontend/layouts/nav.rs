use crate::api::auth::Logout;
use crate::domain::state::AuthState;
use crate::frontend::state::use_auth;
use leptos::prelude::*;

/// Navigation component that adapts based on auth state
#[component]
pub fn Nav() -> impl IntoView {
    let auth = use_auth();

    let on_logout = move |_| {
        auth.logout.dispatch(Logout {});
    };

    view! {
        <nav class="main-nav">
            <div class="nav-brand">
                <a href="/">"User App"</a>
            </div>
            <div class="nav-links">
                // `auth.state()` lee un Resource: dentro de <Transition/> el
                // SSR espera la sesión y se evita el parpadeo al hidratar.
                <Transition fallback=|| view! { <span class="nav-loading">"..."</span> }>
                    {move || match auth.state() {
                        AuthState::Authenticated(user) => view! {
                            <span class="nav-user">
                                "Hola, " {user.username.clone()}
                                {user.role.is_admin().then(|| view! {
                                    <span class="role-badge role-admin">"admin"</span>
                                })}
                            </span>
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
                </Transition>
            </div>
        </nav>
    }
}
