import type { Blazers } from '@blazers/sdk';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import RecordsTab from '../RecordsTab';
import { makeBlazersStub, type BlazersStub } from './stub';

const holder = vi.hoisted(() => ({ bz: null as unknown as Blazers }));

vi.mock('@/lib/blazers', () => ({
  getBlazers: () => holder.bz,
  isAuthed: () => true,
  saveSession: vi.fn(),
  clearSession: vi.fn(),
}));

let stub: BlazersStub;

beforeEach(() => {
  stub = makeBlazersStub();
  holder.bz = stub.raw as unknown as Blazers;
});

const COL = {
  name: 'posts',
  schema: '{}',
  list_rule: '',
  view_rule: '',
  create_rule: '',
  update_rule: '',
  delete_rule: '',
  type: 'records',
  created_at: '2025-01-01T00:00:00Z',
  updated_at: '2025-01-01T00:00:00Z',
};

const PAGE = {
  records: [
    {
      id: 'r1',
      data: { title: 'Hello world' },
      created_at: '2025-01-01T00:00:00Z',
      updated_at: '2025-01-01T00:00:00Z',
    },
  ],
  total: 1,
  page: 1,
  per_page: 50,
  total_pages: 1,
};

async function selectPosts() {
  fireEvent.click(await screen.findByRole('button', { name: 'posts' }));
  await screen.findByText(/Hello world/);
}

describe('RecordsTab', () => {
  beforeEach(() => {
    stub.raw.collections.list.mockResolvedValue([COL]);
    stub.recordClient.list.mockResolvedValue(PAGE);
  });

  it('lists records of the selected collection', async () => {
    render(<RecordsTab />);
    await selectPosts();
    expect(stub.raw.collection).toHaveBeenCalledWith('posts');
    expect(stub.recordClient.list).toHaveBeenCalled();
    expect(screen.getByText('1 record')).toBeTruthy();
  });

  it('passes filter and sort to the list query', async () => {
    render(<RecordsTab />);
    await selectPosts();

    fireEvent.change(screen.getByPlaceholderText('status="live"'), {
      target: { value: 'a="b"' },
    });
    fireEvent.change(screen.getByPlaceholderText('-created_at'), {
      target: { value: '-updated_at' },
    });

    await waitFor(() =>
      expect(stub.recordClient.list).toHaveBeenCalledWith(
        expect.objectContaining({ filter: 'a="b"', sort: '-updated_at' })
      )
    );
  });

  it('creates a record from JSON and reloads', async () => {
    render(<RecordsTab />);
    await selectPosts();
    const callsBefore = stub.recordClient.list.mock.calls.length;

    fireEvent.change(screen.getByPlaceholderText('{"title": "Hello"}'), {
      target: { value: '{"title":"Yo"}' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Create' }));

    await waitFor(() =>
      expect(stub.recordClient.create).toHaveBeenCalledWith({ title: 'Yo' })
    );
    await waitFor(() =>
      expect(stub.recordClient.list.mock.calls.length).toBeGreaterThan(
        callsBefore
      )
    );
  });

  it('rejects invalid JSON', async () => {
    render(<RecordsTab />);
    await selectPosts();

    fireEvent.change(screen.getByPlaceholderText('{"title": "Hello"}'), {
      target: { value: 'not json' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Create' }));

    expect(
      await screen.findByText('Record data must be a JSON object')
    ).toBeTruthy();
    expect(stub.recordClient.create).not.toHaveBeenCalled();
  });
});
