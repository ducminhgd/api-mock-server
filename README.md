# API Mock Server

A self-hosted API mock server built with Rust, [Leptos](https://leptos.dev) (full-stack SSR + WASM), and SQLite.
Define collections of mock endpoints and serve deterministic HTTP responses without a real backend.

## Features

- Manage **collections**, **endpoints**, and **shares** via a browser UI
- Mock any HTTP method and path with configurable status code, headers, and response body
- Path parameters using `{param}` syntax (e.g. `/users/{id}`)
- Import/export collections from Postman and Bruno formats
- User and group management with role-based access (Admin / Regular)
- JWT authentication
- SQLite by default — zero external dependencies for storage

## Requirements

| Tool | Version |
|---|---|
| Rust | 1.95+ |
| cargo-leptos | 0.3.6 |
| wasm32-unknown-unknown target | (added via rustup) |
| Docker + Compose | for containerised runs |

## Quick start (local)

```bash
# 1. Add the WASM target (once)
rustup target add wasm32-unknown-unknown

# 2. Install cargo-leptos (once)
cargo install cargo-leptos --version 0.3.6 --locked

# 3. Copy and edit the environment file
cp .env.example .env          # set JWT_SECRET at minimum

# 4. Run migrations (SQLite file is created automatically)
DATABASE_URL=sqlite://./dev.db sqlx migrate run

# 5. Start the dev server with hot-reload
DATABASE_URL=sqlite://./dev.db JWT_SECRET=dev-secret cargo leptos watch
```

Open http://localhost:3000.

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | — | SQLite: `sqlite:///path/to/app.db` · Postgres: `postgres://user:pass@host/db` |
| `JWT_SECRET` | — | **Required.** Secret used to sign JWT tokens. |
| `PORT` | `3000` | HTTP port the server listens on. |
| `LEPTOS_SITE_ADDR` | `127.0.0.1:3000` | Full bind address (overrides `PORT`). |
| `LEPTOS_SITE_ROOT` | `site` | Directory that contains the compiled frontend assets. |

## Docker

### Single container — SQLite (default)

```bash
docker build -t api-mock-server .

docker run -d \
  -e JWT_SECRET=change-me \
  -p 3000:3000 \
  -v api_mock_data:/app/data \
  --name api-mock-server \
  api-mock-server
```

The SQLite database is stored in the `/app/data` volume and persists across restarts.

### Docker Compose — production

Pass `-f docker-compose.yml` to skip the dev override and build the optimised `prod` image stage:

```bash
# Start (builds if needed)
JWT_SECRET=change-me docker compose -f docker-compose.yml up -d

# Override the host port (default 3000)
APP_PORT=8080 JWT_SECRET=change-me docker compose -f docker-compose.yml up -d
```

The `app_data` named volume is created automatically and holds the SQLite file.
`JWT_SECRET` defaults to `change-me-in-production` — always override it in production.

### Docker Compose — development (hot-reload)

Running without `-f` merges `docker-compose.override.yml` automatically.
The override targets the `dev` image stage, mounts source files into the container,
and runs `cargo leptos watch` so changes rebuild without restarting the container.
Cargo registry and build artefacts are cached in named volumes (`cargo_cache`,
`target_cache`) to avoid full recompiles on restart.

```bash
docker compose up
```

## Using PostgreSQL instead of SQLite

Pass a Postgres connection URL via `DATABASE_URL`:

```bash
# docker run
docker run -d \
  -e DATABASE_URL=postgres://user:pass@db-host/api_mock_server \
  -e JWT_SECRET=change-me \
  -p 3000:3000 \
  api-mock-server

# docker compose — add a db service or point at an external host
DATABASE_URL=postgres://user:pass@db-host/api_mock_server \
JWT_SECRET=change-me \
docker compose up -d
```

The same binary and migrations support SQLite, PostgreSQL, and MySQL — the driver is
selected automatically from the URL scheme.

## Development

```bash
make dev        # cargo leptos watch
make test       # cargo test --features ssr
make lint       # clippy --fix + fmt
make build      # cargo leptos build --release
make pre-commit # lint + test + release build
```

## Project layout

```
src/
├── domain/          # Business entities and rules (no framework imports)
├── application/     # Use cases, DTOs, repository interfaces
├── adapters/
│   ├── http/        # Axum REST handlers
│   └── ui/          # Leptos components (SSR + WASM)
└── infrastructure/  # SQLx repositories, JWT, bcrypt, config
migrations/          # SQLx versioned SQL migrations
style/               # CSS
public/              # Static assets
```

## API reference

All REST endpoints are prefixed with `/api`.

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/auth/login` | Obtain a JWT token |
| `GET` | `/api/collections` | List collections |
| `POST` | `/api/collections` | Create a collection |
| `GET` | `/api/collections/:id` | Get a collection |
| `PUT` | `/api/collections/:id` | Update a collection |
| `DELETE` | `/api/collections/:id` | Delete a collection |
| `POST` | `/api/collections/:id/duplicate` | Duplicate a collection |
| `POST` | `/api/collections/:id/transfer` | Transfer ownership |
| `GET` | `/api/collections/:id/shares` | List shares |
| `POST` | `/api/collections/:id/shares` | Add a share |
| `PUT` | `/api/collections/:id/shares/:sid` | Update share role |
| `DELETE` | `/api/collections/:id/shares/:sid` | Remove a share |
| `GET` | `/api/collections/:id/endpoints` | List endpoints |
| `POST` | `/api/collections/:id/endpoints` | Create an endpoint |
| `GET` | `/api/collections/:id/endpoints/:eid` | Get an endpoint |
| `PUT` | `/api/collections/:id/endpoints/:eid` | Update an endpoint |
| `DELETE` | `/api/collections/:id/endpoints/:eid` | Delete an endpoint |
| `POST` | `/api/collections/:id/endpoints/:eid/duplicate` | Duplicate an endpoint |
| `GET` | `/api/groups` | List groups |
| `POST` | `/api/groups` | Create a group |
| `GET` | `/api/groups/:id` | Get a group |
| `PUT` | `/api/groups/:id` | Update a group |
| `DELETE` | `/api/groups/:id` | Delete a group |
| `GET` | `/api/users` | List users |
| `POST` | `/api/users` | Create a user |
| `GET` | `/api/users/:id` | Get a user |
| `PUT` | `/api/users/:id` | Update a user |
| `DELETE` | `/api/users/:id` | Delete a user |
| `POST` | `/api/users/:id/reset-password` | Reset a user's password |

Mock endpoints are served under `/mocks/:collection_id/*path`.
