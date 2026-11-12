import Link from 'next/link';
import type { FC } from 'react';

const GITHUB = 'https://github.com/hieudoanm/blazers';

const CTA: FC = () => (
  <section className='bg-bg-alt py-20'>
    <div className='mx-auto w-full max-w-[72rem] px-5 text-center'>
      <h2>Give your next side project a real backend.</h2>
      <p className='text-text-muted mt-3 text-[1.0625rem]'>
        Self-host it, embed it, or point your AI agent at it over MCP.
      </p>
      <div className='mt-8 flex flex-wrap justify-center gap-3'>
        <Link
          href={GITHUB}
          target='_blank'
          rel='noreferrer'
          className='btn btn-primary btn-lg'>
          Star on GitHub
        </Link>
        <Link href='/docs/' className='btn btn-ghost btn-lg'>
          Read the docs
        </Link>
      </div>
    </div>
  </section>
);

export default CTA;
