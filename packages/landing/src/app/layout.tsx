import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import type { FC, ReactNode } from 'react';
import './globals.css';

const inter = Inter({ subsets: ['latin'], variable: '--font-inter' });

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
    images: ['/og.png'],
  },
  twitter: { card: 'summary_large_image', images: ['/og.png'] },
};

interface RootLayoutProps {
  children: ReactNode;
}

const RootLayout: FC<RootLayoutProps> = ({ children }) => (
  <html lang='en' className={inter.variable}>
    <body>{children}</body>
  </html>
);

export default RootLayout;
