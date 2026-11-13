# Edge functions

Blazers embeds a JavaScript runtime ([QuickJS](https://bellard.org/quickjs/))
directly in the server binary — no Node, Deno, or sidecar required. Functions
are stored in the database and executed synchronously per invocation.

## How it works

1. An admin stores a function: a name plus JavaScript source.
2. Anyone can invoke it with `POST /api/functions/{name}` and an arbitrary JSON
   `input`.
3. The runtime evaluates the source, calls `handler(ctx)` (or `main(ctx)` as a
   fallback), and returns the handler's return value as `{ "result": ... }`.

The handler receives a single context object:

```json
{
  "input": { "...": "whatever the caller sent" },
  "auth": { "user_id": "...", "email": "..." } // or null
}
```

Send a `Bearer` token with the invoke request to populate `auth`; without one,
`auth` is `null`.

```js
function handler(ctx) {
  if (!ctx.auth) return { error: 'sign in required' };
  return { hello: ctx.input.name ?? 'world', by: ctx.auth.email };
}
```

## Sandbox

Each invocation gets a fresh QuickJS runtime and context:

- **No I/O** — no `fetch`, filesystem, sockets, or process access.
- **No timers** — `setTimeout`/`setInterval` are absent; execution is
  synchronous and must terminate.
- **Memory & stack caps** — 64 MiB heap, 8 MiB stack per invocation.
- **JSON only** — the return value must be JSON-serializable; functions,
  `undefined`, and cyclic objects are not supported.

JS exceptions and infinite loops surface as `400` errors (the runtime aborts the
evaluation). Treat functions as pure compute — validation, transforms, pricing
rules, feature flags — not as a place for side effects.

## Managing functions

CRUD is admin-only (`_permissions` `admin` role on `*`):

| Method   | Path                 | Purpose               |
| -------- | -------------------- | --------------------- |
| `GET`    | `/api/functions`     | list stored functions |
| `POST`   | `/api/functions`     | create or replace     |
| `DELETE` | `/api/functions/{n}` | delete                |
| `POST`   | `/api/functions/{n}` | invoke (any caller)   |

See the [API reference](../api-reference/functions.md) for request/response
shapes.
