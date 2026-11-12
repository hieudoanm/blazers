import type { FC } from 'react';
import type { IconType } from 'react-icons';

export interface Feature {
  title: string;
  body: string;
  icon: IconType;
}

interface FeaturesProps {
  title: string;
  description: string;
  features: Feature[];
}

const Features: FC<FeaturesProps> = ({ title, description, features }) => (
  <section id='features' className='py-20'>
    <div className='mx-auto w-full max-w-6xl px-5'>
      <div className='mx-auto mb-12 max-w-2xl text-center'>
        <h2 className='m-0 text-3xl leading-tight font-bold tracking-normal text-balance text-base-content'>
          {title}
        </h2>
        <p className='text-base-content/60 mt-3 text-lg'>{description}</p>
      </div>
      <div className='grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3'>
        {features.map((f) => (
          <article
            key={f.title}
            className='card border border-base-300 bg-base-100 p-6'>
            <div className='mb-3.5 flex h-11 w-11 items-center justify-center rounded-lg bg-accent/10 text-accent'>
              <f.icon className='h-6 w-6' />
            </div>
            <h3 className='m-0 mb-2 text-lg leading-tight text-balance text-base-content'>
              {f.title}
            </h3>
            <p className='text-base-content/60 text-sm'>{f.body}</p>
          </article>
        ))}
      </div>
    </div>
  </section>
);

export default Features;
