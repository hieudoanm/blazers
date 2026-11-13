import { vi } from 'vitest';

/** Raw (untyped) Blazers stub — tests configure the vi.fn()s and cast to
 * `Blazers` when handing it to the mocked `getBlazers()`. */
export function makeBlazersStub() {
  const recordClient = {
    list: vi.fn(),
    get: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    delete: vi.fn(),
  };

  const raw = {
    http: {
      post: vi.fn(),
      baseUrl: 'http://test',
      authToken: 'tok',
    },
    auth: {},
    collections: {
      list: vi.fn().mockResolvedValue([]),
      get: vi.fn(),
      create: vi.fn().mockResolvedValue({ name: 'new' }),
      update: vi.fn().mockResolvedValue({}),
      delete: vi.fn().mockResolvedValue(undefined),
    },
    collection: vi.fn(() => recordClient),
    files: {
      listBuckets: vi.fn().mockResolvedValue([]),
      createBucket: vi.fn().mockResolvedValue({ name: 'b' }),
      getBucket: vi.fn(),
      deleteBucket: vi.fn().mockResolvedValue(undefined),
      listFiles: vi.fn().mockResolvedValue({ files: [], total: 0 }),
      upload: vi.fn().mockResolvedValue({ id: 'f1' }),
      downloadUrl: vi.fn(() => 'http://test/file'),
      download: vi.fn(),
      fileToken: vi.fn(),
      deleteFile: vi.fn().mockResolvedValue(undefined),
    },
    realtime: {
      connect: vi.fn().mockResolvedValue(undefined),
      subscribe: vi.fn().mockResolvedValue(undefined),
      unsubscribe: vi.fn().mockResolvedValue(undefined),
      presenceJoin: vi.fn().mockResolvedValue(undefined),
      presenceUpdate: vi.fn().mockResolvedValue(undefined),
      presenceLeave: vi.fn().mockResolvedValue(undefined),
      onMessage: vi.fn<(handler: (data: unknown) => void) => () => void>(
        () => () => {}
      ),
      close: vi.fn(),
      connected: true,
    },
    batch: vi.fn(),
    health: vi.fn(),
  };

  return { raw, recordClient };
}

export type BlazersStub = ReturnType<typeof makeBlazersStub>;
