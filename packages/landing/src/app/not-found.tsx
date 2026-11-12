import type { NextPage } from 'next';
import Link from 'next/link';

const NotFound: NextPage = () => (
  <div className="flex min-h-screen flex-col items-center justify-center gap-4 px-5 text-center">
    <h1 className="text-4xl font-bold">404</h1>
    <p className="text-gray-500">This page does not exist.</p>
    <Link href="/" className="btn btn-primary">
      Back to blazers
    </Link>
  </div>
);

export default NotFound;
