'use client';

import type { Bucket, FileRecord } from '@blazers/sdk';
import { useCallback, useEffect, useRef, useState } from 'react';
import { getBlazers } from '@/lib/blazers';
import { timeAgo } from '@/lib/format';

export default function FilesTab() {
  const [buckets, setBuckets] = useState<Bucket[]>([]);
  const [bucket, setBucket] = useState('');
  const [files, setFiles] = useState<FileRecord[]>([]);
  const [newBucket, setNewBucket] = useState('');
  const [isPublic, setIsPublic] = useState(false);
  const [error, setError] = useState('');
  const fileInput = useRef<HTMLInputElement>(null);

  const loadBuckets = useCallback(async () => {
    try {
      const bs = await getBlazers().files.listBuckets();
      setBuckets(bs);
      setBucket((cur) => cur || bs[0]?.name || '');
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  const loadFiles = useCallback(async () => {
    if (!bucket) return;
    try {
      const page = await getBlazers().files.listFiles(bucket);
      setFiles(page.files);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [bucket]);

  useEffect(() => {
    void loadBuckets();
  }, [loadBuckets]);

  useEffect(() => {
    void loadFiles();
  }, [loadFiles]);

  async function createBucket() {
    if (!newBucket.trim()) return;
    setError('');
    try {
      await getBlazers().files.createBucket(newBucket.trim(), isPublic);
      setNewBucket('');
      setIsPublic(false);
      await loadBuckets();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function upload(file: File) {
    if (!bucket) return;
    setError('');
    try {
      await getBlazers().files.upload(bucket, file);
      if (fileInput.current) fileInput.current.value = '';
      await loadFiles();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function remove(f: FileRecord) {
    try {
      await getBlazers().files.deleteFile(bucket, f.id);
      await loadFiles();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="grid gap-6 lg:grid-cols-[280px_1fr]">
      <div className="flex flex-col gap-4">
        <div className="card border-base-300 bg-base-100 border">
          <div className="card-body gap-2 p-4">
            <h2 className="card-title text-base">Buckets</h2>
            {buckets.map((b) => (
              <button
                key={b.name}
                className={`btn btn-ghost justify-start ${bucket === b.name ? 'btn-active' : ''}`}
                onClick={() => setBucket(b.name)}>
                {b.name}
                {b.is_public && (
                  <span className="badge badge-xs badge-success">public</span>
                )}
              </button>
            ))}
            {buckets.length === 0 && (
              <p className="text-base-content/60 text-sm">No buckets yet.</p>
            )}
          </div>
        </div>

        <div className="card border-base-300 bg-base-100 border">
          <div className="card-body gap-2 p-4">
            <h2 className="card-title text-base">New bucket</h2>
            <input
              className="input input-sm border-base-300 font-mono"
              placeholder="avatars"
              value={newBucket}
              onChange={(e) => setNewBucket(e.target.value)}
            />
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                className="checkbox checkbox-sm"
                checked={isPublic}
                onChange={(e) => setIsPublic(e.target.checked)}
              />
              Public (no auth/token needed to download)
            </label>
            <button
              className="btn btn-primary btn-sm"
              onClick={() => void createBucket()}>
              Create
            </button>
          </div>
        </div>
      </div>

      <div className="flex flex-col gap-4">
        {bucket ? (
          <>
            <div className="card border-base-300 bg-base-100 border">
              <div className="card-body gap-2 p-4">
                <h2 className="card-title text-base">Upload to {bucket}</h2>
                <input
                  ref={fileInput}
                  type="file"
                  className="file-input file-input-sm border-base-300"
                  onChange={(e) => {
                    const f = e.target.files?.[0];
                    if (f) void upload(f);
                  }}
                />
                {error && (
                  <div className="alert alert-error py-2 text-sm">{error}</div>
                )}
              </div>
            </div>

            <div className="rounded-box border-base-300 bg-base-100 overflow-x-auto border">
              <table className="table">
                <thead>
                  <tr>
                    <th>Filename</th>
                    <th>Type</th>
                    <th>Size</th>
                    <th>Uploaded</th>
                    <th />
                  </tr>
                </thead>
                <tbody>
                  {files.map((f) => (
                    <tr key={f.id}>
                      <td className="font-mono text-sm">{f.filename}</td>
                      <td className="text-xs">{f.mime_type}</td>
                      <td className="text-xs">
                        {(f.size / 1024).toFixed(1)} KB
                      </td>
                      <td className="text-xs">{timeAgo(f.created_at)}</td>
                      <td className="flex gap-1">
                        <a
                          className="btn btn-ghost btn-xs"
                          href={getBlazers().files.downloadUrl(bucket, f.id)}
                          target="_blank"
                          rel="noreferrer">
                          open
                        </a>
                        <button
                          className="btn btn-ghost btn-xs text-error"
                          onClick={() => void remove(f)}>
                          del
                        </button>
                      </td>
                    </tr>
                  ))}
                  {files.length === 0 && (
                    <tr>
                      <td
                        colSpan={5}
                        className="text-base-content/50 py-8 text-center">
                        No files
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </>
        ) : (
          <div className="alert">Create a bucket on the left.</div>
        )}
      </div>
    </div>
  );
}
