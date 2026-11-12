import type { Metadata, NextPage } from 'next';
import Image from 'next/image';
import Link from 'next/link';
import type { FC } from 'react';
import Footer from '@/components/Footer';
import Features, { type Feature } from '@/components/Features';
import Hero from '@/components/Hero';
import CTA from '@/components/CTA';
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
  prompt: 'text-accent select-none',
  cmd: 'font-semibold text-sky-300',
  flag: 'text-amber-400',
  str: 'text-green-300',
  key: 'text-violet-300',
  url: 'text-cyan-300',
  dim: 'text-slate-400',
  ok: 'text-gray-200',
  err: 'text-red-400',
};

const TERM_FIGURE =
  'm-0 overflow-hidden rounded-2xl border border-gray-800 bg-gray-900 shadow-xl';
const TERM_CAPTION =
  'border-b border-gray-800 bg-slate-950 px-3.5 pt-2 pb-2.5 text-xs text-slate-400';
const TERM_PRE =
  'm-0 overflow-x-auto px-5 py-4 font-mono text-xs leading-relaxed text-gray-200 whitespace-pre tabular-nums';

interface TermProps {
  caption: string;
  lines: Tok[][];
}

const Term: FC<TermProps> = ({ caption, lines }) => (
  <figure className={TERM_FIGURE}>
    <figcaption className={TERM_CAPTION}>{caption}</figcaption>
    <pre className={TERM_PRE}>
      {lines.map((line, i) => (
        <span key={i} className='block min-h-[1.7em]'>
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

const FEATURES: Feature[] = [
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
    body: 'AES-256-GCM encrypted key/value store for credentials, plus a Redis-style cache — strings, lists, hashes, atomic INCR/APPEND, TTL — persisted to SQLite.',
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

const Home: NextPage = () => (
  <>
    <header className='border-gray-200 bg-white/90 sticky top-0 z-50 border-b backdrop-blur'>
      <div className='mx-auto flex h-16 w-full max-w-6xl items-center gap-6 px-5'>
        <Link
          href='#top'
          className='text-gray-900 flex items-center gap-2 text-lg font-bold no-underline'>
          <Image src={LOGO} alt='' width={28} height={28} className='rounded-md' />
          <span>blazers</span>
        </Link>
        <nav className='ml-auto hidden items-center gap-5 lg:flex'>
          <Link
            href='#features'
            className='text-gray-500 hover:text-gray-900 text-sm no-underline'>
            Features
          </Link>
          <Link
            href='#compare'
            className='text-gray-500 hover:text-gray-900 text-sm no-underline'>
            vs PocketBase
          </Link>
          <Link
            href='#quickstart'
            className='text-gray-500 hover:text-gray-900 text-sm no-underline'>
            Quickstart
          </Link>
          <Link
            href='/docs/'
            className='text-gray-500 hover:text-gray-900 text-sm no-underline'>
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
      <Hero
        title='Your whole backend, in one blazing fast binary.'
        description='Blazers is a Back-end as a Service written in Rust: auth, a schema-less database, file storage, realtime streams, webhooks, cron jobs, encrypted secrets, pub/sub and an admin dashboard — all persisted to a single SQLite file. No Docker, no accounts, no cloud.'
        primary={{ label: 'Get started', href: '#quickstart' }}
        secondary={{ label: 'Source on GitHub', href: GITHUB, external: true }}
        stats={[
          { value: '1', label: 'binary' },
          { value: '1', label: 'SQLite file' },
          { value: '20+', label: 'API features' },
        ]}>
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
      </Hero>

      <Features
        title="Everything a BaaS should do — nothing it shouldn't."
        description='Modelled on PocketBase&apos;s "one file, one binary" philosophy and Supabase&apos;s feature breadth, shipped as a Cargo workspace you can audit crate by crate.'
        features={FEATURES}
      />


      {/* compare */}
      <section id='compare' className='bg-gray-50 py-20'>
        <div className='mx-auto w-full max-w-6xl px-5'>
          <div className='mx-auto mb-12 max-w-2xl text-center'>
            <h2 className='m-0 text-3xl leading-tight tracking-normal text-balance'>Same ideas as PocketBase — more of Supabase&apos;s surface.</h2>
            <p className='text-gray-500 mt-3 text-lg'>
              One binary and one SQLite file like PocketBase; auth, storage, realtime,
              functions-adjacent jobs and an admin UI like Supabase — without Postgres, GoTrue or a
              fleet of services.
            </p>
          </div>
          <div className='border-gray-200 overflow-x-auto rounded-2xl border'>
            <table className='w-full border-collapse text-sm [&_tbody_tr:last-child>td]:border-b-0'>
              <thead>
                <tr>
                  <th className='border-gray-200 bg-gray-50 w-44 border-b px-5 py-3.5 text-left align-bottom font-semibold'>
                    Feature
                  </th>
                  <th className='border-gray-200 bg-gray-50 border-b border-l px-5 py-3.5 text-left align-bottom font-semibold'>
                    PocketBase
                  </th>
                  <th className='border-gray-200 bg-primary/10 border-b border-l px-5 py-3.5 text-left align-bottom'>
                    <span className='text-primary flex items-center gap-2 font-bold'>
                      Blazers
                      <span className='badge badge-primary uppercase'>
                        rust
                      </span>
                    </span>
                  </th>
                  <th className='border-gray-200 bg-gray-50 border-b border-l px-5 py-3.5 text-left align-bottom font-semibold'>
                    Supabase
                  </th>
                </tr>
              </thead>
              <tbody>
                {COMPARE.map((row) => (
                  <tr key={row.feature} className='group'>
                    <td className='border-gray-200 text-gray-500 border-b px-5 py-3.5 align-top font-medium'>
                      {row.feature}
                    </td>
                    <td className='border-gray-200 text-gray-500 border-b border-l px-5 py-3.5 align-top'>
                      {row.pb}
                    </td>
                    <td className='border-gray-200 bg-primary/5 text-gray-900 group-hover:bg-primary/10 border-b border-l px-5 py-3.5 align-top font-medium'>
                      {row.bz}
                    </td>
                    <td className='border-gray-200 text-gray-500 border-b border-l px-5 py-3.5 align-top'>
                      {row.sb}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className='text-gray-500 mt-4 text-center text-sm'>
            Comparing deployment model, auth, storage, realtime and ops surface — not raw
            throughput. Blazers aims for PocketBase&apos;s simplicity with more of Supabase&apos;s
            feature set.
          </p>
        </div>
      </section>

      {/* quickstart */}
      <section id='quickstart' className='py-20'>
        <div className='mx-auto w-full max-w-6xl px-5'>
          <div className='mx-auto mb-12 max-w-2xl text-center'>
            <h2 className='m-0 text-3xl leading-tight tracking-normal text-balance'>Up and running in under a minute.</h2>
            <p className='text-gray-500 mt-3 text-lg'>
              Prebuilt binaries on the{' '}
              <Link href={`${GITHUB}/releases`} className='text-primary'>
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
          <p className='text-gray-500 mt-8 text-center text-sm'>
            Dashboard at <code className='rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm'>/</code> · API docs at <code className='rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm'>/api/docs</code> · Full docs at{' '}
            <Link href='/docs/' className='text-primary'>
              /docs
            </Link>{' '}
            · MCP server: <code className='rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm'>blazers mcp serve</code>. Data in <code className='rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm'>~/.blazers/</code> — set{' '}
            <code className='rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm'>BLAZERS_DATA</code>, <code className='rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm'>PORT</code> and <code className='rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm'>JWT_SECRET</code> to configure.
          </p>
        </div>
      </section>

      <CTA
        title="Give your next side project a real backend."
        description="Self-host it, embed it, or point your AI agent at it over MCP."
        primary={{ label: 'Star on GitHub', href: GITHUB, external: true }}
        secondary={{ label: 'Read the docs', href: '/docs/' }}
      />
    </main>

    <Footer />
  </>
);

export default Home;
