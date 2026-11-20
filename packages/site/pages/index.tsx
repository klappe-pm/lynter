import Link from 'next/link';
import {Bench} from '@/components/bench';
import {CodeBlock} from '@/components/code-block';
import {Layout} from '@/components/layout';
import {ScopePicker} from '@/components/scope-picker';
import {home, INSTALL_URL, NOTIFY_USER, site} from '@/content';
import {cliSnippet} from '@/samples';

export default function Home() {
  return (
    <Layout>
      <section className="hero wrap">
        <h1>{home.headline}</h1>
        <p className="lede">{home.lede}</p>
        <div className="actions">
          <a className="button primary" href={INSTALL_URL}>{site.install}</a>
          <Link className="button quiet" href="/playground/">{home.secondary}</Link>
        </div>
        <p className="bench-note">{home.benchNote}</p>
        <Bench />
      </section>

      <section className="section" aria-label="What Lynter does"><div className="wrap">
        <div className="pillars">
          {home.pillars.map((p) => (
            <div key={p.title}>
              <h3>{p.title}</h3>
              <p>{p.body}</p>
            </div>
          ))}
        </div>
      </div></section>

      <section className="section"><div className="wrap split">
        <div>
          <h2>{home.scope.title}</h2>
          <p className="intro">{home.scope.body}</p>
        </div>
        <ScopePicker />
      </div></section>

      <section className="section"><div className="wrap split">
        <div>
          <h2>{home.cli.title}</h2>
          <p className="intro">{home.cli.body}</p>
        </div>
        <CodeBlock code={cliSnippet} label={home.cli.title} />
      </div></section>

      <section className="section"><div className="wrap">
        <h2>{home.useCases.title}</h2>
        <p className="intro">{home.useCases.body}</p>
        <ul className="strip">
          {home.useCases.items.map((i) => (
            <li key={i.title}><Link className="card" href={i.href}><h3>{i.title}</h3><p>{i.body}</p></Link></li>
          ))}
        </ul>
      </div></section>

      <section className="section"><div className="wrap">
        <h2>{home.integrations.title}</h2>
        <p className="intro">{home.integrations.body}</p>
        <ul className="strip four">
          {home.integrations.items.map((i) => (
            <li key={i.title}><Link className="card" href={i.href}><h3>{i.title}</h3><p>{i.body}</p></Link></li>
          ))}
        </ul>
      </div></section>

      <section className="section"><div className="wrap">
        <h2>{home.compare.title}</h2>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>{home.compare.columns.map((c, i) => <th key={i} scope="col">{c}</th>)}</tr>
            </thead>
            <tbody>
              {home.compare.rows.map(([label, ...cells]) => (
                <tr key={label}>
                  <th scope="row">{label}</th>
                  {cells.map((c, i) => <td key={i}>{c}</td>)}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div></section>

      <section className="section"><div className="wrap">
        <h2>{home.faq.title}</h2>
        <div className="faq-list">
          {home.faq.items.map((f) => (
            <div key={f.q}><h3>{f.q}</h3><p>{f.a}</p></div>
          ))}
        </div>
        <div className="actions" style={{marginTop: 24}}><Link className="button quiet" href="/faq/">{home.faq.more}</Link></div>
      </div></section>

      <section className="section"><div className="wrap split">
        <div>
          <h2>{home.switch.title}</h2>
          <p className="intro">{home.switch.body}</p>
        </div>
        <div className="actions"><Link className="button primary" href="/switch/">{home.switch.action}</Link></div>
      </div></section>

      {NOTIFY_USER ? (
        <section className="section"><div className="wrap">
          <h2>{home.notify.title}</h2>
          <p className="intro">{home.notify.body}</p>
          <form className="inline-form" action={`https://buttondown.com/api/emails/embed-subscribe/${NOTIFY_USER}`} method="post">
            <label className="field">{home.notify.label}<input type="email" name="email" required autoComplete="email" /></label>
            <button type="submit" className="button primary">{home.notify.action}</button>
          </form>
        </div></section>
      ) : null}
    </Layout>
  );
}
