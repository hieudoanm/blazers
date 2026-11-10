# blazers landing page

The landing page and documentation site for blazers — a Next.js (App Router)
static-export site with Tailwind CSS v4, deployed to GitHub Pages at
<https://hieudoanm.github.io/blazers>.

## Structure

- `src/app/` — Next.js pages: `page.tsx` (landing), `docs/[[...slug]]/page.tsx`
  (docs site), `not-found.tsx`, `icon.svg`/`favicon.ico` conventions
- `src/app/globals.css` — Tailwind v4 `@theme` tokens (flame-orange accent,
  light theme, dark terminal blocks) + docs markdown styles
- `docs/` — the documentation source (Markdown), rendered at build time to
  `/docs/*` routes; `lib/docs.ts` handles loading, link rewriting and
  heading anchors
- `public/` — `logo.svg`, `favicon.svg`, `favicon.ico`, `robots.txt`,
  `sitemap.xml`, `humans.txt`

## Develop

```bash
cd packages/landing
pnpm install
pnpm run dev      # http://localhost:3000/blazers/
pnpm run build    # static export → out/
```

`basePath`/`assetPrefix` are `/blazers` in `next.config.ts` (GitHub Pages
project site). `lib/docs.ts` `BASE_PATH` must match.

## Deploy

CI runs `pnpm install --frozen-lockfile && pnpm run build` and uploads `packages/landing/out` to
GitHub Pages (see `.github/workflows/ci.yaml`, `pages` job).
