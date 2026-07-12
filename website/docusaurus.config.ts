import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import remarkMath from 'remark-math';
import rehypeKatex from 'rehype-katex';
import rewriteLinks from './src/remark/rewriteLinks.mjs';

// Three oneLight token colours fall below WCAG AA (4.5:1) on the light code
// background (#fafafa): the variable/operator/function blue, the string/
// punctuation green, and the comment grey. Darken each one's *lightness* at the
// theme source — hue/saturation unchanged, so the palette still reads as
// oneLight — so every code block clears AA with no per-token !important hacks.
const CODE_CONTRAST_FIXES: Record<string, string> = {
  'hsl(221, 87%, 60%)': 'hsl(221, 87%, 42%)', // variable / operator / function
  'hsl(119, 34%, 47%)': 'hsl(119, 34%, 34%)', // string / punctuation / builtin
  'hsl(230, 4%, 64%)': 'hsl(230, 4%, 42%)', // comment
  'hsl(35, 99%, 36%)': 'hsl(35, 99%, 30%)', // class-name / number / constant
};
function accessibleCodeTheme(theme: typeof prismThemes.oneLight) {
  return {
    ...theme,
    styles: theme.styles.map((entry) => {
      const fixed = entry.style?.color && CODE_CONTRAST_FIXES[entry.style.color];
      return fixed ? {...entry, style: {...entry.style, color: fixed}} : entry;
    }),
  };
}

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

// Shared Markdown pipeline: rewrite cross-instance/repo links to site routes,
// then render math. Applied to both docs instances.
const contentPlugins = {
  // rewriteLinks must run BEFORE Docusaurus's default Markdown-link resolver
  // (which would otherwise throw on the cross-instance .md links first).
  beforeDefaultRemarkPlugins: [rewriteLinks],
  remarkPlugins: [remarkMath],
  rehypePlugins: [rehypeKatex],
};

// Single source of truth for the site's base path: the KaTeX stylesheet href
// below is derived from it, so changing the base (fork, custom domain) can't
// silently ship unstyled math.
const BASE_URL = '/knuth-taocp/';

const config: Config = {
  title: 'The Art of Computer Programming',
  tagline: "Knuth's essence, implemented — a hands-on course in Rust",
  favicon: 'img/favicon.svg',

  future: {
    v4: true,
  },

  url: 'https://marcelaldecoa.github.io',
  baseUrl: BASE_URL,
  organizationName: 'marcelaldecoa',
  projectName: 'knuth-taocp',

  // Strict: cross-instance/repo links are normalized to site routes by the
  // rewriteLinks remark plugin, so any remaining broken link is a real error.
  onBrokenLinks: 'throw',
  onBrokenAnchors: 'throw',
  markdown: {
    // Parse .md as CommonMark (the course prose uses bare { } and < freely);
    // reserve MDX/JSX for .mdx lessons that embed interactive components.
    format: 'detect',
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        // Instance 1 (default): the main course, sourced from ../course.
        docs: {
          id: 'course',
          path: '../course',
          routeBasePath: 'course',
          sidebarPath: './sidebars.ts',
          editUrl:
            'https://github.com/marcelaldecoa/knuth-taocp/edit/main/',
          ...contentPlugins,
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  plugins: [
    // Instance 2: the supporting handbook, sourced from ../docs.
    [
      '@docusaurus/plugin-content-docs',
      {
        id: 'handbook',
        path: '../docs',
        routeBasePath: 'handbook',
        sidebarPath: './sidebars-handbook.ts',
        editUrl: 'https://github.com/marcelaldecoa/knuth-taocp/edit/main/',
        ...contentPlugins,
      },
    ],
  ],

  // Offline local search — builds a lunr index at build time and serves it from
  // the site itself; no Algolia, no CDN, no runtime network call (matches the
  // course's offline ethos). Indexes both docs instances (course + handbook).
  themes: [
    [
      require.resolve('@easyops-cn/docusaurus-search-local'),
      {
        hashed: true,
        // This site has no docs instance with the default id "default" — the
        // course preset uses id "course" and the handbook uses "handbook".
        // Point the search bar at the course instance so version lookup works.
        docsPluginIdForPreferredVersion: 'course',
        docsRouteBasePath: ['course', 'handbook'],
        indexBlog: false,
        highlightSearchTermsOnTargetPage: true,
      },
    ],
  ],

  // Apply the saved cover-accent theme before React hydrates, so the accent
  // never flashes from the oxblood default to the chosen volume on load.
  clientModules: ['./src/clientModules/themeInit.ts'],

  // Self-hosted KaTeX stylesheet (no CDN — the course never touches the network).
  stylesheets: [
    {
      href: `${BASE_URL}katex/katex.min.css`,
      type: 'text/css',
    },
  ],

  themeConfig: {
    image: 'img/docusaurus-social-card.jpg',
    // No light/dark split: every theme is a parchment-based "cover accent".
    // The classic light/dark toggle slot is repurposed (swizzled
    // src/theme/ColorModeToggle) into a TAOCP volume-cover palette picker, so
    // disableSwitch stays false to keep that navbar slot rendering. The page is
    // pinned to light and never follows the OS dark preference.
    colorMode: {
      defaultMode: 'light',
      disableSwitch: false,
      respectPrefersColorScheme: false,
    },
    navbar: {
      title: 'TAOCP',
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'course',
          docsPluginId: 'course',
          position: 'left',
          label: 'Course',
        },
        {
          type: 'docSidebar',
          sidebarId: 'handbook',
          docsPluginId: 'handbook',
          position: 'left',
          label: 'Handbook',
        },
        {
          // The Museum of Algorithms: self-contained single-file HTML exhibits
          // served from website/static/museum/. `pathname://` links straight to
          // the static file, bypassing the SPA router and strict link checker.
          to: 'pathname:///museum/',
          label: 'Museum',
          position: 'left',
        },
        {
          href: 'https://github.com/marcelaldecoa/knuth-taocp',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Course',
          items: [
            // The course map (homepage) is the real module index — a footer
            // can't enumerate 22 modules, and a lone "Module 01" read as a
            // broken list. Offer the map plus a clear entry point instead.
            {label: 'Course map', to: '/'},
            {label: 'Start the course →', to: '/course/module-01-algorithms/'},
          ],
        },
        {
          title: 'Handbook',
          items: [
            {label: 'New to Knuth?', to: '/handbook/for-newcomers'},
            {label: 'Getting started', to: '/handbook/getting-started'},
            {label: 'Glossary', to: '/handbook/glossary'},
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/marcelaldecoa/knuth-taocp',
            },
          ],
        },
      ],
      copyright: `Course material after Donald E. Knuth's <em>The Art of Computer Programming</em>. Built with Docusaurus.`,
    },
    prism: {
      theme: accessibleCodeTheme(prismThemes.oneLight),
      darkTheme: prismThemes.oneDark,
      additionalLanguages: ['rust', 'bash', 'toml'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
