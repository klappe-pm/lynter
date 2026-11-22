import type {GetStaticPaths, GetStaticProps} from 'next';
import {DocsPage} from '@/components/docs-page';
import {DOCS_INDEX, docSlugs, loadDoc, type DocPage} from '@/docs';

export const getStaticPaths: GetStaticPaths = async () => ({
  paths: docSlugs().filter((slug) => slug !== DOCS_INDEX).map((slug) => ({params: {slug}})),
  fallback: false,
});

export const getStaticProps: GetStaticProps<{doc: DocPage}> = async ({params}) => ({
  props: {doc: await loadDoc(String(params?.slug))},
});

export default function DocsSlug({doc}: {doc: DocPage}) {
  return <DocsPage doc={doc} />;
}
