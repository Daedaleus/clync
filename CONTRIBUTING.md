# Contributing

Thanks for your interest in WhatsUp!

## Dev environment

Follow the [Quick start](README.md#quick-start-local-development) in the README. You need `mise` and Docker / Podman.

```sh
mise install          # pin Node + Rust versions
mise run docker       # start SurrealDB + Keycloak
mise run backend      # Rust API on :3000
mise run frontend     # Vite dev server on :5173
```

## Branching

Branch names follow the same type vocabulary as commits:

| Type | Pattern | Example |
|---|---|---|
| Feature | `feat/<topic>` | `feat/group-invitations` |
| Bug fix | `fix/<topic>` | `fix/keycloak-redirect` |
| Refactor | `refactor/<topic>` | `refactor/session-service` |
| Docs | `docs/<topic>` | `docs/api-endpoints` |
| Build / infra | `build/<topic>` | `build/nginx-hardening` |
| CI | `ci/<topic>` | `ci/add-clippy-check` |
| Chore | `chore/<topic>` | `chore/update-deps` |

- Lowercase and hyphens only
- One logical change per branch
- Never commit directly to `main`

## Commit convention

This project follows [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <emoji> <short description>
```

Common types: `feat` ✨ `fix` 🐛 `refactor` ♻️ `test` 🧪 `docs` 📚 `build` 🔧 `chore` 🔨

Examples:
```
feat(backend): ✨ add group invitation endpoint
fix(frontend): 🐛 resolve token refresh loop on inactivity
test(backend): 🧪 add unit tests for session service
```

## Code quality

Run all checks before opening a PR:

```sh
mise run check
```

This runs `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `npm run lint` and `npm run test:run`.

### Backend (Rust)
- `cargo fmt` — mandatory, no exceptions
- `cargo clippy -- -D warnings` — zero warnings
- No `unwrap()` or `expect()` in production paths

### Frontend (TypeScript)
- `npm run lint` — zero ESLint errors
- `npm run test:run` — all unit tests must pass
- `npm run build` — TypeScript check must succeed

## Pull requests

- One logical change per PR
- Title follows the same format as a commit message
- CI must be green before merge
- PRs target `main` via a feature branch — no direct pushes

## Reporting issues

Please use the GitHub issue tracker. For security vulnerabilities, see [SECURITY.md](SECURITY.md).
