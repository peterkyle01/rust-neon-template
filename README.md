# Rust Neon Template

A production-ready Rust API template built with [Axum](https://github.com/tokio-rs/axum) and [Neon](https://neon.tech) (serverless Postgres with built-in Auth and Data APIs).

## Features

- **Axum web framework** – fast, ergonomic, and async-first
- **Neon Auth** – sign-up, sign-in, session management, and sign-out via the Neon Auth API
- **Health check** – ready-to-extend health endpoint
- **Structured logging** – `tracing` + `tracing-subscriber` with environment-variable filtering
- **Unified error handling** – `AppError` enum that maps cleanly to HTTP responses
- **No system OpenSSL** – uses `rustls` for TLS

## Project Structure

```
src/
├── main.rs           # Entry point — loads config, builds router, starts server
├── config/mod.rs     # Environment-based configuration (AUTH_URL, PORT, etc.)
├── error.rs          # AppError type with IntoResponse for Axum
├── models/
│   ├── mod.rs
│   ├── auth.rs       # Auth request/response types (SignUpRequest, Session, etc.)
│   └── user.rs       # User data type
├── routes/
│   ├── mod.rs        # Router builder — nests all route groups
│   ├── auth.rs       # POST /api/v1/auth/{sign-up,sign-in,sign-out,session}
│   └── health.rs     # GET /health
└── services/
    ├── mod.rs
    └── auth.rs       # NeonAuthClient — wraps the Neon Auth REST API
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

> **Note:** You can use different `.env` files per environment (`.env.production`, `.env.test`, etc.).

### 3. Run the server

```bash
cargo run
```

### 4. Verify it's alive

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

#### Example: Sign up

```bash
curl -X POST http://localhost:8080/api/v1/auth/sign-up \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "name": "Alice", "password": "s3cret"}'
```

#### Example: Sign in

```bash
curl -X POST http://localhost:8080/api/v1/auth/sign-in \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "s3cret"}'
```

Both sign-up and sign-in return:

```json
{ "token": "<jwt-token>" }
```

## Development

### Useful commands

```bash
cargo check       # Check for compilation errors (fast)
cargo build       # Build the project
cargo run         # Run the server
cargo test        # Run tests
cargo fmt         # Format code
cargo clippy      # Lint
```

### Adding a new route

1. Create a file in `src/routes/` (e.g. `src/routes/items.rs`)
2. Define your handlers and a `pub fn routes()` that returns an `axum::Router`
3. Register it in `src/routes/mod.rs` with `.nest("/api/v1/items", items::routes())`
4. If you need shared state, add it to the `Config` struct and pass it via `with_state`

## License

MIT
