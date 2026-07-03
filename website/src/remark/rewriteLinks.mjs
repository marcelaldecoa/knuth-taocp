// Rewrites the repo's relative Markdown links to site routes at BUILD time only.
//
// The course/ and docs/ Markdown is read on two surfaces: github.com (where
// relative `../foo.md` links work) and this site (two separate docs plugin
// instances, where cross-instance relative links can't resolve). Rather than
// hard-code site URLs into the source (which would break the GitHub view), we
// rewrite them here so the source stays GitHub-correct and the site gets valid
// routes — letting us set onBrokenLinks: 'throw'.
//
// No unist-util-visit dependency — a tiny manual walk keeps the site's install
// lean.

const GH_BLOB = 'https://github.com/marcelaldecoa/knuth-taocp/blob/main';

// Root-level repo files that aren't part of the site → link to GitHub.
const ROOT_DOCS = /^(?:\.\.\/)+(SYLLABUS|CONVENTIONS|README|CONTRIBUTING)\.md(#[\w-]+)?$/;
// course/ ↔ handbook (docs/) cross-links.
const TO_HANDBOOK = /^(?:\.\.\/)+docs\/([\w.-]+)\.md(#[\w-]+)?$/;
// handbook → a module's README (its index route has no /README).
const TO_MODULE_INDEX = /^(?:\.\.\/)+course\/([\w-]+)\/README\.md(#[\w-]+)?$/;
// handbook → any other course page.
const TO_MODULE_PAGE = /^(?:\.\.\/)+course\/([\w-]+)\/([\w-]+)\.md(#[\w-]+)?$/;
// The old dashboard, superseded by the homepage course map.
const DASHBOARD = /(?:^|\/)dashboard\.html$/;

function rewrite(url) {
  if (typeof url !== 'string' || /^(https?:|mailto:|#|\/)/.test(url)) return url;

  let m;
  if ((m = url.match(TO_HANDBOOK))) return `/handbook/${m[1]}${m[2] ?? ''}`;
  if ((m = url.match(TO_MODULE_INDEX))) return `/course/${m[1]}/${m[2] ?? ''}`;
  if ((m = url.match(TO_MODULE_PAGE))) return `/course/${m[1]}/${m[2]}${m[3] ?? ''}`;
  if ((m = url.match(ROOT_DOCS))) return `${GH_BLOB}/${m[1]}.md${m[2] ?? ''}`;
  if (DASHBOARD.test(url)) return '/';
  return url;
}

function walk(node) {
  if (!node || typeof node !== 'object') return;
  if ((node.type === 'link' || node.type === 'definition') && node.url) {
    node.url = rewrite(node.url);
  }
  if (Array.isArray(node.children)) node.children.forEach(walk);
}

export default function rewriteLinks() {
  return (tree) => walk(tree);
}
