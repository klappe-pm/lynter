import {useEffect, useId, useState} from 'react';
import {parse as parseYaml, stringify} from 'yaml';
import {builderCopy as copy} from '@/content';
import {optionKind, ruleState, toList, type CatalogRule, type OptionValue} from '@/builder';
import hintsJson from '@/option-hints.json';

interface OptionHint {description: string; choices?: string[]}
/** What each option does and, where the engine accepts a fixed set of values, which ones. Read from the Rust source. */
const hints = hintsJson as Record<string, Record<string, OptionHint>>;

/** A list edited one entry per line. The draft is local so a new empty line can be typed. */
export function ListField({label, value, onChange, hint}: {label: string; value: string[]; onChange: (next: string[]) => void; hint?: string}) {
  const id = useId();
  const joined = value.join('\n');
  const [draft, setDraft] = useState(joined);
  useEffect(() => {
    setDraft((current) => (toList(current).join('\n') === joined ? current : joined));
  }, [joined]);
  return (
    <div className="b-field">
      <label htmlFor={id}>{label} <span className="b-hint">{hint ? `${hint} ` : ''}{copy.rules.onePerLine}</span></label>
      <textarea
        id={id}
        className="mono"
        rows={Math.min(8, Math.max(2, draft.split('\n').length))}
        value={draft}
        spellCheck={false}
        onChange={(e) => {
          setDraft(e.target.value);
          onChange(toList(e.target.value));
        }}
      />
    </div>
  );
}

/** A nested value edited as YAML; it is written only while it parses. */
function YamlField({label, value, onChange, hint}: {label: string; value: OptionValue; onChange: (next: OptionValue) => void; hint?: string}) {
  const id = useId();
  const text = stringify(value, {lineWidth: 0}).trimEnd();
  const [draft, setDraft] = useState(text);
  const [bad, setBad] = useState(false);
  useEffect(() => {
    setDraft((current) => {
      try {
        return JSON.stringify(parseYaml(current)) === JSON.stringify(value) ? current : text;
      } catch {
        return current;
      }
    });
  }, [text, value]);
  return (
    <div className="b-field">
      <label htmlFor={id}>{label} <span className="b-hint">{hint ? `${hint} ` : ''}{copy.rules.yamlHint}</span></label>
      <textarea
        id={id}
        className="mono"
        rows={Math.min(10, Math.max(2, draft.split('\n').length))}
        value={draft}
        spellCheck={false}
        aria-invalid={bad}
        onChange={(e) => {
          setDraft(e.target.value);
          try {
            const next = parseYaml(e.target.value) as OptionValue | null;
            setBad(next === null || next === undefined);
            if (next !== null && next !== undefined) onChange(next);
          } catch {
            setBad(true);
          }
        }}
      />
    </div>
  );
}

function OptionField({name, fallback, value, hint, onChange}: {name: string; fallback: OptionValue; value: OptionValue; hint?: OptionHint; onChange: (next: OptionValue) => void}) {
  const id = useId();
  const kind = optionKind(fallback);
  if (kind === 'toggle') {
    return (
      <label className="b-check" title={hint?.description}>
        <input type="checkbox" checked={value === true} onChange={(e) => onChange(e.target.checked)} />
        <code>{name}</code>
      </label>
    );
  }
  if (kind === 'list') return <ListField label={name} value={Array.isArray(value) ? value.map(String) : []} onChange={onChange} hint={hint?.description} />;
  if (kind === 'yaml') return <YamlField label={name} value={value} onChange={onChange} hint={hint?.description} />;
  const choices = hint?.choices;
  return (
    <div className="b-field">
      <label htmlFor={id}><code>{name}</code>{hint ? <span className="b-hint">{hint.description}</span> : null}</label>
      {choices ? (
        <select id={id} value={typeof value === 'string' ? value : ''} onChange={(e) => onChange(e.target.value)}>
          {/* A value typed into the YAML that the engine does not list still shows, so it is not silently replaced. */}
          {typeof value === 'string' && !choices.includes(value) ? <option value={value}>{value === '' ? copy.rules.unset : value}</option> : null}
          {choices.map((c) => <option key={c} value={c}>{c === '' ? copy.rules.unset : c}</option>)}
        </select>
      ) : null}
      {choices ? null : (<>
      {kind === 'number' ? (
        <input id={id} type="number" value={typeof value === 'number' ? value : 0} onChange={(e) => e.target.value !== '' && onChange(Number(e.target.value))} />
      ) : (
        <input id={id} type="text" className="mono" value={typeof value === 'string' ? value : ''} spellCheck={false} onChange={(e) => onChange(e.target.value)} />
      )}
      </>)}
    </div>
  );
}

/** One rule: its switch, what it does, and its options once it is in the file. */
export function RuleCard({rule, data, profile, onEnable, onOption, onRemove}: {
  rule: CatalogRule;
  data: unknown;
  profile: string | null;
  onEnable: (enabled: boolean) => void;
  onOption: (name: string, value: OptionValue) => void;
  onRemove: () => void;
}) {
  const state = ruleState(data, profile, rule);
  const names = Object.keys(rule.options).filter((n) => n !== 'enabled');
  const toggles = names.filter((n) => optionKind(rule.options[n]) === 'toggle');
  const others = names.filter((n) => optionKind(rule.options[n]) !== 'toggle');
  return (
    <div className={state.enabled ? 'b-rule on' : 'b-rule'}>
      <div className="b-rule-head">
        <label className="b-switch">
          <input type="checkbox" checked={state.enabled} onChange={(e) => onEnable(e.target.checked)} aria-label={`${rule.id}: ${copy.rules.on}`} />
          <code>{rule.id}</code>
        </label>
        {rule.host === 'code' ? <span className="soon">{copy.rules.cliOnly}</span> : null}
        {state.present ? <button type="button" className="b-link" onClick={onRemove}>{copy.rules.remove}</button> : null}
      </div>
      <p>{rule.description}</p>
      {names.length && state.present ? (
        <details className="b-options" open={names.length <= 6}>
          <summary>{copy.rules.options(names.length)}</summary>
          {others.map((n) => <OptionField key={n} name={n} fallback={rule.options[n]} value={state.options[n]} hint={hints[rule.id]?.[n]} onChange={(v) => onOption(n, v)} />)}
          {toggles.length ? (
            <div className="b-checks">
              {toggles.map((n) => <OptionField key={n} name={n} fallback={rule.options[n]} value={state.options[n]} hint={hints[rule.id]?.[n]} onChange={(v) => onOption(n, v)} />)}
            </div>
          ) : null}
        </details>
      ) : null}
    </div>
  );
}
