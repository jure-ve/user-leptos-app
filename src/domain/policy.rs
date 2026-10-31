//! Reglas de autorización como funciones puras.
//!
//! Este módulo es isomórfico: el **servidor** lo usa para *imponer* las
//! reglas (seguridad) y el **cliente** lo usa para *reflejarlas* en la UI
//! (ocultar botones, deshabilitar campos). Una sola fuente de verdad.
//!
//! Importante: que el cliente oculte un botón es solo UX. La seguridad real
//! está en que cada server function llame a estas reglas antes de actuar.

use super::error::AppError;
use super::user::{User, UserRole};

/// Cualquier usuario autenticado puede ver el listado.
pub fn can_view_users(_actor: &User) -> Result<(), AppError> {
    Ok(())
}

/// Solo los administradores pueden crear, editar o eliminar usuarios.
pub fn can_manage_users(actor: &User) -> Result<(), AppError> {
    match actor.role {
        UserRole::Admin => Ok(()),
        UserRole::User => Err(AppError::forbidden(
            "Solo los administradores pueden gestionar usuarios",
        )),
    }
}

/// Un administrador no puede eliminarse a sí mismo (evita quedarse sin admins).
pub fn can_delete_user(actor: &User, target_id: i64) -> Result<(), AppError> {
    can_manage_users(actor)?;
    if actor.id == target_id {
        return Err(AppError::forbidden("No puedes eliminar tu propia cuenta"));
    }
    Ok(())
}

/// Un administrador no puede quitarse su propio rol de administrador.
pub fn can_set_role(actor: &User, target_id: i64, new_role: &UserRole) -> Result<(), AppError> {
    can_manage_users(actor)?;
    if actor.id == target_id && *new_role != UserRole::Admin {
        return Err(AppError::forbidden(
            "No puedes quitarte tu propio rol de administrador",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(id: i64, role: UserRole) -> User {
        User {
            id,
            username: format!("user{id}"),
            email: format!("user{id}@example.com"),
            role,
        }
    }

    #[test]
    fn regular_users_cannot_manage_users() {
        let actor = user(2, UserRole::User);
        assert!(matches!(can_manage_users(&actor), Err(AppError::Forbidden(_))));
        assert!(can_delete_user(&actor, 3).is_err());
        assert!(can_set_role(&actor, 3, &UserRole::User).is_err());
    }

    #[test]
    fn admins_can_manage_other_users() {
        let admin = user(1, UserRole::Admin);
        assert!(can_manage_users(&admin).is_ok());
        assert!(can_delete_user(&admin, 2).is_ok());
        assert!(can_set_role(&admin, 2, &UserRole::Admin).is_ok());
        assert!(can_set_role(&admin, 2, &UserRole::User).is_ok());
    }

    #[test]
    fn admins_cannot_delete_themselves() {
        let admin = user(1, UserRole::Admin);
        assert!(matches!(can_delete_user(&admin, 1), Err(AppError::Forbidden(_))));
    }

    #[test]
    fn admins_cannot_demote_themselves() {
        let admin = user(1, UserRole::Admin);
        assert!(can_set_role(&admin, 1, &UserRole::Admin).is_ok());
        assert!(matches!(
            can_set_role(&admin, 1, &UserRole::User),
            Err(AppError::Forbidden(_))
        ));
    }
}

