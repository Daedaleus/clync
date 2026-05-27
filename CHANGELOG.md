# Changelog

All notable changes to this project are documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [0.4.0] — 2026-05-27

### Fixed

- Session form now uses a single `datetime-local` input instead of separate date and time fields — eliminates the overlap and sizing inconsistency on iOS

### Dependencies

- `react-router-dom` 7.15.0 → 7.15.1
- `@vitejs/plugin-react` 6.0.1 → 6.0.2
- `eslint` 10.3.0 → 10.4.0
- `storybook` 10.4.0 → 10.4.1
- `@types/node` 25.8.0 → 25.9.1
- `serde_json` 1.0.149 → 1.0.150 (backend)
- `tower-http` 0.6.10 → 0.6.11 (backend)

---

## [0.3.0] — 2026-05-21

### Added

- **Session invitations** — any session participant can invite mutual friends directly to a session; pending invitations appear in the inbox
- **Push notifications for invitations** — friend requests, group invitations and session invitations trigger a Web Push to the recipient
- **Session-start push notification** — creator and all participants are notified 5 minutes before the session begins
- **Session notes** — optional free-text field (max 500 characters) for sharing server address, password, Discord link, etc.
- **Stale-data cleanup** — hourly background task removes sessions more than 24 h past their start time and expired session invitations
- **Storybook** — component library covering all atoms, molecules and organisms

### Fixed

- Sessions disappeared immediately after creation because `scheduled_at` was stored as a string, breaking SurrealDB datetime comparisons
- Date and time pickers had different heights and overlapped on iOS (iPhone)
- `ORDER BY created_at` in session-invitation query failed in SurrealDB 2.x (field must appear in SELECT)
- Push subscription is now re-created automatically when iOS silently revokes it — no manual re-enable required
- `EventSource` reconnects with a fresh Keycloak token after the access token is refreshed, preventing silent 401 errors reported as CORS failures in Firefox

---

## [0.2.0] — 2026-05-15

### Added

- **Friend system** — send, accept and decline friend requests; bidirectional friendship gates social data visibility
- **User profiles** — public groups visible on other users' profiles
- **Discord invite link** — group admins can attach a Discord invite URL to a group
- **Late-join window** — sessions remain visible and joinable for up to 2 hours after their scheduled start time
- **SVG logo and favicon** — replaced raster assets

### Fixed

- Deploy script image-tag mismatch and missing `git pull` reminder

---

## [0.1.0] — 2026-05-14

Initial release.

### Added

- **Invite-only registration** — admin creates single-use invite links; Keycloak open registration stays disabled
- **Groups** — public (open join) and private (invite-only); Discord invite link
- **Game library** — shared catalogue with name, description, genre and thumbnail; optional RAWG.io autofill
- **Sessions** — schedule play sessions per game (15-min slots); global or group scope
- **Real-time updates** — session created / joined / deleted events streamed via SSE
- **Web Push (VAPID)** — background notifications as an installable PWA
- **Admin role** — granted in Keycloak; unlocks invite creation and destructive operations
- **Infrastructure** — Docker Compose for local dev and production (Traefik reverse proxy); `deploy.sh` builds with Podman and streams images to the server via SSH
- **CI** — GitHub Actions pipeline with Clippy, tests, and frontend build checks
