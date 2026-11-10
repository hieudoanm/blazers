import type { Metadata, NextPage } from 'next';
import Image from 'next/image';
import Link from 'next/link';
import type { FC } from 'react';
import type { IconType } from 'react-icons';
import {
  LuBookOpen,
  LuCpu,
  LuDatabase,
  LuFolderUp,
  LuKeyRound,
  LuLayoutDashboard,
  LuLock,
  LuWebhook,
  LuZap,
} from 'react-icons/lu';

export const metadata: Metadata = {
  alternates: { canonical: '/' },
};

const GITHUB = 'https://github.com/hieudoanm/blazers';
// next/image with unoptimized does not apply basePath — mirror next.config.ts.
const LOGO = process.env.NODE_ENV === 'production' ? '/blazers/logo.svg' : '/logo.svg';

const FEATURES: { title: string; body: string; icon: IconType }[] = [
  {
    title: 'Auth & permissions',
    body: 'Register, login, JWT bearer tokens, bcrypt passwords and a per-collection permission system — public, user and admin rules enforced on every request.',
    icon: LuLock,
  },
  {
    title: 'Collections & records',
    body: 'Schema-less collections stored as dynamic SQLite tables. Full CRUD with paging, search, sorting and JSON-schema validation.',
    icon: LuDatabase,
  },
  {
    title: 'File buckets',
    body: 'Public and private buckets, streaming uploads, downloads and on-the-fly image thumbnails — files live next to the database.',
    icon: LuFolderUp,
  },
  {
    title: 'Realtime',
    body: 'WebSocket hub plus Server-Sent Events streams for notifications, logs and pub/sub topics — subscribe from any browser or client.',
    icon: LuZap,
  },
  {
    title: 'Webhooks & cron',
    body: 'HMAC-signed webhook delivery with logs, and scheduled HTTP jobs managed entirely through the API — no code required.',
    icon: LuWebhook,
  },
  {
    title: 'Secrets vault & cache',
    body: 'AES-256-GCM encrypted key/value store for credentials, plus an in-memory cache API persisted to SQLite with TTL eviction.',
    icon: LuKeyRound,
  },
  {
    title: 'Admin dashboard',
    body: 'An HTMX admin UI embedded in the binary at / — manage collections, records, files, webhooks, secrets and cron jobs from the browser. No build step.',
    icon: LuLayoutDashboard,
  },
  {
    title: 'MCP server',
    body: 'blazers mcp serve exposes collections and records as Model Context Protocol tools — let an AI agent read and write your backend over stdio.',
    icon: LuCpu,
  },
  {
    title: 'OpenAPI & SDK-ready',
    body: 'Swagger UI at /api/docs, a runtime-generated OpenAPI spec, JSON import/export snapshots and a backup endpoint.',
    icon: LuBookOpen,
  },
];

interface CompareRow {
  feature: string;
  pb: string;
  bz: string;
  sb: string;
}

const COMPARE: CompareRow[] = [
  {
    feature: 'Deployment',
    pb: '1 Go binary + pb_data/',
    bz: '1 Rust binary + ~/.blazers/',
    sb: 'Postgres + GoTrue + PostgREST + Realtime + Storage',
  },
  {
    feature: 'Database',
    pb: 'SQLite',
    bz: 'SQLite',
    sb: 'Postgres',
  },
  {
    feature: 'Auth',
    pb: 'Email + OAuth2, request rules',
    bz: 'Email + JWT (bcrypt), RBAC roles',
    sb: 'GoTrue — email, OAuth2, MFA',
  },
  {
    feature: 'File storage',
    pb: 'Buckets',
    bz: 'Buckets + thumbnails',
    sb: 'S3-compatible storage',
  },
  {
    feature: 'Realtime',
    pb: 'SSE subscriptions',
    bz: 'WebSocket + SSE + pub/sub',
    sb: 'Realtime (Postgres changes)',
  },
  {
    feature: 'Extensibility',
    pb: 'JS hooks (pb_hooks)',
    bz: 'HMAC webhooks + cron → HTTP',
    sb: 'Edge Functions (Deno)',
  },
  {
    feature: 'Scheduled jobs',
    pb: '—',
    bz: 'Cron expressions, built-in',
    sb: 'Scheduled Edge Functions',
  },
  {
    feature: 'Admin UI',
    pb: 'Bundled SPA',
    bz: 'Embedded HTMX at /',
    sb: 'Supabase Studio',
  },
  {
    feature: 'API spec',
    pb: '—',
    bz: 'OpenAPI + Swagger UI',
    sb: 'OpenAPI (PostgREST)',
  },
  {
    feature: 'AI / agent access',
    pb: '—',
    bz: 'MCP server (stdio)',
    sb: '—',
  },
  {
    feature: 'Ops extras',
    pb: 'Backups, logs',
    bz: 'Cache, secrets vault, notifications, logs',
    sb: 'Managed add-ons (Auth, Realtime, …)',
  },
];

type TokKind = 'prompt' | 'cmd' | 'flag' | 'str' | 'key' | 'url' | 'dim' | 'ok' | 'err';
interface Tok {
  t: string;
  c?: TokKind;
}

const TK: Record<TokKind, string> = {
  prompt: 'prompt',
  cmd: 'tk-cmd',
  flag: 'tk-flag',
  str: 'tk-str',
  key: 'tk-key',
  url: 'tk-url',
  dim: 'tk-dim',
  ok: 'tk-ok',
  err: 'tk-err',
};

interface TermProps {
  caption: string;
  lines: Tok[][];
}

const Term: FC<TermProps> = ({ caption, lines }) => (
  <figure className='terminal'>
    <figcaption>{caption}</figcaption>
    <pre>
      {lines.map((line, i) => (
        <span key={i} className='tk-line'>
          {line.map((tok, j) => (
            <span key={j} className={TK[tok.c ?? 'ok']}>
              {tok.t}
            </span>
          ))}
        </span>
      ))}
    </pre>
  </figure>
);

const Home: NextPage = () => (
  <>
    <header className='border-border bg-bg/90 sticky top-0 z-50 border-b backdrop-blur'>
      <div className='mx-auto flex h-16 w-full max-w-[72rem] items-center gap-6 px-5'>
        <Link
          href='#top'
          className='text-text flex items-center gap-2 text-lg font-bold no-underline'>
          <Image src={LOGO} alt='' width={28} height={28} className='rounded-md' />
          <span>blazers</span>
        </Link>
        <nav className='ml-auto hidden items-center gap-5 lg:flex'>
          <Link
            href='#features'
            className='text-text-muted hover:text-text text-[0.9375rem] no-underline'>
            Features
          </Link>
          <Link
            href='#compare'
            className='text-text-muted hover:text-text text-[0.9375rem] no-underline'>
            vs PocketBase
          </Link>
          <Link
            href='#quickstart'
            className='text-text-muted hover:text-text text-[0.9375rem] no-underline'>
            Quickstart
          </Link>
          <Link
            href='/docs/'
            className='text-text-muted hover:text-text text-[0.9375rem] no-underline'>
            Docs
          </Link>
        </nav>
        <Link href={GITHUB} target='_blank' rel='noreferrer' className='btn btn-primary btn-sm'>
          GitHub
        </Link>
      </div>
    </header>

    <main id='top'>
      {/* hero */}
      <section className='pt-20 pb-16'>
        <div className='mx-auto grid w-full max-w-[72rem] items-center gap-12 px-5 text-left lg:grid-cols-[1.05fr_1fr]'>
          <div>
            <h1>Your whole backend, in one blazing fast binary.</h1>
            <p className='text-text-muted mt-6 max-w-xl text-[1.1875rem]'>
              Blazers is a Back-end as a Service written in Rust: auth, a schema-less database, file
              storage, realtime streams, webhooks, cron jobs, encrypted secrets, pub/sub and an
              admin dashboard — all persisted to a single SQLite file. No Docker, no accounts, no
              cloud.
            </p>
            <div className='mt-8 flex flex-wrap justify-start gap-3'>
              <Link href='#quickstart' className='btn btn-primary btn-lg'>
                Get started
              </Link>
              <Link href={GITHUB} target='_blank' rel='noreferrer' className='btn btn-ghost btn-lg'>
                Source on GitHub
              </Link>
            </div>
            <dl className='mt-10 grid grid-cols-3 gap-6'>
              <div>
                <dt className='text-2xl font-bold tabular-nums'>1</dt>
                <dd className='text-text-muted m-0 text-sm'>binary</dd>
              </div>
              <div>
                <dt className='text-2xl font-bold tabular-nums'>1</dt>
                <dd className='text-text-muted m-0 text-sm'>SQLite file</dd>
              </div>
              <div>
                <dt className='text-2xl font-bold tabular-nums'>20+</dt>
                <dd className='text-text-muted m-0 text-sm'>API features</dd>
              </div>
            </dl>
          </div>
          <Term
            caption='zero to backend'
            lines={[
              [{ t: '$ ', c: 'prompt' }, { t: 'cargo', c: 'cmd' }, { t: ' install blazers' }],
              [
                { t: '$ ', c: 'prompt' },
                { t: 'blazers', c: 'cmd' },
              ],
              [
                { t: 'Server running at ', c: 'dim' },
                { t: 'http://localhost:8080', c: 'url' },
              ],
              [
                { t: '$ ', c: 'prompt' },
                { t: 'curl', c: 'cmd' },
                { t: ' -X ', c: 'flag' },
                { t: 'POST', c: 'flag' },
                { t: ' localhost:8080/api/auth/register ', c: 'url' },
                { t: '\\', c: 'dim' },
              ],
              [
                { t: '    -d ', c: 'flag' },
                { t: "'{", c: 'dim' },
                { t: '"email"', c: 'key' },
                { t: ':', c: 'dim' },
                { t: '"a@b.c"', c: 'str' },
                { t: ', ', c: 'dim' },
                { t: '"password"', c: 'key' },
                { t: ':', c: 'dim' },
                { t: '"secret1"', c: 'str' },
                { t: "}'", c: 'dim' },
              ],
              [
                { t: '{', c: 'dim' },
                { t: '"id"', c: 'key' },
                { t: ':', c: 'dim' },
                { t: '"82ff…"', c: 'str' },
                { t: ', ', c: 'dim' },
                { t: '"email"', c: 'key' },
                { t: ':', c: 'dim' },
                { t: '"a@b.c"', c: 'str' },
                { t: '}', c: 'dim' },
              ],
              [
                { t: '$ ', c: 'prompt' },
                { t: 'curl', c: 'cmd' },
                { t: ' localhost:8080/api/docs', c: 'url' },
              ],
              [{ t: 'Swagger UI — every endpoint documented', c: 'dim' }],
            ]}
          />
        </div>
      </section>

      {/* features */}
      <section id='features' className='py-20'>
        <div className='mx-auto w-full max-w-[72rem] px-5'>
          <div className='mx-auto mb-12 max-w-[44rem] text-center'>
            <h2>Everything a BaaS should do — nothing it shouldn&apos;t.</h2>
            <p className='text-text-muted mt-3 text-[1.0625rem]'>
              Modelled on PocketBase&apos;s &quot;one file, one binary&quot; philosophy and
              Supabase&apos;s feature breadth, shipped as a Cargo workspace you can audit crate by
              crate.
            </p>
          </div>
          <div className='grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3'>
            {FEATURES.map((f) => (
              <article
                key={f.title}
                className='border-border bg-surface rounded-[var(--radius-card)] border p-6'>
                <div className='bg-accent-soft text-accent mb-3.5 flex h-11 w-11 items-center justify-center rounded-lg'>
                  <f.icon className='h-[1.375rem] w-[1.375rem]' />
                </div>
                <h3 className='mb-2'>{f.title}</h3>
                <p className='text-text-muted text-[0.9375rem]'>{f.body}</p>
              </article>
            ))}
          </div>
        </div>
      </section>

      {/* compare */}
      <section id='compare' className='bg-bg-alt py-20'>
        <div className='mx-auto w-full max-w-[72rem] px-5'>
          <div className='mx-auto mb-12 max-w-[44rem] text-center'>
            <h2>Same ideas as PocketBase — more of Supabase&apos;s surface.</h2>
            <p className='text-text-muted mt-3 text-[1.0625rem]'>
              One binary and one SQLite file like PocketBase; auth, storage, realtime,
              functions-adjacent jobs and an admin UI like Supabase — without Postgres, GoTrue or a
              fleet of services.
            </p>
          </div>
          <div className='border-border overflow-x-auto rounded-[var(--radius-card)] border'>
            <table className='w-full border-collapse text-[0.9375rem] [&_tbody_tr:last-child>td]:border-b-0'>
              <thead>
                <tr>
                  <th className='border-border bg-bg-alt w-44 border-b px-5 py-3.5 text-left align-bottom font-semibold'>
                    Feature
                  </th>
                  <th className='border-border bg-bg-alt border-b border-l px-5 py-3.5 text-left align-bottom font-semibold'>
                    PocketBase
                  </th>
                  <th className='border-border bg-accent-soft border-b border-l px-5 py-3.5 text-left align-bottom'>
                    <span className='text-accent flex items-center gap-2 font-bold'>
                      Blazers
                      <span className='bg-accent rounded-full px-2 py-0.5 text-[0.6875rem] font-semibold tracking-wide text-white uppercase'>
                        rust
                      </span>
                    </span>
                  </th>
                  <th className='border-border bg-bg-alt border-b border-l px-5 py-3.5 text-left align-bottom font-semibold'>
                    Supabase
                  </th>
                </tr>
              </thead>
              <tbody>
                {COMPARE.map((row) => (
                  <tr key={row.feature} className='group'>
                    <td className='border-border text-text-muted border-b px-5 py-3.5 align-top font-medium'>
                      {row.feature}
                    </td>
                    <td className='border-border text-text-muted border-b border-l px-5 py-3.5 align-top'>
                      {row.pb}
                    </td>
                    <td className='border-border bg-accent-soft/50 text-text group-hover:bg-accent-soft border-b border-l px-5 py-3.5 align-top font-medium'>
                      {row.bz}
                    </td>
                    <td className='border-border text-text-muted border-b border-l px-5 py-3.5 align-top'>
                      {row.sb}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className='text-text-muted mt-4 text-center text-sm'>
            Comparing deployment model, auth, storage, realtime and ops surface — not raw
            throughput. Blazers aims for PocketBase&apos;s simplicity with more of Supabase&apos;s
            feature set.
          </p>
        </div>
      </section>

      {/* quickstart */}
      <section id='quickstart' className='py-20'>
        <div className='mx-auto w-full max-w-[72rem] px-5'>
          <div className='mx-auto mb-12 max-w-[44rem] text-center'>
            <h2>Up and running in under a minute.</h2>
            <p className='text-text-muted mt-3 text-[1.0625rem]'>
              Prebuilt binaries on the{' '}
              <Link href={`${GITHUB}/releases`} className='text-accent'>
                releases page
              </Link>
              , or build from source with Cargo.
            </p>
          </div>
          <div className='grid grid-cols-1 gap-6 md:grid-cols-2'>
            <Term
              caption='build from source'
              lines={[
                [
                  { t: '$ ', c: 'prompt' },
                  { t: 'git', c: 'cmd' },
                  { t: ' clone https://github.com/hieudoanm/blazers', c: 'url' },
                ],
                [
                  { t: '$ ', c: 'prompt' },
                  { t: 'cd', c: 'cmd' },
                  { t: ' blazers', c: 'dim' },
                ],
                [
                  { t: '$ ', c: 'prompt' },
                  { t: 'cargo', c: 'cmd' },
                  { t: ' build ', c: 'dim' },
                  { t: '--release', c: 'flag' },
                  { t: ' -p ', c: 'dim' },
                  { t: 'blazers', c: 'flag' },
                ],
                [
                  { t: '$ ', c: 'prompt' },
                  { t: './target/release/blazers', c: 'cmd' },
                ],
                [
                  { t: '➜ ', c: 'prompt' },
                  { t: 'Local:   ', c: 'dim' },
                  { t: 'http://localhost:8080', c: 'url' },
                ],
                [
                  { t: '➜ ', c: 'prompt' },
                  { t: 'Network: ', c: 'dim' },
                  { t: 'http://192.168.x.x:8080', c: 'url' },
                ],
              ]}
            />
            <Term
              caption='first API calls'
              lines={[
                [
                  { t: '$ ', c: 'prompt' },
                  { t: 'curl', c: 'cmd' },
                  { t: ' -X ', c: 'flag' },
                  { t: 'POST', c: 'flag' },
                  { t: ' localhost:8080/api/auth/register ', c: 'url' },
                  { t: '\\', c: 'dim' },
                ],
                [
                  { t: "    -H '", c: 'flag' },
                  { t: 'Content-Type: application/json', c: 'str' },
                  { t: "' \\", c: 'flag' },
                ],
                [
                  { t: "    -d '", c: 'flag' },
                  { t: '{"email":"admin@example.com","password":"secret123"}', c: 'str' },
                  { t: "'", c: 'flag' },
                ],
                [
                  { t: '$ ', c: 'prompt' },
                  { t: 'TOKEN=$(curl -s -X POST localhost:8080/api/auth/login \\', c: 'dim' },
                ],
                [{ t: "    -d '{…}' | jq -r .token)", c: 'dim' }],
                [
                  { t: '$ ', c: 'prompt' },
                  { t: 'curl', c: 'cmd' },
                  { t: ' -X ', c: 'flag' },
                  { t: 'POST', c: 'flag' },
                  { t: ' localhost:8080/api/collections ', c: 'url' },
                  { t: '\\', c: 'dim' },
                ],
                [{ t: '    -H "Authorization: Bearer $TOKEN" \\', c: 'flag' }],
                [
                  { t: "    -d '", c: 'flag' },
                  { t: '{"name":"posts","schema":{"title":"string"}}', c: 'str' },
                  { t: "'", c: 'flag' },
                ],
                [
                  { t: '{', c: 'dim' },
                  { t: '"name"', c: 'key' },
                  { t: ':', c: 'dim' },
                  { t: '"posts"', c: 'str' },
                  { t: ', …}', c: 'dim' },
                ],
              ]}
            />
          </div>
          <p className='text-text-muted mt-8 text-center text-[0.9375rem]'>
            Dashboard at <code>/</code> · API docs at <code>/api/docs</code> · Full docs at{' '}
            <Link href='/docs/' className='text-accent'>
              /docs
            </Link>{' '}
            · MCP server: <code>blazers mcp serve</code>. Data in <code>~/.blazers/</code> — set{' '}
            <code>BLAZERS_DATA</code>, <code>PORT</code> and <code>JWT_SECRET</code> to configure.
          </p>
        </div>
      </section>

      {/* CTA */}
      <section className='bg-bg-alt py-20'>
        <div className='mx-auto w-full max-w-[72rem] px-5 text-center'>
          <h2>Give your next side project a real backend.</h2>
          <p className='text-text-muted mt-3 text-[1.0625rem]'>
            Self-host it, embed it, or point your AI agent at it over MCP.
          </p>
          <div className='mt-8 flex flex-wrap justify-center gap-3'>
            <Link href={GITHUB} target='_blank' rel='noreferrer' className='btn btn-primary btn-lg'>
              Star on GitHub
            </Link>
            <Link href='/docs/' className='btn btn-ghost btn-lg'>
              Read the docs
            </Link>
          </div>
        </div>
      </section>
    </main>

    <footer className='border-border text-text-muted border-t py-10 text-sm'>
      <div className='mx-auto flex w-full max-w-[72rem] flex-wrap items-center gap-x-8 gap-y-4 px-5'>
        <span>blazers — blazing fast Back-end as a Service</span>
        <span className='ml-auto'>
          Rust · SQLite · MIT-style single binary ·{' '}
          <Link
            href={GITHUB}
            target='_blank'
            rel='noreferrer'
            className='text-text-muted hover:text-text no-underline'>
            GitHub
          </Link>
        </span>
      </div>
    </footer>
  </>
);

export default Home;
