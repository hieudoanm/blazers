# Blazers Demo App

Live demo of the [Blazers](../../README.md) BaaS — Next.js + Tailwind CSS +
DaisyUI, powered by the [`@blazers/sdk`](../sdk) TypeScript SDK (`workspace:*`
dependency).

## Features

- **Auth** — register / sign in, email OTP, MFA challenge handling; session
  token persisted in `localStorage`.
- **Records** — pick a collection, list records with `filter`/`sort`, create
  from JSON, delete.
- **Collections** — create/edit collections incl. the five rule expressions.
- **Realtime** — WebSocket subscribe with per-subscription filter, `*` wildcard,
  presence channel join, live event log.
- **Files** — bucket CRUD (public flag), multipart upload, download links.

## Run

```bash
# 1. SDK must be built once
cd packages/sdk && npm install && npm run build && cd ..

# 2. Blazers server (any data dir)
BLAZERS_DATA=/tmp/blazers-demo PORT=8090 JWT_SECRET=dev ./target/debug/blazers serve

# 3. Demo app
cd packages/app && npm install && npm run dev   # http://localhost:3001
```

Configure the server URL with `NEXT_PUBLIC_BLAZERS_URL` (default
`http://localhost:8090`).

> **Note:** there is no bootstrap superuser — the first account you register has
> no permissions. Grant yourself admin once via curl:

```bash
TOKEN=$(curl -s -X POST localhost:8090/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"you@x.dev","password":"pw"}' | jq -r .token)
UID=$(curl -s localhost:8090/api/auth/me -H "Authorization: Bearer $TOKEN" | jq -r .id)
curl -X POST localhost:8090/api/permissions -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d "{\"user_id\":\"$UID\",\"collection\":\"*\",\"role\":\"admin\"}"
```

## Scripts

```bash
npm run dev          # next dev -p 3001
npm run build        # production build
npm run lint         # eslint (eslint-config-next)
npm run format       # prettier --write
npm run typecheck    # tsc --noEmit
npm test             # vitest
```
