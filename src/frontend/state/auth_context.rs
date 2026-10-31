use crate::api::auth::{get_current_user, Login, Logout};
use crate::domain::error::AppError;
use crate::domain::state::AuthState;
use crate::domain::user::User;
use leptos::prelude::*;

/// Contexto de autenticación global.
///
/// La sesión se obtiene con un `Resource` que se vuelve a pedir cada vez que
/// termina un login o un logout (`version()` de cada acción). A diferencia de
/// sincronizar un `RwSignal` con `Effect`s (que solo corren en el navegador),
/// un `Resource` se resuelve también durante el SSR: el HTML llega ya con el
/// usuario correcto y `<ProtectedRoute>` puede redirigir desde el servidor.
#[derive(Clone, Copy)]
pub struct AuthContext {
    pub login: ServerAction<Login>,
    pub logout: ServerAction<Logout>,
    pub session: Resource<Result<Option<User>, AppError>>,
}

impl AuthContext {
    /// Estado actual. Debe leerse dentro de `<Suspense/>` o `<Transition/>`.
    pub fn state(&self) -> AuthState {
        match self.session.get() {
            None => AuthState::Loading,
            Some(Ok(Some(user))) => AuthState::Authenticated(user),
            Some(_) => AuthState::Unauthenticated,
        }
    }

    pub fn user(&self) -> Option<User> {
        self.state().user().cloned()
    }

    /// `Some(true)` con sesión, `Some(false)` sin ella y `None` mientras carga.
    /// Es justo la firma que espera `condition` en `<ProtectedRoute/>`.
    pub fn is_authenticated(&self) -> Option<bool> {
        match self.state() {
            AuthState::Loading => None,
            state => Some(state.is_authenticated()),
        }
    }
}

/// Crea el contexto de autenticación y lo provee a todo el árbol.
/// Se llama una vez en la raíz de la aplicación.
pub fn provide_auth() -> AuthContext {
    let login = ServerAction::<Login>::new();
    let logout = ServerAction::<Logout>::new();

    let session = Resource::new(
        move || (login.version().get(), logout.version().get()),
        |_| get_current_user(),
    );

    let context = AuthContext {
        login,
        logout,
        session,
    };
    provide_context(context);
    context
}

/// Hook para obtener el `AuthContext` desde cualquier componente.
pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("AuthContext not found. Call provide_auth() in App.")
}
