import Link from 'next/link';

const FEATURES = [
  {
    title: 'Auth',
    body: 'Email/password JWT, OTP, OAuth2, TOTP MFA, refresh tokens, managed users collection.',
  },
  {
    title: 'Records',
    body: 'Schemaless collections with PocketBase-style rules, filter/sort/expand and batch requests.',
  },
  {
    title: 'Realtime',
    body: 'WebSocket record events with oldRecord diffs, per-subscription filters and presence.',
  },
  {
    title: 'Files',
    body: 'Buckets (public/protected), multipart upload, thumbnails and record-bound file tokens.',
  },
];

export default function Home() {
  return (
    <main className="mx-auto max-w-4xl px-6 py-16">
      <div className="mockup-code mb-8">
        <pre data-prefix="$">
          <code>blazers serve --port 8090</code>
        </pre>
      </div>

      <h1 className="text-4xl font-bold">Blazers demo</h1>
      <p className="text-base-content/70 mt-3 max-w-2xl">
        A live playground for the Blazers BaaS — a single-binary backend with
        auth, a schemaless database, file storage and realtime, inspired by
        PocketBase and Supabase.
      </p>

      <div className="mt-8 flex gap-3">
        <Link href="/login" className="btn btn-primary">
          Open demo
        </Link>
        <a
          href="http://localhost:8090/api/docs"
          target="_blank"
          rel="noreferrer"
          className="btn btn-outline">
          API docs ↗
        </a>
      </div>

      <div className="mt-12 grid gap-4 sm:grid-cols-2">
        {FEATURES.map((f) => (
          <div
            key={f.title}
            className="card border-base-300 bg-base-100 border">
            <div className="card-body">
              <h2 className="card-title text-lg">{f.title}</h2>
              <p className="text-base-content/70 text-sm">{f.body}</p>
            </div>
          </div>
        ))}
      </div>

      <p className="text-base-content/50 mt-10 text-sm">
        Server URL: <code>NEXT_PUBLIC_BLAZERS_URL</code> (default{' '}
        <code>http://localhost:8090</code>)
      </p>
    </main>
  );
}
