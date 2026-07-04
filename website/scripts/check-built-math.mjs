// Fails if the *built* site contains any KaTeX render error.
//
// `check:math` is a fast pre-build lint, but its Markdown math extractor is
// necessarily looser than remark-math's real parser — e.g. it renders a glued
// or list-indented `$$\begin{aligned}…` block fine, while remark-math splits it
// and KaTeX (throwOnError:false) emits a red `katex-error` span into the HTML.
// This is the authoritative gate: it greps the actual deployed pages, so
// whatever a visitor would see broken, CI sees too. Requires `npm run build`
// first (run via `npm run check:built-math`, or `npm run verify:site`).
import {readFileSync, existsSync, readdirSync, statSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {dirname, resolve, join, relative} from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const buildRoot = resolve(here, '..', 'build');

if (!existsSync(buildRoot)) {
  console.error('check:built-math — build/ not found. Run `npm run build` first.');
  process.exit(1);
}

// Walk build/ for .html files (skip the JS bundles — the search index copies
// error strings out of the pages, which would double-report).
function htmlFiles(dir) {
  const out = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) out.push(...htmlFiles(p));
    else if (name.endsWith('.html')) out.push(p);
  }
  return out;
}

const bad = [];
for (const file of htmlFiles(buildRoot)) {
  const html = readFileSync(file, 'utf8');
  if (!html.includes('katex-error')) continue;
  // Pull the first error title for a helpful message.
  const m = html.match(/katex-error[^>]*title="([^"]*)"/);
  const msg = m
    ? m[1].replace(/&amp;/g, '&').replace(/&gt;/g, '>').replace(/&lt;/g, '<').replace(/^ParseError:\s*/, '')
    : '(unknown KaTeX error)';
  bad.push({file: relative(buildRoot, file), msg});
}

if (bad.length === 0) {
  console.log('check:built-math — no KaTeX render errors in the built pages ✓');
  process.exit(0);
}

console.error(`check:built-math — KaTeX render error(s) in ${bad.length} built page(s):`);
for (const b of bad) console.error(`  ${b.file}\n      ${b.msg.slice(0, 140)}`);
console.error(
  '\nUsually a display block with its $$ glued to content or indented in a list.' +
    ' Put $$ on their own lines at the block’s indentation.',
);
process.exit(1);
