import {useMemo, useState} from 'react';
import {profileChain, useEngine} from '@/engine';
import {home} from '@/content';
import {contractionsFor, scopeConfig, scopeTree} from '@/samples';

/** Folder tree; picking a folder shows the resolved profile chain from the real engine. */
export function ScopePicker() {
  const [selected, setSelected] = useState('work/projects/client');
  const {ready} = useEngine();
  const chain = useMemo(() => profileChain(scopeConfig, `${selected}/note.md`), [selected, ready]);
  const on = contractionsFor(chain);
  const copy = home.scope;

  return (
    <div>
      <ul className="tree" aria-label="Vault folders">
        {scopeTree.map((node) => (
          <li key={node.path}>
            <button
              type="button"
              aria-pressed={node.path === selected}
              onClick={() => setSelected(node.path)}
              style={{paddingLeft: 12 + node.depth * 22}}
            >
              {node.path.split('/').pop()}/
            </button>
          </li>
        ))}
      </ul>
      <dl className="resolution" aria-live="polite">
        <div>
          <dt>{copy.chainLabel}</dt>
          <dd className="chain">
            {chain.map((name, i) => (
              <span key={name}>{i > 0 ? <span className="sep" aria-hidden="true">{'> '}</span> : null}{name}</span>
            ))}
          </dd>
        </div>
        <div>
          <dt>{copy.contractionsLabel}</dt>
          <dd className={on ? 'state-on' : 'state-off'}>{on ? copy.on : copy.off}</dd>
        </div>
      </dl>
    </div>
  );
}
