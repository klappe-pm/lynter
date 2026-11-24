import Head from 'next/head';
import Link from 'next/link';
import {useRouter} from 'next/router';
import type {ReactNode} from 'react';
import {Icon} from '@/components/icon';
import {INSTALL_URL, LAPPE_URL, SITE_URL, site} from '@/content';

export {Icon};

/** Absolute URL of a route, without query or fragment and with the trailing slash the export uses. */
export function canonicalUrl(asPath: string): string {
  const path = asPath.split(/[?#]/)[0];
  return SITE_URL + (path.endsWith('/') ? path : `${path}/`);
}

export function Wordmark({icon = true}: {icon?: boolean}) {
  return (
    <Link href="/" className="wordmark" aria-label="Lynter home">
      {icon ? <Icon /> : null}
      <span aria-hidden="true">Lynter</span>
    </Link>
  );
}

/**
 * "A lappe.ai product", the sign-off every lappe.ai product footer ends with.
 * The wordmark keeps its own colours (one span per colour run) and typeface;
 * it is never recoloured to match this site.
 */
function LappeSignoff() {
  const [before, after] = site.footer.signoff.split('lappe.ai');
  return (
    <a className="signoff" href={LAPPE_URL} aria-label={site.footer.signoff}>
      {before}
      <span className="lappe-ai" aria-hidden="true">
        <span>l</span><span>a</span><span>p</span><span>p</span><span>e</span><span>.ai</span>
      </span>
      {after}
    </a>
  );
}

export function Layout({title, description, children}: {title?: string; description?: string; children: ReactNode}) {
  const {asPath, pathname} = useRouter();
  const fullTitle = title ? `${title}: ${site.name}` : site.title;
  const desc = description || site.description;
  // The 404 page is served for every unknown path, so it has no URL of its own.
  const url = pathname === '/404' ? null : canonicalUrl(asPath);
  const card = `${SITE_URL}/brand/og-card.png`;
  return (
    <>
      <Head>
        <title>{fullTitle}</title>
        <meta name="description" content={desc} />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <meta key="theme-light" name="theme-color" content="#F4F9FD" media="(prefers-color-scheme: light)" />
        <meta key="theme-dark" name="theme-color" content="#13304F" media="(prefers-color-scheme: dark)" />
        {url ? <link rel="canonical" href={url} /> : <meta name="robots" content="noindex" />}
        <meta property="og:type" content="website" />
        <meta property="og:site_name" content={site.name} />
        {url ? <meta property="og:url" content={url} /> : null}
        <meta property="og:title" content={fullTitle} />
        <meta property="og:description" content={desc} />
        <meta property="og:image" content={card} />
        <meta property="og:image:width" content="1200" />
        <meta property="og:image:height" content="630" />
        <meta property="og:image:alt" content={site.title} />
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:image" content={card} />
        <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
        <link rel="apple-touch-icon" href="/brand/apple-touch-icon.png" />
        <link rel="preload" href="/fonts/Geist-Variable.woff2" as="font" type="font/woff2" crossOrigin="anonymous" />
      </Head>
      <a className="skip" href="#main">Skip to content</a>
      <header className="site-header">
        <div className="wrap">
          <Wordmark />
          <nav className="site-nav" aria-label="Main">
            {site.nav.map((item) => (
              <Link key={item.href} href={item.href} aria-current={asPath.startsWith(item.href) ? 'page' : undefined}>
                {item.label}
              </Link>
            ))}
            <a className="button primary small" href={INSTALL_URL}>{site.install}</a>
          </nav>
        </div>
      </header>
      <main id="main">{children}</main>
      <footer className="site-footer">
        <div className="wrap">
          <div className="footer-main">
            <div className="footer-brand">
              <Wordmark icon={false} />
              <p>{site.footer.line}</p>
              <Link className="button primary" href={site.footer.action.href}>{site.footer.action.label}</Link>
            </div>
            {site.footer.columns.map((column) => (
              <nav key={column.label} aria-label={column.label}>
                <h2>{column.label}</h2>
                {column.links.map((l) => <Link key={l.href} href={l.href}>{l.label}</Link>)}
              </nav>
            ))}
          </div>
          <div className="footer-base">
            <span>{site.footer.copyright}</span>
            <LappeSignoff />
          </div>
        </div>
      </footer>
    </>
  );
}
