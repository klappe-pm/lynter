import {useState} from 'react';
import {home} from '@/content';

/** Dark code panel with a copy button that confirms in place. */
export function CodeBlock({code, label}: {code: string; label: string}) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 1600);
    } catch {
      setCopied(false);
    }
  };
  return (
    <figure className="code" aria-label={label}>
      <pre><code>{code}</code></pre>
      <button type="button" className="button quiet small copy" onClick={copy} aria-live="polite">
        {copied ? home.cli.copied : home.cli.action}
      </button>
    </figure>
  );
}
