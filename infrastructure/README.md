# Infrastructure

Docker Compose configurations for local development and production deployment.

## Files

| File | Purpose |
|---|---|
| `docker-compose.local.yml` | Local development — services with exposed ports, dev-mode Keycloak |
| `docker-compose.yml` | Production — Traefik-ready, no exposed ports, healthchecks |
| `docker-compose.paths.yml` | Optional override to mount local source paths |
| `.env.example` | Template for production environment variables |
| `keycloak/Clync-realm.json` | Keycloak realm export (auto-imported in local dev) |

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

The Keycloak realm `Clync` is imported automatically on first start from `keycloak/Clync-realm.json`.

> **First user:** create an invite link via the app's user menu, visit the link to register, then optionally assign the `admin` role in Keycloak.

## Production deployment

Designed for a server running Traefik as reverse proxy. Traefik handles TLS termination and routing — no ports are exposed directly on the host.

### Prerequisites

- Podman (local) and Docker (server)
- Traefik running on the server, attached to an external network named `proxy`
- DNS records pointing to your domain(s)

Create the proxy network on the server once if it does not exist yet:

```sh
docker network create proxy
```

### Deploying

Use the interactive deploy script from your local machine — it handles everything:

```sh
cd infrastructure
./deploy.sh
```

On first run it will ask for:
- SSH target and remote path on the server
- Routing mode: **subdomains** (frontend.de / api.frontend.de / auth.frontend.de) or **paths** (one domain, `/api` and `/auth` prefixes)
- Domain name(s)
- VAPID keys (can be generated automatically)
- All passwords (can be generated automatically)
- RAWG API key (optional)

It then builds images locally, uploads `docker-compose.yml` + `.env` to the server, streams the images via SSH and optionally restarts services. All answers are saved in `.deploy.conf` (gitignored) so subsequent runs only ask for confirmation.

> **Keycloak note:** Production mode does not auto-import the realm. After first startup, configure the `Clync` realm manually via the Keycloak admin UI or import `keycloak/Clync-realm.json` via `kcadm.sh`.

### Reference: environment variables

See `.env.example` (subdomains) or `.env-paths.example` (paths routing) for all available variables.

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
