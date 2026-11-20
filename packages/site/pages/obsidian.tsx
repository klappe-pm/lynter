import {MdPage} from '@/components/md-page';
import {mdPages} from '@/content';
import {mdProps} from '@/md';

const page = mdPages.obsidian;

export const getStaticProps = mdProps(page.slug);

export default function Page({html}: {html: string}) {
  return <MdPage title={page.title} lede={page.lede} html={html} />;
}
