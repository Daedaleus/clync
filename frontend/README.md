# Frontend

React SPA built with Vite. Handles auth via Keycloak JS, communicates with the backend REST API, and supports Web Push as an installable PWA.

## Technology

- **React 19** + **TypeScript**
- **Vite 8** — build tool and dev server
- **Tailwind CSS 4** — utility-first styling (dark mode, mobile-first)
- **React Router 7** — client-side routing
- **Keycloak JS 26** — OpenID Connect auth (token auto-refresh)
- **Vitest** + **React Testing Library** — unit tests
- **Atomic Design** — component hierarchy (atoms → molecules → organisms → templates → pages)

## Setup

**Prerequisites:** Node 26, running Keycloak and backend (see `infrastructure/`).

```sh
cd frontend
npm install
npm run dev     # http://localhost:5173
```

## Scripts

| Script | Description |
|---|---|
| `npm run dev` | Vite dev server with HMR |
| `npm run build` | TypeScript check + production build → `dist/` |
| `npm run preview` | Preview the production build locally |
| `npm run lint` | ESLint |
| `npm test` | Vitest in watch mode |
| `npm run test:run` | Vitest single run (CI) |

## Configuration

All environment variables are centralised in `src/config.ts`. Never read `import.meta.env` directly elsewhere.

| Variable | Default | Description |
|---|---|---|
| `VITE_API_URL` | `http://localhost:3000` | Backend base URL |
| `VITE_KEYCLOAK_URL` | `http://localhost:8080` | Keycloak base URL |
| `VITE_KEYCLOAK_REALM` | `Clync` | Keycloak realm |
| `VITE_KEYCLOAK_CLIENT_ID` | `clync` | Keycloak client ID |

Defaults are committed in `.env`. For local overrides create `.env.local` (gitignored):

```sh
cp .env.local.example .env.local
```

> **Note:** Vite bakes `VITE_*` variables into the bundle at build time. Changing them after `npm run build` requires a rebuild.

## Routes

| Path | Page | Auth | Description |
|---|---|---|---|
| `/` | → `/me` | Required | Redirect |
| `/me` | MePage | Required | Dashboard — own sessions, wishlist, pending group invitations |
| `/profile` | ProfilePage | Required | Edit games, Steam/Discord handles and visibility, Keycloak account link |
| `/groups` | GroupsPage | Required | Browse public groups, create groups |
| `/groups/:id` | GroupDetailPage | Required | Members, common/possible games, session calendar, invite friends |
| `/friends` | FriendsPage | Required | Friend list, user search, add/remove |
| `/users/:id` | UserPage | Required | Public profile — games, common games, social handles (respects visibility) |
| `/sessions` | SessionsPage | Required | Full session feed (groups + global) |
| `/sessions/:id` | SessionDetailPage | Required | Session detail, participant list, join/leave |
| `/library` | LibraryPage | Required | Game library — browse, add games, RAWG autofill |
| `/library/:name` | GameDetailPage | Required | Game detail — edit metadata, upload thumbnail, delete |
| `/join/:token` | JoinPage | Public | Invite link registration form |

## Component structure

```
src/
├── components/
│   ├── atoms/
│   │   ├── Avatar           # User avatar circle with initials
│   │   ├── Badge            # Coloured status chip
│   │   ├── Button           # Styled button
│   │   ├── Input            # Styled text input
│   │   ├── SectionLabel     # Section heading
│   │   └── VisibilitySelect # Public / Group / Friends visibility selector
│   ├── molecules/
│   │   ├── AutofillPicker   # RAWG candidate card grid (thumbnail selection)
│   │   ├── ErrorBanner      # Dismissible error message
│   │   ├── GameCard         # Game tile with thumbnail, genre badge
│   │   ├── GameChip         # Compact game name tag
│   │   ├── GameInput        # Free-text game name input (legacy)
│   │   ├── GamePicker       # Searchable game picker backed by /library
│   │   ├── InviteModal      # Friend invite overlay for private groups
│   │   ├── SearchBar        # Debounced search field
│   │   ├── SessionRow       # Single session list entry with join/delete actions
│   │   └── UserRow          # User list entry with friend controls
│   ├── organisms/
│   │   ├── AppHeader        # Top nav — links, user menu, notifications bell
│   │   ├── GameSection      # Game wishlist manager (add / remove)
│   │   ├── SessionCreateForm# New session form — game picker, date/time, scope
│   │   ├── SessionList      # Rendered list of SessionRows with empty state
│   │   └── WeekCalendar     # Weekly grid of sessions
│   └── templates/
│       └── PageLayout       # Wraps header + page content
├── pages/                   # Route-level components (see Routes above)
├── services/
│   ├── api.ts               # Authenticated fetch wrapper — all API calls go here
│   └── auth.ts              # Keycloak instance (singleton)
├── hooks/
│   └── useNotifications.ts  # Web Push subscription state (enable / disable / notify)
├── utils/
│   ├── auth.ts              # isAdmin() — UI-only gate, reads Keycloak token
│   └── date.ts              # Date formatting helpers
├── config.ts                # Environment variable access point
└── types.ts                 # Shared TypeScript types
```

## Tests

```sh
npm run test:run
```

Tests live next to their components (`*.test.tsx` / `*.test.ts`). Setup file at `src/test/setup.ts`. Components that use React Router are wrapped in `MemoryRouter`.

## Docker

Vite bakes environment variables at build time, so pass them as build args:

```sh
docker build \
  --build-arg VITE_API_URL=https://api.example.com \
  --build-arg VITE_KEYCLOAK_URL=https://auth.example.com \
  -t clync-frontend frontend/
```

Multi-stage build:
- `node:22-alpine` — `npm ci --ignore-scripts` + `npm run build`
- `nginxinc/nginx-unprivileged:1.27-alpine` — serves static files on **port 8080** as a non-root user (UID 101)

nginx features:
- SPA fallback routing (`try_files`)
- `Cache-Control: immutable` on hashed Vite asset files (`/assets/`)
- `Cache-Control: no-store` on service worker and PWA manifest
- `server_tokens off` — no nginx version in headers or error pages
- gzip for JS, CSS, JSON, SVG
- Security headers on all responses: `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`, `Permissions-Policy`
- `HEALTHCHECK` against `http://127.0.0.1:8080/`

JS bundle is split into three chunks for better browser caching:
- `vendor` — React, React DOM, React Router (~231 KB)
- `keycloak` — Keycloak JS (~26 KB)
- `index` — application code (~76 KB)
