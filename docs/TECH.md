# Technology Stack

## Repository Layout

Backend and frontend live together in a flat monorepo. Each is self-contained with its own `Dockerfile`; a `docker-compose.yml` at the root orchestrates all services.

```
api-mock-server/
├── backend/                          # Go service
│   ├── cmd/
│   │   └── server/
│   │       └── main.go               # Entry point — wires dependencies, starts HTTP server
│   ├── internal/
│   │   ├── domain/                   # Entities, value objects, domain errors
│   │   ├── application/              # Use cases, repository interfaces, DTOs
│   │   ├── adapters/                 # HTTP handlers, request/response schemas
│   │   └── infrastructure/           # GORM models, concrete repos, config
│   ├── migrations/                   # Versioned SQL migration files
│   ├── go.mod
│   ├── go.sum
│   ├── Makefile
│   └── Dockerfile
├── frontend/                         # ReactJS app
│   ├── src/
│   │   ├── domain/                   # TypeScript interfaces, typed domain errors
│   │   ├── application/              # Custom hooks (use cases), repository interfaces
│   │   ├── adapters/                 # HTTP repository implementations
│   │   ├── infrastructure/           # Axios config, storage adapters, env config
│   │   ├── ui/                       # Pages, layouts, presentational components
│   │   └── shared/                   # Cross-cutting utilities and hooks
│   ├── public/
│   ├── package.json
│   ├── tsconfig.json
│   └── Dockerfile
├── docker-compose.yml                # Production-like compose (built images)
├── docker-compose.override.yml       # Local dev overrides (volume mounts, hot reload)
├── .env.example                      # Environment variable template
├── docs/
├── Makefile                          # Top-level: make dev, make build, make test
└── README.md
```

---

## Backend

| Technology | Role |
|---|---|
| [Go](https://go.dev/) | Primary language. Compiled, statically typed, high-performance. |
| [go-chi/chi](https://github.com/go-chi/chi) | Lightweight HTTP router. Defines REST routes, applies middleware, and groups handlers. |
| [GORM](https://gorm.io/) | ORM for database access. Supports SQLite3, PostgreSQL, and MariaDB via driver swap. Handles migrations, queries, and associations. |

### Architecture

The backend follows **Clean Architecture** with four layers. Dependencies point inward only.

```
infrastructure  →  adapters  →  application  →  domain
```

| Layer | Path | Responsibility |
|---|---|---|
| Domain | `internal/domain/` | Entities, value objects, typed domain errors. No framework imports. |
| Application | `internal/application/` | Use cases, repository interfaces, DTOs. |
| Adapters | `internal/adapters/` | HTTP handlers (chi), request/response schemas. |
| Infrastructure | `internal/infrastructure/` | GORM models, concrete repository implementations, config loading. |

### Key Conventions

- All HTTP handlers live under `internal/adapters/http/handler/`.
- Repository interfaces are defined in `internal/application/` and implemented in `internal/infrastructure/`.
- `context.Context` is the first parameter on every function that performs I/O.
- Errors are wrapped with `fmt.Errorf("...: %w", err)` for call-site context.

---

## Frontend

| Technology | Role |
|---|---|
| [ReactJS](https://react.dev/) | UI framework for the admin web interface. |
| [TypeScript](https://www.typescriptlang.org/) | Type safety across all layers. Strict mode enabled. |
| [Vite](https://vitejs.dev/) | Build tool and dev server. Proxies `/api` and `/mocks` to the backend in development. |
| [Axios](https://axios-http.com/) | HTTP client. Used only inside the `adapters/` layer. |

### Architecture

The frontend mirrors the backend's Clean Architecture layering.

| Layer | Path | Responsibility |
|---|---|---|
| Domain | `src/domain/` | TypeScript interfaces and typed domain errors. No React or Axios imports. |
| Application | `src/application/` | Custom hooks encapsulating use-case logic. Repository interfaces (abstract ports). |
| Adapters | `src/adapters/` | Concrete HTTP repository implementations. The only layer that imports Axios. |
| Infrastructure | `src/infrastructure/` | Axios instance config, local storage adapters, environment variable access. |
| UI | `src/ui/` | React pages, layout components, and reusable presentational components. |
| Shared | `src/shared/` | Cross-cutting utilities and hooks (e.g. `useDebounce`, `formatDate`). |

### Key Conventions

- Business logic lives in custom hooks under `application/`, not in React components.
- Components in `ui/` are purely presentational; they receive data and callbacks via props.
- All HTTP calls go through `adapters/`; components and hooks never import Axios directly.

---

## Database

See [FEATURES.md — Databases](FEATURES.md#databases) for the supported backends (SQLite3, PostgreSQL, MariaDB).

GORM manages schema migrations and query construction. Switching backends requires only a driver and DSN configuration change — no application code changes.

---

## Infrastructure and Deployment

### Docker

Each service has a multi-stage `Dockerfile`:

- **`dev` stage**: includes hot-reload tooling (`air` for Go, `npm run dev` / Vite for React).
- **`prod` stage**: produces a minimal final image (distroless or Alpine).

### docker-compose

| File | Purpose |
|---|---|
| `docker-compose.yml` | Production-like setup. Builds images, no volume mounts. |
| `docker-compose.override.yml` | Local development. Mounts source directories, enables hot reload. Applied automatically by Docker Compose when both files are present. |

Three services are defined:

| Service | Image | Port |
|---|---|---|
| `db` | `postgres:16-alpine` | internal only |
| `backend` | built from `./backend` | `8080` |
| `frontend` | built from `./frontend` | `3000` (prod) / `5173` (dev) |

The frontend's Vite dev proxy (and the Nginx config in production) forwards `/api/*` and `/mocks/*` to the backend, so the browser never makes cross-origin requests.

### Environment Variables

Copy `.env.example` to `.env` and populate before running `docker compose up`.

| Variable | Used By | Description |
|---|---|---|
| `DB_NAME` | `db`, `backend` | Database name |
| `DB_USER` | `db`, `backend` | Database user |
| `DB_PASSWORD` | `db`, `backend` | Database password |
| `PORT` | `backend` | HTTP port the Go server listens on (default `8080`) |
| `VITE_API_BASE` | `frontend` | Base path for API calls (default `/api`) |
