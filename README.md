# Rust Neon Template

A production-ready Rust API template built with [Axum](https://github.com/tokio-rs/axum) and [Neon](https://neon.tech) (serverless Postgres with built-in Auth and Data APIs).

## Features

- **Axum web framework** – fast, ergonomic, and async-first
- **Neon Auth** – sign-up, sign-in, session management, and sign-out via the Neon Auth API
- **Neon Data API CRUD** – generic `create`/`get_all`/`get_one`/`update`/`delete` methods on `NeonClient` — works for any table
- **`NeonClient`** – shared HTTP client that holds the JWT token; automatically extracted from `Authorization: Bearer` via Axum's `FromRequestParts`
- **Auto-generated types** – `utility-types` derives reduce boilerplate (e.g. `SignInRequest` derived from `SignUpRequest` by omitting `name`)
- **Health check** – ready-to-extend health endpoint
- **Structured logging** – `tracing` + `tracing-subscriber` with environment-variable filtering
- **Unified error handling** – `AppError` enum that maps cleanly to HTTP responses
- **No system OpenSSL** – uses `rustls` for TLS

## Project Structure

```
src/
├── main.rs            # pub fn routes() — full router wiring + boot logic
├── error.rs           # AppError type with IntoResponse for Axum
├── config/
│   ├── mod.rs         # Config struct (environment settings)
│   └── client.rs      # NeonClient (struct + impl + FromRequestParts extractor)
│                      # + auth types (SignUpRequest, AuthResponse, Session, …)
└── handlers/
    ├── mod.rs
    ├── auth.rs        # Handler functions only (sign_up, sign_in, sign_out, get_session)
    ├── notes.rs       # Note model + handler functions (create_note, get_my_notes, …)
    └── health.rs      # health_check
```

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.85+ (edition 2024)
- A [Neon](https://neon.tech) project with the **Auth** feature enabled

## Getting Started

### 1. Clone and enter the project

```bash
git clone <your-repo-url> && cd rust-neon-template
```

### 2. Set environment variables

Create a `.env` file in the project root:

```env
AUTH_URL=https://<your-neon-project>.us-east-1.aws.neon.tech
DATA_API_URL=https://<your-neon-project>.us-east-1.aws.neon.tech/sql
PORT=8080
HOST=0.0.0.0
```

| Variable        | Description                                    | Required |
|-----------------|------------------------------------------------|----------|
| `AUTH_URL`      | Base URL of your Neon Auth API                 | Yes      |
| `DATA_API_URL`  | Base URL of your Neon Data API                 | Yes      |
| `PORT`          | Port the server listens on (default `8080`)    | No       |
| `HOST`          | Host the server binds to (default `0.0.0.0`)   | No       |

### 3. Create the notes table

Run this SQL in your Neon console's SQL editor:

```sql
CREATE TABLE notes (
    id      SERIAL PRIMARY KEY,
    title   TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT ''
);
```

### 4. Run the server

```bash
cargo run
```

### 5. Verify it's alive

```bash
curl http://localhost:8080/health
# {"status":"ok"}
```

## API Endpoints

### Health

| Method | Path       | Description       |
|--------|------------|-------------------|
| GET    | `/health`  | Health check      |

### Authentication

All auth endpoints are nested under `/api/v1/auth`.

| Method | Path                     | Description            |
|--------|--------------------------|------------------------|
| POST   | `/api/v1/auth/sign-up`   | Register a new user    |
| POST   | `/api/v1/auth/sign-in`   | Sign in an user        |
| POST   | `/api/v1/auth/sign-out`  | Sign out               |
| POST   | `/api/v1/auth/session`   | Get current session    |

### Notes

All notes endpoints require an `Authorization: Bearer <token>` header.

| Method | Path                  | Description              |
|--------|-----------------------|--------------------------|
| GET    | `/api/v1/notes`       | List all user notes      |
| POST   | `/api/v1/notes`       | Create a note            |
| GET    | `/api/v1/notes/{id}`  | Get a note by ID         |
| PATCH  | `/api/v1/notes/{id}`  | Update a note            |
| DELETE | `/api/v1/notes/{id}`  | Delete a note            |

### Example flow

```bash
# Sign up
curl -X POST http://localhost:8080/api/v1/auth/sign-up \
  -H "Content-Type: application/json" \
  -d '{"email": "alice@example.com", "name": "Alice", "password": "s3cret"}'

# Sign in (save the token)
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/sign-in \
  -H "Content-Type: application/json" \
  -d '{"email": "alice@example.com", "password": "s3cret"}' | jq -r '.token')

# Create a note
curl -X POST http://localhost:8080/api/v1/notes \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title": "Hello", "content": "Neon!"}'

# List notes
curl http://localhost:8080/api/v1/notes \
  -H "Authorization: Bearer $TOKEN"
```

## NeonClient

[`NeonClient`](src/config/client.rs) is the shared HTTP client for both the Neon Auth API and the Neon Data API. After a successful sign-up or sign-in it stores the JWT token, so every subsequent data API call is automatically authenticated.

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
| Data API | `delete(resource, id)`                      | Delete a record                    |

The generic CRUD methods work with any Neon Data API table — just pass the resource name (e.g. `"notes"`, `"users"`) and the return type `T`.

## Adding a new resource

1. **Create the model** — define your struct with `#[derive(Serialize, Deserialize)]` in a new handler file (e.g. `handlers/items.rs`)
2. **Write handlers** — use `client: NeonClient` with the generic CRUD methods
3. **Wire routes** — add the paths to `pub fn routes()` in `main.rs`

## Development

```bash
cargo check       # Check for compilation errors (fast)
cargo build       # Build the project
cargo run         # Run the server
cargo test        # Run tests
cargo fmt         # Format code
cargo clippy      # Lint
```

## License

MIT
