import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import type { FC, ReactNode } from 'react';
import Footer from '@/components/molecules/Footer';
import Navbar from '@/components/molecules/Navbar';
import './globals.css';

const inter = Inter({ subsets: ['latin'] });

const GITHUB = 'https://github.com/hieudoanm/blazers';
const LOGO =
  process.env.NODE_ENV === 'production' ? '/blazers/logo.svg' : '/logo.svg';
const NAV_LINKS = [
  { label: 'Features', href: '/#features' },
  { label: 'Versus', href: '/#compare' },
  { label: 'Quickstart', href: '/#quickstart' },
  { label: 'Docs', href: '/docs/' },
];

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
  <html
    lang="en"
    data-theme="blazers"
    className={`scroll-smooth antialiased motion-reduce:scroll-auto ${inter.className}`}>
    <body className="bg-base-100 text-base-content m-0 leading-relaxed">
      <Navbar
        name="blazers"
        logoSrc={LOGO}
        links={NAV_LINKS}
        githubUrl={GITHUB}
      />
      {children}
      <Footer
        name="blazers"
        tagline="Blazing fast Back-end as a Service · Rust · SQLite · MIT"
        githubUrl={GITHUB}
      />
      
    </body>
  </html>
);

export default RootLayout;
