import {Layout} from '@/components/layout';

export function MdPage({title, lede, html}: {title: string; lede: string; html: string}) {
  return (
    <Layout title={title} description={lede}>
      <div className="wrap">
        <div className="page-head">
          <h1>{title}</h1>
          <p className="lede">{lede}</p>
        </div>
        <div className="prose" dangerouslySetInnerHTML={{__html: html}} />
      </div>
    </Layout>
  );
}
