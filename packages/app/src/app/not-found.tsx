import Link from 'next/link';

export default function NotFound() {
  return (
    <main className="mx-auto flex min-h-screen max-w-md flex-col items-center justify-center gap-4 px-6 text-center">
      <p className="text-primary text-6xl font-bold">404</p>
      <h1 className="text-2xl font-semibold">Page not found</h1>
      <p className="text-base-content/60">
        The page you are looking for does not exist.
      </p>
      <Link href="/" className="btn btn-primary">
        Back to demo
      </Link>
    </main>
  );
}
