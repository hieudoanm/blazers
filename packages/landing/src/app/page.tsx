import type { Metadata, NextPage } from 'next';
import Image from 'next/image';
import Link from 'next/link';
import type { FC } from 'react';
import Features, { type Feature } from '@/components/molecules/Features';
import Hero from '@/components/molecules/Hero';
import CTA from '@/components/molecules/CTA';
import {
  LuBookOpen,
  LuCheck,
  LuCpu,
  LuDatabase,
  LuFolderUp,
  LuKeyRound,
  LuLayoutDashboard,
  LuLock,
  LuWebhook,
  LuX,
  LuZap,
} from 'react-icons/lu';

export const metadata: Metadata = {
  alternates: { canonical: '/' },
};

const GITHUB = 'https://github.com/hieudoanm/blazers';
// next/image with unoptimized does not apply basePath — mirror next.config.ts.
const LOGO =
  process.env.NODE_ENV === 'production' ? '/blazers/logo.svg' : '/logo.svg';

type Support = boolean | 'na';

interface CompareRow {
  feature: string;
  pb: Support;
  bz: Support;
  sb: Support;
}

interface CompareGroup {
  title: string;
  rows: CompareRow[];
}

const COMPARE_GROUPS: CompareGroup[] = [
  {
    title: 'Core & data',
    rows: [
      { feature: 'SQLite database', pb: true, bz: true, sb: false },
      {
        feature: 'Schema-less collections',
        pb: false,
        bz: true,
        sb: false,
      },
      {
        feature: 'Typed collection schemas',
        pb: true,
        bz: true,
        sb: true,
      },
      { feature: 'Record CRUD & querying', pb: true, bz: true, sb: true },
      { feature: 'Filter / sort / expand', pb: true, bz: true, sb: true },
      {
        feature: 'Per-collection record rules',
        pb: true,
        bz: true,
        sb: true,
      },
      { feature: 'Batch requests', pb: true, bz: true, sb: 'na' },
      { feature: 'Import / export', pb: false, bz: true, sb: true },
    ],
  },
  {
    title: 'Auth & users',
    rows: [
      { feature: 'Email + password auth', pb: true, bz: true, sb: true },
      {
        feature: 'Email verification & password reset',
        pb: true,
        bz: true,
        sb: true,
      },
      { feature: 'OAuth2 social login', pb: true, bz: true, sb: true },
      { feature: 'OTP / magic-code login', pb: true, bz: true, sb: true },
      { feature: 'MFA (TOTP)', pb: true, bz: true, sb: true },
      { feature: 'User impersonation', pb: true, bz: true, sb: true },
      { feature: 'Anonymous sign-in', pb: false, bz: true, sb: true },
      { feature: 'SSO / SAML', pb: false, bz: false, sb: true },
    ],
  },
  {
    title: 'Files & storage',
    rows: [
      { feature: 'File buckets', pb: true, bz: true, sb: true },
      { feature: 'Public file URLs', pb: true, bz: true, sb: true },
      {
        feature: 'Protected files & tokens',
        pb: true,
        bz: true,
        sb: true,
      },
      {
        feature: 'Tokens honor record rules',
        pb: true,
        bz: true,
        sb: true,
      },
      { feature: 'Image thumbnails', pb: true, bz: true, sb: true },
      { feature: 'S3-compatible storage', pb: false, bz: false, sb: true },
      { feature: 'Local disk storage', pb: true, bz: true, sb: false },
      { feature: 'CDN-backed delivery', pb: false, bz: false, sb: true },
    ],
  },
  {
    title: 'Realtime',
    rows: [
      { feature: 'Record change events', pb: true, bz: true, sb: true },
      {
        feature: 'oldRecord on update/delete',
        pb: true,
        bz: true,
        sb: true,
      },
      {
        feature: 'Per-subscription filters',
        pb: true,
        bz: true,
        sb: true,
      },
      { feature: 'Presence', pb: false, bz: true, sb: true },
      { feature: 'SSE streaming', pb: true, bz: true, sb: false },
      { feature: 'Pub/sub channels', pb: false, bz: true, sb: true },
      { feature: 'WebSocket transport', pb: false, bz: true, sb: true },
      {
        feature: 'Wildcard subscriptions',
        pb: false,
        bz: true,
        sb: true,
      },
    ],
  },
  {
    title: 'Extensibility & ops',
    rows: [
      { feature: 'Outbound webhooks', pb: false, bz: true, sb: true },
      {
        feature: 'Server-side record hooks',
        pb: true,
        bz: true,
        sb: true,
      },
      {
        feature: 'JS / edge-function runtime',
        pb: true,
        bz: true,
        sb: true,
      },
      { feature: 'Cron jobs', pb: true, bz: true, sb: true },
      { feature: 'Secrets vault', pb: false, bz: true, sb: true },
      { feature: 'Admin dashboard', pb: true, bz: true, sb: true },
      { feature: 'OpenAPI + API docs', pb: false, bz: true, sb: true },
      { feature: 'MCP server', pb: false, bz: true, sb: true },
    ],
  },
];

function support(value: Support) {
  if (value === 'na') return <span className="text-gray-300">-</span>;
  return value ? (
    <LuCheck aria-label="Supported" className="text-success size-4" />
  ) : (
    <LuX aria-label="Not supported" className="text-error size-4" />
  );
}

type TokKind =
  'prompt' | 'cmd' | 'flag' | 'str' | 'key' | 'url' | 'dim' | 'ok' | 'err';
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

interface TerminalProps {
  lines: Tok[][];
}

const Terminal: FC<TerminalProps> = ({ lines }) => (
  <div className="mockup-code">
    {lines.map((line, i) => {
      const prompt = line[0]?.c === 'prompt' ? line[0].t.trim() : null;
      const tokens = prompt ? line.slice(1) : line;
      return (
        <pre key={i} {...(prompt ? { 'data-prefix': prompt } : {})}>
          <code>
            {tokens.map((tok, j) => (
              <span key={j} className={TK[tok.c ?? 'ok']}>
                {tok.t}
              </span>
            ))}
          </code>
        </pre>
      );
    })}
  </div>
);

const FEATURES: Feature[] = [
  {
    title: 'Auth & permissions',
    body: 'Register, login, JWT bearer tokens, bcrypt passwords, password reset and email verification — plus per-collection record rules with @request.auth expressions.',
    icon: LuLock,
  },
  {
    title: 'Collections & records',
    body: 'Schema-less collections stored as dynamic SQLite tables. CRUD with paging, search, filter expressions, sorting, relation expand and JSON-schema validation.',
    icon: LuDatabase,
  },
  {
    title: 'File buckets',
    body: 'Public and private buckets, uploads, downloads, on-the-fly thumbnails and time-limited file tokens for protected files.',
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
    <main id="top">
      {/* hero */}
      <Hero
        title="Your whole backend, in one blazing fast binary."
        description="Blazers is a Back-end as a Service written in Rust: auth, a schema-less database, file storage, realtime streams, webhooks, cron jobs, encrypted secrets, pub/sub and an admin dashboard — all persisted to a single SQLite file. No Docker, no accounts, no cloud."
        primary={{ label: 'Get started', href: '#quickstart' }}
        secondary={{ label: 'Source on GitHub', href: GITHUB, external: true }}
        stats={[
          { value: '1', label: 'binary' },
          { value: '1', label: 'SQLite file' },
          { value: '20+', label: 'API features' },
        ]}>
        <Terminal
          lines={[
            [
              { t: '$ ', c: 'prompt' },
              { t: 'cargo', c: 'cmd' },
              { t: ' install blazers' },
            ],
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
        description="Modelled on PocketBase's &quot;one file, one binary&quot; philosophy and Supabase's feature breadth, shipped as a Cargo workspace you can audit crate by crate."
        features={FEATURES}
      />

      {/* compare */}
      <section id="compare" className="bg-gray-50 py-20">
        <div className="mx-auto w-full max-w-6xl px-5">
          <div className="mx-auto mb-12 max-w-2xl text-center">
            <h2 className="m-0 text-3xl leading-tight tracking-normal text-balance">
              Same ideas as PocketBase — more of Supabase&apos;s surface.
            </h2>
            <p className="mt-3 text-lg text-gray-500">
              One binary and one SQLite file like PocketBase; auth, storage,
              realtime, functions-adjacent jobs and an admin UI like Supabase —
              without Postgres, GoTrue or a fleet of services.
            </p>
          </div>
          <div className="grid gap-6 md:grid-cols-2">
            {COMPARE_GROUPS.map((group) => (
              <div
                key={group.title}
                className="overflow-x-auto rounded-2xl border border-gray-200">
                <table className="w-full border-collapse text-xs [&_tbody_tr:last-child>td]:border-b-0">
                  <thead>
                    <tr>
                      <th className="border-b border-gray-200 bg-gray-50 px-5 py-3.5 text-left align-bottom font-semibold whitespace-nowrap">
                        {group.title}
                      </th>
                      <th className="border-b border-l border-gray-200 bg-gray-50 px-5 py-3.5 text-left align-bottom font-semibold">
                        <span className="flex flex-col items-start gap-1">
                          <span className="badge badge-xs bg-teal-700 text-teal-50 uppercase">
                            go
                          </span>
                          <span>PocketBase</span>
                        </span>
                      </th>
                      <th className="bg-primary/10 border-b border-l border-gray-200 px-5 py-3.5 text-left align-bottom">
                        <span className="flex flex-col items-start gap-1">
                          <span className="badge badge-primary badge-xs uppercase">
                            rust
                          </span>
                          <span className="text-primary font-bold">
                            Blazers
                          </span>
                        </span>
                      </th>
                      <th className="border-b border-l border-gray-200 bg-gray-50 px-5 py-3.5 text-left align-bottom font-semibold">
                        <span className="flex flex-col items-start gap-1">
                          <span className="badge badge-xs bg-blue-900 text-blue-100 uppercase">
                            ts
                          </span>
                          <span>Supabase</span>
                        </span>
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {group.rows.map((row) => (
                      <tr key={row.feature} className="group">
                        <td className="border-b border-gray-200 px-5 py-3 align-top font-medium whitespace-nowrap text-gray-500">
                          {row.feature}
                        </td>
                        <td className="border-b border-l border-gray-200 px-5 py-3 align-top text-gray-500">
                          {support(row.pb)}
                        </td>
                        <td className="bg-primary/5 group-hover:bg-primary/10 border-b border-l border-gray-200 px-5 py-3 align-top font-medium text-gray-900">
                          {support(row.bz)}
                        </td>
                        <td className="border-b border-l border-gray-200 px-5 py-3 align-top text-gray-500">
                          {support(row.sb)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ))}
          </div>
          <p className="mt-4 text-center text-sm text-gray-500">
            Comparing deployment model, auth, storage, realtime and ops surface
            — not raw throughput. Blazers aims for PocketBase&apos;s simplicity
            with more of Supabase&apos;s feature set.
          </p>
        </div>
      </section>

      {/* quickstart */}
      <section id="quickstart" className="py-20">
        <div className="mx-auto w-full max-w-6xl px-5">
          <div className="mx-auto mb-12 max-w-2xl text-center">
            <h2 className="m-0 text-3xl leading-tight tracking-normal text-balance">
              Up and running in under a minute.
            </h2>
            <p className="mt-3 text-lg text-gray-500">
              Prebuilt binaries on the{' '}
              <Link href={`${GITHUB}/releases`} className="text-primary">
                releases page
              </Link>
              , or build from source with Cargo.
            </p>
          </div>
          <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
            <Terminal
              lines={[
                [
                  { t: '$ ', c: 'prompt' },
                  { t: 'git', c: 'cmd' },
                  {
                    t: ' clone https://github.com/hieudoanm/blazers',
                    c: 'url',
                  },
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
            <Terminal
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
                  {
                    t: '{"email":"admin@example.com","password":"secret123"}',
                    c: 'str',
                  },
                  { t: "'", c: 'flag' },
                ],
                [
                  { t: '$ ', c: 'prompt' },
                  {
                    t: 'TOKEN=$(curl -s -X POST localhost:8080/api/auth/login \\',
                    c: 'dim',
                  },
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
                  {
                    t: '{"name":"posts","schema":{"title":"string"}}',
                    c: 'str',
                  },
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
          <p className="mt-8 text-center text-sm text-gray-500">
            Dashboard at{' '}
            <code className="rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm">
              /
            </code>{' '}
            · API docs at{' '}
            <code className="rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm">
              /api/docs
            </code>{' '}
            · Full docs at{' '}
            <Link href="/docs/" className="text-primary">
              /docs
            </Link>{' '}
            · MCP server:{' '}
            <code className="rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm">
              blazers mcp serve
            </code>
            . Data in{' '}
            <code className="rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm">
              ~/.blazers/
            </code>{' '}
            — set{' '}
            <code className="rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm">
              BLAZERS_DATA
            </code>
            ,{' '}
            <code className="rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm">
              PORT
            </code>{' '}
            and{' '}
            <code className="rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm">
              JWT_SECRET
            </code>{' '}
            to configure.
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
  </>
);

export default Home;
