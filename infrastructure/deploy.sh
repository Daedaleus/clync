#!/usr/bin/env bash
# Build images locally with Podman and stream them directly to the server.
# No registry needed — images are piped over SSH.
#
# Usage: ./deploy.sh user@your-server.com
#
# Prerequisites:
#   - Podman installed locally
#   - SSH access to the server
#   - .env-paths file filled out (copy from .env-paths.example)

set -euo pipefail

SERVER="${1:?Usage: $0 user@server}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
ENV_FILE="${SCRIPT_DIR}/.env-paths"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Error: $ENV_FILE not found — copy .env-paths.example and fill it in."
  exit 1
fi

# Load env vars (skip comments and blank lines)
set -a
# shellcheck disable=SC1090
source <(grep -v '^\s*#' "$ENV_FILE" | grep -v '^\s*$')
set +a

: "${VITE_API_URL:?}"
: "${VITE_KEYCLOAK_URL:?}"
: "${VITE_KEYCLOAK_REALM:=WhatsUp}"
: "${VITE_KEYCLOAK_CLIENT_ID:=whatsup}"

echo "==> Building backend …"
podman build -t whatsup-backend "$ROOT_DIR/backend"

echo "==> Building frontend …"
podman build \
  --build-arg "VITE_API_URL=${VITE_API_URL}" \
  --build-arg "VITE_KEYCLOAK_URL=${VITE_KEYCLOAK_URL}" \
  --build-arg "VITE_KEYCLOAK_REALM=${VITE_KEYCLOAK_REALM}" \
  --build-arg "VITE_KEYCLOAK_CLIENT_ID=${VITE_KEYCLOAK_CLIENT_ID}" \
  -t whatsup-frontend \
  "$ROOT_DIR/frontend"

echo "==> Streaming backend to ${SERVER} …"
podman save whatsup-backend | ssh "$SERVER" docker load
# Ensure the image is reachable under the short name used in docker-compose.paths.yml
ssh "$SERVER" "docker tag localhost/whatsup-backend:latest whatsup-backend:latest 2>/dev/null || true"

echo "==> Streaming frontend to ${SERVER} …"
podman save whatsup-frontend | ssh "$SERVER" docker load
ssh "$SERVER" "docker tag localhost/whatsup-frontend:latest whatsup-frontend:latest 2>/dev/null || true"

echo ""
echo "Done. On the server run:"
echo ""
echo "  # Pull latest compose/config changes first if infrastructure/ changed:"
echo "  git -C ~/whatsup pull"
echo ""
echo "  cd ~/whatsup/infrastructure"
echo "  docker compose -f docker-compose.paths.yml --env-file .env-paths up -d --no-build"
