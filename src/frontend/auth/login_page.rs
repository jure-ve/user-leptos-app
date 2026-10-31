use crate::frontend::auth::login_form::LoginForm;
use leptos::prelude::*;

/// Login page component.
///
/// La redirección a `/users` cuando ya hay sesión la hace `<ProtectedRoute/>`
/// en `app.rs`, tanto en el servidor como tras el login en el cliente.
#[component]
pub fn LoginPage() -> impl IntoView {
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
