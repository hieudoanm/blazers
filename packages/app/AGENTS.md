# AGENTS.md — @blazers/app

Guidance for AI agents working in `packages/app`. See the root
[`AGENTS.md`](../../AGENTS.md) and `packages/landing/AGENTS.md` for shared
conventions.

## What this is

`@blazers/app` — live demo SPA for the Blazers BaaS: Next.js 16 (App Router)

- Tailwind CSS v4 + DaisyUI, powered by `@blazers/sdk` (`workspace:*`).
  Client-side only — all state lives in the browser (token in `localStorage`).

## Layout

| Path                         | What it is                                                                |
| ---------------------------- | ------------------------------------------------------------------------- |
| `src/app/page.tsx`           | Landing: feature cards + quickstart                                       |
| `src/app/login/page.tsx`     | Register/login, email OTP, MFA challenge                                  |
| `src/app/dashboard/page.tsx` | Tab shell — records / collections / realtime / files                      |
| `src/app/globals.css`        | Tailwind + DaisyUI `blazers` theme (same palette as landing)              |
| `src/lib/blazers.ts`         | SDK singleton; token persisted under `blazers_demo_token`                 |
| `src/lib/format.ts`          | Pure display helpers (`shortId`, `timeAgo`, `tryParseJson`) — unit-tested |
| `src/components/dashboard/`  | `RecordsTab`, `CollectionsTab`, `RealtimeTab`, `FilesTab`                 |

## Commands

```bash
npm run dev          # next dev -p 3001 (predev kills the port)
npm run build        # production build (prebuild rimrafs .next)
npm run lint         # eslint (eslint-config-next core-web-vitals + TS)
npm run format       # prettier --write
npm run typecheck    # tsc --noEmit
npm test             # vitest run (src/**/*.test.ts)
```

Requires a running server: `blazers serve` on `:8090` (override with
`NEXT_PUBLIC_BLAZERS_URL`). There is no bootstrap superuser — grant admin via
`POST /api/permissions` (see README for the curl).

## Conventions

- **DaisyUI + Tailwind only** — same rules as landing: utilities in `className`,
  `data-theme="blazers"`, no custom CSS, no new fonts.
- All data access goes through `getBlazers()` from `src/lib/blazers.ts` — never
  instantiate `Blazers` elsewhere or fetch `/api/*` directly.
- Components are `'use client'` — the whole app is a client SPA; keep server
  components out unless a page genuinely needs SSR.
- `react-hooks/set-state-in-effect` is disabled in `eslint.config.mjs`
  (fetch-on-mount is intentional here) — don't re-enable without refactoring.
- Realtime: subscribe/unsubscribe through `bz.realtime`; presence channels are
  demo-only (`demo:room` default). Clean up subscriptions on unmount.
- Pinned dependency versions (same as landing); pnpm workspace at repo root
  manages installs — use `pnpm install`, not `npm install`.

<!-- BEGIN:nextjs-agent-rules -->

## This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` (resolved from this file's directory; in monorepos the `next` package may not be visible from the repo root) before writing any code. Heed deprecation notices.

This block is written and re-added by `next dev` — verify at `node_modules/next/dist/server/lib/generate-agent-files.js`. Removing it from a diff only re-creates the uncommitted change; committing it with your work keeps the tree clean.

<!-- END:nextjs-agent-rules -->
