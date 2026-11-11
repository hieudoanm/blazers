import { describe, expect, it } from 'vitest';
import { DOC_SECTIONS, getDocPages, sectionLabel } from '@/lib/docs';

describe('getDocPages', () => {
  it('returns the docs overview and section pages', () => {
    const pages = getDocPages();
    expect(pages.length).toBeGreaterThan(0);
    expect(pages.some((p) => p.slug === '')).toBe(true);
    for (const page of pages) {
      expect(page.title.length).toBeGreaterThan(0);
      expect(page.html.length).toBeGreaterThan(0);
    }
  });
});

describe('sectionLabel', () => {
  it('returns the label for a known section and the id otherwise', () => {
    expect(sectionLabel(DOC_SECTIONS[0].id)).toBe(DOC_SECTIONS[0].label);
    expect(sectionLabel('unknown')).toBe('unknown');
  });
});
