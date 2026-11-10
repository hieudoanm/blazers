import type { Metadata } from 'next';
import type { FC, ReactNode } from 'react';
import './globals.css';

export const metadata: Metadata = {
  metadataBase: new URL('https://hieudoanm.github.io/blazers/'),
  title: {
    default: 'Blazers — blazing fast Back-end as a Service',
    template: '%s — Blazers',
  },
  description:
    'Auth, a schema-less database, file storage, realtime, webhooks, cron, secrets, pub/sub and an admin dashboard — in one Rust binary on a single SQLite file. Inspired by PocketBase and Supabase.',
  openGraph: {
    siteName: 'Blazers',
    type: 'website',
    title: 'Blazers — blazing fast Back-end as a Service',
    description:
      'Auth, database, files, realtime, webhooks, cron, secrets and an MCP server — one Rust binary, one SQLite file.',
    url: 'https://hieudoanm.github.io/blazers/',
  },
  twitter: { card: 'summary' },
};

interface RootLayoutProps {
  children: ReactNode;
}

const RootLayout: FC<RootLayoutProps> = ({ children }) => (
  <html lang='en'>
    <body>{children}</body>
  </html>
);

export default RootLayout;
