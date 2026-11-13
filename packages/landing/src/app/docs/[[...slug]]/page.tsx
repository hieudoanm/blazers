import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { DOC_SECTIONS, getDocPages, type DocPage } from '@/lib/docs';
import type { JSX } from 'react';

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

export const generateMetadata = async ({
  params,
}: DocsPageProps): Promise<Metadata> => {
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
      <div className="mx-auto flex h-[calc(100dvh-4rem)] w-full max-w-6xl flex-col px-5 py-10">
        <div className="grid min-h-0 flex-1 grid-cols-1 gap-10 md:grid-cols-[16rem_1fr]">
          <aside className="hidden min-h-0 overflow-y-auto md:block">
            <Link
              href="/docs/"
              className="text-sm font-bold text-gray-900 no-underline">
              Documentation
            </Link>
            {overview && (
              <Link
                href="/docs/"
                className={`mt-2 block rounded-lg px-3 py-1.5 text-sm no-underline ${doc.slug === '' ? 'bg-primary/10 text-primary font-semibold' : 'text-gray-500 hover:bg-gray-50 hover:text-gray-900'}`}>
                {overview.title}
              </Link>
            )}
            {DOC_SECTIONS.map((s) => {
              const items = pages.filter((p) => p.section === s.id);
              if (items.length === 0) return null;
              return (
                <div key={s.id}>
                  <div className="mt-5 mb-1.5 text-xs font-semibold tracking-wider text-gray-400 uppercase">
                    {s.label}
                  </div>
                  {items.map((p) => (
                    <Link
                      key={p.slug}
                      href={`/docs/${p.slug}/`}
                      className={`block rounded-lg px-3 py-1.5 text-sm no-underline ${p.slug === doc.slug ? 'bg-primary/10 text-primary font-semibold' : 'text-gray-500 hover:bg-gray-50 hover:text-gray-900'}`}>
                      {p.title}
                    </Link>
                  ))}
                </div>
              );
            })}
          </aside>

          <article className="min-h-0 overflow-y-auto">
            <div className="grid grid-cols-1 gap-8 pb-4 lg:grid-cols-[minmax(0,1fr)_11rem] lg:gap-10">
              {doc.toc.length > 0 && (
                <nav
                  aria-label="On this page"
                  className="self-start lg:sticky lg:top-0 lg:order-2 lg:max-h-[calc(100dvh-7rem)] lg:overflow-y-auto">
                  <p className="text-xs font-semibold tracking-wider text-gray-400 uppercase">
                    On this page
                  </p>
                  <ul className="border-base-content/10 mt-2 space-y-1">
                    {doc.toc.map((h) => (
                      <li key={h.id} className={h.level > 2 ? 'pl-4' : ''}>
                        <a
                          href={`#${h.id}`}
                          className="text-base-content/60 hover:text-primary block py-0.5 text-sm transition-colors">
                          {h.text}
                        </a>
                      </li>
                    ))}
                  </ul>
                </nav>
              )}
              <div
                className="prose max-w-none lg:order-1"
                dangerouslySetInnerHTML={{ __html: doc.html }}
              />
            </div>
          </article>
        </div>
      </div>
    </>
  );
};

export default DocsPage;
