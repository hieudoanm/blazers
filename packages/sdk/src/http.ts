import { BlazersError } from './errors.js';
import type { BatchResult, BatchSubRequest } from './types.js';

export interface ClientOptions {
  /** Base URL of the Blazers server, e.g. `http://localhost:8090`. */
  url: string;
  /** Bearer token attached to every request (set automatically on login). */
  token?: string;
  /** Extra headers sent with every request. */
  headers?: Record<string, string>;
  /** Fetch implementation override (defaults to global `fetch`). */
  fetch?: typeof fetch;
}

/** Low-level transport shared by all sub-clients. */
export class HttpClient {
  readonly baseUrl: string;
  private token?: string;
  private readonly headers: Record<string, string>;
  private readonly fetchImpl: typeof fetch;

  constructor(options: ClientOptions) {
    this.baseUrl = options.url.replace(/\/+$/, '');
    this.token = options.token;
    this.headers = options.headers ?? {};
    // Bind to globalThis: browsers (Safari) require fetch's `this` to be the
    // Window/realm object, and ESM strict mode leaves it undefined otherwise.
    this.fetchImpl = options.fetch ?? fetch.bind(globalThis);
  }

  get authToken(): string | undefined {
    return this.token;
  }

  setToken(token: string | undefined): void {
    this.token = token;
  }

  /** Absolute URL for an API path (used by file downloads etc.). */
  url(path: string): string {
    return `${this.baseUrl}${path.startsWith('/') ? path : `/${path}`}`;
  }

  /** WebSocket URL for `/ws`. */
  wsUrl(): string {
    const u = new URL(this.baseUrl);
    u.protocol = u.protocol === 'https:' ? 'wss:' : 'ws:';
    u.pathname = '/ws';
    u.search = '';
    return u.toString();
  }

  async request<T = unknown>(
    method: string,
    path: string,
    body?: unknown,
    extraHeaders?: Record<string, string>
  ): Promise<T> {
    const headers: Record<string, string> = {
      ...this.headers,
      ...extraHeaders,
    };
    let payload: BodyInit | undefined;
    if (body instanceof FormData) {
      payload = body;
    } else if (body !== undefined) {
      headers['Content-Type'] = 'application/json';
      payload = JSON.stringify(body);
    }
    if (this.token) headers['Authorization'] = `Bearer ${this.token}`;

    const res = await this.fetchImpl(this.url(path), {
      method,
      headers,
      body: payload,
    });
    if (res.status === 204) return undefined as T;

    const text = await res.text();
    let parsed: unknown = text;
    try {
      parsed = text ? JSON.parse(text) : undefined;
    } catch {
      /* non-JSON body */
    }
    if (!res.ok) throw new BlazersError(res.status, parsed);
    return parsed as T;
  }

  get<T = unknown>(path: string): Promise<T> {
    return this.request<T>('GET', path);
  }

  post<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('POST', path, body);
  }

  patch<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('PATCH', path, body);
  }

  put<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('PUT', path, body);
  }

  delete<T = unknown>(path: string): Promise<T> {
    return this.request<T>('DELETE', path);
  }

  /** POST /api/batch — up to 50 sub-requests, per-request errors isolated. */
  async batch(requests: BatchSubRequest[]): Promise<BatchResult[]> {
    return this.post<BatchResult[]>('/api/batch', { requests });
  }
}
