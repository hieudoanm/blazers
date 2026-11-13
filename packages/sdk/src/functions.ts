import type { HttpClient } from './http.js';

/** A stored edge function (JS source executed by the embedded runtime). */
export interface EdgeFunction {
  name: string;
  code: string;
  created_at?: string;
  updated_at?: string;
}

/**
 * Edge functions client — `/api/functions`.
 * Management (list/upsert/remove) requires an admin token; `invoke` is open
 * and uses the client's current Bearer token, if any, as the function's
 * `auth` context.
 */
export class FunctionsClient {
  constructor(private readonly http: HttpClient) {}

  list(): Promise<EdgeFunction[]> {
    return this.http.get<EdgeFunction[]>('/api/functions');
  }

  /** Create or replace a function by name. */
  upsert(fn: EdgeFunction): Promise<EdgeFunction> {
    return this.http.post<EdgeFunction>('/api/functions', {
      name: fn.name,
      code: fn.code,
    });
  }

  remove(name: string): Promise<void> {
    return this.http.delete(`/api/functions/${encodeURIComponent(name)}`);
  }

  /**
   * Invoke `name` with `input`; resolves to the handler's return value.
   * The handler receives `{ input, auth }` — `auth` is populated when the
   * client holds a session token.
   */
  async invoke<T = unknown>(name: string, input?: unknown): Promise<T> {
    const res = await this.http.post<{ result: T }>(
      `/api/functions/${encodeURIComponent(name)}`,
      { input }
    );
    return res.result;
  }
}
