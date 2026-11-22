import type {GetStaticProps} from 'next';
import {DocsPage} from '@/components/docs-page';
import {DOCS_INDEX, loadDoc, type DocPage} from '@/docs';

export const getStaticProps: GetStaticProps<{doc: DocPage}> = async () => ({props: {doc: await loadDoc(DOCS_INDEX)}});

export default function DocsIndex({doc}: {doc: DocPage}) {
  return <DocsPage doc={doc} />;
}
