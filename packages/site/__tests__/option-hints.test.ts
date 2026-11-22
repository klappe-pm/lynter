import fs from 'node:fs';
import path from 'node:path';

type Hint = {description: string; choices?: unknown};
type Rule = {id: string; options: Record<string, unknown>};

const src = path.join(__dirname, '..', 'src');
const catalog: Rule[] = JSON.parse(fs.readFileSync(path.join(src, 'rule-catalog.json'), 'utf8'));
const hints: Record<string, Record<string, Hint>> = JSON.parse(fs.readFileSync(path.join(src, 'option-hints.json'), 'utf8'));
const byId = new Map(catalog.map((r) => [r.id, r]));

describe('option hints', () => {
  test('only name rules the catalog has', () => {
    for (const id of Object.keys(hints)) expect(byId.has(id)).toBe(true);
  });

  test('every option except enabled has a short sentence', () => {
    for (const rule of catalog) {
      for (const name of Object.keys(rule.options)) {
        if (name === 'enabled') continue;
        const hint = hints[rule.id]?.[name];
        expect({rule: rule.id, name, hinted: Boolean(hint)}).toEqual({rule: rule.id, name, hinted: true});
        expect(typeof hint.description).toBe('string');
        expect(hint.description.length).toBeGreaterThan(0);
        expect(hint.description.length).toBeLessThanOrEqual(110);
        expect(hint.description.endsWith('.')).toBe(true);
      }
    }
  });

  test('no hint names an option the rule lacks', () => {
    for (const [id, options] of Object.entries(hints)) {
      const known = Object.keys(byId.get(id)!.options);
      for (const name of Object.keys(options)) expect({id, name, known: known.includes(name)}).toEqual({id, name, known: true});
    }
  });

  test('choices are clean lists that include the default', () => {
    for (const [id, options] of Object.entries(hints)) {
      for (const [name, hint] of Object.entries(options)) {
        if (hint.choices === undefined) continue;
        const where = `${id}.${name}`;
        const def = byId.get(id)!.options[name];
        expect(typeof def).toBe('string');
        expect(Array.isArray(hint.choices)).toBe(true);
        const choices = hint.choices as unknown[];
        expect({where, count: choices.length >= 2}).toEqual({where, count: true});
        for (const c of choices) expect(typeof c).toBe('string');
        expect({where, unique: new Set(choices).size === choices.length}).toEqual({where, unique: true});
        if (typeof def === 'string' && def !== '') expect({where, has: choices.includes(def)}).toEqual({where, has: true});
      }
    }
  });
});
