import {useCallback, useEffect, useMemo, useRef, useState, type ReactNode} from 'react';
import {ListField, RuleCard} from '@/components/builder-fields';
import {
  AdaptersSection, AutomationsSection, CodeChecksSection, NoteTypesSection, ProvidersSection, RuleOrderSection, TemplatesSection,
} from '@/components/builder-sections';
import {builderCopy as copy, playground} from '@/content';
import {
  addProfile, asList, catalog, draftSlot, EMPTY_CONFIG, getIn, parse, profileNameError, profileNames, removeRule,
  ruleGroups, ruleState, setIn, setRuleEnabled, setRuleOption, type CatalogRule, type OptionValue,
} from '@/builder';
import {starterConfig, useEngine, validateConfig} from '@/engine';
import {examples} from '@/examples';
import {sectionsCopy} from '@/sections-copy';

/** The parts of the File System Access API the builder uses; Chromium only. */
interface FileHandle {
  name: string;
  getFile(): Promise<File>;
  createWritable(): Promise<{write(text: string): Promise<void>; close(): Promise<void>}>;
}
interface PickerWindow {
  showOpenFilePicker?: (options: object) => Promise<FileHandle[]>;
  showSaveFilePicker?: (options: object) => Promise<FileHandle>;
}
const pickerOptions = {suggestedName: 'lynter.yaml', types: [{description: 'YAML', accept: {'text/yaml': ['.yaml', '.yml']}}]};

type Mode = 'guided' | 'free';
interface Step {id: string; title: string; intro: string; body: ReactNode}

/**
 * The config builder. `site` is the origin that docs links point at: empty on
 * lynter.dev, the full origin in the standalone file that runs from disk.
 */
export function Builder({site = ''}: {site?: string}) {
  const {ready} = useEngine();
  const [text, setText] = useState(EMPTY_CONFIG);
  const [mode, setMode] = useState<Mode>('guided');
  const [stepId, setStepId] = useState('start');
  const [hydrated, setHydrated] = useState(false);
  const [filter, setFilter] = useState('');
  const [pending, setPending] = useState<string | null>(null);
  const [newScope, setNewScope] = useState('');
  const [copied, setCopied] = useState(false);
  const [fileNote, setFileNote] = useState('');
  const [fileName, setFileName] = useState('');
  const handle = useRef<FileHandle | null>(null);
  const upload = useRef<HTMLInputElement>(null);

  // Restore the draft once, then save every change. Storage can be unavailable; the page still works.
  useEffect(() => {
    try {
      const saved = JSON.parse(window.localStorage.getItem(draftSlot()) ?? 'null') as {text?: string; mode?: Mode; step?: string} | null;
      if (saved?.text) setText(saved.text);
      if (saved?.mode === 'guided' || saved?.mode === 'free') setMode(saved.mode);
      if (saved?.step) setStepId(saved.step);
    } catch {
      // A private window or blocked storage: start fresh.
    }
    setHydrated(true);
  }, []);
  useEffect(() => {
    if (!hydrated) return;
    try {
      window.localStorage.setItem(draftSlot(), JSON.stringify({text, mode, step: stepId}));
    } catch {
      // Nothing to do: the draft lives only in this tab.
    }
  }, [hydrated, text, mode, stepId]);

  const parsed = useMemo(() => parse(text), [text]);
  // The form keeps showing the last text that parsed while the YAML pane holds a syntax error.
  const lastGood = useRef<Record<string, unknown>>({});
  if (parsed.data) lastGood.current = parsed.data;
  const data = parsed.data ?? lastGood.current;
  const problems = useMemo(() => (parsed.error ? [] : validateConfig(text)), [text, parsed.error, ready]);
  const errorCount = problems.filter((p) => p.severity === 'error').length + (parsed.error ? 1 : 0);

  // With a file open, write each valid change straight back to it.
  useEffect(() => {
    const target = handle.current;
    if (!target || !hydrated) return undefined;
    if (errorCount) {
      setFileNote(copy.yaml.fileHeld(target.name));
      return undefined;
    }
    const timer = setTimeout(async () => {
      try {
        const out = await target.createWritable();
        await out.write(text);
        await out.close();
        setFileNote(copy.yaml.fileSaved(target.name, new Date().toLocaleTimeString()));
      } catch {
        setFileNote(copy.yaml.fileFailed(target.name));
      }
    }, 400);
    return () => clearTimeout(timer);
  }, [text, errorCount, hydrated, fileName]);

  /** Apply a change from the form. Ignored while the YAML pane has a syntax error, so nothing is lost. */
  const edit = useCallback((change: (current: string) => string) => setText((current) => (parse(current).error ? current : change(current))), []);

  const replaceWith = (next: string) => {
    const untouched = text.trim() === EMPTY_CONFIG.trim() || text.trim() === '';
    if (untouched || next === text) {
      setText(next);
      setPending(null);
    } else {
      setPending(next);
    }
  };

  const openFile = async () => {
    const picker = (window as unknown as PickerWindow).showOpenFilePicker;
    if (!picker) {
      upload.current?.click();
      return;
    }
    try {
      const [picked] = await picker(pickerOptions);
      const content = await (await picked.getFile()).text();
      handle.current = picked;
      setFileName(picked.name);
      setPending(null);
      setText(content || EMPTY_CONFIG);
    } catch {
      // The picker was dismissed.
    }
  };

  const saveFile = async () => {
    const picker = (window as unknown as PickerWindow).showSaveFilePicker;
    if (!picker) {
      download();
      return;
    }
    try {
      handle.current = await picker(pickerOptions);
      setFileName(handle.current.name);
    } catch {
      // The picker was dismissed.
    }
  };

  const download = () => {
    const url = URL.createObjectURL(new Blob([text], {type: 'text/yaml'}));
    const link = document.createElement('a');
    link.href = url;
    link.download = 'lynter.yaml';
    link.click();
    URL.revokeObjectURL(url);
  };

  const copyText = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 1600);
    } catch {
      setCopied(false);
    }
  };

  const ruleCards = (rules: CatalogRule[], profile: string | null) => rules.map((rule) => (
    <RuleCard
      key={`${profile ?? ''}/${rule.id}`}
      rule={rule}
      data={data}
      profile={profile}
      onEnable={(on) => edit((t) => setRuleEnabled(t, profile, rule.id, on))}
      onOption={(name: string, value: OptionValue) => edit((t) => setRuleOption(t, profile, rule, name, value))}
      onRemove={() => edit((t) => removeRule(t, profile, rule.id))}
    />
  ));

  const needle = filter.trim().toLowerCase();
  const matches = (rule: CatalogRule) => !needle || rule.id.includes(needle) || rule.description.toLowerCase().includes(needle);
  const scopes = profileNames(data);
  const scopeError = newScope ? profileNameError(newScope, scopes) : null;
  const enabledCount = catalog.filter((rule) => ruleState(data, null, rule).enabled).length;
  const renameMode = String(getIn(data, ['rename', 'mode']) ?? '');

  const steps: Step[] = [
    {
      id: 'start',
      title: copy.start.title,
      intro: copy.start.intro,
      body: (
        <>
          <div className="b-choices">
            <button type="button" className="b-choice" disabled={!ready} onClick={() => replaceWith(starterConfig() ?? EMPTY_CONFIG)}>
              <strong>{copy.start.starter}</strong><span>{copy.start.starterHint}</span>
            </button>
            <button type="button" className="b-choice" onClick={() => replaceWith(EMPTY_CONFIG)}>
              <strong>{copy.start.empty}</strong><span>{copy.start.emptyHint}</span>
            </button>
            <button type="button" className="b-choice" onClick={openFile}>
              <strong>{copy.start.open}</strong><span>{copy.start.openHint}</span>
            </button>
          </div>
          <label className="b-field">
            {copy.start.example}
            <select value="" onChange={(e) => { const found = examples.find((x) => x.id === e.target.value); if (found) replaceWith(found.config); }}>
              <option value="">{playground.exampleLabel}</option>
              {examples.map((x) => <option key={x.id} value={x.id}>{x.title}</option>)}
            </select>
          </label>
          {pending !== null ? (
            <div className="b-confirm" role="alertdialog" aria-label={copy.start.replace}>
              <span>{copy.start.replace}</span>
              <button type="button" className="button primary small" onClick={() => { setText(pending); setPending(null); }}>{copy.start.confirm}</button>
              <button type="button" className="button quiet small" onClick={() => setPending(null)}>{copy.start.cancel}</button>
            </div>
          ) : null}
        </>
      ),
    },
    ...ruleGroups.map((group): Step => {
      const rules = catalog.filter((rule) => rule.group === group.id && matches(rule));
      return {
        id: group.id,
        title: group.title,
        intro: group.intro,
        body: (
          <>
            {rules.length ? ruleCards(rules, null) : <p className="b-empty">{copy.rules.none}</p>}
            <p className="b-more"><a href={`${site}/docs/rules-overview/`}>{copy.rules.docs}</a></p>
          </>
        ),
      };
    }),
    {
      id: 'scopes',
      title: copy.scopes.title,
      intro: copy.scopes.intro,
      body: (
        <>
          {scopes.length === 0 ? <p className="b-empty">{copy.scopes.empty}</p> : null}
          {scopes.map((name) => {
            const present = catalog.filter((rule) => ruleState(data, name, rule).present);
            const match = (key: string) => asList(getIn(data, ['profiles', name, 'match', key]));
            const setMatch = (key: string, next: string[]) => edit((t) => setIn(t, ['profiles', name, 'match', key], next.length ? next : undefined, 3));
            return (
              <section key={name} className="b-scope" aria-label={name}>
                <div className="b-rule-head">
                  <h3><code>{name}</code></h3>
                  <button type="button" className="b-link" onClick={() => edit((t) => setIn(t, ['profiles', name], undefined, 0))}>{copy.scopes.remove}</button>
                </div>
                <ListField label={copy.scopes.paths} value={match('path')} onChange={(next) => setMatch('path', next)} />
                <ListField label={copy.scopes.extensions} value={match('extension')} onChange={(next) => setMatch('extension', next)} />
                <ListField label={copy.scopes.tags} value={match('tag')} onChange={(next) => setMatch('tag', next)} />
                <h4>{copy.scopes.overrides}</h4>
                {ruleCards(present, name)}
                <label className="b-field">
                  {copy.scopes.addRule}
                  <select value="" onChange={(e) => { const id = e.target.value; if (id) edit((t) => setRuleEnabled(t, name, id, false)); }}>
                    <option value="">{copy.scopes.choose}</option>
                    {catalog.filter((rule) => !present.includes(rule)).map((rule) => <option key={rule.id} value={rule.id}>{rule.id}</option>)}
                  </select>
                </label>
              </section>
            );
          })}
          <form className="b-add" onSubmit={(e) => { e.preventDefault(); if (newScope && !scopeError) { edit((t) => addProfile(t, newScope)); setNewScope(''); } }}>
            <label className="b-field">
              {copy.scopes.name}
              <input className="mono" value={newScope} onChange={(e) => setNewScope(e.target.value.trim())} placeholder="work" spellCheck={false} aria-invalid={scopeError !== null} />
            </label>
            <button type="submit" className="button quiet" disabled={!newScope || scopeError !== null}>{copy.scopes.add}</button>
            {scopeError ? <p className="b-error" role="alert">{scopeError}</p> : null}
          </form>
          <p className="b-more"><a href={`${site}/docs/scopes-and-profiles/`}>{copy.scopes.docs}</a></p>
        </>
      ),
    },
    {
      id: 'files',
      title: copy.files.title,
      intro: copy.files.intro,
      body: (
        <>
          <label className="b-field">
            {copy.files.rename}
            <select value={renameMode} onChange={(e) => edit((t) => setIn(t, ['rename', 'mode'], e.target.value || undefined, 0))}>
              <option value="">{copy.files.unset}</option>
              {Object.entries(copy.files.renameModes).map(([value, label]) => <option key={value} value={value}>{label}</option>)}
            </select>
          </label>
          <ListField label={copy.files.folders} value={asList(getIn(data, ['ignore', 'folders']))} onChange={(next) => edit((t) => setIn(t, ['ignore', 'folders'], next.length ? next : undefined, 0))} />
          <ListField label={copy.files.filesIgnored} value={asList(getIn(data, ['ignore', 'files']))} onChange={(next) => edit((t) => setIn(t, ['ignore', 'files'], next.length ? next : undefined, 0))} />
        </>
      ),
    },
    {id: 'note-type-forms', title: sectionsCopy.noteTypes.title, intro: sectionsCopy.noteTypes.intro, body: <NoteTypesSection data={data} edit={edit} />},
    {id: 'templates', title: sectionsCopy.templates.title, intro: sectionsCopy.templates.intro, body: <TemplatesSection data={data} edit={edit} />},
    {id: 'code-checks', title: sectionsCopy.codeChecks.title, intro: sectionsCopy.codeChecks.intro, body: <CodeChecksSection data={data} edit={edit} />},
    {id: 'adapters', title: sectionsCopy.adapters.title, intro: sectionsCopy.adapters.intro, body: <AdaptersSection data={data} edit={edit} />},
    {id: 'rule-order', title: sectionsCopy.ruleOrder.title, intro: sectionsCopy.ruleOrder.intro, body: <RuleOrderSection data={data} edit={edit} />},
    {id: 'automations', title: sectionsCopy.automations.title, intro: sectionsCopy.automations.intro, body: <AutomationsSection data={data} edit={edit} />},
    {id: 'providers', title: sectionsCopy.providers.title, intro: sectionsCopy.providers.intro, body: <ProvidersSection data={data} edit={edit} />},
    {
      id: 'finish',
      title: copy.finish.title,
      intro: copy.finish.intro,
      body: (
        <>
          <p className="b-summary"><span className="chip">{copy.finish.enabled(enabledCount)}</span> <span className="chip">{copy.finish.scopes(scopes.length)}</span></p>
          <div className="actions">
            <button type="button" className="button primary" onClick={saveFile}>{copy.yaml.save}</button>
            <button type="button" className="button quiet" onClick={download}>{copy.yaml.download}</button>
            <button type="button" className="button quiet" onClick={copyText}>{copied ? copy.yaml.copied : copy.yaml.copy}</button>
          </div>
          <p>{copy.finish.next}</p>
          {site === '' ? (
            <>
              <h3>{copy.finish.offlineTitle}</h3>
              <p>{copy.finish.offline} <a href="/lynter-config-builder.html" download>{copy.finish.offlineLink}</a>.</p>
            </>
          ) : null}
          <p><a href={`${site}/docs/reference/`}>{copy.finish.reference}</a></p>
        </>
      ),
    },
  ];

  const index = Math.max(0, steps.findIndex((s) => s.id === stepId));
  const shown = mode === 'guided' ? [steps[index]] : steps;
  const isRuleStep = (id: string) => ruleGroups.some((g) => g.id === id);
  const go = (id: string) => {
    setStepId(id);
    if (mode === 'free') document.getElementById(`b-${id}`)?.scrollIntoView({block: 'start'});
    else window.scrollTo({top: 0});
  };

  return (
      <div className="wrap">
        <div className="page-head">
          <h1>{copy.title}</h1>
          <p className="lede">{copy.lede}</p>
          <fieldset className="b-mode">
            <legend>{copy.modeLabel}</legend>
            {(['guided', 'free'] as Mode[]).map((m) => (
              <label key={m} className={mode === m ? 'on' : undefined}>
                <input type="radio" name="b-mode" checked={mode === m} onChange={() => setMode(m)} />
                <strong>{m === 'guided' ? copy.guided : copy.free}</strong>
                <span>{m === 'guided' ? copy.guidedHint : copy.freeHint}</span>
              </label>
            ))}
          </fieldset>
        </div>

        <div className="builder">
          <nav className="b-steps" aria-label={copy.stepsLabel}>
            <ol>
              {steps.map((s, i) => (
                <li key={s.id}>
                  <button type="button" aria-current={mode === 'guided' && i === index ? 'step' : undefined} onClick={() => go(s.id)}>{s.title}</button>
                </li>
              ))}
            </ol>
          </nav>

          <div className="b-main">
            {mode === 'free' || isRuleStep(steps[index].id) ? (
              <label className="b-field b-filter">
                {copy.rules.search}
                <input type="search" value={filter} onChange={(e) => setFilter(e.target.value)} />
              </label>
            ) : null}
            {shown.map((s) => (
              <section key={s.id} id={`b-${s.id}`} className="b-step" aria-labelledby={`b-${s.id}-title`}>
                {mode === 'guided' ? <p className="b-progress">{copy.stepOf(index + 1, steps.length)}</p> : null}
                <h2 id={`b-${s.id}-title`}>{s.title}</h2>
                <p className="b-intro">{s.intro}</p>
                {s.body}
              </section>
            ))}
            {mode === 'guided' ? (
              <div className="b-pager">
                <button type="button" className="button quiet" disabled={index === 0} onClick={() => go(steps[index - 1].id)}>{copy.back}</button>
                <button type="button" className="button primary" disabled={index === steps.length - 1} onClick={() => go(steps[index + 1].id)}>{copy.next}</button>
              </div>
            ) : null}
          </div>

          <aside className="b-yaml pane" aria-label={copy.yaml.label}>
            <div className="pane-head">
              <label htmlFor="b-yaml">{copy.yaml.label}</label>
              {ready || parsed.error ? (
                <span className={errorCount ? 'config-state bad' : problems.length ? 'config-state warn' : 'config-state'}>
                  {errorCount ? playground.configProblems(errorCount) : problems.length ? playground.configWarnings(problems.length) : playground.configOk}
                </span>
              ) : null}
            </div>
            <textarea id="b-yaml" value={text} onChange={(e) => setText(e.target.value)} spellCheck={false} autoCapitalize="off" autoCorrect="off" aria-invalid={errorCount > 0} aria-describedby="b-yaml-notes" />
            <div id="b-yaml-notes">
              {parsed.error || problems.length ? (
                <ul className="config-problems" role="status" aria-live="polite">
                  {parsed.error ? <li className="error"><strong>{copy.yaml.syntax}</strong><span>{parsed.error}</span></li> : null}
                  {problems.map((p, i) => (
                    <li key={i} className={p.severity}>
                      <strong>{p.severity === 'error' ? playground.configErrorLabel : playground.configWarningLabel}</strong>
                      {p.line ? <span className="chip">{playground.configLine} {p.line}</span> : null}
                      <span>{p.path ? `${p.path}: ` : ''}{p.message}</span>
                    </li>
                  ))}
                </ul>
              ) : null}
              <p className="b-status" role="status">{fileNote || (hydrated ? copy.saved : '')} <span>{copy.yaml.hint}</span></p>
            </div>
            <div className="b-actions">
              <button type="button" className="button quiet small" onClick={openFile}>{copy.yaml.open}</button>
              <button type="button" className="button quiet small" onClick={saveFile}>{copy.yaml.save}</button>
              <button type="button" className="button quiet small" onClick={download}>{copy.yaml.download}</button>
              <button type="button" className="button quiet small" onClick={copyText}>{copied ? copy.yaml.copied : copy.yaml.copy}</button>
              <button type="button" className="button quiet small" onClick={() => { handle.current = null; setFileName(''); setFileNote(''); setText(EMPTY_CONFIG); setStepId('start'); }}>{copy.yaml.reset}</button>
            </div>
            <input
              ref={upload}
              type="file"
              accept=".yaml,.yml,text/yaml"
              hidden
              onChange={async (e) => {
                const file = e.target.files?.[0];
                if (file) setText((await file.text()) || EMPTY_CONFIG);
                e.target.value = '';
              }}
            />
          </aside>
        </div>
      </div>
  );
}
