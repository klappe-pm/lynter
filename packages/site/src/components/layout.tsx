import Head from 'next/head';
import Link from 'next/link';
import {useRouter} from 'next/router';
import type {ReactNode} from 'react';
import {INSTALL_URL, LAB_URL, site} from '@/content';

/** Merge Y icon. Colors come from CSS variables so it follows the theme. */
export function Icon() {
  return (
    <svg viewBox="0 0 48 48" aria-hidden="true" focusable="false" fill="none" strokeLinecap="round" strokeWidth="6">
      <path className="arm" d="M12 8 L24 26" />
      <path className="arm" d="M36 8 L24 26" />
      <path className="stem" d="M24 26 V41" />
    </svg>
  );
}

export function Wordmark() {
  return (
    <Link href="/" className="wordmark" aria-label="Lynter home">
      <Icon />
      <span aria-hidden="true">Lynter</span>
    </Link>
  );
}

export function Layout({title, description, children}: {title?: string; description?: string; children: ReactNode}) {
  const {asPath} = useRouter();
  const fullTitle = title ? `${title}: ${site.name}` : site.title;
  const desc = description || site.description;
  return (
    <>
      <Head>
        <title>{fullTitle}</title>
        <meta name="description" content={desc} />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <meta name="theme-color" content="#F4F9FD" media="(prefers-color-scheme: light)" />
        <meta name="theme-color" content="#13304F" media="(prefers-color-scheme: dark)" />
        <meta property="og:title" content={fullTitle} />
        <meta property="og:description" content={desc} />
        <meta property="og:image" content="/brand/og-card.png" />
        <meta name="twitter:card" content="summary_large_image" />
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
          <Wordmark />
          <nav aria-label="Footer">
            {site.footerLinks.map((l) => <Link key={l.href} href={l.href}>{l.label}</Link>)}
          </nav>
          <div className="meta">
            <a href={LAB_URL}>{site.footer.project}</a>
          </div>
        </div>
      </footer>
    </>
  );
}
