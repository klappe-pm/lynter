import {useEffect, useMemo, useRef, useState} from 'react';
import {changedLines} from '@/diff';
import {lint, todayIso, useEngine, validateConfig} from '@/engine';
import {playground as copy} from '@/content';
import {sampleConfig, sampleNote, samplePath} from '@/samples';

/**
 * Config, note, and result side by side, running the real core on every
 * keystroke. Used in the home hero and on the playground page.
 */
export function Bench({showPath = false, seed}: {showPath?: boolean; seed?: {config: string; note: string; path: string}}) {
  const start = seed ?? {config: sampleConfig, note: sampleNote, path: samplePath};
  const [config, setConfig] = useState(start.config);
  const [note, setNote] = useState(start.note);
  const [path, setPath] = useState(start.path);
  const [today, setToday] = useState('2026-01-01');
  useEffect(() => setToday(todayIso()), []);
  const {ready, slow} = useEngine();

  const outcome = useMemo(() => lint(config, note, path, today), [config, note, path, today, ready]);
  const problems = useMemo(() => validateConfig(config), [config, ready]);
  const errorCount = problems.filter((p) => p.severity === 'error').length;
  const configRef = useRef<HTMLTextAreaElement>(null);
  /** Put the caret on a 1-based line of the config, so a problem is one click from its fix. */
  const goToLine = (line: number) => {
    const el = configRef.current;
    if (!el) return;
    const lines = config.split('\n');
    const start = lines.slice(0, line - 1).reduce((n, l) => n + l.length + 1, 0);
    el.focus();
    el.setSelectionRange(start, start + (lines[line - 1]?.length ?? 0));
  };
  const marks = useMemo(() => (outcome.ok ? changedLines(note, outcome.text) : []), [note, outcome]);

  const reset = () => {
    setConfig(start.config);
    setNote(start.note);
    setPath(start.path);
  };

  return (
    <div className="bench">
      <section className="pane" aria-label={copy.configLabel}>
        <div className="pane-head">
          <label htmlFor="bench-config">{copy.configLabel}</label>
          {ready ? <span className={errorCount ? 'config-state bad' : problems.length ? 'config-state warn' : 'config-state'}>{errorCount ? copy.configProblems(errorCount) : problems.length ? copy.configWarnings(problems.length) : copy.configOk}</span> : null}
        </div>
        <textarea ref={configRef} id="bench-config" value={config} onChange={(e) => setConfig(e.target.value)} spellCheck={false} autoCapitalize="off" autoCorrect="off" aria-invalid={errorCount > 0} aria-describedby={problems.length ? 'bench-config-problems' : undefined} />
        {problems.length ? (
          <ul id="bench-config-problems" className="config-problems" role="status" aria-live="polite">
            {problems.map((p, i) => (
              <li key={i} className={p.severity}>
                <strong>{p.severity === 'error' ? copy.configErrorLabel : copy.configWarningLabel}</strong>
                {p.line ? <button type="button" onClick={() => goToLine(p.line!)}>{copy.configLine} {p.line}</button> : null}
                <span>{p.message}</span>
              </li>
            ))}
          </ul>
        ) : null}
      </section>
      <section className="pane" aria-label={copy.noteLabel}>
        <div className="pane-head"><label htmlFor="bench-note">{copy.noteLabel}</label></div>
        <textarea id="bench-note" value={note} onChange={(e) => setNote(e.target.value)} spellCheck={false} />
      </section>
      <section className="pane" aria-label={copy.outputLabel}>
        <div className="pane-head"><span>{copy.outputLabel}</span></div>
        {outcome.ok ? (
          <div className="out" role="status" aria-live="polite" tabIndex={0}>
            {outcome.text.split('\n').map((line, i) => (
              <span key={i} className={marks[i] ? 'ln changed' : 'ln'}>{line}</span>
            ))}
          </div>
        ) : (
          <div className="out error">{copy.configError}</div>
        )}
      </section>
      <div className="bench-foot">
        {slow ? <span role="status">{copy.loading}</span> : null}
        {showPath ? (
          <label className="path-field">
            {copy.pathLabel}
            <input className="mono" value={path} onChange={(e) => setPath(e.target.value)} spellCheck={false} />
          </label>
        ) : null}
        {outcome.ok ? (
          <>
            <span className="chips" aria-label={copy.chainLabel}>
              {copy.chainLabel}: {outcome.profileChain.map((p) => <code key={p} className="chip">{p}</code>)}
            </span>
            <span className="chips" aria-label={copy.violationsLabel}>
              {outcome.rules.length ? <>{copy.violationsLabel}: {outcome.rules.map((r) => <code key={r} className="chip">{r}</code>)}</> : copy.noViolations}
            </span>
          </>
        ) : null}
        {showPath ? <button type="button" className="button quiet small" onClick={reset}>{copy.reset}</button> : null}
      </div>
    </div>
  );
}
