# Infrastructure

Docker Compose configurations for local development and production deployment.

## Files

| File | Purpose |
|---|---|
| `docker-compose.local.yml` | Local development — services with exposed ports, dev-mode Keycloak |
| `docker-compose.yml` | Production — Traefik-ready, no exposed ports, healthchecks |
| `docker-compose.paths.yml` | Optional override to mount local source paths |
| `.env.example` | Template for production environment variables |
| `keycloak/WhatsUp-realm.json` | Keycloak realm export (auto-imported in local dev) |

## Local development

Starts Keycloak (with auto-imported realm), SurrealDB, and PostgreSQL. Backend and frontend run separately on the host.

```sh
docker compose -f docker-compose.local.yml up -d
```

| Service | URL | Credentials |
|---|---|---|
| Keycloak | http://localhost:8080 | admin / admin |
| SurrealDB | ws://localhost:8000 | root / root |

Then in separate terminals:

```sh
# Backend
cd ../backend && cargo run

# Frontend
cd ../frontend && npm run dev
```

The Keycloak realm `WhatsUp` is imported automatically on first start from `keycloak/WhatsUp-realm.json`.

> **First user:** create an invite link via the app's user menu, visit the link to register, then optionally assign the `admin` role in Keycloak.

## Production deployment

Designed for a server running Traefik as reverse proxy. Traefik handles TLS termination and routing — no ports are exposed directly on the host.

### Prerequisites

- Docker / Podman with Compose
- Traefik running and attached to an external network named `proxy`
- DNS records pointing to the three hostnames

Create the proxy network once if it does not exist yet:

```sh
docker network create proxy
```

### First deployment

```sh
# 1. Create environment file
cp .env.example .env

# 2. Fill in all values (see Environment variables below)
$EDITOR .env

# 3. Build images (frontend bakes VITE_* URLs at this step)
docker compose build

# 4. Start
docker compose up -d
```

> **Keycloak note:** Production mode (`start`) does not auto-import the realm. Configure the `WhatsUp` realm manually via the Keycloak admin UI after first startup, or use `kcadm.sh` / the REST API to import `keycloak/WhatsUp-realm.json`.

### Subsequent deploys

```sh
docker compose build   # rebuild changed images
docker compose up -d   # rolling restart
```

## Environment variables

Copy `.env.example` to `.env` and fill in every value. The file is gitignored.

```sh
# Hostnames — Traefik uses these for routing rules
FRONTEND_HOST=whatsup.example.com
API_HOST=api.whatsup.example.com
KEYCLOAK_HOST=auth.whatsup.example.com

# Frontend URLs — baked into the image at docker compose build time
VITE_API_URL=https://api.whatsup.example.com
VITE_KEYCLOAK_URL=https://auth.whatsup.example.com
VITE_KEYCLOAK_REALM=WhatsUp
VITE_KEYCLOAK_CLIENT_ID=whatsup

# Backend CORS origin
FRONTEND_URL=https://whatsup.example.com

# VAPID keys for Web Push — generate fresh keys for production:
#   npx web-push generate-vapid-keys
VAPID_PUBLIC_KEY=
VAPID_PRIVATE_KEY=
VAPID_SUBJECT=mailto:admin@whatsup.example.com

# RAWG API key for game library autofill (optional, free at rawg.io/apidocs)
RAWG_API_KEY=

# SurrealDB credentials
SURREALDB_USER=surrealdb
SURREALDB_PASS=<strong-password>

# Keycloak bootstrap admin (used only on first start)
KC_ADMIN_USER=admin
KC_ADMIN_PASS=<strong-password>

# PostgreSQL (Keycloak backend only)
POSTGRES_USER=keycloak
POSTGRES_PASS=<strong-password>
```

## Network architecture

```
Internet
    │
  Traefik  ── proxy network (external)
    ├── frontend   :8080  (nginx-unprivileged, static SPA, non-root)
    ├── backend    :3000  (Rust API)
    └── keycloak   :8080  (OIDC)

internal network (private, no Traefik attachment)
    ├── backend    ──→ surrealdb :8000
    ├── backend    ──→ keycloak  :8080  (JWKS refresh, bypasses Traefik)
    ├── keycloak   ──→ postgres  :5432
    ├── surrealdb  (internal only)
    └── postgres   (internal only)
```

## Healthchecks

| Service | Tool | Endpoint | Start period |
|---|---|---|---|
| frontend | `wget` (busybox) | `http://localhost:8080/` | 10 s |
| backend | `wget` | `http://localhost:3000/api/v1/health` | 15 s |
| keycloak | bash TCP | port 9000 `/health/ready` | 90 s |
| surrealdb | `/surreal is-ready` | `http://localhost:8000` | 20 s |
| postgres | `pg_isready` | — | — |

The backend waits for `surrealdb` and `keycloak` to be healthy before starting (`depends_on: condition: service_healthy`).
