/** Error thrown for non-2xx API responses. */
export class BlazersError extends Error {
  /** HTTP status code. */
  readonly status: number;
  /** Parsed JSON error body when available, otherwise the raw text. */
  readonly body: unknown;

  constructor(status: number, body: unknown) {
    const detail =
      typeof body === 'object' && body !== null && 'error' in body
        ? String((body as { error: unknown }).error)
        : typeof body === 'string' && body.length > 0
          ? body
          : `HTTP ${status}`;
    super(detail);
    this.name = 'BlazersError';
    this.status = status;
    this.body = body;
  }
}
