import type { Blazers } from '@blazers/sdk';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import CollectionsTab from '../CollectionsTab';
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

describe('CollectionsTab', () => {
  it('lists collections', async () => {
    stub.raw.collections.list.mockResolvedValue([COL]);
    render(<CollectionsTab />);
    expect(await screen.findByText('posts')).toBeTruthy();
    expect(screen.getByText('records')).toBeTruthy();
  });

  it('creates a collection', async () => {
    stub.raw.collections.list.mockResolvedValue([]);
    render(<CollectionsTab />);
    fireEvent.change(await screen.findByPlaceholderText('posts'), {
      target: { value: 'comments' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Create' }));
    await waitFor(() =>
      expect(stub.raw.collections.create).toHaveBeenCalledWith('comments', '{}')
    );
  });

  it('edits rule expressions', async () => {
    stub.raw.collections.list.mockResolvedValue([
      { ...COL, list_rule: 'status="live"' },
    ]);
    render(<CollectionsTab />);
    fireEvent.click(await screen.findByRole('button', { name: 'edit' }));

    const ruleInput = await screen.findByDisplayValue('status="live"');
    fireEvent.change(ruleInput, {
      target: { value: 'owner = @request.auth.id' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));

    await waitFor(() =>
      expect(stub.raw.collections.update).toHaveBeenCalledWith(
        'posts',
        expect.objectContaining({ list_rule: 'owner = @request.auth.id' })
      )
    );
  });
});
