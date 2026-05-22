# Features

## Root Admin

The root admin account is the first privileged user bootstrapped into the system via the CLI. It is required before the web interface can be used.

### CLI Commands

| Command | Description |
|---|---|
| `admin create [--username USERNAME]` | Create the root admin account. `USERNAME` defaults to `administrator`. |

**Password setup**: The CLI prompts for a plain-text password. The password is hashed (bcrypt or equivalent) before being stored in the database. The plain-text password is never persisted.

---

## Manage Users and Groups

Admin users can manage the access control structure of the system.

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
| Reset password | Generate a random password. The new password is displayed **once** at generation time and is not retrievable afterwards. |
| List | Paginated list with search (by username) and filter (by group, status). |

> **Note**: Only admins can create and edit users and groups.

---

## Manage Mock APIs

Authenticated users can organise and define mock API endpoints within collections.

### Collections

A collection is a logical grouping of mock API endpoints, similar to a Postman collection.

| Action | Details |
|---|---|
| Create | Provide name, description, and status (`active` / `inactive`). |
| Edit | Users can edit their own collections. Admins can edit any collection. |
| Delete | Users can delete their own collections. Admins can delete any collection. |

### Mock API Endpoints

Each mock API endpoint belongs to one collection and defines the behaviour the server should simulate.

| Field | Description |
|---|---|
| Collection | The parent collection this endpoint belongs to. |
| Name | A human-readable label for the endpoint. |
| Type | Protocol type: `RESTful`, `GraphQL`, or `gRPC`. |
| Endpoint (path) | The URL path fragment, e.g. `notes` or `users/{id}`. |
| Allowed HTTP Methods | Whitelist of methods (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`). Defaults to none allowed. |
| Description | Optional free-text description. |
| Delay (ms) | Artificial response delay in milliseconds. Defaults to `0`. |
| Response — Headers | Key-value pairs to include in the response headers. |
| Response — Payload | The response body, typically a JSON document. |
| Response — HTTP Status Code | The HTTP status code to return (e.g. `200`, `201`, `404`). |

---

## Routes

Given the server is running at `https://example.com`, the URL structure is:

| Purpose | Base Path | Example |
|---|---|---|
| Admin web UI | `/admin` | `https://example.com/admin` |
| User-group management page | `/admin/user-groups` | `https://example.com/admin/user-groups` |
| Internal API | `/api` | `https://example.com/api` |
| User-group management API | `/api/user-groups` | `https://example.com/api/user-groups` |
| Mock endpoints | `/mocks/` | `https://example.com/mocks/` |

**Mock endpoint example**: If a mock API has the path `notes`, it is accessible at `https://example.com/mocks/notes`.

---

## Databases

The application supports three database backends. The backend is selected via configuration.

| Database | Notes |
|---|---|
| SQLite3 | Default. Suitable for local development and single-instance deployments. |
| PostgreSQL | Recommended for production multi-user deployments. |
| MariaDB | Alternative relational option; production-ready. |

The database layer is abstracted via GORM, so switching backends requires only a configuration change and no code changes.
