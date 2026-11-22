import {useRouter} from 'next/router';
import {useEffect, useState} from 'react';
import {Bench} from '@/components/bench';
import {Layout} from '@/components/layout';
import {playground} from '@/content';
import {examples, findExample} from '@/examples';

export default function Playground() {
  const {query, isReady, replace} = useRouter();
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  const id = typeof query.example === 'string' ? query.example : '';
  const example = id ? findExample(id) : undefined;
  return (
    <Layout title={playground.title} description={playground.lede}>
      <div className="wrap">
        <div className="page-head">
          <h1>{playground.title}</h1>
          <p className="lede">{playground.lede}</p>
        </div>
        <label className="path-field example-picker">
          {playground.exampleLabel}
          <select value={example?.id ?? ''} onChange={(e) => void replace(e.target.value ? `/playground/?example=${e.target.value}` : '/playground/', undefined, {shallow: true})}>
            <option value="">{playground.exampleDefault}</option>
            {examples.slice(1).map((e) => <option key={e.id} value={e.id}>{e.title}</option>)}
          </select>
        </label>
        {mounted && isReady ? <Bench key={example?.id ?? 'default'} showPath seed={example} /> : null}
      </div>
    </Layout>
  );
}
