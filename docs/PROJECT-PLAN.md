# Project Plan

## Phase 0 — Foundation

1. Initialise monorepo structure (`backend/`, `frontend/`, `docs/`)
2. Backend: Go module, Clean Architecture directory scaffold
3. Backend: Config loading (env vars, `.env` support, DB driver switch)
4. Backend: GORM setup with SQLite, PostgreSQL, and MariaDB drivers
5. Frontend: React + TypeScript + Vite scaffold, Clean Architecture directory layout
6. Frontend: Axios instance, base API client, environment config
7. Docker: multi-stage `Dockerfile` for backend (dev / prod)
8. Docker: multi-stage `Dockerfile` for frontend (dev / prod)
9. Docker: `docker-compose.yml` and `docker-compose.override.yml`
10. CI: lint + test pipeline (GitHub Actions or equivalent)

---

## Phase 1 — Database Schema and Migrations

11. Domain models: `User`, `Group`
12. Domain models: `Collection`, `CollectionShare`
13. Domain models: `Endpoint`
14. Migrations: create all tables with indexes and constraints
15. Seed: fixture data for local development

---

## Phase 2 — Authentication and Root Admin

16. Backend: password hashing (bcrypt)
17. Backend: JWT issue and validation middleware
18. Backend: `admin create` CLI command
19. API: `POST /api/auth/login` — issue JWT
20. API: `POST /api/auth/logout` — invalidate token
21. Frontend: login page
22. Frontend: auth context, protected route wrapper, token storage

---

## Phase 3 — User and Group Management

23. API: `GET/POST /api/groups` — list and create groups
24. API: `GET/PUT/DELETE /api/groups/{id}` — read, edit, delete group
25. API: `GET/POST /api/users` — list and create users
26. API: `GET/PUT/DELETE /api/users/{id}` — read, edit, delete user
27. API: `POST /api/users/{id}/reset-password` — generate and return new password once
28. Frontend: group list page (search, filter, pagination)
29. Frontend: create / edit group form
30. Frontend: user list page (search, filter, pagination)
31. Frontend: create / edit user form
32. Frontend: reset password action

---

## Phase 4 — Collection Management

33. API: `GET/POST /api/collections` — list (scoped to caller) and create
34. API: `GET/PUT/DELETE /api/collections/{id}` — read, edit, delete
35. API: `POST /api/collections/{id}/duplicate` — clone collection and all its endpoints
36. API: `GET/POST /api/collections/{id}/shares` — list and add shares (user or group, with role)
37. API: `PUT/DELETE /api/collections/{id}/shares/{shareId}` — update role or remove share
38. API: `PUT /api/collections/{id}/transfer` — transfer ownership
39. Frontend: collection list page (scoped, with visibility indicator)
40. Frontend: create / edit collection form (name, description, status, visibility)
41. Frontend: sharing panel (add user / group, assign role, remove)

---

## Phase 5 — Endpoint Management

42. API: `GET/POST /api/collections/{id}/endpoints` — list and create endpoints
43. API: `GET/PUT/DELETE /api/collections/{id}/endpoints/{eid}` — read, edit, delete
44. API: `POST /api/collections/{id}/endpoints/{eid}/duplicate` — clone endpoint
45. Frontend: endpoint list page within a collection
46. Frontend: create / edit endpoint form
47. Frontend: response payload code editor (Monaco or CodeMirror, JSON / XML / plain text toggle)

---

## Phase 6 — Mock Router

48. Backend: mount `/mocks/*` router (public, no auth)
49. Backend: path parameter matching (`{param}` syntax)
50. Backend: exact-match-over-wildcard priority resolution
51. Backend: check collection status → return `503` if inactive
52. Backend: check endpoint status → return `503` if inactive
53. Backend: check allowed HTTP methods → return `405` if method not permitted
54. Backend: apply configured delay before responding
55. Backend: write configured headers and payload to response

---

## Phase 7 — Import and Export

56. Backend: Postman Collection v2.1 parser (import)
57. Backend: Bruno format parser (import)
58. Backend: Postman Collection v2.1 serialiser (export)
59. Backend: Bruno format serialiser (export)
60. API: `POST /api/collections/import` — upload file, create collection
61. API: `GET /api/collections/{id}/export?format=postman|bruno` — download file
62. Frontend: import button (file picker, format detection)
63. Frontend: export button (format selector dropdown)

---

## Phase 8 — UI Polish

64. Frontend: dark mode via `prefers-color-scheme` (CSS / Tailwind)
65. Frontend: consistent error states and empty states across all pages
66. Frontend: responsive layout for tablet and desktop

---

## Phase 9 — Testing and Documentation

67. Backend: unit tests for domain and application layers
68. Backend: integration tests for all API endpoints
69. Backend: mock router integration tests (path matching, status precedence, delay)
70. Frontend: component tests for forms and the payload editor
71. OpenAPI spec (`/api` routes) — generated or hand-authored
72. Update `README.md` with setup, configuration, and usage instructions

---

## Advanced Features (Next Phase)

> Not in scope for the current delivery. Tracked here for sequencing.

- A. Full request log: record method, path, headers, body, response status, timestamp per mock hit
- B. Response templating: built-in variables (`{{now}}`, `{{uuid}}`) resolved at response time
