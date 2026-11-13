#!/usr/bin/env bash
# One-shot demo setup: installs the workspace, builds the SDK + binary,
# starts `blazers serve`, seeds a demo account, then runs the Next.js app.
#
#   ./scripts/setup.sh          # full setup + dev server on :3001
#   BLAZERS_DATA=/tmp/x PORT=9090 ./scripts/setup.sh
#
# Ctrl+C stops the app and the Blazers server.
set -euo pipefail

APP_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROOT_DIR="$(cd "$APP_DIR/../.." && pwd)"
cd "$APP_DIR"

DATA_DIR="${BLAZERS_DATA:-$APP_DIR/.blazers-data}"
PORT="${PORT:-8090}"
API="http://localhost:${PORT}"
DEMO_EMAIL="${DEMO_EMAIL:-demo@blazers.dev}"
DEMO_PASSWORD="${DEMO_PASSWORD:-demo1234}"
BINARY="$ROOT_DIR/target/debug/blazers"
SERVER_PID=""

cleanup() {
  [ -n "$SERVER_PID" ] && kill "$SERVER_PID" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

say() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }

# --- 1. JS workspace deps + SDK build -------------------------------------
say "Installing workspace dependencies (pnpm)"
(cd "$ROOT_DIR" && pnpm install)

say "Building @blazers/sdk"
pnpm --filter @blazers/sdk run build

# --- 2. Blazers binary ------------------------------------------------------
if [ ! -x "$BINARY" ]; then
  say "Building blazers binary (cargo build -p blazers)"
  (cd "$ROOT_DIR" && cargo build -p blazers)
fi

# --- 3. Start server ---------------------------------------------------------
mkdir -p "$DATA_DIR"
say "Starting blazers serve on :$PORT (data: $DATA_DIR)"
BLAZERS_DATA="$DATA_DIR" PORT="$PORT" JWT_SECRET="${JWT_SECRET:-dev}" \
  "$BINARY" serve &
SERVER_PID=$!

say "Waiting for $API/api/health"
for _ in $(seq 1 50); do
  curl -sf "$API/api/health" >/dev/null 2>&1 && break
  sleep 0.2
done
curl -sf "$API/api/health" >/dev/null || {
  echo "server failed to start" >&2
  exit 1
}

# --- 4. Seed demo account (idempotent) ---------------------------------------
say "Seeding demo account ($DEMO_EMAIL / $DEMO_PASSWORD)"

json_get() { # json_get <json> <key>
  printf '%s' "$1" | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{const o=JSON.parse(s);const k=process.argv[1];console.log(k.split(".").reduce((a,c)=>a?.[c],o)??"")})' "$2"
}

LOGIN=$(curl -sf -X POST "$API/api/auth/login" \
  -H 'Content-Type: application/json' \
  -d "{\"email\":\"$DEMO_EMAIL\",\"password\":\"$DEMO_PASSWORD\"}" || true)

if [ -z "$LOGIN" ]; then
  curl -sf -X POST "$API/api/auth/register" \
    -H 'Content-Type: application/json' \
    -d "{\"email\":\"$DEMO_EMAIL\",\"password\":\"$DEMO_PASSWORD\"}" >/dev/null
  LOGIN=$(curl -sf -X POST "$API/api/auth/login" \
    -H 'Content-Type: application/json' \
    -d "{\"email\":\"$DEMO_EMAIL\",\"password\":\"$DEMO_PASSWORD\"}")
fi

TOKEN=$(json_get "$LOGIN" token)
USER_ID=$(json_get "$LOGIN" user.id)
AUTH="Authorization: Bearer $TOKEN"
JSON_CT='Content-Type: application/json'

# Grant global admin (no bootstrap superuser exists).
curl -sf -X POST "$API/api/permissions" -H "$AUTH" -H "$JSON_CT" \
  -d "{\"user_id\":\"$USER_ID\",\"collection\":\"*\",\"role\":\"admin\"}" >/dev/null || true

# Sample collection + records.
COLS=$(curl -sf "$API/api/collections" -H "$AUTH")
if ! printf '%s' "$COLS" | grep -q '"posts"'; then
  curl -sf -X POST "$API/api/collections" -H "$AUTH" -H "$JSON_CT" \
    -d '{"name":"posts","schema":"{}"}' >/dev/null
fi

RECORDS=$(curl -sf "$API/api/collections/posts/records?per_page=1" -H "$AUTH")
if printf '%s' "$RECORDS" | grep -q '"total":0'; then
  curl -sf -X POST "$API/api/collections/posts/records" -H "$AUTH" -H "$JSON_CT" \
    -d '{"data":{"title":"Hello from the demo","status":"live","views":42}}' >/dev/null
fi

# --- 5. Dev server ------------------------------------------------------------
say "Demo ready"
echo "  App:     http://localhost:3001"
echo "  API:     $API  (docs: $API/api/docs)"
echo "  Login:   $DEMO_EMAIL / $DEMO_PASSWORD"
echo "  Data:    $DATA_DIR"
echo
exec pnpm dev
