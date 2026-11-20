import {useState} from 'react';
import {Layout} from '@/components/layout';
import {switchPage as copy} from '@/content';
import {convertDataJson, useEngine} from '@/engine';

export default function Switch() {
  const [input, setInput] = useState('');
  const [output, setOutput] = useState('');
  const [error, setError] = useState(false);
  const [copied, setCopied] = useState(false);
  const {ready} = useEngine();

  const convert = () => {
    const yaml = convertDataJson(input);
    setError(yaml === null);
    setOutput(yaml ?? '');
    setCopied(false);
  };

  const copyOut = async () => {
    try {
      await navigator.clipboard.writeText(output);
      setCopied(true);
    } catch {
      setCopied(false);
    }
  };

  return (
    <Layout title={copy.title} description={copy.lede}>
      <div className="wrap">
        <div className="page-head">
          <h1>{copy.title}</h1>
          <p className="lede">{copy.lede}</p>
        </div>
        <div className="converter">
          <section className="pane">
            <div className="pane-head"><label htmlFor="data-json">{copy.inputLabel}</label></div>
            <textarea id="data-json" value={input} placeholder={copy.placeholder} onChange={(e) => setInput(e.target.value)} spellCheck={false} />
          </section>
          <section className="pane">
            <div className="pane-head"><span>{copy.outputLabel}</span>{output ? <button type="button" className="button quiet small" onClick={copyOut}>{copied ? copy.copied : copy.copy}</button> : null}</div>
            <div className="out" tabIndex={0} aria-live="polite">{output}</div>
          </section>
        </div>
        <div className="actions" style={{marginTop: 16}}>
          <button type="button" className="button primary" onClick={convert} disabled={!ready || !input.trim()}>{copy.action}</button>
        </div>
        {error ? <p className="message error" role="alert">{copy.error}</p> : <p className="message">{copy.note}</p>}
      </div>
    </Layout>
  );
}
