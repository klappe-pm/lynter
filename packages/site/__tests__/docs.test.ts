import {labelBeforeAfter, markProseTables} from '../src/docs';

// marked ships as an ES module only, and these tests never render Markdown.
jest.mock('marked', () => ({marked: {}}));

const block = (lang: string, body: string) => `<pre><code class="language-${lang}">${body}</code></pre>`;

describe('labelBeforeAfter', () => {
  it('pairs two adjacent blocks in the same language', () => {
    const out = labelBeforeAfter(`<p>Intro</p>\n${block('markdown', 'a')}\n${block('markdown', 'b')}\n<p>End</p>`);
    expect(out).toBe(
        '<p>Intro</p>\n<div class="before-after">' +
        `<figure><figcaption>Before</figcaption>${block('markdown', 'a')}</figure>` +
        `<figure><figcaption>After</figcaption>${block('markdown', 'b')}</figure></div>\n<p>End</p>`,
    );
  });

  it('leaves a single block alone', () => {
    const html = `<p>Intro</p>\n${block('yaml', 'a')}\n<p>End</p>`;
    expect(labelBeforeAfter(html)).toBe(html);
  });

  it('leaves adjacent blocks in different languages alone', () => {
    const html = `${block('yaml', 'a')}\n${block('markdown', 'b')}`;
    expect(labelBeforeAfter(html)).toBe(html);
  });

  it('does not pair blocks with prose between them', () => {
    const html = `${block('markdown', 'a')}\n<p>Then:</p>\n${block('markdown', 'b')}`;
    expect(labelBeforeAfter(html)).toBe(html);
  });

  it('does not stretch a lone block across prose to a later pair', () => {
    const lone = `${block('markdown', 'x')}\n<p>Then:</p>\n`;
    const out = labelBeforeAfter(`${lone}${block('markdown', 'a')}\n${block('markdown', 'b')}`);
    expect(out.startsWith(lone)).toBe(true);
    expect(out).toContain(`<figcaption>Before</figcaption>${block('markdown', 'a')}</figure>`);
  });

  it('pairs each example on a page separately', () => {
    const pair = `${block('markdown', 'a')}\n${block('markdown', 'b')}`;
    const out = labelBeforeAfter(`${pair}\n<h3>Next rule</h3>\n${pair}`);
    expect(out.match(/class="before-after"/g)).toHaveLength(2);
    expect(out).toContain('</div>\n<h3>Next rule</h3>\n<div class="before-after">');
  });
});

describe('markProseTables', () => {
  const table = (...heads: string[]) =>
    `<table>\n<thead>\n<tr>\n${heads.map((h) => `<th>${h}</th>\n`).join('')}</tr>\n</thead>\n<tbody></tbody></table>`;

  it('marks a table whose last column is prose', () => {
    expect(markProseTables(table('Option', 'Type', 'Default', 'Meaning'))).toContain('<table class="ends-in-prose">');
    expect(markProseTables(table('Command', 'What it does'))).toContain('<table class="ends-in-prose">');
  });

  it('leaves a table that ends in a short column alone', () => {
    const html = table('Feature', 'CLI', 'Obsidian plugin', 'Browser playground');
    expect(markProseTables(html)).toBe(html);
  });

  it('does not mark a table because a later table ends in prose', () => {
    const short = table('Meaning', 'Fixes');
    const out = markProseTables(`${short}\n${table('Key', 'Meaning')}`);
    expect(out.startsWith(short)).toBe(true);
    expect(out.match(/ends-in-prose/g)).toHaveLength(1);
  });
});
