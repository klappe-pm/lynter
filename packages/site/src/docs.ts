/**
 * The docs wiki: its sidebar structure and the build-time loader that turns
 * content/docs/<slug>.md into a page. Runs only in getStaticProps.
 */
import fs from 'node:fs';
import path from 'node:path';
import {marked} from 'marked';

/** Sidebar groups, in order. Each slug is a file in content/docs/, apart from `reference`. */
export const docsGroups: {title: string; slugs: string[]}[] = [
  {title: 'Start here', slugs: ['overview', 'architecture']},
  {title: 'Configuration', slugs: ['configuration', 'reference', 'scopes-and-profiles', 'note-types']},
  {title: 'Rules', slugs: ['rules-overview', 'rules-frontmatter', 'rules-content', 'rules-spacing-and-structure', 'code-linting']},
  {title: 'Where it runs', slugs: ['cli', 'obsidian-plugin', 'wasm-api']},
  {title: 'Project', slugs: ['importing-settings', 'testing', 'releases']},
];

/** The page served at /docs/ itself. */
export const DOCS_INDEX = 'overview';

/** Rendered from the repository's docs/lynter-yaml.md, so the key reference never drifts. */
const REFERENCE = {
  slug: 'reference',
  title: 'lynter.yaml reference',
  lede: 'Every key in lynter.yaml: defaults, profiles, note types, code checks, adapters, and the command line flags that read them.',
};

export interface DocNavItem {slug: string; title: string; href: string}
export interface DocNavGroup {title: string; items: DocNavItem[]}
export interface DocHeading {id: string; text: string}
export interface DocPage {
  slug: string;
  title: string;
  lede: string;
  html: string;
  headings: DocHeading[];
  nav: DocNavGroup[];
  prev: DocNavItem | null;
  next: DocNavItem | null;
  /** Repository path of the source file, for the edit link. */
  source: string;
}

export const docSlugs = (): string[] => docsGroups.flatMap((g) => g.slugs);

export const docHref = (slug: string): string => (slug === DOCS_INDEX ? '/docs/' : `/docs/${slug}/`);

const contentDir = () => path.join(process.cwd(), 'content', 'docs');

/** Split `---` frontmatter (title and lede only) from a Markdown body. */
export function splitFrontmatter(source: string): {meta: Record<string, string>; body: string} {
  const match = source.match(/^---\n([\s\S]*?)\n---\n?/);
  if (!match) return {meta: {}, body: source};
  const meta: Record<string, string> = {};
  for (const line of match[1].split('\n')) {
    const at = line.indexOf(':');
    if (at > 0) meta[line.slice(0, at).trim()] = line.slice(at + 1).trim().replace(/^["'](.*)["']$/, '$1');
  }
  return {meta, body: source.slice(match[0].length)};
}

function read(slug: string): {title: string; lede: string; body: string; source: string} {
  if (slug === REFERENCE.slug) {
    const file = path.join(process.cwd(), '..', '..', 'docs', 'lynter-yaml.md');
    return {...REFERENCE, body: fs.readFileSync(file, 'utf8').replace(/^# .*\n/, ''), source: 'docs/lynter-yaml.md'};
  }
  const {meta, body} = splitFrontmatter(fs.readFileSync(path.join(contentDir(), `${slug}.md`), 'utf8'));
  if (!meta.title || !meta.lede) throw new Error(`content/docs/${slug}.md needs title and lede frontmatter`);
  return {title: meta.title, lede: meta.lede, body, source: `packages/site/content/docs/${slug}.md`};
}

const unescape = (s: string) => s.replace(/&gt;/g, '>').replace(/&lt;/g, '<').replace(/&amp;/g, '&').replace(/&quot;/g, '"').replace(/&#39;/g, "'");
const escape = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

/** A ```flow block: each line is a chain `a -> b -> c`, drawn as a row of boxes. */
export function renderFlows(html: string): string {
  return html.replace(/<pre><code class="language-flow">([\s\S]*?)<\/code><\/pre>/g, (_m, raw: string) => {
    const rows = unescape(raw).split('\n').map((l) => l.trim()).filter(Boolean);
    const body = rows.map((row) => {
      const nodes = row.split(/\s*->\s*/).filter(Boolean);
      const cells = nodes.map((n) => `<li>${escape(n)}</li>`).join('');
      return `<ol class="flow-row" aria-label="${escape(nodes.join(', then '))}">${cells}</ol>`;
    }).join('');
    return `<div class="flow">${body}</div>`;
  });
}

/** Give h2 and h3 stable ids and collect the h2s for the on-page contents list. */
export function anchorHeadings(html: string): {html: string; headings: DocHeading[]} {
  const headings: DocHeading[] = [];
  const seen = new Map<string, number>();
  const out = html.replace(/<(h[23])>([\s\S]*?)<\/\1>/g, (_m, tag: string, inner: string) => {
    const text = unescape(inner.replace(/<[^>]+>/g, '')).trim();
    const base = text.toLowerCase().replace(/[^a-z0-9 -]/g, '').trim().replace(/\s+/g, '-') || 'section';
    const n = seen.get(base) ?? 0;
    seen.set(base, n + 1);
    const id = n ? `${base}-${n + 1}` : base;
    if (tag === 'h2') headings.push({id, text});
    return `<${tag} id="${id}"><a class="anchor" href="#${id}">${inner}</a></${tag}>`;
  });
  return {html: out, headings};
}

/**
 * On the rule pages, two code blocks in a row in the same language are an
 * example before and after the rule runs. Label them and set them side by side.
 */
export function labelBeforeAfter(html: string): string {
  // The body may not cross a closing tag, or a lone block would pair with a later example.
  const body = '(?:(?!</code>)[\\s\\S])*';
  return html.replace(
      new RegExp(`(<pre><code class="language-([a-z]+)">${body}</code></pre>)\\s*(<pre><code class="language-\\2">${body}</code></pre>)`, 'g'),
      (_m, before: string, _lang: string, after: string) =>
        `<div class="before-after"><figure><figcaption>Before</figcaption>${before}</figure><figure><figcaption>After</figcaption>${after}</figure></div>`,
  );
}

/**
 * Tables that end in a prose column ("Meaning", "What it does") get a class, so
 * that column keeps a readable width on a phone. Tables that end in a short
 * column such as Yes or No are left to size themselves.
 */
export function markProseTables(html: string): string {
  const cell = '<th[^>]*>(?:(?!</th>)[\\s\\S])*</th>\\s*';
  return html.replace(
      new RegExp(`<table>(?=\\s*<thead>\\s*<tr>\\s*(?:${cell})*<th[^>]*>(?:Meaning|What it does)</th>\\s*</tr>)`, 'g'),
      '<table class="ends-in-prose">',
  );
}

/** "Sources: ..." paragraphs become a quiet footnote under their section. */
const markSources = (html: string) => html.replace(/<p>Sources: /g, '<p class="sources"><span>Sources</span> ');

export async function loadDoc(slug: string): Promise<DocPage> {
  const all = docSlugs().map((s) => ({slug: s, title: read(s).title, href: docHref(s)}));
  const at = all.findIndex((d) => d.slug === slug);
  if (at < 0) throw new Error(`unknown docs page: ${slug}`);
  const doc = read(slug);
  const rendered = markProseTables(renderFlows(await marked.parse(doc.body)));
  const {html, headings} = anchorHeadings(slug.startsWith('rules-') ? labelBeforeAfter(rendered) : rendered);
  return {
    slug,
    title: doc.title,
    lede: doc.lede,
    html: markSources(html),
    headings,
    nav: docsGroups.map((g) => ({title: g.title, items: g.slugs.map((s) => all.find((d) => d.slug === s)!)})),
    prev: all[at - 1] ?? null,
    next: all[at + 1] ?? null,
    source: doc.source,
  };
}
