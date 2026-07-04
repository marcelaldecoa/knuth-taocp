import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import remarkMath from 'remark-math';
import rehypeKatex from 'rehype-katex';
import rewriteLinks from './src/remark/rewriteLinks.mjs';

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

const config: Config = {
  title: 'The Art of Computer Programming',
  tagline: "Knuth's essence, implemented — a hands-on course in Rust",
  favicon: 'img/favicon.svg',

  future: {
    v4: true,
  },

  url: 'https://marcelaldecoa.github.io',
  baseUrl: '/knuth-taocp/',
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

  // Apply the saved cover-accent theme before React hydrates, so the accent
  // never flashes from the oxblood default to the chosen volume on load.
  clientModules: ['./src/clientModules/themeInit.ts'],

  // Self-hosted KaTeX stylesheet (no CDN — the course never touches the network).
  stylesheets: [
    {
      href: '/knuth-taocp/katex/katex.min.css',
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
      theme: prismThemes.oneLight,
      darkTheme: prismThemes.oneDark,
      additionalLanguages: ['rust', 'bash', 'toml'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
