/** Wire types mirroring the Blazers REST API (`/api/*`). */

export interface User {
  id: string;
  email: string;
  /** Custom user fields (schemaless JSON). */
  data: Record<string, unknown>;
  verified: boolean;
  mfa_enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface LoginResponse {
  user: User;
  token: string;
}

/** `POST /api/auth/login` returns this union: full session or an MFA challenge. */
export type LoginResult =
  | ({ mfa_required?: false } & LoginResponse)
  | { mfa_required: true; user: User };

export interface RecordData {
  id: string;
  /** Schemaless document; `expand_<field>` keys appear here after `?expand=`. */
  data: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface RecordsPage {
  records: RecordData[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface ListOptions {
  page?: number;
  perPage?: number;
  /** Free-text search across record data. */
  search?: string;
  /** PocketBase-style filter expression, e.g. `status="live" && views>10`. */
  filter?: string;
  /** Comma-separated fields; prefix with `-` for descending. */
  sort?: string;
  /** Comma-separated relation fields to resolve into `data.expand_<field>`. */
  expand?: string;
}

export interface CollectionRules {
  list_rule?: string;
  view_rule?: string;
  create_rule?: string;
  update_rule?: string;
  delete_rule?: string;
}

export interface Collection {
  name: string;
  /** JSON-schema string validating `record.data`. */
  schema: string;
  list_rule: string;
  view_rule: string;
  create_rule: string;
  update_rule: string;
  delete_rule: string;
  /** "records" for data collections, "auth" for the managed users collection. */
  type: 'records' | 'auth' | string;
  created_at: string;
  updated_at: string;
}

export interface Bucket {
  name: string;
  is_public: boolean;
  created_at: string;
  updated_at: string;
}

export interface FileRecord {
  id: string;
  bucket: string;
  filename: string;
  mime_type: string;
  size: number;
  created_at: string;
  updated_at: string;
}

export interface FilesPage {
  files: FileRecord[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface BatchSubRequest {
  method: string;
  /** Path or absolute URL under the same origin, e.g. `/api/collections/posts/records`. */
  url: string;
  headers?: Record<string, string>;
  body?: unknown;
}

export interface BatchResult {
  status: number;
  body: unknown;
}

export interface FileToken {
  token: string;
  expires_in: number;
}

// ---- Realtime (WebSocket `/ws`) ----

export type RecordAction = 'create' | 'update' | 'delete';

export interface RecordEvent {
  type: 'record';
  action: RecordAction;
  collection: string;
  record: RecordData;
  /** Present on update/delete: the record as it was before the mutation. */
  oldRecord?: RecordData;
}

export type PresenceAction = 'join' | 'leave' | 'state';

export interface PresenceEvent {
  type: 'presence';
  action: PresenceAction;
  channel: string;
  /** Client id of the member that changed. */
  client: string;
  state?: unknown;
}

/** Any other frame the server pushes (manual broadcasts, pings, …). */
export interface RawEvent {
  type: string;
  [key: string]: unknown;
}

export type RealtimeEvent = RecordEvent | PresenceEvent | RawEvent;
