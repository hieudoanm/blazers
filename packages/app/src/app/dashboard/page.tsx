'use client';

import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { clearSession, getBlazers, isAuthed } from '@/lib/blazers';
import CollectionsTab from '@/components/dashboard/CollectionsTab';
import FilesTab from '@/components/dashboard/FilesTab';
import RealtimeTab from '@/components/dashboard/RealtimeTab';
import RecordsTab from '@/components/dashboard/RecordsTab';

const TABS = [
  { id: 'records', label: 'Records' },
  { id: 'collections', label: 'Collections' },
  { id: 'realtime', label: 'Realtime' },
  { id: 'files', label: 'Files' },
] as const;

type TabId = (typeof TABS)[number]['id'];

export default function DashboardPage() {
  const router = useRouter();
  const [tab, setTab] = useState<TabId>('records');
  const [ready, setReady] = useState(false);

  useEffect(() => {
    if (!isAuthed()) router.replace('/login');
    else setReady(true);
  }, [router]);

  if (!ready) return null;

  return (
    <main className="mx-auto max-w-6xl px-6 py-8">
      <header className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Dashboard</h1>
          <p className="text-base-content/60 text-sm">
            {getBlazers().http.baseUrl}
          </p>
        </div>
        <div className="flex gap-2">
          <Link href="/" className="btn btn-ghost btn-sm">
            Home
          </Link>
          <button
            className="btn btn-outline btn-sm"
            onClick={() => {
              clearSession();
              router.push('/login');
            }}>
            Sign out
          </button>
        </div>
      </header>

      <div role="tablist" className="tabs tabs-boxed mb-6">
        {TABS.map((t) => (
          <button
            key={t.id}
            role="tab"
            className={`tab ${tab === t.id ? 'tab-active' : ''}`}
            onClick={() => setTab(t.id)}>
            {t.label}
          </button>
        ))}
      </div>

      {tab === 'records' && <RecordsTab />}
      {tab === 'collections' && <CollectionsTab />}
      {tab === 'realtime' && <RealtimeTab />}
      {tab === 'files' && <FilesTab />}
    </main>
  );
}
