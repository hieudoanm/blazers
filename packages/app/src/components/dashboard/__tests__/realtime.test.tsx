import type { Blazers } from '@blazers/sdk';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import RealtimeTab from '../RealtimeTab';
import { makeBlazersStub, type BlazersStub } from './stub';

const holder = vi.hoisted(() => ({ bz: null as unknown as Blazers }));

vi.mock('@/lib/blazers', () => ({
  getBlazers: () => holder.bz,
  isAuthed: () => true,
  saveSession: vi.fn(),
  clearSession: vi.fn(),
}));

let stub: BlazersStub;
let messageHandler: ((data: unknown) => void) | undefined;

beforeEach(() => {
  stub = makeBlazersStub();
  holder.bz = stub.raw as unknown as Blazers;
  messageHandler = undefined;
  stub.raw.realtime.onMessage.mockImplementation((h: (d: unknown) => void) => {
    messageHandler = h;
    return () => {};
  });
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

describe('RealtimeTab', () => {
  beforeEach(() => {
    stub.raw.collections.list.mockResolvedValue([COL]);
  });

  it('connects and subscribes to a collection', async () => {
    render(<RealtimeTab />);
    expect(await screen.findByText('connected')).toBeTruthy();

    fireEvent.click(await screen.findByRole('button', { name: 'posts' }));
    expect(stub.raw.realtime.subscribe).toHaveBeenCalledWith(
      'posts',
      expect.anything()
    );
  });

  it('passes the subscription filter', async () => {
    render(<RealtimeTab />);
    fireEvent.change(await screen.findByPlaceholderText('status="live"'), {
      target: { value: 'a > 1' },
    });
    fireEvent.click(await screen.findByRole('button', { name: 'posts' }));
    expect(stub.raw.realtime.subscribe).toHaveBeenCalledWith('posts', {
      filter: 'a > 1',
    });
  });

  it('appends incoming record events to the log', async () => {
    render(<RealtimeTab />);
    await screen.findByText('connected');
    expect(messageHandler).toBeTruthy();

    act(() => {
      messageHandler?.({
        type: 'record',
        action: 'create',
        collection: 'posts',
        record: { id: 'r1', data: { title: 'Live!' } },
      });
    });

    expect(await screen.findByText(/Live!/)).toBeTruthy();
  });

  it('joins a presence channel', async () => {
    render(<RealtimeTab />);
    fireEvent.click(
      await screen.findByRole('button', { name: 'Join channel' })
    );
    expect(stub.raw.realtime.presenceJoin).toHaveBeenCalledWith(
      'demo:room',
      expect.objectContaining({ state: { demo: true } })
    );
  });
});
