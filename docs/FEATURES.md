# Features

## Root Admin

The root admin account is the first privileged user bootstrapped into the system via the CLI. It is required before the web interface can be used.

### CLI Commands

| Command | Description |
|---|---|
| `admin create [--username USERNAME]` | Create the root admin account. `USERNAME` defaults to `administrator`. |

**Password setup**: The CLI prompts for a plain-text password. The password is hashed (bcrypt or equivalent) before being stored in the database. The plain-text password is never persisted.

---

## Users and Groups

Admin users manage the access control structure of the system.

### Groups

| Action | Details |
|---|---|
| Create | Provide name, description, and status (`active` / `inactive`). |
| Edit | Update any field of an existing group. |
| List | Paginated list with search (by name) and filter (by status). |

### Users

| Action | Details |
|---|---|
| Create | Provide username, password, group assignment, and status. |
| Edit | Update username, group, and status. Password cannot be changed via this form. |
| Reset password | Generate a random password. Displayed **once** at generation time; not retrievable afterwards. |
| List | Paginated list with search (by username) and filter (by group, status). |

> Only admins can create and edit users and groups.

---

## Mock API Collections

A collection is a logical grouping of mock API endpoints, similar to a Postman collection.

### Ownership and Permissions

- Every collection has an **owner** (the user who created it).
- The owner and admins are the only ones who can **delete** or **transfer ownership** of a collection.
- Collections can be shared with individual users or groups, each share assigned one of two roles:

| Role | Can call mock endpoints | Can edit collection and endpoints |
|---|---|---|
| `viewer` | Yes | No |
| `editor` | Yes | Yes |

### Visibility

A collection has one of four visibility levels:

| Level | Who can see and call its endpoints |
|---|---|
| Private | Owner only |
| Shared with users | Owner + explicitly listed users (with their assigned role) |
| Shared with groups | Owner + members of explicitly listed groups (with their assigned role) |
| Public | All authenticated users on the server |

> Visibility levels are not mutually exclusive — a collection can be shared with specific users, specific groups, and set to public simultaneously.

### Collection Actions

| Action | Who |
|---|---|
| Create | Any authenticated user |
| Edit | Owner, editors, admins |
| Delete | Owner, admins |
| Transfer ownership | Owner, admins |
| Duplicate | Any authenticated user (duplicate is owned by the duplicating user) |
| Import (Postman v2.1, Bruno) | Any authenticated user |
| Export (Postman v2.1, Bruno) | Any user with at least `viewer` access |

### Status

A collection has an `active` or `inactive` status. When a collection is **inactive**, all its endpoints return `503 Service Unavailable` regardless of their individual status.

---

## Mock API Endpoints

Each endpoint belongs to one collection and defines the behaviour the server should simulate.

### Endpoint Fields

| Field | Description |
|---|---|
| Collection | The parent collection this endpoint belongs to. |
| Name | A human-readable label. |
| Type | Protocol type: `RESTful`, `GraphQL`, or `gRPC`. |
| Path | URL path fragment appended to `/mocks/`. Supports path parameters using `{param}` syntax (e.g. `v1/users/{id}/notes`). |
| Allowed HTTP Methods | Whitelist of permitted methods (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`). Defaults to none. |
| Description | Optional free-text description. |
| Delay (ms) | Fixed artificial response delay in milliseconds. Defaults to `0`. |
| Status | `active` or `inactive`. An inactive endpoint returns `503 Service Unavailable`. |

### Response Fields

| Field | Description |
|---|---|
| HTTP Status Code | The status code to return (e.g. `200`, `201`, `404`). |
| Headers | Key-value pairs included in the response headers. |
| Payload | The response body. Edited in a code editor with syntax highlighting and a language mode toggle: **JSON**, **XML**, or **plain text**. |

### Path Matching Rules

- Path parameters use `{param}` syntax: a single segment defined as `users/{id}/notes` matches `/mocks/users/1/notes`, `/mocks/users/99/notes`, etc.
- **Exact match takes priority** over parameterised paths. If both `users/me/notes` and `users/{id}/notes` are defined, a request to `/mocks/users/me/notes` is served by `users/me/notes`.

### Endpoint Actions

| Action | Who |
|---|---|
| Create | Owner, editors, admins |
| Edit | Owner, editors, admins |
| Delete | Owner, admins |
| Duplicate (within same or different collection) | Owner, editors, admins |

### Status Precedence

Both the collection and endpoint status independently gate responses:

| Collection status | Endpoint status | Result |
|---|---|---|
| Active | Active | Configured response returned |
| Active | Inactive | `503 Service Unavailable` |
| Inactive | Active | `503 Service Unavailable` |
| Inactive | Inactive | `503 Service Unavailable` |

---

## Mock Endpoint Routing

Given the server is running at `https://example.com`:

| Purpose | Base Path | Example |
|---|---|---|
| Admin web UI | `/admin` | `https://example.com/admin` |
| User-group management page | `/admin/user-groups` | `https://example.com/admin/user-groups` |
| Internal management API | `/api` | `https://example.com/api` |
| User-group management API | `/api/user-groups` | `https://example.com/api/user-groups` |
| Mock endpoints | `/mocks/` | `https://example.com/mocks/` |

**Mock endpoint examples**:

| Endpoint path defined | Full URL |
|---|---|
| `notes` | `https://example.com/mocks/notes` |
| `v1/users/{id}/notes` | `https://example.com/mocks/v1/users/42/notes` |

Mock endpoints (`/mocks/*`) are **public** — no authentication is required to call them. Authentication is enforced only on the admin UI and management API.

---

## Import and Export

Collections and their endpoints can be imported and exported in industry-standard formats:

| Format | Import | Export |
|---|---|---|
| Postman Collection v2.1 | Yes | Yes |
| Bruno | Yes | Yes |

---

## UI

- **Dark mode**: follows the OS `prefers-color-scheme` setting automatically. No manual toggle.
- **Payload editor**: embedded code editor with syntax highlighting and a language mode toggle (JSON, XML, plain text).

---

## Databases

The application supports three database backends, selected via configuration. Default is SQLite3.

| Database | Notes |
|---|---|
| SQLite3 | Default. Suitable for local development and single-instance deployments. |
| PostgreSQL | Recommended for production multi-user deployments. |
| MariaDB | Alternative relational option; production-ready. |

The database layer is abstracted via GORM. Switching backends requires only a configuration change.

---

## Advanced Features (Next Phase)

The following features are planned but out of scope for the current phase:

| Feature | Description |
|---|---|
| Request logging | Full log per request: timestamp, method, path, headers, request body, response status code. Viewable in the UI. |
| Response templating | Built-in variables in the payload (e.g. `{{now}}`, `{{uuid}}`) resolved at response time. |
