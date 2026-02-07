use crate::domain::state::AuthState;
use crate::domain::user::User;
use crate::frontend::state::use_auth;
use crate::frontend::users::user_modal::{ModalMode, UserModal};
use crate::frontend::users::user_row::UserRow;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

// Server function for listing users
#[server(ListUsers, "/api")]
pub async fn list_users() -> Result<Vec<User>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::users::list_users().await
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("SSR only"))
    }
}

// Server function for deleting users
#[server(DeleteUser, "/api")]
pub async fn delete_user(id: i64) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::users::delete_user(id).await
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("SSR only"))
    }
}

/// Users list page with CRUD operations
#[component]
pub fn UsersListPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    // Redirect to login if not authenticated
    Effect::new(move |_| {
        if let AuthState::Unauthenticated = auth.state.get() {
            navigate("/login", Default::default());
        }
    });

    // Refresh trigger
    let refresh_trigger = RwSignal::new(0);

    // Fetch users
    let users = Resource::new(
        move || refresh_trigger.get(),
        |_| async move { list_users().await },
    );

    // Modal state
    let show_modal = RwSignal::new(false);
    let modal_mode = RwSignal::new(Option::<ModalMode>::None);

    // Delete action
    let delete_action = Action::new(move |id: &i64| {
        let id = *id;
        async move { delete_user(id).await }
    });

    // Watch delete action and refresh
    Effect::new(move |_| {
        if let Some(Ok(())) = delete_action.value().get() {
            refresh_trigger.update(|v| *v += 1);
        }
    });

    // Open create modal
    let open_create_modal = move |_| {
        modal_mode.set(Some(ModalMode::Create));
        show_modal.set(true);
    };

    // Open edit modal
    let on_edit = Callback::new(move |user: User| {
        modal_mode.set(Some(ModalMode::Edit(user)));
        show_modal.set(true);
    });

    // Delete user
    let on_delete = Callback::new(move |id: i64| {
        delete_action.dispatch(id);
    });

    // Modal save callback
    let on_modal_save = Callback::new(move |_result: Result<User, String>| {
        show_modal.set(false);
        modal_mode.set(None);
        refresh_trigger.update(|v| *v += 1);
    });

    // Modal close callback
    let on_modal_close = Callback::new(move |_| {
        show_modal.set(false);
        modal_mode.set(None);
    });

    view! {
        <div class="users-page">
            <div class="page-header">
                <h1>"Gestión de Usuarios"</h1>
                <button class="btn btn-primary" on:click=open_create_modal>
                    "+ Nuevo Usuario"
                </button>
            </div>

            <div class="users-table-container">
                <Suspense fallback=move || view! { <div class="loading">"Cargando usuarios..."</div> }>
                    {move || {
                        users.get().map(|result: Result<Vec<User>, ServerFnError>| {
                            match result {
                                Ok(user_list) => {
                                    if user_list.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <p>"No hay usuarios registrados."</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <table class="users-table">
                                                <thead>
                                                    <tr>
                                                        <th>"ID"</th>
                                                        <th>"Usuario"</th>
                                                        <th>"Email"</th>
                                                        <th>"Rol"</th>
                                                        <th>"Acciones"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    <For
                                                        each=move || user_list.clone()
                                                        key=|user| user.id
                                                        children=move |user| {
                                                            view! {
                                                                <UserRow
                                                                    user=user
                                                                    on_edit=on_edit
                                                                    on_delete=on_delete
                                                                />
                                                            }
                                                        }
                                                    />
                                                </tbody>
                                            </table>
                                        }.into_any()
                                    }
                                }
                                Err(e) => view! {
                                    <div class="error-state">
                                        <p>"Error cargando usuarios: " {e.to_string()}</p>
                                    </div>
                                }.into_any()
                            }
                        })
                    }}
                </Suspense>
            </div>

            {move || {
                show_modal.get().then(|| {
                    modal_mode.get().map(|mode| {
                        view! {
                            <UserModal
                                mode=mode
                                on_save=on_modal_save
                                on_close=on_modal_close
                            />
                        }
                    })
                }).flatten()
            }}
        </div>
    }
}
