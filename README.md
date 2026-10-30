# User Leptos App

Aplicación full-stack con **Leptos 0.9** que demuestra autenticación con cookies, CRUD de usuarios, y arquitectura en capas.

## Características

- **Autenticación** - Login/logout con cookies de sesión y bcrypt
- **CRUD de Usuarios** - Crear, listar, editar y eliminar usuarios
- **SQLite** - Base de datos ligera con sqlx
- **SSR + Hidratación** - Server-side rendering con cliente WASM reactivo
- **UI Moderna** - Tema oscuro con diseño responsive
- **Validaciones** - Cliente y servidor con la crate `validator`

## Arquitectura

```
src/
├── domain/           # Modelos compartidos (isomórfico)
│   ├── user.rs       # User, LoginCredentials, etc.
│   ├── state.rs      # AuthState, FormState
│   └── validation.rs # Validaciones
│
├── server/           # Lógica del servidor (#[cfg(feature = "ssr")])
│   ├── db.rs         # SQLite pool, schema
│   ├── auth.rs       # Login, logout, sesiones
│   └── users.rs      # Operaciones CRUD
│
├── frontend/         # Componentes UI
│   ├── state/        # AuthContext (estado global)
│   ├── auth/         # Página y formulario de login
│   ├── users/        # Tabla, filas, modal CRUD
│   └── layouts/      # Navegación, layout principal
│
├── app.rs            # Router
├── lib.rs            # Exports
└── main.rs           # Entry point
```

## Inicio Rápido

### Prerrequisitos

```bash
# Instalar cargo-leptos
cargo install cargo-leptos

# Añadir target WASM
rustup target add wasm32-unknown-unknown
```

### Ejecutar

```bash
# Modo desarrollo con hot-reload
cargo leptos watch
```

Navegar a **http://127.0.0.1:3000**

### Credenciales por Defecto

| Usuario | Contraseña |
|---------|------------|
| `admin` | `admin123` |

## Comandos Útiles

```bash
# Verificar compilación SSR
cargo check --features ssr

# Verificar compilación cliente (WASM)
cargo check --features hydrate

# Build de producción
cargo leptos build --release

# Ejecutar tests
cargo test
```

## Dependencias Principales

| Crate | Uso |
|-------|-----|
| `leptos` 0.9 | Framework reactivo full-stack |
| `axum` | Servidor HTTP |
| `sqlx` | Conexión SQLite asíncrona |
| `bcrypt` | Hash de contraseñas |
| `validator` | Validación de datos |
| `axum-extra` | Cookies de sesión |

## Flujo de Autenticación

```mermaid
sequenceDiagram
    participant U as Usuario
    participant F as Frontend
    participant S as Server
    participant DB as SQLite

    U->>F: Ingresa credenciales
    F->>F: Validación cliente
    F->>S: login(credentials)
    S->>DB: SELECT user
    S->>S: bcrypt::verify()
    S->>DB: INSERT session
    S->>F: Set-Cookie + User
    F->>F: AuthContext → Authenticated
    F->>U: Redirige a /users
```

## Estructura de Base de Datos

```sql
-- Usuarios
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT DEFAULT 'user'
);

-- Sesiones
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    expires_at TEXT NOT NULL
);
```

## Rutas

| Ruta | Descripción |
|------|-------------|
| `/` | Página de inicio |
| `/login` | Formulario de login |
| `/users` | Gestión de usuarios (requiere auth) |

## Documentación y Recursos

- **Artículo:** [Arquitectura Full-Stack en Rust: Estructurando aplicaciones con Leptos sin morir en el intento](https://juredev.com/blog/2026/02/full-stack-en-rust-estructurando-aplicaciones-con-leptos/)
- **Documento de arquitectura:** [docs/arquitectura-leptos-fullstack.md](docs/arquitectura-leptos-fullstack.md)

## Licencia

MIT License - ver [LICENSE](LICENSE)
