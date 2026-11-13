import { AuthClient } from './auth.js';
import { FilesClient } from './files.js';
import { FunctionsClient } from './functions.js';
import { HttpClient, type ClientOptions } from './http.js';
import { CollectionClient, CollectionsClient } from './records.js';
import { RealtimeClient } from './realtime.js';
import type { BatchSubRequest, BatchResult } from './types.js';

export { BlazersError } from './errors.js';
export type {
  BatchResult,
  BatchSubRequest,
  Bucket,
  Collection,
  CollectionRules,
  FileRecord,
  FileToken,
  FilesPage,
  ListOptions,
  LoginResponse,
  LoginResult,
  PresenceEvent,
  RawEvent,
  RecordAction,
  RecordData,
  RecordEvent,
  RecordsPage,
  RealtimeEvent,
  User,
} from './types.js';
export type { ClientOptions } from './http.js';
export type { Credentials, MfaSetup, UpdateMeOptions } from './auth.js';
export type { EdgeFunction } from './functions.js';
export type { FileTokenOptions } from './files.js';
export type { PresenceOptions, SubscribeOptions } from './realtime.js';
export {
  AuthClient,
  CollectionClient,
  CollectionsClient,
  FilesClient,
  FunctionsClient,
  HttpClient,
  RealtimeClient,
};

export interface BlazersOptions extends ClientOptions {
  /** When true, `collection()` results are cached per name. Default true. */
  cacheCollections?: boolean;
}

/**
 * Blazers BaaS client — single entry point composing auth, records, files,
 * realtime and batch against a running `blazers serve` instance.
 *
 * ```ts
 * import { Blazers } from "blazers";
 *
 * const bz = new Blazers({ url: "http://localhost:8090" });
 * await bz.auth.login({ email: "a@b.c", password: "secret" });
 * const page = await bz.collection("posts").list({ filter: 'status="live"' });
 * ```
 */
export class Blazers {
  readonly http: HttpClient;
  readonly auth: AuthClient;
  readonly collections: CollectionsClient;
  readonly files: FilesClient;
  readonly functions: FunctionsClient;
  readonly realtime: RealtimeClient;

  private readonly collectionCache = new Map<string, CollectionClient>();

  constructor(options: BlazersOptions) {
    this.http = new HttpClient(options);
    this.auth = new AuthClient(this.http);
    this.collections = new CollectionsClient(this.http);
    this.files = new FilesClient(this.http);
    this.functions = new FunctionsClient(this.http);
    this.realtime = new RealtimeClient(this.http);
  }

  /** Client for one collection's records. */
  collection(name: string): CollectionClient {
    let c = this.collectionCache.get(name);
    if (!c) {
      c = new CollectionClient(this.http, name);
      this.collectionCache.set(name, c);
    }
    return c;
  }

  /** `GET /api/health` — liveness probe. */
  health(): Promise<unknown> {
    return this.http.get('/api/health');
  }

  /** `POST /api/batch` — up to 50 sub-requests in one round-trip. */
  batch(requests: BatchSubRequest[]): Promise<BatchResult[]> {
    return this.http.batch(requests);
  }
}
