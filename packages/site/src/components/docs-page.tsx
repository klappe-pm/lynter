import Link from 'next/link';
import {useEffect, useState} from 'react';
import {Layout} from '@/components/layout';
import {docsCopy, REPO_URL} from '@/content';
import type {DocPage} from '@/docs';

/** One wiki page: section sidebar, the article, and a contents list for the page. */
export function DocsPage({doc}: {doc: DocPage}) {
  // Open for the desktop sidebar; on a narrow screen it starts as a closed menu above the article.
  const [navOpen, setNavOpen] = useState(true);
  useEffect(() => setNavOpen(!window.matchMedia('(max-width: 960px)').matches), [doc.slug]);
  return (
    <Layout title={doc.title} description={doc.lede}>
      <div className="wrap docs">
        <nav className="docs-nav" aria-label={docsCopy.navLabel}>
          <details open={navOpen} onToggle={(e) => setNavOpen(e.currentTarget.open)}>
            <summary>{docsCopy.navLabel}</summary>
            {doc.nav.map((group) => (
              <div key={group.title} className="docs-group">
                <h2>{group.title}</h2>
                <ul>
                  {group.items.map((item) => (
                    <li key={item.slug}>
                      <Link href={item.href} aria-current={item.slug === doc.slug ? 'page' : undefined}>{item.title}</Link>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </details>
        </nav>

        <article className="docs-article">
          <header className="page-head">
            <h1>{doc.title}</h1>
            <p className="lede">{doc.lede}</p>
          </header>
          <div className="prose" dangerouslySetInnerHTML={{__html: doc.html}} />
          <footer className="docs-foot">
            <div className="docs-pager">
              {doc.prev ? <Link className="prev" href={doc.prev.href}><span>{docsCopy.prev}</span>{doc.prev.title}</Link> : <span />}
              {doc.next ? <Link className="next" href={doc.next.href}><span>{docsCopy.next}</span>{doc.next.title}</Link> : <span />}
            </div>
            <a href={`${REPO_URL}/blob/master/${doc.source}`}>{docsCopy.edit}</a>
          </footer>
        </article>

        {doc.headings.length > 1 ? (
          <nav className="docs-toc" aria-label={docsCopy.tocLabel}>
            <h2>{docsCopy.tocLabel}</h2>
            <ul>
              {doc.headings.map((h) => <li key={h.id}><a href={`#${h.id}`}>{h.text}</a></li>)}
            </ul>
          </nav>
        ) : null}
      </div>
    </Layout>
  );
}
