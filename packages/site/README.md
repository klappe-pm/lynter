# lynter.dev

The Lynter website: a Next.js static export with a playground that runs the Rust core, compiled to WebAssembly (`crates/lynter-wasm`, rebuilt with `npm run build:wasm`), in the browser, so every example on the site is real engine output.

## run

```bash
cd /path/to/lynter && npm ci
cd /path/to/lynter && npm run dev -w @lynter/site
```

The dev server listens on http://localhost:4310.

## build

```bash
cd /path/to/lynter && npm run build -w @lynter/site
```

The build compiles the core first and writes static files to `packages/site/out/`. Deploy that folder to any static host (Vercel, Cloudflare Pages) and point lynter.dev at it.

## edit

| Source | Purpose |
|---|---|
| `src/content.ts` | Every visible string and outbound link, including `INSTALL_URL` |
| `src/samples.ts` | Playground seed config and note, scope demo config, CLI snippets and `CLI_BIN` |
| `src/engine.ts` | Thin wrapper over the core: lint, profile chain, data.json conversion |
| `styles/globals.css` | Tokens for light and dark themes and all layout |
| `pages/docs.tsx` | Renders `docs/lynter-yaml.md` at build time, so the reference never drifts |

Set `NEXT_PUBLIC_BUTTONDOWN_USER` at build time to show the notify form; without it the section is hidden.

When the plugin is listed in the Obsidian community directory, change `INSTALL_URL` to `obsidian://show-plugin?id=lynter`. When the CLI is renamed, change `CLI_BIN`.

## brand and fonts

Geist Sans and Geist Mono come from the `geist` npm package; the variable woff2 files are copied to `public/fonts/` and loaded by `@font-face` in `styles/globals.css`, so no remote fonts load. Logo files live in `public/brand/`. Regenerate `og-card.png` and `apple-touch-icon.png` with `node scripts/brand.cjs` (needs Playwright). `node scripts/shots.cjs <dir>` screenshots every route after a build and reports overflow and console errors. The Geist license is served beside the fonts as `public/fonts/geist-license.txt`.

## checks

`npm run check:export -w @lynter/site` runs after a build and fails on a broken internal link, a page missing from `public/sitemap.xml`, or a page without an absolute canonical link and social card; CI and the deploy workflow both run it. `npm run smoke -w @lynter/site` fetches every page and the playground assets from https://lynter.dev (or the origin given as an argument); the deploy workflow runs it after each deploy. Add a new page to `public/sitemap.xml`.

## content

Long pages are Markdown in `content/*.md`, rendered at build time by `src/md.ts`. `{{CLI}}` expands to `CLI_BIN`. The examples gallery data is in `src/examples.ts`; starter configs are in the repository `examples/` folder, and `__tests__/examples.test.ts` lints each one.
