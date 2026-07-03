// The "Handbook" sidebar — supporting material sourced from ../docs.
// Hand-ordered orientation → setup → reference (alphabetical autogen would
// scramble the intended reading order). dashboard.html is not Markdown, so the
// docs plugin ignores it.
import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  handbook: [
    {
      type: 'category',
      label: 'Orientation',
      collapsed: false,
      items: ['for-newcomers', 'why-knuth-matters'],
    },
    {
      type: 'category',
      label: 'Getting set up',
      collapsed: false,
      items: ['getting-started'],
    },
    {
      type: 'category',
      label: 'Reference',
      collapsed: false,
      items: ['toolkit', 'concrete-mathematics', 'glossary'],
    },
  ],
};

export default sidebars;
