# Technology Stack

## Repository Layout

The project is a single Rust workspace. Backend logic, frontend UI, and shared domain code live
together in one crate, compiled with feature flags that gate server-only or client-only code.
`cargo-leptos` builds and orchestrates both targets; a `Dockerfile` and `docker-compose.yml` at the
root orchestrate deployment.

```
api-mock-server/
├── src/
│   ├── main.rs                   # Entry point (ssr only) — wires Axum, starts HTTP server
│   ├── lib.rs                    # Leptos app root — shared by ssr and hydrate builds
│   ├── domain/                   # Entities, value objects, domain errors (no framework imports)
│   ├── application/              # Use cases, repository interfaces, DTOs
│   ├── adapters/
│   │   ├── http/                 # Axum handlers for /api/* and /mocks/* (ssr only)
│   │   └── ui/                   # Leptos components and pages (shared)
│   └── infrastructure/           # SQLx repos, config loading (ssr only)
├── style/                        # Global CSS / Tailwind entry point
├── public/                       # Static assets served directly
├── migrations/                   # Versioned SQL migration files (sqlx-cli)
├── Cargo.toml                    # Workspace + crate manifest
├── Cargo.lock
├── Makefile                      # make dev, make build, make test, make migrate
├── Dockerfile
├── docker-compose.yml            # Production-like compose (built image)
├── docker-compose.override.yml   # Local dev overrides (hot reload via cargo-leptos)
├── .env.example                  # Environment variable template
├── docs/
└── README.md
```

---

## Technology Choices

| Technology | Role |
|---|---|
| [Rust](https://www.rust-lang.org/) | Primary language. Memory-safe, zero-cost abstractions, single binary output. |
| [Leptos](https://leptos.dev/) | Full-stack reactive UI framework. Components compile to WASM (client) and render on Axum (server). Replaces both React and the chi-based API layer for the admin UI. |
| [Axum](https://github.com/tokio-rs/axum) | HTTP server framework (used by Leptos SSR integration). Handles `/api/*` management routes and `/mocks/*` mock-serving routes directly. |
| [SQLx](https://github.com/launchbadge/sqlx) | Async database library with compile-time query verification. Supports SQLite3, PostgreSQL, and MariaDB. Replaces GORM. |
| [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) | Build tool and dev server. Compiles the WASM frontend and the Axum backend, serves them together, and provides hot reload in development. |

---

## Architecture

### Build Targets and Feature Flags

Leptos uses Cargo feature flags to separate code that runs only on the server from code that runs
only in the browser (WASM):

| Feature | Where it runs | What it includes |
|---|---|---|
| `ssr` | Axum server (native binary) | Axum handlers, SQLx repos, config loading, server functions |
| `hydrate` | Browser (WASM) | Client-side Leptos hydration, browser-only interactivity |

Shared code (domain entities, use cases, Leptos component tree) compiles under both features.
`src/lib.rs` is the shared entry point; `src/main.rs` is `ssr`-only.

### Clean Architecture Layers

The same Clean Architecture layering used previously is preserved. Dependencies point inward only.

```
infrastructure  →  adapters  →  application  →  domain
```

| Layer | Path | Responsibility |
|---|---|---|
| Domain | `src/domain/` | Entities, value objects, typed domain errors. No framework imports. `cfg`-free. |
| Application | `src/application/` | Use cases, repository interfaces (traits), DTOs. |
| Adapters | `src/adapters/http/` | Axum handlers for `/api` and `/mocks`. `#[server]` functions for admin UI actions. |
| Adapters | `src/adapters/ui/` | Leptos components and pages. No direct SQLx or Axum imports. |
| Infrastructure | `src/infrastructure/` | SQLx repository implementations, config loading. `ssr`-only. |

### Server Functions

Leptos server functions (`#[server]` macro) replace the REST-over-Axios pattern used in the
previous React frontend. A component calls a server function as a plain async Rust function; Leptos
serialises the call to an HTTP request automatically. This eliminates the adapters/api layer that
previously lived in the React codebase.

Direct REST endpoints (`/api/*`) are still exposed as Axum route handlers for external API
consumers and are independent of the Leptos component tree.

### Key Conventions

- `#[cfg(feature = "ssr")]` gates all server-only code: SQLx, config, Axum extractors.
- `#[cfg(feature = "hydrate")]` gates all browser-only code: `window`, `localStorage`, JS interop.
- Repository traits are defined in `src/application/` and implemented in `src/infrastructure/`.
- `context.Context` is replaced by Tokio's `async`/`await` and Axum's `Extension` / `State` extractors.
- Errors are typed: domain errors in `src/domain/errors.rs`; use `thiserror` for `Display`/`Error` derives.
- All SQL queries use `sqlx::query!` / `sqlx::query_as!` macros (compile-time checked).

---

## Database

See [FEATURES.md — Databases](FEATURES.md#databases) for supported backends (SQLite3, PostgreSQL, MariaDB).

SQLx manages query execution. `sqlx-cli` runs versioned migration files in `migrations/`.
Switching backends requires only a driver feature flag (`sqlx/sqlite`, `sqlx/postgres`,
`sqlx/mysql`) and a DSN configuration change — no application code changes.

---

## Infrastructure and Deployment

### Docker

The `Dockerfile` is a multi-stage build:

- **`dev` stage**: installs `cargo-leptos` and `sqlx-cli`; runs `cargo leptos watch` for hot reload.
- **`builder` stage**: compiles both the WASM bundle and the native Axum binary via `cargo leptos build --release`.
- **`prod` stage**: copies the compiled binary and `public/` assets into a minimal Chainguard or distroless image.

### docker-compose

| File | Purpose |
|---|---|
| `docker-compose.yml` | Production-like setup. Builds the image, no volume mounts. |
| `docker-compose.override.yml` | Local development. Mounts source, runs `cargo leptos watch`. |

Two services are defined:

| Service | Image | Port |
|---|---|---|
| `db` | `postgres:16-alpine` | internal only |
| `app` | built from `.` | `3000` (prod) / `3000` (dev, cargo-leptos) |

The single `app` service replaces the previous separate `backend` and `frontend` services.
Leptos SSR serves the admin UI, `/api/*`, and `/mocks/*` from the same binary on one port.

### Environment Variables

Copy `.env.example` to `.env` and populate before running `docker compose up`.

| Variable | Description |
|---|---|
| `DATABASE_URL` | Full DSN passed to SQLx (e.g. `postgres://user:pass@db/dbname` or `sqlite://data.db`) |
| `PORT` | HTTP port the Axum server listens on (default `3000`) |
| `LEPTOS_SITE_ADDR` | Address cargo-leptos / Axum binds to (default `0.0.0.0:3000`) |
| `LEPTOS_SITE_ROOT` | Path to compiled WASM and static assets (default `site`) |
