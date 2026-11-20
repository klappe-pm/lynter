import fs from 'node:fs';
import path from 'node:path';
import type {GetStaticProps} from 'next';
import {marked} from 'marked';
import {CodeBlock} from '@/components/code-block';
import {Layout} from '@/components/layout';
import {docs} from '@/content';
import {CLI_BIN} from '@/samples';

const buildSteps = `git clone https://github.com/klappe-pm/lynter.git
cd lynter/plugin && npm ci && npm run build
cd .. && cargo install --path crates/lynter-cli`;

const cliSteps = `cd /path/to/notes
${CLI_BIN} init
${CLI_BIN} check .`;

/** Renders the repository's own lynter.yaml reference at build time, so docs never drift. */
export const getStaticProps: GetStaticProps<{reference: string}> = async () => {
  const file = path.join(process.cwd(), '..', '..', 'docs', 'lynter-yaml.md');
  const source = fs.readFileSync(file, 'utf8').replace(/linter\.yaml/g, 'lynter.yaml').replace(/^# .*\n/, '');
  return {props: {reference: await marked.parse(source)}};
};

export default function Docs({reference}: {reference: string}) {
  return (
    <Layout title={docs.title}>
      <div className="wrap">
        <div className="page-head"><h1>{docs.title}</h1></div>
        <div className="prose">
          <h2 id="install">{docs.installTitle}</h2>
          <p>{docs.installBody}</p>
          <CodeBlock code={buildSteps} label={docs.installTitle} />
          <h3>{docs.pluginTitle}</h3>
          <p>{docs.pluginBody}</p>
          <h3 id="cli">{docs.cliTitle}</h3>
          <CodeBlock code={cliSteps} label={docs.cliTitle} />
          <h2 id="reference">{docs.referenceTitle}</h2>
          <div dangerouslySetInnerHTML={{__html: reference}} />
        </div>
      </div>
    </Layout>
  );
}
