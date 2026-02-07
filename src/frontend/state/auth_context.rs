use crate::domain::state::AuthState;
use crate::domain::user::{LoginCredentials, User};
use leptos::prelude::*;

// Server function for login
#[server(Login, "/api")]
pub async fn login(credentials: LoginCredentials) -> Result<User, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::login(credentials).await
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("SSR only"))
    }
}

// Server function for logout
#[server(Logout, "/api")]
pub async fn logout() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::logout().await
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("SSR only"))
    }
}

// Server function to get current user
#[server(GetCurrentUser, "/api")]
pub async fn get_current_user() -> Result<Option<User>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::auth::get_current_user().await
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("SSR only"))
    }
}

/// Authentication context that lives at the root of the app
#[derive(Clone)]
pub struct AuthContext {
    pub state: RwSignal<AuthState>,
    pub login_action: Action<LoginCredentials, Result<User, ServerFnError>>,
    pub logout_action: Action<(), Result<(), ServerFnError>>,
}

impl AuthContext {
    pub fn is_authenticated(&self) -> bool {
        self.state.get().is_authenticated()
    }

    pub fn user(&self) -> Option<User> {
        self.state.get().user().cloned()
    }
}

/// Hook to get the AuthContext from any component
pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("AuthContext not found. Wrap your app in AuthProvider.")
}

/// Provider component that wraps the app and provides AuthContext
#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let state = RwSignal::new(AuthState::Loading);

    // Action for login
    let login_action = Action::new(move |credentials: &LoginCredentials| {
        let credentials = credentials.clone();
        async move { login(credentials).await }
    });

    // Action for logout
    let logout_action = Action::new(move |_: &()| async move { logout().await });

    // Effect to update state when login succeeds
    Effect::new(move |_| {
        if let Some(Ok(user)) = login_action.value().get() {
            state.set(AuthState::Authenticated(user));
        } else if let Some(Err(_)) = login_action.value().get() {
            state.set(AuthState::Unauthenticated);
        }
    });

    // Effect to update state when logout succeeds
    Effect::new(move |_| {
        if let Some(Ok(())) = logout_action.value().get() {
            state.set(AuthState::Unauthenticated);
        }
    });

    // Check session on mount using Resource
    let initial_session = Resource::new(|| (), |_| async move { get_current_user().await });

    Effect::new(move |_| {
        if let Some(result) = initial_session.get() {
            match result {
                Ok(Some(user)) => state.set(AuthState::Authenticated(user)),
                _ => state.set(AuthState::Unauthenticated),
            }
        }
    });

    let context = AuthContext {
        state,
        login_action,
        logout_action,
    };

    provide_context(context);

    children()
}
