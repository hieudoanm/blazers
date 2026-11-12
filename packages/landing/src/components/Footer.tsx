import Link from 'next/link';
import type { FC } from 'react';

const GITHUB = 'https://github.com/hieudoanm/blazers';

const linkClass =
  'text-text-muted hover:text-text no-underline transition-colors';

const Footer: FC = () => (
  <footer className='border-border text-text-muted border-t py-10 text-sm'>
    <div className='mx-auto flex w-full max-w-[72rem] flex-wrap items-center gap-x-8 gap-y-4 px-5'>
      <span>© {new Date().getFullYear()} blazers</span>
      <span>Blazing fast Back-end as a Service · Rust · SQLite · MIT</span>
      <nav aria-label='Footer' className='ml-auto flex items-center gap-5'>
        <Link href={GITHUB} target='_blank' rel='noreferrer' className={linkClass}>
          GitHub
        </Link>
        <Link
          href={`${GITHUB}#readme`}
          target='_blank'
          rel='noreferrer'
          className={linkClass}>
          README
        </Link>
      </nav>
    </div>
  </footer>
);

export default Footer;
