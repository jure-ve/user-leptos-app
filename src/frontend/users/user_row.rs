use crate::domain::user::User;
use leptos::prelude::*;

/// Row component for displaying a user in the table
#[component]
pub fn UserRow(
    user: User,
    /// Si el usuario actual puede editar/eliminar (regla de `domain::policy`).
    #[prop(into)]
    can_manage: Signal<bool>,
    /// Si la fila corresponde al usuario autenticado.
    is_self: bool,
    on_edit: Callback<User>,
    on_delete: Callback<i64>,
) -> impl IntoView {
    let user_for_edit = user.clone();
    let user_id = user.id;
    let username = user.username.clone();
    let email = user.email.clone();
    let role_str = user.role.as_str().to_string();
    let role_class = format!("role-badge role-{}", user.role.as_str());

    view! {
        <tr class="user-row">
            <td class="user-id">{user.id}</td>
            <td class="user-username">
                {username}
                {is_self.then(|| view! { " (tú)" })}
            </td>
            <td class="user-email">{email}</td>
            <td class="user-role">
                <span class=role_class>
                    {role_str}
                </span>
            </td>
            <td class="user-actions">
                <Show
                    when=move || can_manage.get()
                    fallback=|| view! { <span class="text-muted">"Solo lectura"</span> }
                >
                    <button
                        class="btn btn-sm btn-secondary"
                        on:click={
                            let user_for_edit = user_for_edit.clone();
                            move |_| on_edit.run(user_for_edit.clone())
                        }
                    >
                        "Editar"
                    </button>
                    <button
                        class="btn btn-sm btn-danger"
                        // Coherente con `policy::can_delete_user`: nadie se borra a sí mismo.
                        disabled=is_self
                        title=if is_self { "No puedes eliminar tu propia cuenta" } else { "" }
                        on:click=move |_| on_delete.run(user_id)
                    >
                        "Eliminar"
                    </button>
                </Show>
            </td>
        </tr>
    }
}
