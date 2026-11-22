/**
 * Entry point of the standalone config builder: one HTML file that runs from
 * disk with no network. The engine is embedded as gzipped base64 and unpacked
 * in the browser, so the checks are the same ones the site and the plugin run.
 */
import {createRoot} from 'react-dom/client';
import {Builder} from '@/components/builder';
import {Icon} from '@/components/icon';
import {SITE_URL, site} from '@/content';
import {setEngineLoader} from '@/engine';
import * as glue from '../public/wasm/lynter_wasm.mjs';
import packed from 'lynter-wasm-gz';

setEngineLoader(async () => {
  const gz = Uint8Array.from(atob(packed), (c) => c.charCodeAt(0));
  const stream = new Blob([gz]).stream().pipeThrough(new DecompressionStream('gzip'));
  glue.initSync({module: await new Response(stream).arrayBuffer()});
  return glue;
});

createRoot(document.getElementById('root')!).render(
  <>
    <header className="site-header">
      <div className="wrap">
        <a className="wordmark" href={SITE_URL}><Icon /><span>{site.name}</span></a>
      </div>
    </header>
    <main id="main"><Builder site={SITE_URL} /></main>
  </>,
);
