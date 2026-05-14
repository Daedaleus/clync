# General

- Challenge me critically when needed before doing anything
- Use best practices and current versions for each language/ecosystem
- No `Co-Authored-By` lines in commit messages

---

# Project Overview

**WhatsUp** is a fullstack group gaming coordination app.

- **Backend**: Rust — REST API, business logic, real-time events
- **Frontend**: React SPA — UI, API integration, PWA
- **Infrastructure**: Docker-based dev and production environment

Users form groups, maintain game wishlists, and schedule play sessions with live updates and push notifications.

---

# Architecture Principles

- Strict frontend/backend separation
- API-first design — no business logic in the frontend
- Stateless backend services
- Containerised deployment and container-assisted development
- All API calls go through `frontend/src/services/api.ts` — never fetch directly in components

---

# Backend (Rust)

## Technology

- **Rust** (stable)
- **Axum 0.8** — HTTP framework
- **SurrealDB 2** — database via WebSocket client
- **Keycloak 26** — OIDC/JWT authentication (JWKS validation, 1 h cache)
- **SSE** — real-time events via `tokio::sync::broadcast`
- **Web Push (VAPID)** — push notifications
- **RAWG API** — optional game metadata autofill

## Principles

- Structure domains modularly (controller → service → repository)
- No `unwrap()` — use proper error propagation with `?`
- DTOs separate from domain models
- `AppError` enum: `Unauthorized` / `Validation` (user-facing) / `Database` / `Internal` (logged, generic response)
- Consistent API error shape: `{ "error": "message" }`

## Code Quality

- `cargo fmt` mandatory before commit
- `cargo clippy -- -D warnings` must pass with zero warnings
- `cargo test` must pass

---

# Frontend (React)

## Technology

- **React 19** + **TypeScript**
- **Vite 8** — build tool and dev server
- **Tailwind CSS 4** — utility-first styling
- **React Router 7** — client-side routing
- **Keycloak JS 26** — OIDC auth with automatic token refresh
- **Vitest** + **React Testing Library** — unit tests
- **Atomic Design** — atoms → molecules → organisms → templates → pages

## Principles

- UI / logic separation — no side effects in pure UI components
- API calls only in `src/services/` — never inline `fetch` in components
- Reusability — components receive data via props, not by fetching it themselves
- No business logic in the frontend — validate only at system boundaries

## Code Quality

- `npm run lint` must pass with zero errors
- `npm run test:run` must pass
- `npm run build` (TypeScript check + Vite build) must succeed

---

# API

- REST/JSON, versioned under `/api/v1/`
- Centralised API client layer (`api.ts`) — authenticated fetch wrapper
- Uniform error responses: `{ "error": "message" }`
- SSE for real-time events: `/api/v1/groups/{id}/events?token=<jwt>`

---

# Infrastructure

## Goals

- Reproducibility
- Simple local development
- Clear service separation

## Local dev stack

| Service | URL | Credentials |
|---|---|---|
| Keycloak | http://localhost:8080 | admin / admin |
| SurrealDB | ws://localhost:8000 | root / root |

Backend runs on `http://localhost:3000`, frontend dev server on `http://localhost:5173`.

---

# Security

- **No secrets in code or git** — use `.env.local` / `config/app.local.yaml` (both gitignored)
- **No `.env` files with real secrets committed** — only example files with placeholder values
- **Input validation in the backend** — never trust frontend data
- **CORS explicitly configured** — allow only the known frontend origin
- **Security headers** on all responses (backend middleware + nginx): `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`, `Permissions-Policy`
- **Body size limits** — 2 MB global JSON limit, manual 5 MB check for thumbnail uploads
- **Least privilege** — Docker containers run as non-root users where possible
- **No secrets in logs** — never log tokens, passwords, or private keys

---

# Code Standards

## Rust

- `rustfmt` enforced
- `clippy -- -D warnings` — zero warnings policy
- Clean error handling — no `unwrap()`, no `expect()` in production paths

## React / TypeScript

- ESLint — zero errors policy
- TypeScript strict — no `any` unless genuinely unavoidable
- No side effects in render functions

---

# Branching Strategy

Branch names follow the same type vocabulary as commits:

| Type | Branch pattern | Example |
|---|---|---|
| New feature | `feat/<topic>` | `feat/group-invitations` |
| Bug fix | `fix/<topic>` | `fix/keycloak-redirect` |
| Refactor | `refactor/<topic>` | `refactor/session-service` |
| Documentation | `docs/<topic>` | `docs/api-endpoints` |
| Build / infra | `build/<topic>` | `build/nginx-hardening` |
| CI/CD | `ci/<topic>` | `ci/add-clippy-check` |
| Chore | `chore/<topic>` | `chore/update-deps` |

- Use lowercase and hyphens — no underscores, no camelCase
- Keep topics short and descriptive (2–4 words max)
- One logical change per branch

---

# Commit Convention

This project follows [Conventional Commits](https://www.conventionalcommits.org/).

## Format

```
<type>(<scope>): <emoji> <short description>

[optional body — bullet points for details]
```

- **type** — see table below
- **scope** — affected area (see scope examples)
- **emoji** — one emoji directly after the colon, before the description
- **description** — imperative mood, lowercase, no period, English

## Types

| Type | Emoji | When to use |
|---|---|---|
| `feat` | ✨ | New user-facing feature |
| `fix` | 🐛 | Bug fix |
| `refactor` | ♻️ | Code improvement without behaviour change |
| `style` | 🎨 | Formatting, UI-only styling |
| `perf` | ⚡ | Performance improvement |
| `test` | 🧪 | Adding or updating tests |
| `docs` | 📚 | Documentation only |
| `build` | 🔧 | Build system, Docker, dependencies, infra |
| `ci` | ⚙️ | CI/CD pipeline changes |
| `chore` | 🔨 | Tooling, config, maintenance (no production code) |
| `revert` | ⏪ | Reverts a previous commit |

> **Security fixes** use `fix` with a security-relevant description and 🔐 emoji.
> **Infrastructure changes** use `build` — this maps to the standard Conventional Commits `build` type.

## Scope Examples

`backend` · `frontend` · `api` · `auth` · `db` · `ui` · `docker` · `ci` · `infra` · `deps`

## Examples

```
feat(backend): ✨ add group invitation endpoint
fix(frontend): 🐛 resolve token refresh loop on inactivity
refactor(backend): ♻️ extract session validation into service layer
build(docker): 🔧 switch nginx to unprivileged image on port 8080
fix(auth): 🔐 box surrealdb::Error to reduce AppError enum size
docs(readme): 📚 update deployment instructions for path-based routing
ci(github): ⚙️ add clippy -D warnings check to backend job
test(frontend): 🧪 wrap SessionList tests in MemoryRouter
```

## Rules

- One logical change per commit
- Clear technical description — no vague messages ("fix bug", "update code", "stuff")
- Imperative mood ("add", "fix", "remove" — not "added", "fixed", "removed")
- English as standard
- Body optional but encouraged for non-obvious changes

---

# Pull Request Guidelines

## Title

Follows the same format as a commit message:
```
feat(backend): ✨ add RAWG autofill for game library
```

## Body

```markdown
## Summary
- What changed and why (2–5 bullet points)

## Test plan
- [ ] Unit tests pass (`cargo test` / `npm run test:run`)
- [ ] Linting passes (`cargo clippy -- -D warnings` / `npm run lint`)
- [ ] Manually tested: <what you clicked/tested>
- [ ] No regressions in adjacent features

## Related
Closes #<issue> (if applicable)
```

## Rules

- PRs target `main` via a feature/fix branch — no direct pushes to `main`
- One logical feature or fix per PR — keep scope small
- Self-review before requesting review — read your own diff
- CI must be green before merge
