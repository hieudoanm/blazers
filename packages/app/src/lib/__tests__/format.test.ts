import { describe, expect, it } from 'vitest';
import { shortId, timeAgo, tryParseJson } from '../format';

describe('shortId', () => {
  it('truncates long ids', () => {
    expect(shortId('abcdef1234567890')).toBe('abcdef12…');
    expect(shortId('abc')).toBe('abc');
  });
});

describe('timeAgo', () => {
  it('formats relative times', () => {
    const now = Date.parse('2025-01-01T12:00:00Z');
    expect(timeAgo('2025-01-01T11:59:30Z', now)).toBe('30s ago');
    expect(timeAgo('2025-01-01T11:30:00Z', now)).toBe('30m ago');
    expect(timeAgo('2025-01-01T11:00:00Z', now)).toBe('1h ago');
    expect(timeAgo('2024-12-31T12:00:00Z', now)).toBe('1d ago');
    expect(timeAgo('not-a-date', now)).toBe('not-a-date');
  });
});

describe('tryParseJson', () => {
  it('parses objects only', () => {
    expect(tryParseJson('{"a":1}')).toEqual({ a: 1 });
    expect(tryParseJson('[1,2]')).toBeNull();
    expect(tryParseJson('nope')).toBeNull();
    expect(tryParseJson('null')).toBeNull();
  });
});
