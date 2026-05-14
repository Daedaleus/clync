# WhatsUp

Group gaming coordination app. Users form groups, list games they want to play, and arrange sessions — with live updates and push notifications.

## Stack

| Layer | Technology |
|---|---|
| Backend | Rust · Axum 0.8 · SurrealDB 2 |
| Frontend | React 19 · TypeScript · Vite 8 · Tailwind CSS 4 |
| Auth | Keycloak 26 (OpenID Connect / JWT) · invite-only registration |
| Real-time | Server-Sent Events |
| Push | Web Push (VAPID) — works as installable PWA |
| Infrastructure | Docker / Podman · Traefik (production) |

## Quick start (local development)

**Prerequisites:** Rust (stable), Node 22, Docker / Podman

```sh
# 1 — Start backing services
docker compose -f infrastructure/docker-compose.local.yml up -d

# 2 — Backend (http://localhost:3000)
cd backend && cargo run

# 3 — Frontend (http://localhost:5173)
cd frontend && npm install && npm run dev
```

Login at `http://localhost:5173` — Keycloak runs on `http://localhost:8080` (admin: admin/admin).

> **First user:** create an invite link via the user menu → "Einladen", visit the link, register — then assign the `admin` role in Keycloak if needed.

## Project structure

```
whatsup/
├── backend/          # Rust API — see backend/README.md
├── frontend/         # React SPA — see frontend/README.md
├── infrastructure/   # Docker Compose configs — see infrastructure/README.md
└── CLAUDE.md         # Coding standards and commit convention
```

## Core features

- **Invite-only registration** — admin creates invite links; Keycloak open registration stays disabled
- **User profiles** — game wishlists, Steam and Discord handles (with per-field visibility: public / group members / mutual friends only)
- **Friend system** — bidirectional; mutual friendship gates social data visibility
- **Groups** — public (joinable by anyone) and private (invite-only)
- **Game library** — shared catalogue of games with name, description, genre, and thumbnail; RAWG.io autofill
- **Sessions** — schedule play sessions per game with date/time (15-min slots); group or global scope
- **Real-time updates** — session created/joined/deleted events via SSE
- **Web Push notifications** — foreground and background notifications as an installable PWA
- **Admin role** — granted via Keycloak; unlocks invite creation and destructive operations
