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

const Features: FC = () => (
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
);

export default Features;
