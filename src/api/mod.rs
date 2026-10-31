//! Server functions de la aplicación: **la frontera de seguridad**.
//!
//! Cada `#[server]` se convierte en un endpoint HTTP público
//! (`POST /api/<endpoint>`), aunque en el código parezca una llamada local.
//! Por eso todas siguen el mismo patrón:
//!
//! 1. Autenticación: `require_user()` / `require_admin()`  → 401
//! 2. Autorización: reglas de `domain::policy`              → 403
//! 3. Validación de entrada                                 → 422
//! 4. Delegar en el repositorio (`server::*`)
//!
//! Este módulo es isomórfico: en el cliente (WASM) el cuerpo de cada función
//! se reemplaza por una petición HTTP, así que puede referenciar `crate::server`
//! sin que ese código llegue al bundle.

pub mod auth;
pub mod users;

