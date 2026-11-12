# AGENTS.md — blazers landing page

Guidance for AI agents working on the landing page (`packages/landing`). See the
root [`AGENTS.md`](../../AGENTS.md) for repository-wide conventions.

## Overview

Next.js 16 (App Router) static-export site + Tailwind CSS v4, deployed to GitHub
Pages at `https://hieudoanm.github.io/blazers` (project site —
`basePath: "/blazers"`).

## Layout

- `src/app/page.tsx` — landing page (hero, features, PocketBase comparison,
  quickstart terminals, CTA, footer)
- `src/app/docs/[[...slug]]/page.tsx` — docs catch-all; renders every Markdown
  file under `docs/` as a static page at `/blazers/docs/<path>/`
- `src/app/globals.css` — `@import 'tailwindcss'` + official plugins only:
  `@tailwindcss/typography` for docs prose and DaisyUI with a custom `blazers`
  theme (see Styling below)
- `lib/docs.ts` — Markdown → HTML pipeline (`marked`), relative-link rewriting
  to `/blazers/docs/*`, GitHub-style heading ids
- `docs/` — documentation source (moved from the repo-root `docs/`)
- `public/` — static assets served under `/blazers/`
- `src/app/icon.svg`, `src/app/favicon.ico` — Next metadata file conventions

## Commands

```sh
cd packages/landing
pnpm install
pnpm run dev     # dev server (basePath applies: /blazers/)
pnpm run build   # static export → out/ (uploaded to Pages by CI)
```

## Conventions

- **Fonts:** Inter (sans) via `next/font` (`--font-inter`) or the Google Fonts
  link in `index.html`; mono stack
  `ui-monospace, 'SF Mono', SFMono-Regular, Menlo, Consolas, monospace`. Tokens:
  `--font-sans` / `--font-mono`. Do not add other typefaces.

- **Keep `output: "export"`** — the site must build to static HTML; no server
  features, no API routes, no `next/image` optimization.
- **`basePath`/`assetPrefix` are `/blazers`.** `next/link` and metadata get it
  automatically; raw `<a href>` and generated HTML links must include it (see
  `BASE_PATH` in `lib/docs.ts`).
- **DaisyUI + Tailwind v4** — `globals.css` has only `@import 'tailwindcss'`
  plus official plugins; the `blazers` theme (set via `data-theme` on `<html>`)
  maps `primary` to flame orange (`#c2410c`). Use DaisyUI components (`btn`,
  `badge`, `card`) and theme tokens (`bg-base-100`, `text-base-content`,
  `text-primary`, `bg-primary/10`) — no custom CSS.
- **Docs are Markdown**, rendered at build time — no MDX, no client JS. Links
  between docs use relative `.md` paths; `lib/docs.ts` rewrites them. Links
  escaping the docs tree (e.g. `../../README.md`) are pointed at the GitHub repo
  blob.
- **Pin dependency versions** in `package.json`; CI runs
  `pnpm install --frozen-lockfile` against `pnpm-lock.yaml` — commit lockfile
  changes.
- **Deploy:** CI builds `out/` and uploads via `upload-pages-artifact` +
  `deploy-pages` (see `.github/workflows/ci.yaml`, `pages` job).

## Footer

The footer follows the shared contract via `src/components/Footer.tsx`:
`© {year} {repo-name}` (lowercase repo name, `new Date().getFullYear()`), a
one-line tagline/license descriptor, and a right-aligned
`<nav aria-label="Footer">` with exactly two links — `GitHub` (repo root) and
`README` (`<repo>#readme`), both `target="_blank" rel="noreferrer"`. Reuse the
component on every page that shows a footer.

## Features & CTA sections

The landing page composes `Features` and `CTA` components (same directory as
`Footer`). Features: centered head (h2 + supporting p), responsive card grid,
card = icon chip + h3 + p. CTA: alt/surface background, centered h2 + p, primary
action button plus at most one ghost button. Keep copy and link targets
per-repo; keep the structure shared.

## Styling

Tailwind CSS only. `globals.css` contains just `@import 'tailwindcss';` plus
official plugins where required (`@tailwindcss/typography` for docs prose,
DaisyUI in fdr). No `@theme`, `@layer`, `@utility`, or custom selectors — style
with utilities in `className` — no shared class-string modules. Inter comes from
`next/font` `className` on `<html>`; smooth scroll uses
`scroll-smooth motion-reduce:scroll-auto`. Markdown-rendered HTML gets utility
classes emitted by `lib/docs.ts`, never custom CSS. Inline `style={{}}` is only
for genuinely dynamic values.
