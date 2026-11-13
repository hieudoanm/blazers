'use client';

import type { Collection, RealtimeEvent } from '@blazers/sdk';
import { useEffect, useRef, useState } from 'react';
import { getBlazers } from '@/lib/blazers';
import { timeAgo } from '@/lib/format';

const MAX_EVENTS = 100;

interface LogEntry {
  at: number;
  event: RealtimeEvent;
}

export default function RealtimeTab() {
  const [collections, setCollections] = useState<Collection[]>([]);
  const [selected, setSelected] = useState('');
  const [filter, setFilter] = useState('');
  const [events, setEvents] = useState<LogEntry[]>([]);
  const [connected, setConnected] = useState(false);
  const [presenceChannel, setPresenceChannel] = useState('demo:room');
  const logRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    getBlazers()
      .collections.list()
      .then((cols) => setCollections(cols.filter((c) => c.type !== 'auth')))
      .catch(() => setCollections([]));
  }, []);

  useEffect(() => {
    const rt = getBlazers().realtime;
    let alive = true;
    void rt.connect().then(() => alive && setConnected(true));
    const off = rt.onMessage((data) => {
      setEvents((prev) =>
        [{ at: Date.now(), event: data as RealtimeEvent }, ...prev].slice(
          0,
          MAX_EVENTS
        )
      );
    });
    return () => {
      alive = false;
      off();
    };
  }, []);

  useEffect(() => {
    logRef.current?.scrollTo?.({ top: 0 });
  }, [events]);

  async function toggleSubscribe(name: string) {
    const rt = getBlazers().realtime;
    if (selected === name) {
      await rt.unsubscribe(name);
      setSelected('');
      return;
    }
    if (selected) await rt.unsubscribe(selected);
    await rt.subscribe(name, { filter: filter || undefined });
    setSelected(name);
  }

  async function joinPresence() {
    await getBlazers().realtime.presenceJoin(presenceChannel, {
      state: { demo: true },
    });
  }

  return (
    <div className="grid gap-6 lg:grid-cols-[280px_1fr]">
      <div className="flex flex-col gap-4">
        <div className="card border-base-300 bg-base-100 border">
          <div className="card-body gap-2 p-4">
            <div className="flex items-center justify-between">
              <h2 className="card-title text-base">Subscribe</h2>
              <span
                className={`badge badge-sm ${connected ? 'badge-success' : 'badge-ghost'}`}>
                {connected ? 'connected' : 'connecting…'}
              </span>
            </div>
            <fieldset className="fieldset">
              <legend className="fieldset-legend text-xs">
                Filter (optional)
              </legend>
              <input
                className="input input-sm border-base-300 font-mono"
                placeholder='status="live"'
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
              />
            </fieldset>
            <div className="flex max-h-64 flex-col gap-1 overflow-y-auto">
              {collections.map((c) => (
                <button
                  key={c.name}
                  className={`btn btn-ghost justify-start ${selected === c.name ? 'btn-active' : ''}`}
                  onClick={() => void toggleSubscribe(c.name)}>
                  {c.name}
                </button>
              ))}
              <button
                className={`btn btn-ghost justify-start ${selected === '*' ? 'btn-active' : ''}`}
                onClick={() => void toggleSubscribe('*')}>
                * (all collections)
              </button>
            </div>
          </div>
        </div>

        <div className="card border-base-300 bg-base-100 border">
          <div className="card-body gap-2 p-4">
            <h2 className="card-title text-base">Presence</h2>
            <fieldset className="fieldset">
              <legend className="fieldset-legend text-xs">Channel</legend>
              <input
                className="input input-sm border-base-300 font-mono"
                value={presenceChannel}
                onChange={(e) => setPresenceChannel(e.target.value)}
              />
            </fieldset>
            <button
              className="btn btn-outline btn-sm"
              onClick={() => void joinPresence()}>
              Join channel
            </button>
            <p className="text-base-content/50 text-xs">
              Open this page in two tabs to see join/state/leave events.
            </p>
          </div>
        </div>
      </div>

      <div
        ref={logRef}
        className="rounded-box border-base-300 bg-neutral text-neutral-content h-[32rem] overflow-y-auto border p-3 font-mono text-xs">
        {events.length === 0 && (
          <p className="text-neutral-content/50">
            Waiting for events — create/update/delete a record in the Records
            tab.
          </p>
        )}
        {events.map((e, i) => (
          <div key={i} className="border-neutral-content/10 mb-2 border-b pb-2">
            <span className="text-neutral-content/50">
              {new Date(e.at).toLocaleTimeString()} ·{' '}
              {timeAgo(new Date(e.at).toISOString())}
            </span>
            <pre className="break-all whitespace-pre-wrap">
              {JSON.stringify(e.event, null, 2)}
            </pre>
          </div>
        ))}
      </div>
    </div>
  );
}
