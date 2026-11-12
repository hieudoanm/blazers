import type { Metadata } from 'next';
import Image from 'next/image';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { DOC_SECTIONS, getDocPages, type DocPage } from '@/lib/docs';
import Footer from '@/components/Footer';
import type { JSX } from 'react';

// next/image with unoptimized does not apply basePath — mirror next.config.ts.
const LOGO = process.env.NODE_ENV === 'production' ? '/blazers/logo.svg' : '/logo.svg';

export const generateStaticParams = () =>
  getDocPages().map((p) => ({
    slug: p.slug === '' ? [] : p.slug.split('/'),
  }));

const findDoc = (slug: string[]): DocPage | undefined => {
  const key = slug.join('/');
  return getDocPages().find((p) => p.slug === key);
};

interface DocsPageProps {
  params: Promise<{ slug?: string[] }>;
}

export const generateMetadata = async ({ params }: DocsPageProps): Promise<Metadata> => {
  const { slug = [] } = await params;
  const doc = findDoc(slug);
  return { title: doc ? doc.title : 'Docs' };
};

const DocsPage = async ({ params }: DocsPageProps): Promise<JSX.Element> => {
  const { slug = [] } = await params;
  const pages = getDocPages();
  const doc = findDoc(slug);
  if (!doc) notFound();
  const overview = pages.find((p) => p.slug === '');

  return (
    <>
      <header className='border-border bg-bg/90 sticky top-0 z-50 border-b backdrop-blur'>
        <div className='mx-auto flex h-16 w-full max-w-[72rem] items-center gap-6 px-5'>
          <Link
            href='/'
            className='text-text flex items-center gap-2 text-lg font-bold no-underline'>
            <Image src={LOGO} alt='' width={28} height={28} className='rounded-md' />
            <span>blazers</span>
          </Link>
          <nav className='ml-auto flex items-center gap-5 text-[0.9375rem]'>
            <Link href='/' className='text-text-muted hover:text-text no-underline'>
              Home
            </Link>
            <Link href='/docs/' className='text-accent font-semibold no-underline'>
              Docs
            </Link>
            <Link
              href='https://github.com/hieudoanm/blazers'
              target='_blank'
              rel='noreferrer'
              className='btn btn-primary btn-sm'>
              GitHub
            </Link>
          </nav>
        </div>
      </header>

      <div className='mx-auto w-full max-w-[72rem] px-5 py-10'>
        <div className='docs-layout grid grid-cols-[16rem_1fr] gap-10'>
          <aside className='docs-sidebar'>
            <Link href='/docs/' className='text-text text-sm font-bold no-underline'>
              Documentation
            </Link>
            {overview && (
              <Link
                href='/docs/'
                className={`docs-nav-link mt-2 block ${doc.slug === '' ? 'active' : ''}`}>
                {overview.title}
              </Link>
            )}
            {DOC_SECTIONS.map((s) => {
              const items = pages.filter((p) => p.section === s.id);
              if (items.length === 0) return null;
              return (
                <div key={s.id}>
                  <div className='docs-nav-section'>{s.label}</div>
                  {items.map((p) => (
                    <Link
                      key={p.slug}
                      href={`/docs/${p.slug}/`}
                      className={`docs-nav-link ${p.slug === doc.slug ? 'active' : ''}`}>
                      {p.title}
                    </Link>
                  ))}
                </div>
              );
            })}
          </aside>

          <article>
            <div className='docs-content' dangerouslySetInnerHTML={{ __html: doc.html }} />
          </article>
        </div>
      </div>

      <Footer />
    </>
  );
};

export default DocsPage;
