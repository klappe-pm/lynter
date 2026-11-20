import Link from 'next/link';
import {Layout} from '@/components/layout';
import {notFound} from '@/content';

export default function NotFound() {
  return (
    <Layout title={notFound.title}>
      <div className="wrap page-head">
        <h1>{notFound.title}</h1>
        <p className="lede">{notFound.body}</p>
        <div className="actions" style={{marginTop: 24}}>
          <Link className="button primary" href="/">{notFound.action}</Link>
        </div>
      </div>
    </Layout>
  );
}
