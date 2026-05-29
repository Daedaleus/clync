# Backend

Rust REST API built with Axum. Handles authentication, business logic, and real-time events.

## Technology

- **Axum 0.8** — HTTP framework
- **SurrealDB 2** — Database (WebSocket client)
- **Keycloak** — Auth via JWKS JWT validation (1 h cache)
- **SSE** — Real-time group events via `tokio::sync::broadcast`
- **Web Push** — VAPID push notifications
- **RAWG API** — Optional game metadata autofill (description, genre, thumbnail)
- **`config` crate** — Layered YAML configuration

## Setup

**Prerequisites:** Rust (stable), running SurrealDB and Keycloak (see `infrastructure/`).

```sh
cd backend
cargo run
```

Server listens on `http://localhost:3000`.

## Configuration

Configuration is layered — later sources override earlier ones:

| Source | File | Committed |
|---|---|---|
| Base defaults | `config/app.yaml` | Yes |
| Local overrides | `config/app.local.yaml` | No (gitignored) |
| Environment | `APP__<SECTION>__<KEY>=value` | — |

Copy the example and adjust:

```sh
cp config/app.local.yaml.example config/app.local.yaml
```

Key settings in `config/app.yaml`:

```yaml
server:
  port: 3000
database:
  url: "127.0.0.1:8000"
  namespace: clync
  name: clync
keycloak:
  jwks_uri: "http://localhost:8080/realms/Clync/protocol/openid-connect/certs"
  admin_url: "http://localhost:8080"
  realm: "Clync"
cors:
  frontend_url: "http://localhost:5173"
vapid:
  public_key: ""   # required for push notifications
  private_key: ""  # set in app.local.yaml
  subject: "mailto:dev@clync.local"
rawg:
  api_key: ""      # optional — get a free key at https://rawg.io/apidocs
```

Environment variable format (useful for Docker):

```sh
APP__DATABASE__URL=surrealdb:8000
APP__DATABASE__PASSWORD=secret
APP__VAPID__PRIVATE_KEY=...
APP__RAWG__API_KEY=...
```

Generate VAPID keys:

```sh
npx web-push generate-vapid-keys
```

## API

All routes are prefixed with `/api/v1`.

### Public (no auth required)

| Method | Path | Description |
|---|---|---|
| `GET` | `/health` | Health check |
| `GET` | `/push/vapid-public-key` | VAPID public key for push subscription |
| `GET` | `/invites/{token}` | Validate an invite link |
| `POST` | `/invites/{token}/register` | Register via invite link |
| `GET` | `/library/{name}/thumbnail` | Serve game thumbnail (used directly by `<img>`) |

### Protected (Bearer JWT required)

**Profile & social**

| Method | Path | Description |
|---|---|---|
| `GET` | `/me` | Own profile, games, and social handles |
| `PUT` | `/me/profile` | Update Steam / Discord handles and their visibility |
| `GET` | `/users/search?q=` | Search users by username |
| `GET` | `/users/{id}` | Public user profile (respects visibility settings) |

**Friends**

| Method | Path | Description |
|---|---|---|
| `GET` | `/friends` | Friend list |
| `POST` | `/friends/{id}` | Add friend |
| `DELETE` | `/friends/{id}` | Remove friend |

**Games (wishlist)**

| Method | Path | Description |
|---|---|---|
| `POST` | `/me/games` | Add game to own wishlist |
| `DELETE` | `/me/games?name=` | Remove game from own wishlist |

**Game library**

| Method | Path | Description |
|---|---|---|
| `GET` | `/library` | List all games |
| `POST` | `/library` | Create a new game |
| `GET` | `/library/{name}` | Game detail |
| `PUT` | `/library/{name}` | Update name, description, or genre |
| `DELETE` | `/library/{name}` | Delete game (only if not in any wishlist) |
| `POST` | `/library/{name}/thumbnail` | Upload thumbnail (multipart, max 5 MB) |
| `GET` | `/library/{name}/autofill` | Fetch RAWG candidates for autofill |
| `POST` | `/library/{name}/autofill` | Confirm autofill from RAWG |

**Groups**

| Method | Path | Description |
|---|---|---|
| `GET` | `/groups?q=` | Search public groups |
| `POST` | `/groups` | Create group (public or private) |
| `GET` | `/groups/mine` | Own and member groups |
| `GET` | `/groups/{id}` | Group detail |
| `POST` | `/groups/{id}/join` | Join a public group |
| `DELETE` | `/groups/{id}` | Delete group |
| `GET` | `/groups/{id}/sessions` | Sessions belonging to group |
| `GET` | `/groups/{id}/invitable` | Friends eligible for group invitation |
| `POST` | `/groups/{id}/invite` | Invite a friend to a private group |

**Group invitations**

| Method | Path | Description |
|---|---|---|
| `GET` | `/invitations` | Pending group invitations for current user |
| `POST` | `/invitations/{id}/accept` | Accept group invitation |
| `DELETE` | `/invitations/{id}` | Decline group invitation |

**Sessions**

| Method | Path | Description |
|---|---|---|
| `GET` | `/sessions` | Session feed (own groups + global, future only) |
| `GET` | `/sessions/mine` | Own created sessions |
| `POST` | `/sessions` | Create session |
| `GET` | `/sessions/{id}` | Session detail |
| `POST` | `/sessions/{id}/join` | Join session |
| `DELETE` | `/sessions/{id}/join` | Leave session |
| `DELETE` | `/sessions/{id}` | Delete session |

**Push notifications**

| Method | Path | Description |
|---|---|---|
| `POST` | `/push/subscribe` | Save push subscription |
| `DELETE` | `/push/subscribe` | Remove push subscription |

**Admin only**

| Method | Path | Description |
|---|---|---|
| `POST` | `/invites` | Create a one-time user invite link |

### Server-Sent Events

| Path | Auth | Description |
|---|---|---|
| `GET /groups/{id}/events?token=<jwt>` | Query param | Real-time session events per group |

Events: `session_created`, `session_joined`, `session_deleted`.

## Module structure

```
src/
├── config/       # Settings loader, AppState, DB connection
├── controller/   # Axum route handlers (one file per domain)
├── dto/          # Request / response types (separate from domain models)
├── error/        # AppError enum (Unauthorized, Validation, Database, Internal)
├── middleware/
│   ├── auth.rs       # JWT validation, AuthUser extraction
│   └── security.rs   # Security response headers (X-Frame-Options, CSP, …)
├── model/        # Domain models (User, Group, Session, Game, …)
├── repository/   # SurrealDB queries, trait abstractions
└── service/      # Business logic
```

## Tests

```sh
cargo test
```

Repositories are abstracted behind traits (`async-trait`) so services are unit-testable with `PanicRepo` mock implementations that panic on any call not exercised by the test.

## Docker

```sh
# Build (from repo root)
docker build -t clync-backend backend/

# Override config at runtime via environment variables
docker run \
  -e APP__DATABASE__URL=surrealdb:8000 \
  -e APP__DATABASE__PASSWORD=secret \
  -e APP__VAPID__PRIVATE_KEY=... \
  clync-backend

# Or mount a config override file
docker run -v ./prod.yaml:/app/config/app.local.yaml:ro clync-backend
```

Multi-stage build:
- `rust:1-bookworm` — deps cache layer + full compile with `--release`
- `debian:bookworm-slim` — runtime (~60 MB); only `libssl3`, `ca-certificates`, `wget`

Security:
- Runs as dedicated system user `appuser` (UID 10001, no home, no login shell)
- Release profile: `lto = "thin"`, `codegen-units = 1`, `strip = true`, `panic = "abort"`
- `HEALTHCHECK` via `wget` against `/api/v1/health`
