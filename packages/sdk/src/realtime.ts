import type { HttpClient } from './http.js';
import type { PresenceEvent, RecordEvent, RealtimeEvent } from './types.js';

export interface SubscribeOptions {
  /**
   * PocketBase-style filter expression evaluated server-side per event,
   * e.g. `status="live"`. `@request.auth.*` resolves to null on WS.
   */
  filter?: string;
  /** Called for record create/update/delete events on this subscription. */
  onRecord?: (event: RecordEvent) => void;
  /** Called for any frame not handled by a more specific callback. */
  onEvent?: (event: RealtimeEvent) => void;
}

export interface PresenceOptions {
  /** Initial presence state shared with channel members. */
  state?: unknown;
  onEvent?: (event: PresenceEvent) => void;
}

type MessageHandler = (data: unknown) => void;

/**
 * WebSocket realtime client for `/ws`. Lazily connects on first use and
 * reconnects with backoff. Protocol frames are JSON:
 *
 * - subscribe:   `{"action":"subscribe","collection":"posts","filter":"..."}`
 * - unsubscribe: `{"action":"unsubscribe","collection":"posts"}`
 * - presence:    `{"action":"presence_join"|"presence"|"presence_leave","channel":"c","state":{...}}`
 *
 * Server → client: `{"type":"record",...}`, `{"type":"presence",...}`.
 */
export class RealtimeClient {
  private ws?: WebSocket;
  private readonly subs = new Map<string, SubscribeOptions>();
  private readonly presences = new Map<string, PresenceOptions>();
  private readonly handlers = new Set<MessageHandler>();
  private reconnectTimer?: ReturnType<typeof setTimeout>;
  private attempts = 0;
  private closed = false;

  constructor(private readonly http: HttpClient) {}

  /** Current connection state. */
  get connected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /** Resolves once the socket is open (connects if needed). */
  connect(): Promise<void> {
    if (this.connected) return Promise.resolve();
    if (this.ws && this.ws.readyState === WebSocket.CONNECTING) {
      return new Promise((resolve, reject) => {
        this.ws!.addEventListener('open', () => resolve(), { once: true });
        this.ws!.addEventListener(
          'error',
          () => reject(new Error('ws connect failed')),
          {
            once: true,
          }
        );
      });
    }
    this.closed = false;
    const ws = new WebSocket(this.http.wsUrl());
    this.ws = ws;
    ws.addEventListener('message', (ev) => this.dispatch(ev.data));
    ws.addEventListener('close', () => this.scheduleReconnect());
    ws.addEventListener('error', () => ws.close());
    return new Promise((resolve, reject) => {
      ws.addEventListener(
        'open',
        () => {
          this.attempts = 0;
          this.resubscribeAll();
          resolve();
        },
        { once: true }
      );
      ws.addEventListener(
        'error',
        () => reject(new Error('ws connect failed')),
        {
          once: true,
        }
      );
    });
  }

  /** Subscribe to record events for `collection` (`"*"` = all collections). */
  async subscribe(
    collection: string,
    options: SubscribeOptions = {}
  ): Promise<void> {
    this.subs.set(collection, options);
    await this.connect();
    this.send({
      action: 'subscribe',
      collection,
      ...(options.filter ? { filter: options.filter } : {}),
    });
  }

  async unsubscribe(collection: string): Promise<void> {
    this.subs.delete(collection);
    if (this.connected) this.send({ action: 'unsubscribe', collection });
  }

  /** Join a presence channel (or update state with action `"presence"`). */
  async presenceJoin(
    channel: string,
    opts: PresenceOptions = {}
  ): Promise<void> {
    this.presences.set(channel, opts);
    await this.connect();
    this.send({
      action: 'presence_join',
      channel,
      ...(opts.state !== undefined ? { state: opts.state } : {}),
    });
  }

  async presenceUpdate(channel: string, state: unknown): Promise<void> {
    if (this.connected) {
      this.send({ action: 'presence', channel, state });
    }
  }

  async presenceLeave(channel: string): Promise<void> {
    this.presences.delete(channel);
    if (this.connected) this.send({ action: 'presence_leave', channel });
  }

  /** Raw frame listener for frames not covered by subscribe/presence. */
  onMessage(handler: MessageHandler): () => void {
    this.handlers.add(handler);
    return () => this.handlers.delete(handler);
  }

  /** Permanently closes the socket (no reconnect). */
  close(): void {
    this.closed = true;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.ws?.close();
    this.ws = undefined;
  }

  private send(frame: Record<string, unknown>): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(frame));
    }
  }

  private dispatch(data: unknown): void {
    let event: RealtimeEvent;
    try {
      event = JSON.parse(String(data)) as RealtimeEvent;
    } catch {
      return;
    }
    for (const handler of this.handlers) handler(event);
    if (event.type === 'record') {
      const rec = event as RecordEvent;
      for (const key of [rec.collection, '*']) {
        const opts = this.subs.get(key);
        opts?.onRecord?.(rec);
        opts?.onEvent?.(rec);
      }
    } else if (event.type === 'presence') {
      this.presences
        .get((event as PresenceEvent).channel)
        ?.onEvent?.(event as PresenceEvent);
    }
  }

  private resubscribeAll(): void {
    for (const [collection, opts] of this.subs) {
      this.send({
        action: 'subscribe',
        collection,
        ...(opts.filter ? { filter: opts.filter } : {}),
      });
    }
    for (const [channel, opts] of this.presences) {
      this.send({
        action: 'presence_join',
        channel,
        ...(opts.state !== undefined ? { state: opts.state } : {}),
      });
    }
  }

  private scheduleReconnect(): void {
    if (this.closed || this.reconnectTimer) return;
    const delay = Math.min(1000 * 2 ** this.attempts++, 15000);
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = undefined;
      this.connect().catch(() => this.scheduleReconnect());
    }, delay);
  }
}
