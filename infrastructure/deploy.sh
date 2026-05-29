#!/usr/bin/env bash
# Clync deploy script
# Asks all config questions, generates .env + uploads it along with
# docker-compose.yml, builds images locally and streams them to the server.
# All answers are saved in .deploy.conf so subsequent runs only need Y/n.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
CONF_FILE="${SCRIPT_DIR}/.deploy.conf"

# ── Colours ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GRN='\033[0;32m'; YLW='\033[1;33m'
CYN='\033[0;36m'; BLD='\033[1m';   RST='\033[0m'

info() { echo -e "${CYN}==>${RST} $*"; }
ok()   { echo -e "${GRN} ✔${RST}  $*"; }
warn() { echo -e "${YLW} ⚠${RST}  $*"; }
err()  { echo -e "${RED} ✖${RST}  $*" >&2; exit 1; }
hr()   { echo -e "${BLD}────────────────────────────────────────${RST}"; }

# ── Load saved config ──────────────────────────────────────────────────────────
[[ -f "$CONF_FILE" ]] && source "$CONF_FILE"

# ── Helpers ────────────────────────────────────────────────────────────────────
ask() {
  # ask VARNAME "Label" "default"
  local _v="$1" _l="$2" _d="${3:-}" _i
  if [[ -n "$_d" ]]; then
    read -rp "$(echo -e "  ${BLD}${_l}${RST} [${CYN}${_d}${RST}]: ")" _i
    printf -v "$_v" '%s' "${_i:-$_d}"
  else
    read -rp "$(echo -e "  ${BLD}${_l}${RST}: ")" _i
    while [[ -z "$_i" ]]; do
      warn "Required."
      read -rp "$(echo -e "  ${BLD}${_l}${RST}: ")" _i
    done
    printf -v "$_v" '%s' "$_i"
  fi
}

ask_secret() {
  # ask_secret VARNAME "Label" "default (masked)"
  local _v="$1" _l="$2" _d="${3:-}" _i
  if [[ -n "$_d" ]]; then
    read -rsp "$(echo -e "  ${BLD}${_l}${RST} [saved, Enter to keep]: ")" _i
    echo ""
    printf -v "$_v" '%s' "${_i:-$_d}"
  else
    read -rsp "$(echo -e "  ${BLD}${_l}${RST}: ")" _i
    echo ""
    while [[ -z "$_i" ]]; do
      warn "Required."
      read -rsp "$(echo -e "  ${BLD}${_l}${RST}: ")" _i
      echo ""
    done
    printf -v "$_v" '%s' "$_i"
  fi
}

confirm() {
  local _l="$1" _d="${2:-y}" _i
  read -rp "$(echo -e "  ${BLD}${_l}${RST} [$([ "$_d" = y ] && echo "Y/n" || echo "y/N")]: ")" _i
  _i="${_i:-$_d}"
  [[ "${_i,,}" == "y" ]]
}

save_conf() {
  cat > "$CONF_FILE" <<EOF
# Clync deploy config — gitignored, do not commit
CONF_SERVER="${SERVER}"
CONF_REMOTE_DIR="${REMOTE_DIR}"
CONF_ROUTING="${ROUTING}"
CONF_DOMAIN="${DOMAIN}"
CONF_FRONTEND_HOST="${FRONTEND_HOST}"
CONF_API_HOST="${API_HOST}"
CONF_KEYCLOAK_HOST="${KEYCLOAK_HOST}"
CONF_VAPID_PUBLIC="${VAPID_PUBLIC_KEY}"
CONF_VAPID_PRIVATE="${VAPID_PRIVATE_KEY}"
CONF_VAPID_SUBJECT="${VAPID_SUBJECT}"
CONF_RAWG_API_KEY="${RAWG_API_KEY:-}"
CONF_DB_USER="${SURREALDB_USER}"
CONF_DB_PASS="${SURREALDB_PASS}"
CONF_KC_ADMIN_USER="${KC_ADMIN_USER}"
CONF_KC_ADMIN_PASS="${KC_ADMIN_PASS}"
CONF_PG_USER="${POSTGRES_USER}"
CONF_PG_PASS="${POSTGRES_PASS}"
EOF
  ok "Config saved to .deploy.conf"
}

# ── Banner ─────────────────────────────────────────────────────────────────────
echo ""
echo -e "${BLD}  Clync Deploy${RST}"
hr

# ── Use saved config? ──────────────────────────────────────────────────────────
RECONFIGURE=true
if [[ -n "${CONF_SERVER:-}" ]]; then
  echo ""
  echo -e "  Saved config:"
  echo -e "    Server:   ${CYN}${CONF_SERVER}${RST}"
  echo -e "    Dir:      ${CYN}${CONF_REMOTE_DIR}${RST}"
  echo -e "    Domain:   ${CYN}${CONF_DOMAIN:-?}${RST}"
  echo ""
  if confirm "Use saved config?" y; then
    RECONFIGURE=false
    SERVER="$CONF_SERVER"
    REMOTE_DIR="$CONF_REMOTE_DIR"
    ROUTING="${CONF_ROUTING:-subdomains}"
    DOMAIN="${CONF_DOMAIN:-}"
    FRONTEND_HOST="${CONF_FRONTEND_HOST:-}"
    API_HOST="${CONF_API_HOST:-}"
    KEYCLOAK_HOST="${CONF_KEYCLOAK_HOST:-}"
    VAPID_PUBLIC_KEY="${CONF_VAPID_PUBLIC:-}"
    VAPID_PRIVATE_KEY="${CONF_VAPID_PRIVATE:-}"
    VAPID_SUBJECT="${CONF_VAPID_SUBJECT:-}"
    RAWG_API_KEY="${CONF_RAWG_API_KEY:-}"
    SURREALDB_USER="${CONF_DB_USER:-surrealdb}"
    SURREALDB_PASS="${CONF_DB_PASS:-}"
    KC_ADMIN_USER="${CONF_KC_ADMIN_USER:-admin}"
    KC_ADMIN_PASS="${CONF_KC_ADMIN_PASS:-}"
    POSTGRES_USER="${CONF_PG_USER:-keycloak}"
    POSTGRES_PASS="${CONF_PG_PASS:-}"
  fi
fi

# ── Config questions ───────────────────────────────────────────────────────────
if $RECONFIGURE; then
  echo ""
  echo -e "  ${BLD}── Server ─────────────────────────────${RST}"
  echo ""
  ask SERVER    "SSH target (alias or user@host)" "${CONF_SERVER:-}"
  ask REMOTE_DIR "Remote app directory (docker-compose + .env live here)" "${CONF_REMOTE_DIR:-/opt/clync}"

  echo ""
  echo -e "  ${BLD}── Routing ────────────────────────────${RST}"
  echo ""
  echo -e "    ${BLD}1)${RST} Subdomains  — frontend.de / api.frontend.de / auth.frontend.de"
  echo -e "    ${BLD}2)${RST} Paths       — one domain: domain.de / domain.de/api / domain.de/auth"
  echo ""
  ask ROUTING_CHOICE "Routing mode (1 or 2)" "$([ "${CONF_ROUTING:-subdomains}" = paths ] && echo 2 || echo 1)"
  if [[ "$ROUTING_CHOICE" == "2" ]]; then
    ROUTING="paths"
  else
    ROUTING="subdomains"
  fi

  echo ""
  echo -e "  ${BLD}── Hostnames ──────────────────────────${RST}"
  echo ""
  ask DOMAIN "Domain (e.g. pkeil.de)" "${CONF_DOMAIN:-}"

  if [[ "$ROUTING" == "paths" ]]; then
    FRONTEND_HOST="$DOMAIN"
    API_HOST="$DOMAIN"
    KEYCLOAK_HOST="$DOMAIN"
    ok "All services on ${DOMAIN} — frontend /, API /api, Keycloak /auth"
  else
    ask FRONTEND_HOST "Frontend hostname" "${CONF_FRONTEND_HOST:-${DOMAIN}}"
    ask API_HOST      "API hostname"      "${CONF_API_HOST:-api.${DOMAIN}}"
    ask KEYCLOAK_HOST "Keycloak hostname" "${CONF_KEYCLOAK_HOST:-auth.${DOMAIN}}"
  fi

  echo ""
  echo -e "  ${BLD}── VAPID keys (Web Push) ──────────────${RST}"
  echo ""
  if [[ -n "${CONF_VAPID_PUBLIC:-}" ]]; then
    VAPID_PUBLIC_KEY="$CONF_VAPID_PUBLIC"
    VAPID_PRIVATE_KEY="$CONF_VAPID_PRIVATE"
    VAPID_SUBJECT="$CONF_VAPID_SUBJECT"
    ok "Using saved VAPID keys."
  else
    if confirm "Generate VAPID keys automatically? (requires npx)" y; then
      info "Generating VAPID keys …"
      VAPID_OUTPUT=$(npx --yes web-push generate-vapid-keys 2>/dev/null)
      VAPID_PUBLIC_KEY=$(echo "$VAPID_OUTPUT"  | grep "Public"  | awk '{print $NF}')
      VAPID_PRIVATE_KEY=$(echo "$VAPID_OUTPUT" | grep "Private" | awk '{print $NF}')
      ok "Keys generated."
      echo -e "    Public:  ${CYN}${VAPID_PUBLIC_KEY}${RST}"
    else
      ask VAPID_PUBLIC_KEY  "VAPID public key"  ""
      ask VAPID_PRIVATE_KEY "VAPID private key" ""
    fi
    ask VAPID_SUBJECT "VAPID subject (mailto:...)" "mailto:admin@${DOMAIN}"
  fi

  echo ""
  echo -e "  ${BLD}── Optional ────────────────────────────${RST}"
  echo ""
  ask RAWG_API_KEY "RAWG API key (game autofill, leave empty to skip)" "${CONF_RAWG_API_KEY:-}"

  echo ""
  echo -e "  ${BLD}── Passwords ───────────────────────────${RST}"
  echo ""
  ask SURREALDB_USER "SurrealDB username"      "${CONF_DB_USER:-surrealdb}"
  ask KC_ADMIN_USER  "Keycloak admin username" "${CONF_KC_ADMIN_USER:-admin}"
  ask POSTGRES_USER  "PostgreSQL username"     "${CONF_PG_USER:-keycloak}"
  echo ""

  # If all passwords are already saved, offer to keep them
  if [[ -n "${CONF_DB_PASS:-}" && -n "${CONF_KC_ADMIN_PASS:-}" && -n "${CONF_PG_PASS:-}" ]]; then
    if confirm "Keep saved passwords?" y; then
      SURREALDB_PASS="$CONF_DB_PASS"
      KC_ADMIN_PASS="$CONF_KC_ADMIN_PASS"
      POSTGRES_PASS="$CONF_PG_PASS"
      ok "Using saved passwords."
    else
      CONF_DB_PASS=""; CONF_KC_ADMIN_PASS=""; CONF_PG_PASS=""
    fi
  fi

  if [[ -z "${SURREALDB_PASS:-}" ]]; then
    if confirm "Generate all passwords automatically?" y; then
      SURREALDB_PASS="$(openssl rand -base64 32 | tr -d '=+/' | cut -c1-32)"
      KC_ADMIN_PASS="$(openssl rand -base64 32 | tr -d '=+/' | cut -c1-32)"
      POSTGRES_PASS="$(openssl rand -base64 32 | tr -d '=+/' | cut -c1-32)"
      echo ""
      warn "Generated passwords — save these somewhere safe:"
      echo -e "    SurrealDB:  ${YLW}${SURREALDB_PASS}${RST}"
      echo -e "    Keycloak:   ${YLW}${KC_ADMIN_PASS}${RST}"
      echo -e "    PostgreSQL: ${YLW}${POSTGRES_PASS}${RST}"
      echo ""
      read -rp "  Press Enter to continue …"
    else
      ask_secret SURREALDB_PASS "SurrealDB password"      ""
      ask_secret KC_ADMIN_PASS  "Keycloak admin password" ""
      ask_secret POSTGRES_PASS  "PostgreSQL password"     ""
    fi
  fi

  save_conf
fi

# Derived values — Keycloak URL gets /auth suffix in paths mode
VITE_API_URL="https://${API_HOST}"
FRONTEND_URL="https://${FRONTEND_HOST}"
if [[ "${ROUTING}" == "paths" ]]; then
  VITE_KEYCLOAK_URL="https://${KEYCLOAK_HOST}/auth"
  COMPOSE_SRC="${SCRIPT_DIR}/docker-compose.paths.yml"
else
  VITE_KEYCLOAK_URL="https://${KEYCLOAK_HOST}"
  COMPOSE_SRC="${SCRIPT_DIR}/docker-compose.yml"
fi

# ── Summary ────────────────────────────────────────────────────────────────────
echo ""
hr
echo ""
echo -e "  ${BLD}Deploy plan:${RST}"
echo -e "    Server:         ${CYN}${SERVER}${RST}"
echo -e "    Remote dir:     ${CYN}${REMOTE_DIR}${RST}"
echo -e "    Routing:        ${CYN}${ROUTING}${RST}"
echo -e "    Compose file:   ${CYN}$(basename "$COMPOSE_SRC")${RST}"
echo -e "    Frontend:       ${CYN}https://${FRONTEND_HOST}${RST}"
echo -e "    API:            ${CYN}${VITE_API_URL}${RST}"
echo -e "    Keycloak:       ${CYN}${VITE_KEYCLOAK_URL}${RST}"
echo -e "    Realm / client: ${CYN}Clync / clync${RST}"
echo ""

if ! confirm "Build and deploy now?" y; then
  info "Aborted."
  exit 0
fi

# ── Generate .env ──────────────────────────────────────────────────────────────
echo ""
hr
ENV_TMP="$(mktemp)"
cat > "$ENV_TMP" <<EOF
# Generated by deploy.sh — $(date -u +"%Y-%m-%d %H:%M UTC")

FRONTEND_HOST=${FRONTEND_HOST}
API_HOST=${API_HOST}
KEYCLOAK_HOST=${KEYCLOAK_HOST}

VITE_API_URL=${VITE_API_URL}
VITE_KEYCLOAK_URL=${VITE_KEYCLOAK_URL}
VITE_KEYCLOAK_REALM=Clync
VITE_KEYCLOAK_CLIENT_ID=clync


FRONTEND_URL=${FRONTEND_URL}

VAPID_PUBLIC_KEY=${VAPID_PUBLIC_KEY}
VAPID_PRIVATE_KEY=${VAPID_PRIVATE_KEY}
VAPID_SUBJECT=${VAPID_SUBJECT}

RAWG_API_KEY=${RAWG_API_KEY:-}

SURREALDB_USER=${SURREALDB_USER}
SURREALDB_PASS=${SURREALDB_PASS}

KC_ADMIN_USER=${KC_ADMIN_USER}
KC_ADMIN_PASS=${KC_ADMIN_PASS}

POSTGRES_USER=${POSTGRES_USER}
POSTGRES_PASS=${POSTGRES_PASS}
EOF
ok ".env generated."

# ── Upload .env and docker-compose.yml ────────────────────────────────────────
info "Creating remote directory ${REMOTE_DIR} …"
ssh "$SERVER" "mkdir -p ${REMOTE_DIR}"

info "Ensuring Docker proxy network exists …"
ssh "$SERVER" "docker network inspect proxy >/dev/null 2>&1 || docker network create proxy"
ok "Proxy network ready."

info "Uploading docker-compose.yml …"
scp "$COMPOSE_SRC" "${SERVER}:${REMOTE_DIR}/docker-compose.yml"
ok "docker-compose.yml uploaded."

info "Uploading .env …"
scp "$ENV_TMP" "${SERVER}:${REMOTE_DIR}/.env"
rm -f "$ENV_TMP"
ok ".env uploaded."

info "Uploading keycloak realm …"
ssh "$SERVER" "mkdir -p ${REMOTE_DIR}/keycloak"
scp "${SCRIPT_DIR}/keycloak/Clync-realm.json" "${SERVER}:${REMOTE_DIR}/keycloak/Clync-realm.json"
ok "Realm JSON uploaded."

# ── Build ──────────────────────────────────────────────────────────────────────
echo ""
info "Building backend …"
podman build -t clync-backend "$ROOT_DIR/backend"
ok "Backend image built."

echo ""
info "Building frontend …"
podman build \
  --build-arg "VITE_API_URL=${VITE_API_URL}" \
  --build-arg "VITE_KEYCLOAK_URL=${VITE_KEYCLOAK_URL}" \
  --build-arg "VITE_KEYCLOAK_REALM=Clync" \
  --build-arg "VITE_KEYCLOAK_CLIENT_ID=clync" \
  -t clync-frontend \
  "$ROOT_DIR/frontend"
ok "Frontend image built."

# ── Transfer images ────────────────────────────────────────────────────────────
echo ""
info "Streaming backend to ${SERVER} …"
podman save clync-backend | ssh "$SERVER" docker load
ssh "$SERVER" "docker tag localhost/clync-backend:latest clync-backend:latest 2>/dev/null || true"
ok "Backend transferred."

echo ""
info "Streaming frontend to ${SERVER} …"
podman save clync-frontend | ssh "$SERVER" docker load
ssh "$SERVER" "docker tag localhost/clync-frontend:latest clync-frontend:latest 2>/dev/null || true"
ok "Frontend transferred."

# ── Optional: docker compose up ───────────────────────────────────────────────
echo ""
hr
COMPOSE_CMD="docker compose -f ${REMOTE_DIR}/docker-compose.yml --env-file ${REMOTE_DIR}/.env up -d --no-build"
echo -e "  Will run: ${CYN}${COMPOSE_CMD}${RST}"
echo ""

if confirm "Start/restart services on server now?" y; then
  info "Bringing up services …"
  ssh "$SERVER" "$COMPOSE_CMD"
  ok "Services updated."
else
  warn "Skipped. Run manually on the server:"
  echo ""
  echo "  ${COMPOSE_CMD}"
fi

echo ""
hr
ok "Deploy complete."
echo ""
