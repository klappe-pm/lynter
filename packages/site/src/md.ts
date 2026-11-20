import fs from 'node:fs';
import path from 'node:path';
import type {GetStaticProps} from 'next';
import {marked} from 'marked';
import {CLI_BIN} from '@/samples';

/** Give h2 and h3 headings stable anchor ids. */
export function addHeadingIds(html: string): string {
  return html.replace(/<(h[23])>(.*?)<\/\1>/g, (_m, tag: string, inner: string) => {
    const id = inner.replace(/<[^>]+>/g, '').toLowerCase().replace(/[^a-z0-9 -]/g, '').trim().replace(/\s+/g, '-');
    return `<${tag} id="${id}">${inner}</${tag}>`;
  });
}

/** Render content/<slug>.md at build time. {{CLI}} becomes the current binary name. */
export function mdProps(slug: string): GetStaticProps<{html: string}> {
  return async () => {
    const file = path.join(process.cwd(), 'content', `${slug}.md`);
    const source = fs.readFileSync(file, 'utf8').replace(/\{\{CLI\}\}/g, CLI_BIN);
    const html = addHeadingIds(await marked.parse(source)).replace(/Coming soon/g, '<span class="soon">Coming soon</span>');
    return {props: {html}};
  };
}
