# blazers

TypeScript client SDK for the [Blazers](../../README.md) BaaS — a single-binary
backend (auth, schemaless collections, files, realtime, webhooks, cron) inspired
by PocketBase and Supabase.

Zero runtime dependencies — uses `fetch` and the native `WebSocket` (Node ≥ 22
or any modern browser; Node 18/20 need a `WebSocket` polyfill for realtime).

## Install

```bash
npm install @blazers/sdk   # or: npm install ./packages/sdk
```

## Quick start

```ts
import { Blazers } from '@blazers/sdk';

const bz = new Blazers({ url: 'http://localhost:8090' });

// Auth — token is stored automatically and sent as Bearer on every request
await bz.auth.register({ email: 'a@b.c', password: 'secret' });
// or: await bz.auth.login({ email, password });
// MFA accounts return { mfa_required: true } — finish with bz.auth.mfaLogin(...)

// Records — schemaless documents with PB-style filter/sort/expand
const posts = bz.collection('posts');
await posts.create({ title: 'Hello', status: 'live' });
const page = await posts.list({
  filter: 'status="live" && title~"ello"',
  sort: '-created_at',
  expand: 'author',
  perPage: 20,
});

// Realtime — record events with optional server-side filters + presence
await bz.realtime.subscribe('posts', {
  filter: 'status="live"',
  onRecord: (e) => console.log(e.action, e.record, e.oldRecord),
});
await bz.realtime.presenceJoin('room:lobby', {
  state: { name: 'Ada' },
  onEvent: (e) => console.log(e.action, e.client, e.state),
});

// Files — public buckets need no auth; protected ones use ~1h tokens
await bz.files.createBucket('avatars', true);
const form = new FormData();
form.append('file', fileBlob, 'avatar.png');
const f = await bz.files.upload('avatars', fileBlob, 'avatar.png');
const url = bz.files.downloadUrl('avatars', f.id); // public → direct URL

// Batch — up to 50 sub-requests in one round-trip
const results = await bz.batch([
  { method: 'GET', url: '/api/collections/posts/records' },
  {
    method: 'POST',
    url: '/api/collections/posts/records',
    body: { data: { t: 1 } },
  },
]);
```

## API surface

| Client                   | Covers                                                                                                                                                                                                                                                            |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `bz.auth`                | register, login (incl. MFA challenge), `mfaLogin`, refresh, logout, `me`/`updateMe`, forgot/reset password, email verification, OTP (`requestOtp`/`authWithOtp`), `oauthUrl(provider)` (Google/GitHub/GitLab/Discord), TOTP `mfaEnable`/`mfaConfirm`/`mfaDisable` |
| `bz.collection(name)`    | `list`/`listAll`/`get`/`create`/`update`/`delete` with `filter`, `sort`, `expand`, `search`, pagination                                                                                                                                                           |
| `bz.collections`         | admin CRUD for collections + the five rule expressions                                                                                                                                                                                                            |
| `bz.files`               | bucket CRUD, multipart `upload`, `download`/`downloadUrl`, `fileToken` (optional `{collection, record}` binding to `view_rule`)                                                                                                                                   |
| `bz.realtime`            | `subscribe`/`unsubscribe` (per-subscription `filter`), `presenceJoin`/`presenceUpdate`/`presenceLeave`, raw `onMessage`, auto-reconnect                                                                                                                           |
| `bz.batch` / `bz.health` | `POST /api/batch`, `GET /api/health`                                                                                                                                                                                                                              |

## Notes

- **Token storage** is in-memory on the `HttpClient`; persist `res.token`
  yourself (e.g. `localStorage`) and pass `token` in `BlazersOptions`, or call
  `bz.auth.refresh()` to rotate before expiry (72h access tokens).
- **Realtime auth**: `/ws` connections are anonymous server-side, so
  `@request.auth.*` inside subscription filters resolves to `null`.
- **`?batch=1`**: the REST API also accepts batch-wrapped bodies on any `/api/*`
  route; the SDK exposes the dedicated `POST /api/batch` endpoint.
- Errors throw `BlazersError` with `.status` and `.body`.

## Development

```bash
npm install
npm run build        # tsc → dist/
npm run lint         # eslint (flat config, typescript-eslint)
npm run format       # prettier --write
npm test             # vitest — unit tests always run;
                     # live-server tests run when BLAZERS_URL is set
```
