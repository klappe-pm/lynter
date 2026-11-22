import {Builder} from '@/components/builder';
import {Layout} from '@/components/layout';
import {builderCopy as copy} from '@/content';

export default function Configure() {
  return (
    <Layout title={copy.title} description={copy.lede}>
      <Builder />
    </Layout>
  );
}
