use crate::domain::state::AuthState;
use crate::frontend::auth::login_form::LoginForm;
use crate::frontend::state::use_auth;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// Login page component
#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    // Redirect to users page if already authenticated
    Effect::new(move |_| {
        if let AuthState::Authenticated(_) = auth.state.get() {
            navigate("/users", Default::default());
        }
    });

    view! {
        <div class="login-page">
            <div class="login-container">
                <div class="login-header">
                    <h1>"Bienvenido"</h1>
                    <p>"Inicia sesión para continuar"</p>
                </div>
                <LoginForm />
            </div>
        </div>
    }
}
