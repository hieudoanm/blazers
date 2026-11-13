import { describe, expect, it } from 'vitest';
import { Blazers, BlazersError, HttpClient } from '../src/index';

describe('HttpClient', () => {
  it('builds API and ws URLs', () => {
    const http = new HttpClient({ url: 'https://example.com/' });
    expect(http.url('/api/health')).toBe('https://example.com/api/health');
    expect(http.wsUrl()).toBe('wss://example.com/ws');
    const http2 = new HttpClient({ url: 'http://localhost:8090' });
    expect(http2.wsUrl()).toBe('ws://localhost:8090/ws');
  });

  it('attaches bearer token and parses JSON', async () => {
    const calls: { url: unknown; init: RequestInit }[] = [];
    const fakeFetch = async (url: unknown, init?: RequestInit) => {
      calls.push({ url, init: init ?? {} });
      return new Response(JSON.stringify({ ok: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    };
    const http = new HttpClient({
      url: 'http://x',
      token: 'tok',
      fetch: fakeFetch as typeof fetch,
    });
    const res = await http.get<{ ok: boolean }>('/api/auth/me');
    expect(res).toEqual({ ok: true });
    const headers = calls[0].init.headers as Record<string, string>;
    expect(headers.Authorization).toBe('Bearer tok');
  });

  it('throws BlazersError on non-2xx', async () => {
    const http = new HttpClient({
      url: 'http://x',
      fetch: (async () =>
        new Response(JSON.stringify({ error: 'nope' }), {
          status: 403,
        })) as typeof fetch,
    });
    const err = await http.get('/api/x').catch((e: unknown) => e);
    expect(err).toBeInstanceOf(BlazersError);
    expect((err as BlazersError).status).toBe(403);
    expect((err as BlazersError).message).toBe('nope');
  });
});

describe('CollectionClient query', () => {
  it('serializes list options', async () => {
    const seen: string[] = [];
    const bz = new Blazers({
      url: 'http://x',
      fetch: (async (url: unknown) => {
        seen.push(String(url));
        return new Response(
          JSON.stringify({
            records: [],
            total: 0,
            page: 1,
            per_page: 50,
            total_pages: 0,
          }),
          { status: 200 }
        );
      }) as typeof fetch,
    });
    await bz.collection('posts').list({
      page: 2,
      perPage: 10,
      search: 'hi',
      filter: 'a="b"',
      sort: '-created_at',
      expand: 'author',
    });
    const url = new URL(seen[0]);
    expect(url.pathname).toBe('/api/collections/posts/records');
    expect(url.searchParams.get('page')).toBe('2');
    expect(url.searchParams.get('per_page')).toBe('10');
    expect(url.searchParams.get('search')).toBe('hi');
    expect(url.searchParams.get('filter')).toBe('a="b"');
    expect(url.searchParams.get('sort')).toBe('-created_at');
    expect(url.searchParams.get('expand')).toBe('author');
  });
});

describe.skipIf(!process.env.BLAZERS_URL)('live server', () => {
  const base = process.env.BLAZERS_URL as string;

  it('register → create collection → record round-trip', async () => {
    const bz = new Blazers({ url: base });
    const email = `sdk-${Date.now()}@test.dev`;
    const user = await bz.auth.register({
      email,
      password: 'password123',
    });
    const session = await bz.auth.login({
      email,
      password: 'password123',
    });
    expect(bz.http.authToken).toBeTruthy();

    // No bootstrap superuser: grant self global admin (same as e2e tests).
    await bz.http.post('/api/permissions', {
      user_id: session.user.id ?? user.id,
      collection: '*',
      role: 'admin',
    });

    const name = `sdk_${Date.now()}`;
    await bz.collections.create(name);
    const rec = await bz.collection(name).create({ title: 'hello' });
    expect(rec.data.title).toBe('hello');

    const got = await bz.collection(name).get(rec.id);
    expect(got.id).toBe(rec.id);

    const page = await bz.collection(name).list({ filter: 'title="hello"' });
    expect(page.total).toBe(1);

    await bz.collection(name).delete(rec.id);
    await bz.collections.delete(name);
  });
});
