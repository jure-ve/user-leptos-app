use crate::frontend::layouts::nav::Nav;
use crate::frontend::state::AuthProvider;
use leptos::prelude::*;

/// Main layout wrapper with navigation
#[component]
pub fn MainLayout(children: Children) -> impl IntoView {
    view! {
        <AuthProvider>
            <div class="app-container">
                <Nav />
                <main class="main-content">
                    {children()}
                </main>
                <footer class="main-footer">
                    <p>"© 2026 Jure | User App Leptos"</p>
                </footer>
            </div>
        </AuthProvider>
    }
}
