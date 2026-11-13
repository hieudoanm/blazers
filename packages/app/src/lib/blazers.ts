'use client';

import { Blazers } from '@blazers/sdk';

const TOKEN_KEY = 'blazers_demo_token';

let client: Blazers | undefined;

/** Shared SDK client — token persisted in localStorage across reloads. */
export function getBlazers(): Blazers {
  if (!client) {
    client = new Blazers({
      url: process.env.NEXT_PUBLIC_BLAZERS_URL ?? 'http://localhost:8090',
    });
    const token =
      typeof window !== 'undefined' ? localStorage.getItem(TOKEN_KEY) : null;
    if (token) client.http.setToken(token);
  }
  return client;
}

export function saveSession(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
  getBlazers().http.setToken(token);
}

export function clearSession(): void {
  localStorage.removeItem(TOKEN_KEY);
  getBlazers().http.setToken(undefined);
}

export function isAuthed(): boolean {
  return !!getBlazers().http.authToken;
}
