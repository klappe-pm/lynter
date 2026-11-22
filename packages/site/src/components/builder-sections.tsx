import {useEffect, useId, useState, type ReactNode} from 'react';
import {catalog, getIn, setIn, type Path} from '@/builder';
import {
  ADAPTERS, AUTOMATION_ACTIONS, AUTOMATION_FAILURES, AUTOMATION_LOGS, AUTOMATION_REPORTS, AUTOMATION_TRIGGERS, BUILTIN_CHECK_IDS,
  CHECK_FLAGS, DATE_KEYS, GLOBAL_TEMPLATE, MATCH_LISTS, SCOPED_TEMPLATES, TEMPLATE_LISTS, addAutomation, addCheck, addNoteType,
  addProvider, addScopedTemplate, customCheckIds, entryNameIssue, fixReplacementAt, flagAt, isRecord, itemsAt, keysAt, listAt,
  mappingText, nameIssue, parseMapping, removeEntry, removeProviderRule, scopedEntry, setAt, setBuiltinCheck, setChoice, setFix,
  setList, setProviderRule, setTrigger, setTriState, textAt, toggleCheckFlag, triStateAt, triggerNeedsSchedule,
  type MappingKind, type TriState,
} from '@/builder-sections';
import {sectionsCopy as copy} from '@/sections-copy';
import {ListField} from './builder-fields';

export interface SectionProps {
  data: unknown;
  edit: (change: (text: string) => string) => void;
}

/* ---------- shared fields ---------- */

function Label({text, hint}: {text: string; hint?: string}) {
  return <>{text}{hint ? <> <span className="b-hint">{hint}</span></> : null}</>;
}

function TextField({label, hint, value, onChange, mono}: {label: string; hint?: string; value: string; onChange: (next: string) => void; mono?: boolean}) {
  const id = useId();
  return (
    <div className="b-field">
      <label htmlFor={id}><Label text={label} hint={hint} /></label>
      <input id={id} type="text" className={mono ? 'mono' : undefined} value={value} spellCheck={false} onChange={(e) => onChange(e.target.value)} />
    </div>
  );
}

/** Multi-line text. The draft is local so a trailing newline can be typed and kept. */
function AreaField({label, hint, value, onChange}: {label: string; hint?: string; value: string; onChange: (next: string) => void}) {
  const id = useId();
  const [draft, setDraft] = useState(value);
  useEffect(() => {
    setDraft((current) => (current === value ? current : value));
  }, [value]);
  return (
    <div className="b-field">
      <label htmlFor={id}><Label text={label} hint={hint} /></label>
      <textarea
        id={id}
        className="mono"
        rows={Math.min(10, Math.max(3, draft.split('\n').length))}
        value={draft}
        spellCheck={false}
        onChange={(e) => {
          setDraft(e.target.value);
          onChange(e.target.value);
        }}
      />
    </div>
  );
}

/** A YAML mapping typed into a textarea; it is written only while it parses and fits the key. */
function MappingField({label, hint, kind, value, onChange}: {
  label: string;
  hint: string;
  kind: MappingKind;
  value: unknown;
  onChange: (next: Record<string, unknown> | undefined) => void;
}) {
  const id = useId();
  const errorId = useId();
  const text = mappingText(value);
  const [draft, setDraft] = useState(text);
  const [issue, setIssue] = useState<'syntax' | 'shape' | null>(null);
  // Runs only when the file's own text for this key changes, so a draft that does not parse is left alone while typing.
  useEffect(() => {
    setDraft((current) => {
      const parsed = parseMapping(current, kind);
      return parsed.ok && mappingText(parsed.value) === text ? current : text;
    });
    setIssue(null);
  }, [text, kind]);
  return (
    <div className="b-field">
      <label htmlFor={id}><Label text={label} hint={hint} /></label>
      <textarea
        id={id}
        className="mono"
        rows={Math.min(10, Math.max(2, draft.split('\n').length))}
        value={draft}
        spellCheck={false}
        aria-invalid={issue !== null}
        aria-describedby={issue ? errorId : undefined}
        onChange={(e) => {
          setDraft(e.target.value);
          const parsed = parseMapping(e.target.value, kind);
          if (parsed.ok) {
            setIssue(null);
            onChange(parsed.value);
          } else {
            setIssue(parsed.reason);
          }
        }}
      />
      {issue ? <p className="b-error" id={errorId} role="alert">{issue === 'syntax' ? copy.common.syntaxError : copy.common.shapeError}</p> : null}
    </div>
  );
}

function SelectField({label, value, options, labels, blank, onChange}: {
  label: string;
  value: string;
  options: readonly string[];
  labels: Record<string, string>;
  blank?: string;
  onChange: (next: string) => void;
}) {
  const id = useId();
  return (
    <div className="b-field">
      <label htmlFor={id}>{label}</label>
      <select id={id} value={options.includes(value) ? value : ''} onChange={(e) => onChange(e.target.value)}>
        {blank !== undefined ? <option value="">{blank}</option> : null}
        {options.map((o) => <option key={o} value={o}>{labels[o] ?? o}</option>)}
      </select>
    </div>
  );
}

function CheckField({label, checked, onChange}: {label: ReactNode; checked: boolean; onChange: (next: boolean) => void}) {
  return (
    <label className="b-check">
      <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} />
      <span>{label}</span>
    </label>
  );
}

const triLabels: Record<string, string> = {on: copy.common.on, off: copy.common.off};

function TriStateField({label, value, onChange}: {label: string; value: TriState; onChange: (next: TriState) => void}) {
  return <SelectField label={label} value={value} options={['on', 'off']} labels={triLabels} blank={copy.common.notSet} onChange={(v) => onChange(v === 'on' || v === 'off' ? v : '')} />;
}

function CardHead({title, onRemove}: {title: ReactNode; onRemove: () => void}) {
  return (
    <div className="b-rule-head">
      <h3>{title}</h3>
      <button type="button" className="b-link" onClick={onRemove}>{copy.common.remove}</button>
    </div>
  );
}

function issueText(issue: 'format' | 'taken' | null): string | null {
  return issue === 'format' ? copy.common.nameFormat : issue === 'taken' ? copy.common.nameTaken : null;
}

/** The row that names and adds a new entry. `strict` limits the name to what a mapping key should look like. */
function AddRow({label, taken, strict, onAdd}: {label: string; taken: string[]; strict: boolean; onAdd: (name: string) => void}) {
  const [name, setName] = useState('');
  const clean = name.trim();
  const issue = !clean ? null : strict ? nameIssue(clean, taken) : taken.includes(clean) ? 'taken' : null;
  const message = issueText(issue);
  return (
    <form
      className="b-add"
      onSubmit={(e) => {
        e.preventDefault();
        if (clean && !issue) {
          onAdd(clean);
          setName('');
        }
      }}
    >
      <label className="b-field">
        {label}
        <input className="mono" value={name} onChange={(e) => setName(strict ? e.target.value.trim() : e.target.value)} spellCheck={false} aria-invalid={issue !== null} />
      </label>
      <button type="submit" className="button quiet" disabled={!clean || issue !== null}>{copy.common.add}</button>
      {message ? <p className="b-error" role="alert">{message}</p> : null}
    </form>
  );
}

/** Path, extension and tag lists of a match block; `block` is `match`, or `scope` for an automation. */
function MatchFields({data, edit, entry, block}: SectionProps & {entry: Path; block: string}) {
  const labels = {path: copy.common.paths, extension: copy.common.extensions, tag: copy.common.tags};
  return (
    <>
      {MATCH_LISTS.map((key) => (
        <ListField
          key={key}
          label={labels[key]}
          hint={copy.common.onePerLine}
          value={listAt(data, [...entry, block, key])}
          onChange={(next) => edit((t) => setList(t, [...entry, block, key], next, entry.length))}
        />
      ))}
      <p className="b-hint">{copy.common.matchHint}</p>
    </>
  );
}

/* ---------- code checks ---------- */

function CheckCard({id, data, edit}: SectionProps & {id: string}) {
  const entry: Path = ['code-checks', id];
  const set = (sub: Path, value: unknown) => edit((t) => setAt(t, entry, sub, value));
  const flags = textAt(data, [...entry, 'flags']);
  const replacement = fixReplacementAt(data, id);
  const c = copy.codeChecks;
  return (
    <section className="b-scope" aria-label={id}>
      <CardHead title={<code>{id}</code>} onRemove={() => edit((t) => setIn(t, entry, undefined, 0))} />
      <CheckField label={c.enabled} checked={flagAt(data, [...entry, 'enabled'])} onChange={(on) => set(['enabled'], on ? true : undefined)} />
      <TextField label={c.description} value={textAt(data, [...entry, 'description'])} onChange={(v) => set(['description'], v === '' ? undefined : v)} />
      <ListField label={c.paths} hint={c.pathsHint} value={listAt(data, [...entry, 'paths'])} onChange={(next) => edit((t) => setList(t, [...entry, 'paths'], next, entry.length))} />
      <ListField label={c.languages} hint={c.languagesHint} value={listAt(data, [...entry, 'languages'])} onChange={(next) => edit((t) => setList(t, [...entry, 'languages'], next, entry.length))} />
      <TextField label={c.pattern} hint={c.patternHint} mono value={textAt(data, [...entry, 'pattern'])} onChange={(v) => set(['pattern'], v === '' ? undefined : v)} />
      <fieldset className="b-field">
        <legend>{c.flags}</legend>
        <div className="b-checks">
          {CHECK_FLAGS.map((f) => (
            <CheckField key={f} label={c.flagLabels[f]} checked={flags.includes(f)} onChange={(on) => set(['flags'], toggleCheckFlag(flags, f, on) || undefined)} />
          ))}
        </div>
      </fieldset>
      <TextField label={c.message} hint={c.messageHint} value={textAt(data, [...entry, 'message'])} onChange={(v) => set(['message'], v === '' ? undefined : v)} />
      <CheckField label={c.fix} checked={replacement !== null} onChange={(on) => edit((t) => setFix(t, id, on ? '' : null))} />
      {replacement !== null ? <TextField label={c.replacement} hint={c.replacementHint} mono value={replacement} onChange={(v) => edit((t) => setFix(t, id, v))} /> : null}
    </section>
  );
}

export function CodeChecksSection({data, edit}: SectionProps) {
  const custom = customCheckIds(data);
  return (
    <>
      <h4>{copy.codeChecks.builtins}</h4>
      <div className="b-checks">
        {BUILTIN_CHECK_IDS.map((id) => (
          <CheckField
            key={id}
            label={<><code>{id}</code> {copy.codeChecks.builtinLabels[id]}</>}
            checked={flagAt(data, ['code-checks', id, 'enabled'])}
            onChange={(on) => edit((t) => setBuiltinCheck(t, id, on))}
          />
        ))}
      </div>
      <h4>{copy.codeChecks.custom}</h4>
      {custom.length === 0 ? <p className="b-empty">{copy.codeChecks.none}</p> : null}
      {custom.map((id) => <CheckCard key={id} id={id} data={data} edit={edit} />)}
      <AddRow label={copy.codeChecks.newId} strict taken={[...BUILTIN_CHECK_IDS, ...custom]} onAdd={(id) => edit((t) => addCheck(t, id))} />
    </>
  );
}

/* ---------- adapters ---------- */

export function AdaptersSection({data, edit}: SectionProps) {
  const c = copy.adapters;
  return (
    <>
      {ADAPTERS.map(({id, select}) => {
        const at = (key: string): Path => ['defaults', 'adapters', id, key];
        // Pruning to the top means a cleared adapter leaves no empty mapping behind.
        const set = (key: string, value: unknown) => edit((t) => setIn(t, at(key), value, 0));
        return (
          <section key={id} className="b-scope" aria-label={c.labels[id]}>
            <div className="b-rule-head"><h3>{c.labels[id]}</h3></div>
            <CheckField label={c.enabled} checked={flagAt(data, at('enabled'))} onChange={(on) => set('enabled', on ? true : undefined)} />
            <TextField label={c.path} hint={c.pathHint} mono value={textAt(data, at('path'))} onChange={(v) => set('path', v === '' ? undefined : v)} />
            <ListField label={c.args} hint={c.argsHint} value={listAt(data, at('args'))} onChange={(next) => edit((t) => setList(t, at('args'), next, 0))} />
            {select ? <ListField label={c.select} hint={c.selectHint} value={listAt(data, at('select'))} onChange={(next) => edit((t) => setList(t, at('select'), next, 0))} /> : null}
          </section>
        );
      })}
    </>
  );
}

/* ---------- rule order ---------- */

export function RuleOrderSection({data, edit}: SectionProps) {
  return <ListField label={copy.ruleOrder.label} hint={copy.ruleOrder.hint} value={listAt(data, ['rule-order'])} onChange={(next) => edit((t) => setList(t, ['rule-order'], next, 0))} />;
}

/* ---------- note types ---------- */

function NoteTypeCard({name, data, edit}: SectionProps & {name: string}) {
  const entry: Path = ['note-types', name];
  const c = copy.noteTypes;
  const set = (sub: Path, value: unknown) => edit((t) => setAt(t, entry, sub, value));
  return (
    <section className="b-scope" aria-label={name}>
      <CardHead title={<code>{name}</code>} onRemove={() => edit((t) => setIn(t, entry, undefined, 0))} />
      <MappingField label={c.required} hint={c.requiredHint} kind="seed" value={getIn(data, [...entry, 'required'])} onChange={(v) => set(['required'], v)} />
      <ListField label={c.keyOrder} hint={c.keyOrderHint} value={listAt(data, [...entry, 'key-order'])} onChange={(next) => edit((t) => setList(t, [...entry, 'key-order'], next, entry.length))} />
      <MappingField label={c.values} hint={c.valuesHint} kind="allowed" value={getIn(data, [...entry, 'values'])} onChange={(v) => set(['values'], v)} />
      <h4>{c.dateKeys}</h4>
      {DATE_KEYS.map((key) => (
        <TextField key={key} label={key === 'created' ? c.created : c.revised} mono value={textAt(data, [...entry, 'date-keys', key])} onChange={(v) => set(['date-keys', key], v === '' ? undefined : v)} />
      ))}
      <h4>{c.match}</h4>
      <MatchFields data={data} edit={edit} entry={entry} block="match" />
      <MappingField label={c.frontmatterMatch} hint={c.frontmatterMatchHint} kind="predicate" value={getIn(data, [...entry, 'match', 'frontmatter'])} onChange={(v) => set(['match', 'frontmatter'], v)} />
    </section>
  );
}

export function NoteTypesSection({data, edit}: SectionProps) {
  const names = keysAt(data, ['note-types']);
  return (
    <>
      <p className="b-hint">{copy.noteTypes.governance}</p>
      {names.length === 0 ? <p className="b-empty">{copy.noteTypes.none}</p> : null}
      {names.map((name) => <NoteTypeCard key={name} name={name} data={data} edit={edit} />)}
      <AddRow label={copy.noteTypes.newName} strict taken={names} onAdd={(name) => edit((t) => addNoteType(t, name))} />
    </>
  );
}

/* ---------- templates ---------- */

/** The keys global and scoped templates share. `where` is the template's place in the data; `write` writes below it. */
function TemplateFields({data, where, write, edit}: {data: unknown; where: Path; write: (text: string, sub: Path, value: unknown) => string; edit: SectionProps['edit']}) {
  const c = copy.templates;
  const labels = {'pinned-keys': c.pinnedKeys, 'key-order': c.keyOrder};
  const hints = {'pinned-keys': c.pinnedKeysHint, 'key-order': c.keyOrderHint};
  return (
    <>
      {TEMPLATE_LISTS.map((key) => (
        <ListField key={key} label={labels[key]} hint={hints[key]} value={listAt(data, [...where, key])} onChange={(next) => edit((t) => write(t, [key], next.length ? next : undefined))} />
      ))}
      <MappingField label={c.frontmatter} hint={c.frontmatterHint} kind="seed" value={getIn(data, [...where, 'frontmatter'])} onChange={(v) => edit((t) => write(t, ['frontmatter'], v))} />
      <AreaField label={c.body} hint={c.bodyHint} value={textAt(data, [...where, 'body'])} onChange={(v) => edit((t) => write(t, ['body'], v === '' ? undefined : v))} />
      <TriStateField label={c.ageLine} value={triStateAt(data, [...where, 'age-line'])} onChange={(v) => edit((t) => write(t, ['age-line'], v === '' ? undefined : v === 'on'))} />
    </>
  );
}

export function TemplatesSection({data, edit}: SectionProps) {
  const c = copy.templates;
  const scoped = itemsAt(data, SCOPED_TEMPLATES);
  const names = scoped.map((s) => (isRecord(s) && typeof s.name === 'string' ? s.name : ''));
  return (
    <>
      <section className="b-scope" aria-label={c.global}>
        <div className="b-rule-head"><h3>{c.global}</h3></div>
        <TemplateFields data={data} edit={edit} where={GLOBAL_TEMPLATE} write={(t, sub, v) => setIn(t, [...GLOBAL_TEMPLATE, ...sub], v, 0)} />
      </section>
      <h4>{c.byScope}</h4>
      {scoped.length === 0 ? <p className="b-empty">{c.none}</p> : null}
      {scoped.map((_, i) => {
        const entry = scopedEntry(i);
        const issue = entryNameIssue(names[i], names);
        return (
          <section key={i} className="b-scope" aria-label={names[i] || String(i + 1)}>
            <CardHead title={<code>{names[i] || String(i + 1)}</code>} onRemove={() => edit((t) => removeEntry(t, SCOPED_TEMPLATES, i))} />
            <TextField label={c.name} mono value={names[i]} onChange={(v) => edit((t) => setAt(t, entry, ['name'], v === '' ? undefined : v))} />
            {issue ? <p className="b-error" role="alert">{issue === 'empty' ? copy.common.nameEmpty : copy.common.nameDuplicate}</p> : null}
            <SelectField label={c.extends} value={textAt(data, [...entry, 'extends'])} options={['global']} labels={{global: c.extendsGlobal}} blank={copy.common.notSet} onChange={(v) => edit((t) => setChoice(t, entry, 'extends', v))} />
            <h4>{c.match}</h4>
            <MatchFields data={data} edit={edit} entry={entry} block="match" />
            <MappingField label={c.toggles} hint={c.togglesHint} kind="toggles" value={getIn(data, [...entry, 'toggles'])} onChange={(v) => edit((t) => setAt(t, entry, ['toggles'], v))} />
            <TemplateFields data={data} edit={edit} where={entry} write={(t, sub, v) => setAt(t, entry, sub, v)} />
          </section>
        );
      })}
      <AddRow label={c.newName} strict={false} taken={names} onAdd={(name) => edit((t) => addScopedTemplate(t, name))} />
    </>
  );
}

/* ---------- automations ---------- */

export function AutomationsSection({data, edit}: SectionProps) {
  const c = copy.automations;
  const entries = itemsAt(data, ['automations']);
  const names = entries.map((e) => (isRecord(e) && typeof e.name === 'string' ? e.name : ''));
  return (
    <>
      {entries.length === 0 ? <p className="b-empty">{c.none}</p> : null}
      {entries.map((_, i) => {
        const entry: Path = ['automations', i];
        const trigger = textAt(data, [...entry, 'trigger']);
        const validTrigger = (AUTOMATION_TRIGGERS as readonly string[]).includes(trigger);
        const issue = entryNameIssue(names[i], names);
        const schedule = textAt(data, [...entry, 'schedule']);
        const choice = (key: string, label: string, options: readonly string[], labels: Record<string, string>) => (
          <SelectField label={label} value={textAt(data, [...entry, key])} options={options} labels={labels} blank={copy.common.notSet} onChange={(v) => edit((t) => setChoice(t, entry, key, v))} />
        );
        return (
          <section key={i} className="b-scope" aria-label={names[i] || String(i + 1)}>
            <CardHead title={<code>{names[i] || String(i + 1)}</code>} onRemove={() => edit((t) => removeEntry(t, ['automations'], i))} />
            <TextField label={copy.common.nameLabel} mono value={names[i]} onChange={(v) => edit((t) => setAt(t, entry, ['name'], v === '' ? undefined : v))} />
            {issue ? <p className="b-error" role="alert">{issue === 'empty' ? copy.common.nameEmpty : copy.common.nameDuplicate}</p> : null}
            <SelectField label={c.trigger} value={trigger} options={AUTOMATION_TRIGGERS} labels={c.triggers} blank={validTrigger ? undefined : c.triggerChoose} onChange={(v) => edit((t) => setTrigger(t, i, v))} />
            {triggerNeedsSchedule(trigger) ? (
              <>
                <TextField label={c.schedule} hint={c.scheduleHint} mono value={schedule} onChange={(v) => edit((t) => setAt(t, entry, ['schedule'], v === '' ? undefined : v))} />
                {schedule === '' ? <p className="b-error" role="alert">{c.scheduleRequired}</p> : null}
              </>
            ) : null}
            {choice('action', c.action, AUTOMATION_ACTIONS, c.actions)}
            {choice('failure', c.failure, AUTOMATION_FAILURES, c.failures)}
            {choice('log', c.log, AUTOMATION_LOGS, c.logs)}
            {choice('report', c.report, AUTOMATION_REPORTS, c.reports)}
            <h4>{c.scope}</h4>
            <MatchFields data={data} edit={edit} entry={entry} block="scope" />
          </section>
        );
      })}
      <AddRow label={c.newName} strict={false} taken={names} onAdd={(name) => edit((t) => addAutomation(t, name))} />
    </>
  );
}

/* ---------- providers ---------- */

export function ProvidersSection({data, edit}: SectionProps) {
  const c = copy.providers;
  const namespaces = keysAt(data, ['providers']);
  return (
    <>
      {namespaces.length === 0 ? <p className="b-empty">{c.none}</p> : null}
      {namespaces.map((ns) => {
        const ids = keysAt(data, ['providers', ns, 'rules']);
        const free = catalog.filter((r) => !ids.includes(r.id));
        return (
          <section key={ns} className="b-scope" aria-label={ns}>
            <CardHead title={<code>{ns}</code>} onRemove={() => edit((t) => setIn(t, ['providers', ns], undefined, 0))} />
            <h4>{c.rules}</h4>
            {ids.length === 0 ? <p className="b-empty">{c.noRules}</p> : null}
            {ids.map((id) => (
              <div key={id} className="b-rule-head">
                <CheckField label={<code>{id}</code>} checked={flagAt(data, ['providers', ns, 'rules', id, 'enabled'])} onChange={(on) => edit((t) => setProviderRule(t, ns, id, on))} />
                <button type="button" className="b-link" onClick={() => edit((t) => removeProviderRule(t, ns, id))}>{copy.common.remove}</button>
              </div>
            ))}
            <label className="b-field">
              {c.addRule}
              <select value="" onChange={(e) => { const id = e.target.value; if (id) edit((t) => setProviderRule(t, ns, id, true)); }}>
                <option value="">{c.choose}</option>
                {free.map((r) => <option key={r.id} value={r.id}>{r.id}</option>)}
              </select>
            </label>
          </section>
        );
      })}
      <AddRow label={c.newName} strict taken={namespaces} onAdd={(name) => edit((t) => addProvider(t, name))} />
    </>
  );
}
