'use client';

import type { FC } from 'react';

const Error: FC<{
  error: Error & { digest?: string };
  reset: () => void;
}> = ({ error, reset }) => (
  <div className="flex min-h-screen flex-col items-center justify-center gap-4 px-5 text-center">
    <h1 className="text-3xl font-bold">Something went wrong</h1>
    <p className="max-w-md text-gray-500">{error.message}</p>
    <button type="button" onClick={reset} className="btn btn-primary">
      Try again
    </button>
  </div>
);

export default Error;
