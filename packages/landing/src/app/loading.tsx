import type { FC } from 'react';

const Loading: FC = () => (
  <div className="flex min-h-screen items-center justify-center">
    <span className="loading loading-spinner loading-lg text-primary" />
  </div>
);

export default Loading;
