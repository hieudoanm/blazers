# AGENTS.md — @blazers/sdk

Guidance for AI agents working in `packages/sdk`. See the root
[`AGENTS.md`](../../AGENTS.md) for repository-wide conventions.

## What this is

`@blazers/sdk` — dependency-free TypeScript client SDK for the Blazers BaaS
(auth incl. OAuth2/OTP/MFA/refresh, collections/records with filter/sort/expand,
realtime WebSocket + presence, file buckets, batch). Published shape: ESM
`dist/` + `.d.ts` built by `tsc`.

## Layout

| Path                | What it is                                                                                 |
| ------------------- | ------------------------------------------------------------------------------------------ |
| `src/index.ts`      | `Blazers` entry point composing the sub-clients + type re-exports                          |
| `src/http.ts`       | `HttpClient` — base URL, Bearer token, JSON/FormData, `BlazersError`, `batch()`            |
| `src/auth.ts`       | `AuthClient` — `/api/auth/*` incl. MFA challenge + TOTP                                    |
| `src/records.ts`    | `CollectionClient` (per-collection CRUD) + `CollectionsClient` (admin/rules)               |
| `src/files.ts`      | `FilesClient` — buckets, multipart upload, download URLs/tokens                            |
| `src/realtime.ts`   | `RealtimeClient` — `/ws` subscribe (+`filter`), presence, auto-reconnect                   |
| `src/types.ts`      | Wire types mirroring the Rust models — keep in sync with `crates/blazers-core/src/models/` |
| `tests/sdk.test.ts` | vitest — unit tests always; live-server test when `BLAZERS_URL` is set                     |

## Commands

```bash
npm run build        # tsc → dist/
npm run typecheck    # tsc --noEmit
npm run lint         # eslint (flat config, typescript-eslint)
npm run lint:fix
npm run format       # prettier --write
npm run format:check
npm test             # vitest run
```

Consumed by `packages/app` via `"@blazers/sdk": "workspace:*"` — **run
`npm run build` here after changing `src/` before the app picks it up** (the app
imports `dist/`, not `src/`).

## Conventions

- Zero runtime dependencies — `fetch` + native `WebSocket` only. Node ≥ 18 for
  HTTP; realtime needs `WebSocket` (Node ≥ 22 or browser).
- Match the server wire format exactly: snake_case fields (`per_page`,
  `is_public`, `current_password`, `otpauth_url`, …). When the Rust API changes,
  update `src/types.ts`/`auth.ts`/etc. and the landing docs together.
- Register returns `User` (not a session) — callers must `login()` after.
- `/ws` connections are anonymous server-side: `@request.auth.*` inside
  subscription filters resolves to `null` — document, don't work around.
- Prettier: single quotes, semi, width 80 (same `.prettierrc` as landing/app).
- Keep files ≲200 lines; split by domain like the rest of the repo.
