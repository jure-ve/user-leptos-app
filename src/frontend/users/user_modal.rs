use crate::api::users::{create_user, update_user};
use crate::domain::user::{CreateUserRequest, UpdateUserRequest, User, UserRole};
use crate::domain::validation::validate;
use leptos::prelude::*;

#[derive(Clone)]
pub enum ModalMode {
    Create,
    Edit(User),
}

#[derive(Clone)]
struct CreateInput(CreateUserRequest);

#[derive(Clone)]
struct UpdateInput {
    id: i64,
    request: UpdateUserRequest,
}

/// Modal component for creating or editing a user
#[component]
pub fn UserModal(
    mode: ModalMode,
    /// Id del usuario autenticado, para no permitir que se quite su rol de admin.
    current_user_id: Option<i64>,
    on_save: Callback<Result<User, String>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let (initial_user, is_edit) = match &mode {
        ModalMode::Create => (None, false),
        ModalMode::Edit(user) => (Some(user.clone()), true),
    };

    let username = RwSignal::new(initial_user.as_ref().map(|u| u.username.clone()).unwrap_or_default());
    let email = RwSignal::new(initial_user.as_ref().map(|u| u.email.clone()).unwrap_or_default());
    let password = RwSignal::new(String::new());
    let role = RwSignal::new(initial_user.as_ref().map(|u| u.role.clone()).unwrap_or(UserRole::User));
    let error = RwSignal::new(Option::<String>::None);
    let submitting = RwSignal::new(false);

    let user_id = initial_user.as_ref().map(|u| u.id);
    // Misma regla que `policy::can_set_role`: el servidor la vuelve a comprobar.
    let editing_self = is_edit && user_id == current_user_id;

    // Create action
    let create_action = Action::new(move |input: &CreateInput| {
        let request = input.0.clone();
        async move { create_user(request).await }
    });

    // Update action
    let update_action = Action::new(move |input: &UpdateInput| {
        let id = input.id;
        let request = input.request.clone();
        async move { update_user(id, request).await }
    });

    // Watch create action result
    Effect::new(move |_| {
        if let Some(result) = create_action.value().get() {
            submitting.set(false);
            match result {
                Ok(user) => on_save.run(Ok(user)),
                Err(e) => error.set(Some(e.to_string())),
            }
        }
    });

    // Watch update action result
    Effect::new(move |_| {
        if let Some(result) = update_action.value().get() {
            submitting.set(false);
            match result {
                Ok(user) => on_save.run(Ok(user)),
                Err(e) => error.set(Some(e.to_string())),
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        error.set(None);
        submitting.set(true);

        let username_val = username.get();
        let email_val = email.get();
        let password_val = password.get();
        let role_val = role.get();

        if is_edit {
            let request = UpdateUserRequest {
                username: username_val,
                email: email_val,
                password: if password_val.is_empty() { None } else { Some(password_val) },
                role: Some(role_val),
            };

            if let Err(msg) = validate(&request) {
                error.set(Some(msg));
                submitting.set(false);
                return;
            }

            update_action.dispatch(UpdateInput {
                id: user_id.unwrap(),
                request,
            });
        } else {
            let request = CreateUserRequest {
                username: username_val,
                email: email_val,
                password: password_val,
            };

            if let Err(msg) = validate(&request) {
                error.set(Some(msg));
                submitting.set(false);
                return;
            }

            create_action.dispatch(CreateInput(request));
        }
    };

    view! {
        <div class="modal-overlay" on:click=move |_| on_close.run(())>
            <div class="modal" on:click=|ev| ev.stop_propagation()>
                <div class="modal-header">
                    <h2>{if is_edit { "Editar Usuario" } else { "Nuevo Usuario" }}</h2>
                    <button class="modal-close" on:click=move |_| on_close.run(())>"×"</button>
                </div>
                <form class="modal-body" on:submit=on_submit>
                    <div class="form-group">
                        <label for="modal-username">"Usuario"</label>
                        <input
                            type="text"
                            id="modal-username"
                            prop:value=move || username.get()
                            on:input=move |ev| username.set(event_target_value(&ev))
                            disabled=move || submitting.get()
                            required
                        />
                    </div>

                    <div class="form-group">
                        <label for="modal-email">"Email"</label>
                        <input
                            type="email"
                            id="modal-email"
                            prop:value=move || email.get()
                            on:input=move |ev| email.set(event_target_value(&ev))
                            disabled=move || submitting.get()
                            required
                        />
                    </div>

                    <div class="form-group">
                        <label for="modal-password">
                            {if is_edit { "Nueva Contraseña (dejar vacío para mantener)" } else { "Contraseña" }}
                        </label>
                        <input
                            type="password"
                            id="modal-password"
                            prop:value=move || password.get()
                            on:input=move |ev| password.set(event_target_value(&ev))
                            disabled=move || submitting.get()
                            required=!is_edit
                        />
                    </div>

                    {is_edit.then(|| view! {
                        <div class="form-group">
                            <label for="modal-role">"Rol"</label>
                            <select
                                id="modal-role"
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    role.set(UserRole::from_str(&value));
                                }
                                disabled=move || submitting.get() || editing_self
                            >
                                <option value="user" selected=move || role.get() == UserRole::User>"Usuario"</option>
                                <option value="admin" selected=move || role.get() == UserRole::Admin>"Administrador"</option>
                            </select>
                            {editing_self.then(|| view! {
                                <small class="text-muted">"No puedes cambiar tu propio rol"</small>
                            })}
                        </div>
                    })}

                    {move || error.get().map(|msg| view! {
                        <div class="form-error">{msg}</div>
                    })}

                    <div class="modal-footer">
                        <button
                            type="button"
                            class="btn btn-secondary"
                            on:click=move |_| on_close.run(())
                        >
                            "Cancelar"
                        </button>
                        <button
                            type="submit"
                            class="btn btn-primary"
                            disabled=move || submitting.get()
                        >
                            {move || if submitting.get() { "Guardando..." } else { "Guardar" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
