import type { HttpClient } from './http.js';
import type {
  Collection,
  CollectionRules,
  ListOptions,
  RecordData,
  RecordsPage,
} from './types.js';

/** CRUD + listing for one collection (`/api/collections/{name}/records*`). */
export class CollectionClient {
  constructor(
    private readonly http: HttpClient,
    readonly name: string
  ) {}

  private path(): string {
    return `/api/collections/${encodeURIComponent(this.name)}/records`;
  }

  private query(opts: ListOptions = {}): string {
    const p = new URLSearchParams();
    if (opts.page !== undefined) p.set('page', String(opts.page));
    if (opts.perPage !== undefined) p.set('per_page', String(opts.perPage));
    if (opts.search) p.set('search', opts.search);
    if (opts.filter) p.set('filter', opts.filter);
    if (opts.sort) p.set('sort', opts.sort);
    if (opts.expand) p.set('expand', opts.expand);
    const qs = p.toString();
    return qs ? `?${qs}` : '';
  }

  list(opts: ListOptions = {}): Promise<RecordsPage> {
    return this.http.get<RecordsPage>(`${this.path()}${this.query(opts)}`);
  }

  /** Fetches every page matching `opts` (bounded by `maxPages`). */
  async listAll(opts: ListOptions = {}, maxPages = 100): Promise<RecordData[]> {
    const perPage = opts.perPage ?? 200;
    const out: RecordData[] = [];
    for (let page = 1; page <= maxPages; page++) {
      const res = await this.list({ ...opts, page, perPage });
      out.push(...res.records);
      if (page >= res.total_pages || res.records.length === 0) break;
    }
    return out;
  }

  get(id: string): Promise<RecordData> {
    return this.http.get<RecordData>(
      `${this.path()}/${encodeURIComponent(id)}`
    );
  }

  /** `data` is the schemaless document; optional `id` sets the record id. */
  create(data: Record<string, unknown>, id?: string): Promise<RecordData> {
    return this.http.post<RecordData>(this.path(), {
      data,
      ...(id ? { id } : {}),
    });
  }

  /** Replaces the record's `data` document (send the complete desired object). */
  update(id: string, data: Record<string, unknown>): Promise<RecordData> {
    return this.http.patch<RecordData>(
      `${this.path()}/${encodeURIComponent(id)}`,
      { data }
    );
  }

  delete(id: string): Promise<void> {
    return this.http.delete<void>(`${this.path()}/${encodeURIComponent(id)}`);
  }
}

/** Admin-level collection management (`/api/collections*`). */
export class CollectionsClient {
  constructor(private readonly http: HttpClient) {}

  list(): Promise<Collection[]> {
    return this.http.get<Collection[]>('/api/collections');
  }

  get(name: string): Promise<Collection> {
    return this.http.get<Collection>(
      `/api/collections/${encodeURIComponent(name)}`
    );
  }

  create(name: string, schema = '{}'): Promise<Collection> {
    return this.http.post<Collection>('/api/collections', { name, schema });
  }

  /** Update schema and/or the five PocketBase-style rule expressions. */
  update(
    name: string,
    opts: { schema?: string } & CollectionRules
  ): Promise<Collection> {
    const body: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(opts)) {
      if (v !== undefined) body[k] = v;
    }
    return this.http.patch<Collection>(
      `/api/collections/${encodeURIComponent(name)}`,
      body
    );
  }

  delete(name: string): Promise<void> {
    return this.http.delete<void>(
      `/api/collections/${encodeURIComponent(name)}`
    );
  }
}
