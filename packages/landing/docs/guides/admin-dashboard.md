# Admin dashboard

Blazers ships a built-in admin UI — no separate frontend to deploy. It is
embedded into the binary (`rust-embed`) and served at `/`.

## Stack

- **HTMX** (`htmx.org`) + **htmx-client-side-templates** for partial swaps
- **Mustache** templates (`<script type="x-tmpl-mustache">`) rendered
  client-side against JSON API responses
- **Pure CSS** (`public/styles.css`) — no Tailwind, no build step
- Vanilla JS for auth token storage, pagination, and modal state

## What's inside

| View          | Covers                                             |
| ------------- | -------------------------------------------------- |
| Dashboard     | counts, recent collections/buckets                 |
| Collections   | schema editor + records browser (search, paginate) |
| Buckets       | file list, upload, download, delete                |
| Webhooks      | CRUD + delivery logs viewer                        |
| Secrets       | vault list, reveal-on-click                        |
| WebSockets    | live connections + message inspector               |
| Cache         | entries, TTLs, flush                               |
| Notifications | list, mark read, live badge via SSE                |
| Logs          | structured log browser, live tail                  |
| Pub/Sub       | topics, message history, publish                   |

## Live updates

The dashboard holds three SSE connections:

- `/api/notifications/stream` — bumps the sidebar badge and reloads the
  notifications view
- `/api/logs/stream` — appends to the logs view when open
- `/api/pubsub/.../stream` — pub/sub message feed

## Template rules

Templates are Mustache, rendered by htmx's client-side-templates
extension. Two pitfalls we've hit:

1. **Never nest `<script>` inside `<script type="x-tmpl-mustache">`** —
   the HTML parser truncates the outer script at the first inner
   `</script>` and the rest of the template leaks as literal `{{...}}`
   text. Post-process rendered DOM in `htmx:afterSwap` instead.
2. Sections like `{{#files}}...{{/files}}` iterate arrays; `{{^files}}`
   renders the empty state. Context must match the API response shape
   (`FilesPage` → `{files: [...]}`, `WebhookLog[]` → `{{#.}}`).

## Customizing

Edit `crates/blazers-server/public/index.html` and `styles.css`, then
rebuild — `rust-embed` bakes the assets into the binary at compile time.
Touch a `.rs` file (or `cargo clean -p blazers-server`) to force the
re-embed.
