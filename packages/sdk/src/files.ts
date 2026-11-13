import type { HttpClient } from './http.js';
import type { Bucket, FileRecord, FilesPage, FileToken } from './types.js';

export interface FileTokenOptions {
  /** When set, the token is only issued if the caller passes the record's
   * `view_rule` — mirrors PocketBase file tokens bound to records. */
  collection?: string;
  record?: string;
}

/** Buckets + file upload/download (`/api/buckets/*`). */
export class FilesClient {
  constructor(private readonly http: HttpClient) {}

  // ---- Buckets ----

  listBuckets(): Promise<Bucket[]> {
    return this.http.get<Bucket[]>('/api/buckets');
  }

  createBucket(name: string, isPublic = false): Promise<Bucket> {
    return this.http.post<Bucket>('/api/buckets', {
      name,
      is_public: isPublic,
    });
  }

  getBucket(name: string): Promise<Bucket> {
    return this.http.get<Bucket>(`/api/buckets/${encodeURIComponent(name)}`);
  }

  deleteBucket(name: string): Promise<void> {
    return this.http.delete<void>(`/api/buckets/${encodeURIComponent(name)}`);
  }

  // ---- Files ----

  listFiles(bucket: string, page = 1, perPage = 50): Promise<FilesPage> {
    return this.http.get<FilesPage>(
      `/api/buckets/${encodeURIComponent(bucket)}/files?page=${page}&per_page=${perPage}`
    );
  }

  /** Multipart upload — accepts `Blob`/`File` (browser) or `Buffer` (Node ≥18
   * via undici `FormData`/`Blob`). Field name must be the file itself. */
  async upload(
    bucket: string,
    file: Blob,
    filename?: string
  ): Promise<FileRecord> {
    const form = new FormData();
    const name = filename ?? (file instanceof File ? file.name : 'file');
    form.append('file', file, name);
    return this.http.request<FileRecord>(
      'POST',
      `/api/buckets/${encodeURIComponent(bucket)}/files`,
      form
    );
  }

  /** URL that serves the file — works unauthenticated for public buckets;
   * protected buckets need `token` (from {@link fileToken}) or a Bearer header. */
  downloadUrl(bucket: string, fileId: string, token?: string): string {
    const base = `/api/buckets/${encodeURIComponent(bucket)}/files/${encodeURIComponent(fileId)}`;
    return this.http.url(
      token ? `${base}?token=${encodeURIComponent(token)}` : base
    );
  }

  /** Fetches the file bytes (uses the session token when set). */
  async download(
    bucket: string,
    fileId: string,
    token?: string
  ): Promise<Blob> {
    const headers: Record<string, string> = {};
    if (this.http.authToken)
      headers['Authorization'] = `Bearer ${this.http.authToken}`;
    const res = await fetch.bind(globalThis)(
      this.downloadUrl(bucket, fileId, token),
      { headers }
    );
    if (!res.ok) throw new Error(`download failed: HTTP ${res.status}`);
    return res.blob();
  }

  /** Issues a ~1h download token for a protected file (auth required). */
  fileToken(
    bucket: string,
    fileId: string,
    opts: FileTokenOptions = {}
  ): Promise<FileToken> {
    const body =
      opts.collection && opts.record
        ? { collection: opts.collection, record: opts.record }
        : undefined;
    return this.http.post<FileToken>(
      `/api/buckets/${encodeURIComponent(bucket)}/files/${encodeURIComponent(fileId)}/token`,
      body
    );
  }

  deleteFile(bucket: string, fileId: string): Promise<void> {
    return this.http.delete<void>(
      `/api/buckets/${encodeURIComponent(bucket)}/files/${encodeURIComponent(fileId)}`
    );
  }
}
