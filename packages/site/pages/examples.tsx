import Link from 'next/link';
import {useEffect, useState} from 'react';
import {Layout} from '@/components/layout';
import {examplesPage as copy} from '@/content';
import {examples} from '@/examples';
import {changedLines} from '@/diff';
import {lint, todayIso, useEngine} from '@/engine';

function Lines({text, marks}: {text: string; marks?: boolean[]}) {
  return (
    <pre className="view">
      {text.split('\n').map((line, i) => <span key={i} className={marks?.[i] ? 'ln changed' : 'ln'}>{line}</span>)}
    </pre>
  );
}

export default function Examples() {
  const [today, setToday] = useState('2026-01-01');
  useEffect(() => setToday(todayIso()), []);
  useEngine();
  return (
    <Layout title={copy.title} description={copy.lede}>
      <div className="wrap">
        <div className="page-head">
          <h1>{copy.title}</h1>
          <p className="lede">{copy.lede}</p>
        </div>
        <div className="gallery">
          {examples.map((e) => {
            const out = lint(e.config, e.note, e.path, today);
            const after = out.ok ? out.text : e.note;
            return (
              <article key={e.id} className="example" id={e.id}>
                <div className="example-head">
                  <div>
                    <h2>{e.title}</h2>
                    <p>{e.blurb}</p>
                    <p className="path">{copy.path}: {e.path}</p>
                  </div>
                  <Link className="button primary" href={`/playground/?example=${e.id}`}>{copy.open}</Link>
                </div>
                <div className="bench">
                  <section className="pane" aria-label={copy.config}>
                    <div className="pane-head"><span>{copy.config}</span></div>
                    <Lines text={e.config} />
                  </section>
                  <section className="pane" aria-label={copy.before}>
                    <div className="pane-head"><span>{copy.before}</span></div>
                    <Lines text={e.note} />
                  </section>
                  <section className="pane" aria-label={copy.after}>
                    <div className="pane-head"><span>{copy.after}</span></div>
                    <Lines text={after} marks={out.ok ? changedLines(e.note, after) : []} />
                  </section>
                </div>
              </article>
            );
          })}
        </div>
      </div>
    </Layout>
  );
}
