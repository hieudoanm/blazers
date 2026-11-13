'use client';

import type { Collection, RecordData } from '@blazers/sdk';
import { useCallback, useEffect, useState } from 'react';
import { getBlazers } from '@/lib/blazers';
import { shortId, timeAgo, tryParseJson } from '@/lib/format';

export default function RecordsTab() {
  const [collections, setCollections] = useState<Collection[]>([]);
  const [name, setName] = useState('');
  const [records, setRecords] = useState<RecordData[]>([]);
  const [total, setTotal] = useState(0);
  const [filter, setFilter] = useState('');
  const [sort, setSort] = useState('');
  const [json, setJson] = useState('{}');
  const [error, setError] = useState('');

  const loadCollections = useCallback(async () => {
    try {
      const cols = await getBlazers().collections.list();
      setCollections(cols.filter((c) => c.type !== 'auth'));
    } catch {
      setCollections([]);
    }
  }, []);

  const loadRecords = useCallback(async () => {
    if (!name) return;
    setError('');
    try {
      const page = await getBlazers()
        .collection(name)
        .list({ filter: filter || undefined, sort: sort || undefined });
      setRecords(page.records);
      setTotal(page.total);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [name, filter, sort]);

  useEffect(() => {
    void loadCollections();
  }, [loadCollections]);

  useEffect(() => {
    void loadRecords();
  }, [loadRecords]);

  async function createRecord() {
    const data = tryParseJson(json);
    if (!data) {
      setError('Record data must be a JSON object');
      return;
    }
    setError('');
    try {
      await getBlazers().collection(name).create(data);
      setJson('{}');
      await loadRecords();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function deleteRecord(id: string) {
    try {
      await getBlazers().collection(name).delete(id);
      await loadRecords();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="grid gap-6 lg:grid-cols-[280px_1fr]">
      <div className="card border-base-300 bg-base-100 border">
        <div className="card-body gap-1 p-4">
          <h2 className="card-title text-base">Collections</h2>
          {collections.length === 0 && (
            <p className="text-base-content/60 text-sm">
              None yet — create one in the Collections tab.
            </p>
          )}
          {collections.map((c) => (
            <button
              key={c.name}
              className={`btn btn-ghost justify-start ${name === c.name ? 'btn-active' : ''}`}
              onClick={() => setName(c.name)}>
              {c.name}
            </button>
          ))}
        </div>
      </div>

      <div className="flex flex-col gap-4">
        {name ? (
          <>
            <div className="card border-base-300 bg-base-100 border">
              <div className="card-body gap-3 p-4">
                <div className="flex flex-wrap items-end gap-2">
                  <fieldset className="fieldset">
                    <legend className="fieldset-legend text-xs">Filter</legend>
                    <input
                      className="input input-sm border-base-300 w-64 font-mono"
                      placeholder='status="live"'
                      value={filter}
                      onChange={(e) => setFilter(e.target.value)}
                    />
                  </fieldset>
                  <fieldset className="fieldset">
                    <legend className="fieldset-legend text-xs">Sort</legend>
                    <input
                      className="input input-sm border-base-300 w-40 font-mono"
                      placeholder="-created_at"
                      value={sort}
                      onChange={(e) => setSort(e.target.value)}
                    />
                  </fieldset>
                  <span className="text-base-content/60 pb-2 text-sm">
                    {total} record{total === 1 ? '' : 's'}
                  </span>
                </div>

                <div className="flex gap-2">
                  <textarea
                    className="textarea border-base-300 h-20 flex-1 font-mono text-xs"
                    placeholder='{"title": "Hello"}'
                    value={json}
                    onChange={(e) => setJson(e.target.value)}
                  />
                  <button
                    className="btn btn-primary btn-sm self-end"
                    onClick={createRecord}>
                    Create
                  </button>
                </div>
                {error && (
                  <div className="alert alert-error py-2 text-sm">{error}</div>
                )}
              </div>
            </div>

            <div className="rounded-box border-base-300 bg-base-100 overflow-x-auto border">
              <table className="table">
                <thead>
                  <tr>
                    <th>ID</th>
                    <th>Data</th>
                    <th>Created</th>
                    <th />
                  </tr>
                </thead>
                <tbody>
                  {records.map((r) => (
                    <tr key={r.id}>
                      <td className="font-mono text-xs">{shortId(r.id)}</td>
                      <td className="max-w-md truncate font-mono text-xs">
                        {JSON.stringify(r.data)}
                      </td>
                      <td className="text-xs">{timeAgo(r.created_at)}</td>
                      <td>
                        <button
                          className="btn btn-ghost btn-xs text-error"
                          onClick={() => void deleteRecord(r.id)}>
                          delete
                        </button>
                      </td>
                    </tr>
                  ))}
                  {records.length === 0 && (
                    <tr>
                      <td
                        colSpan={4}
                        className="text-base-content/50 py-8 text-center">
                        No records
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </>
        ) : (
          <div className="alert">Select a collection on the left.</div>
        )}
      </div>
    </div>
  );
}
