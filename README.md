# Rust Neon Template

A production-ready Rust API template built with [Axum](https://github.com/tokio-rs/axum) and [Neon](https://neon.tech) (serverless Postgres with built-in Auth and Data APIs).

## Features

- **Axum web framework** – fast, ergonomic, and async-first
- **Neon Auth** – sign-up, sign-in, and sign-out via the Neon Auth API
- **Neon Data API CRUD** – generic `create`/`get_all`/`get_one`/`update`/`delete` on `NeonClient` — works for any table
- **`NeonClient` extractor** – automatically pulls the JWT from `Authorization: Bearer` via Axum's `FromRequestParts`
- **Auto-generated types** – `utility-types` reduces boilerplate (e.g. `SignInRequest` derived from `SignUpRequest`)
- **Standard API envelope** – all responses follow `{ "data": ... }` / `{ "error": { "code": "...", "message": "..." } }`
- **Smart health checks** – verifies both Auth and Data API endpoints are reachable
- **Level-based logging** – `INFO` for 2xx, `WARN` for 4xx, `ERROR` for 5xx
- **Comprehensive tests** – integration tests covering the full CRUD flow and error scenarios
- **No system OpenSSL** – uses `rustls` for TLS

## How it works

```
Client                    Axum Server                      Neon Auth                  Neon Data API
  │                          │                                │                           │
  ├─ POST /api/v1/auth/sign-in                                │                           │
  │                          ├─ POST /sign-in/email ─────────►│                           │
  │                          │◄── session token + cookie ─────┤                           │
  │                          ├─ GET /get-session (cookie) ───►│                           │
  │                          │◄── set-auth-jwt: <JWT> ───────┤                           │
  │◄─ { "data": { "token": <JWT> } }                          │                           │
  │                          │                                │                           │
  ├─ GET /api/v1/notes (Bearer JWT)                           │                           │
  │                          ├─ GET /notes (Bearer JWT) ──────┼───────────────►───────────┤
  │◄─ { "data": [...] } ─────┤◄──────────────────────────────┼───────────[...]──────────┤
```

## Project Structure

```
src/
├── lib.rs            # Library root — routes(), TraceLayer, LogOnResponse
├── main.rs           # Binary entry point — calls into lib
├── response.rs       # Standard API envelope (AppError, ok(), created())
├── config/
│   ├── mod.rs        # Config struct (environment settings)
│   └── client.rs     # NeonClient (struct + impl + FromRequestParts)
│                     # + auth types (SignUpRequest, SignInRequest, Session)
└── handlers/
    ├── mod.rs
    ├── auth.rs       # Handler functions (sign_up, sign_in, sign_out)
    ├── notes.rs      # Note model + handler functions (create, list, get, update, delete)
    └── health.rs     # Health check with component-level status
tests/
└── api.rs            # Full integration tests (4 tests, no warnings)
```

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.85+ (edition 2024)
- A [Neon](https://neon.tech) project with **Auth** and **Data API** enabled

## Getting Started

### 1. Set environment variables

Create a `.env` file in the project root:

```env
AUTH_URL=https://<your-project>.neonauth.<region>.aws.neon.tech/neondb/auth
DATA_API_URL=https://<your-project>.apirest.<region>.aws.neon.tech/neondb/rest/v1
PORT=8080
HOST=0.0.0.0
```

| Variable        | Description                                    | Required |
|-----------------|------------------------------------------------|----------|
| `AUTH_URL`      | Your Neon Auth URL (from Console → Auth)       | Yes      |
| `DATA_API_URL`  | Your Data API URL (from Console → Data API)    | Yes      |
| `PORT`          | Port the server listens on (default `8080`)    | No       |
| `HOST`          | Host the server binds to (default `0.0.0.0`)   | No       |

### 2. Create the notes table

Run this SQL in your Neon console's SQL editor:

```sql
CREATE TABLE notes (
    id      SERIAL PRIMARY KEY,
    title   TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT ''
);

-- Disable RLS so the Data API can read/write freely.
ALTER TABLE notes DISABLE ROW LEVEL SECURITY;
```

### 3. Run the server

```bash
cargo run
```

Every request is logged with level-appropriate detail:

```
 INFO listening on 0.0.0.0:8080
 INFO request{method=GET uri=/health}: ok status=200 latency_ms=1310
 INFO request{method=POST uri=/api/v1/auth/sign-in}: ok status=200 latency_ms=937
 WARN request{method=GET uri=/api/v1/notes/9999}: client error status=404 latency_ms=599
 WARN request{method=GET uri=/api/v1/notes}: client error status=401 latency_ms=0
```

### 4. Run the tests

```bash
cargo test
```

All 4 integration tests pass, covering health, auth errors, and the full CRUD lifecycle.

## API Response Format

Every endpoint returns one of two shapes:

**Success (2xx):**
```json
{ "data": <payload> }
```

**Error (4xx/5xx):**
```json
{ "error": { "code": "NOT_FOUND", "message": "note 9999 not found" } }
```

**Error codes:**

| Code            | HTTP Status | When                      |
|-----------------|-------------|---------------------------|
| `BAD_REQUEST`   | 400         | Invalid input             |
| `UNAUTHORIZED`  | 401         | Missing/wrong credentials |
| `NOT_FOUND`     | 404         | Resource doesn't exist    |
| `INTERNAL_ERROR`| 500         | Server error              |

## API Endpoints

### Health

```bash
curl http://localhost:8080/health
# {"data":{"status":"ok","checks":{"auth":"ok","data_api":"ok"}}}
```

### Auth

All auth endpoints are nested under `/api/v1/auth`.

```bash
# Sign up
curl -X POST http://localhost:8080/api/v1/auth/sign-up \
  -H "Content-Type: application/json" \
  -d '{"email": "alice@example.com", "name": "Alice", "password": "s3cret"}'

# Sign in — returns a JWT
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/sign-in \
  -H "Content-Type: application/json" \
  -d '{"email": "alice@example.com", "password": "s3cret"}' | jq -r '.data.token')

# Sign out
curl -X POST http://localhost:8080/api/v1/auth/sign-out \
  -H "Authorization: Bearer $TOKEN"
```

### Notes CRUD

All notes endpoints require `Authorization: Bearer <token>`.

```bash
# Create
curl -X POST http://localhost:8080/api/v1/notes \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title": "Hello", "content": "Neon!"}'

# List
curl http://localhost:8080/api/v1/notes \
  -H "Authorization: Bearer $TOKEN"

# Get by ID
curl http://localhost:8080/api/v1/notes/1 \
  -H "Authorization: Bearer $TOKEN"

# Update
curl -X PATCH http://localhost:8080/api/v1/notes/1 \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title": "Updated", "content": "Edited!"}'

# Delete
curl -X DELETE http://localhost:8080/api/v1/notes/1 \
  -H "Authorization: Bearer $TOKEN"
```

Standard response examples:

```json
// GET /notes (list)
{ "data": [{ "id": 1, "title": "Hello", "content": "Neon!" }] }

// GET /notes/1 (single)
{ "data": { "id": 1, "title": "Hello", "content": "Neon!" } }

// POST /notes (create — 201)
{ "data": { "id": 2, "title": "New", "content": "Note" } }

// PATCH /notes/1 (update)
{ "data": [{ "id": 1, "title": "Updated", "content": "Yes" }] }

// DELETE /notes/1
{ "data": { "message": "deleted" } }

// 404 — non-existent note
{ "error": { "code": "NOT_FOUND", "message": "note 9999 not found" } }

// 401 — missing auth
{ "error": { "code": "UNAUTHORIZED", "message": "missing or invalid Authorization header" } }
```

## NeonClient

[`NeonClient`](src/config/client.rs) is the shared HTTP client for both the Neon Auth API and the Neon Data API. It handles the full token lifecycle:

1. **Sign-in / Sign-up** – calls the Better Auth REST API, extracts the session cookie
2. **JWT exchange** – calls `/get-session` with the cookie, extracts the JWT from the `set-auth-jwt` response header
3. **Data API calls** – uses the JWT as `Authorization: Bearer` for all CRUD operations

In handlers, `client: NeonClient` is extracted directly from the request — the `FromRequestParts` implementation pulls the `Authorization: Bearer` header automatically.

### Available methods

| Category | Method                                      | Description                        |
|----------|---------------------------------------------|------------------------------------|
| Auth     | `sign_up`                                   | Register a new user                |
| Auth     | `sign_in`                                   | Sign in an existing user           |
| Auth     | `get_session`                               | Refresh / validate the session     |
| Auth     | `sign_out`                                  | Sign out and clear the token       |
| Data API | `get_all::<T>(resource)`                    | List all records of a resource     |
| Data API | `get_one::<T>(resource, id)`                | Get a single record by ID          |
| Data API | `create::<T>(resource, body)`               | Create a record                    |
| Data API | `update::<T>(resource, id, body)`           | Update a record                    |
| Data API | `delete(resource, id) -> bool`              | Delete a record (returns false if missing) |

The generic CRUD methods work with any Neon Data API table — just pass the resource name (e.g. `"notes"`, `"users"`) and the return type `T`.

## Adding a new resource

1. **Create the model** — define your struct with `#[derive(Serialize, Deserialize)]` in a new handler file (e.g. `handlers/items.rs`)
2. **Write handlers** — use `client: NeonClient` with the generic CRUD methods; return `response::ok()` or `response::created()`
3. **Wire routes** — add the paths to `pub fn routes()` in `src/lib.rs`

## Development

```bash
cargo check       # Check for compilation errors (fast)
cargo build       # Build the project
cargo run         # Run the server
cargo test        # Run 4 integration tests
cargo fmt         # Format code
cargo clippy      # Lint
```

## License

MIT
