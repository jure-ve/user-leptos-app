use crate::domain::user::LoginCredentials;
use crate::domain::validation::validate;
use crate::frontend::state::use_auth;
use leptos::prelude::*;

/// Login form component with validation and error handling
#[component]
pub fn LoginForm() -> impl IntoView {
    let auth = use_auth();

    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let submitting = RwSignal::new(false);

    // Watch for login action result
    Effect::new(move |_| {
        let pending = auth.login_action.pending().get();
        submitting.set(pending);

        if let Some(result) = auth.login_action.value().get() {
            if let Err(e) = result {
                error.set(Some(e.to_string()));
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        error.set(None);

        let credentials = LoginCredentials {
            username: username.get(),
            password: password.get(),
        };

        // Validate on client side first
        if let Err(msg) = validate(&credentials) {
            error.set(Some(msg));
            return;
        }

        auth.login_action.dispatch(credentials);
    };

    view! {
        <form class="login-form" on:submit=on_submit>
            <div class="form-group">
                <label for="username">"Usuario"</label>
                <input
                    type="text"
                    id="username"
                    name="username"
                    placeholder="Ingrese su usuario"
                    prop:value=move || username.get()
                    on:input=move |ev| username.set(event_target_value(&ev))
                    disabled=move || submitting.get()
                    required
                />
            </div>

            <div class="form-group">
                <label for="password">"Contraseña"</label>
                <input
                    type="password"
                    id="password"
                    name="password"
                    placeholder="Ingrese su contraseña"
                    prop:value=move || password.get()
                    on:input=move |ev| password.set(event_target_value(&ev))
                    disabled=move || submitting.get()
                    required
                />
            </div>

            {move || error.get().map(|msg| view! {
                <div class="form-error">
                    <span class="error-icon">"⚠"</span>
                    {msg}
                </div>
            })}

            <button type="submit" class="btn btn-primary" disabled=move || submitting.get()>
                {move || if submitting.get() { "Iniciando sesión..." } else { "Iniciar Sesión" }}
            </button>
        </form>
    }
}
