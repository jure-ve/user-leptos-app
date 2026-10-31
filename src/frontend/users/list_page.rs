use crate::api::users::{delete_user, list_users};
use crate::domain::error::AppError;
use crate::domain::policy;
use crate::domain::user::User;
use crate::frontend::state::use_auth;
use crate::frontend::users::user_modal::{ModalMode, UserModal};
use crate::frontend::users::user_row::UserRow;
use leptos::prelude::*;

/// Users list page with CRUD operations.
///
/// La ruta está envuelta en `<ProtectedRoute/>` (ver `app.rs`), así que aquí
/// ya no hace falta un `Effect` que redirija a `/login`.
#[component]
pub fn UsersListPage() -> impl IntoView {
    let auth = use_auth();

    // ¿Puede el usuario actual gestionar usuarios? Se usa la MISMA regla que
    // aplica el servidor (`domain::policy`), solo para adaptar la UI.
    let can_manage = Memo::new(move |_| {
        auth.user()
            .map(|u| policy::can_manage_users(&u).is_ok())
            .unwrap_or(false)
    });
    let current_user_id = Memo::new(move |_| auth.user().map(|u| u.id));

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

    // Error de la última acción (p. ej. un 403 devuelto por el servidor)
    let action_error = RwSignal::new(Option::<String>::None);

    // Delete action
    let delete_action = Action::new(move |id: &i64| {
        let id = *id;
        async move { delete_user(id).await }
    });

    // Watch delete action and refresh
    Effect::new(move |_| match delete_action.value().get() {
        Some(Ok(())) => {
            action_error.set(None);
            refresh_trigger.update(|v| *v += 1);
        }
        Some(Err(e)) => action_error.set(Some(e.to_string())),
        None => {}
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
                <Transition fallback=|| ()>
                    {move || can_manage.get().then(|| view! {
                        <button class="btn btn-primary" on:click=open_create_modal>
                            "+ Nuevo Usuario"
                        </button>
                    })}
                </Transition>
            </div>

            {move || action_error.get().map(|msg| view! {
                <div class="form-error">
                    <span class="error-icon">"⚠"</span>
                    {msg}
                </div>
            })}

            <div class="users-table-container">
                <Suspense fallback=move || view! { <div class="loading">"Cargando usuarios..."</div> }>
                    {move || {
                        users.get().map(|result: Result<Vec<User>, AppError>| {
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
                                                            let is_self = current_user_id.get_untracked() == Some(user.id);
                                                            view! {
                                                                <UserRow
                                                                    user=user
                                                                    can_manage=can_manage
                                                                    is_self=is_self
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
                                // Con errores tipados la UI puede reaccionar a cada caso.
                                Err(AppError::Unauthorized) => view! {
                                    <div class="error-state">
                                        <p>"Tu sesión ha expirado. "</p>
                                        <a href="/login" class="btn btn-primary">"Iniciar sesión"</a>
                                    </div>
                                }.into_any(),
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
                                current_user_id=current_user_id.get_untracked()
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
