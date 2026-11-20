import {useRouter} from 'next/router';
import {useEffect, useState} from 'react';
import {Bench} from '@/components/bench';
import {Layout} from '@/components/layout';
import {playground} from '@/content';
import {findExample} from '@/examples';

export default function Playground() {
  const {query, isReady} = useRouter();
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
        {mounted && isReady ? <Bench key={example?.id ?? 'default'} showPath seed={example} /> : null}
      </div>
    </Layout>
  );
}
