import type { Blazers } from '@blazers/sdk';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import FilesTab from '../FilesTab';
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

const BUCKET = {
  name: 'avatars',
  is_public: true,
  created_at: '2025-01-01T00:00:00Z',
  updated_at: '2025-01-01T00:00:00Z',
};

const FILE = {
  id: 'f1',
  bucket: 'avatars',
  filename: 'me.png',
  mime_type: 'image/png',
  size: 2048,
  created_at: '2025-01-01T00:00:00Z',
  updated_at: '2025-01-01T00:00:00Z',
};

describe('FilesTab', () => {
  it('lists buckets and selects one to load files', async () => {
    stub.raw.files.listBuckets.mockResolvedValue([BUCKET]);
    stub.raw.files.listFiles.mockResolvedValue({
      files: [FILE],
      total: 1,
      page: 1,
      per_page: 50,
      total_pages: 1,
    });
    render(<FilesTab />);

    const btn = await screen.findByRole('button', { name: /avatars/ });
    fireEvent.click(btn);

    expect(await screen.findByText('me.png')).toBeTruthy();
    expect(stub.raw.files.listFiles).toHaveBeenCalledWith('avatars');
    expect(screen.getByText('public')).toBeTruthy();
  });

  it('creates a bucket', async () => {
    stub.raw.files.listBuckets.mockResolvedValue([]);
    render(<FilesTab />);

    fireEvent.change(await screen.findByPlaceholderText('avatars'), {
      target: { value: 'docs' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Create' }));

    await waitFor(() =>
      expect(stub.raw.files.createBucket).toHaveBeenCalledWith('docs', false)
    );
  });

  it('uploads a file to the selected bucket', async () => {
    stub.raw.files.listBuckets.mockResolvedValue([BUCKET]);
    render(<FilesTab />);
    fireEvent.click(await screen.findByRole('button', { name: /avatars/ }));
    await screen.findByText('Upload to avatars');

    const input = document.querySelector(
      'input[type="file"]'
    ) as HTMLInputElement;
    const file = new File(['data'], 'pic.png', { type: 'image/png' });
    fireEvent.change(input, { target: { files: [file] } });

    await waitFor(() =>
      expect(stub.raw.files.upload).toHaveBeenCalledWith('avatars', file)
    );
  });
});
