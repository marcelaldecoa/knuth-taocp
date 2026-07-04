// Link-checks the Museum of Algorithms (and the brand guide) against the built
// site.
//
// Docusaurus enforces `onBrokenLinks: 'throw'` for the pages it *renders*, but
// the museum exhibits live in `static/` as hand-written HTML that Docusaurus
// copies verbatim and never validates. This script closes that gap: it parses
// every built museum/brand HTML file, resolves each local href/src against the
// deployed tree in `build/`, and fails if any target is missing — so a renamed
// exhibit, a moved asset, or a dead cross-link into a course page can't
// silently ship. It also verifies the reverse direction: every
// `museum/exhibit-*.html` link inside a course lesson points at a real exhibit,
// so the course↔museum wiring can't drift either.
//
// Resolves against `build/` (not `static/`) so museum links into
// Docusaurus-generated routes like `/course/module-01-algorithms/` validate
// correctly. Requires `npm run build` first; run via `npm run check:museum`
// (or `npm run verify:site`, which builds then checks).
//
// Pure Node, no deps.
import {readFileSync, existsSync, statSync, readdirSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {dirname, resolve, join, normalize} from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const websiteRoot = resolve(here, '..');
const repoRoot = resolve(websiteRoot, '..');
const buildRoot = join(websiteRoot, 'build');
const BASE_URL = '/knuth-taocp/'; // must match docusaurus.config.ts `baseUrl`

if (!existsSync(buildRoot)) {
  console.error(
    'check:museum — build/ not found. Run `npm run build` first ' +
      '(or use `npm run verify:site`).',
  );
  process.exit(1);
}

const REF = /(?:href|src)\s*=\s*["']([^"']+)["']/gi;
const SKIP = /^(https?:|data:|mailto:|javascript:|#|\/\/)/i;

const problems = [];
let checked = 0;

/** Every .html file under static/museum and static/brand. */
function htmlFiles(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((f) => f.endsWith('.html'))
    .map((f) => join(dir, f));
}

const files = [
  ...htmlFiles(join(buildRoot, 'museum')),
  ...htmlFiles(join(buildRoot, 'brand')),
];

for (const file of files) {
  const html = readFileSync(file, 'utf8');
  const baseDir = dirname(file);
  for (const m of html.matchAll(REF)) {
    const raw = m[1].trim();
    if (!raw || SKIP.test(raw)) continue;
    const path = decodeURIComponent(raw.split('#')[0].split('?')[0]);
    if (!path) continue;
    checked++;
    let target;
    if (path.startsWith('/')) {
      // Absolute from the site root; strip the deployed baseUrl prefix.
      const rel = path.startsWith(BASE_URL)
        ? path.slice(BASE_URL.length)
        : path.replace(/^\//, '');
      target = join(buildRoot, rel);
    } else {
      target = normalize(join(baseDir, path));
    }
    let ok = existsSync(target);
    if (ok && statSync(target).isDirectory()) {
      ok = existsSync(join(target, 'index.html'));
    }
    if (!ok) {
      problems.push({
        file: file.replace(repoRoot + '/', ''),
        ref: raw,
        target: target.replace(repoRoot + '/', ''),
      });
    }
  }
}

// Reverse direction: course lessons must only link to museum exhibits that exist.
let lessonLinks = 0;
const courseDir = join(repoRoot, 'course');
if (existsSync(courseDir)) {
  for (const mod of readdirSync(courseDir)) {
    const readme = join(courseDir, mod, 'README.md');
    if (!existsSync(readme)) continue;
    const text = readFileSync(readme, 'utf8');
    for (const m of text.matchAll(/museum\/(exhibit-[0-9.]+-[a-z-]+\.html)/g)) {
      lessonLinks++;
      const exhibit = join(buildRoot, 'museum', m[1]);
      if (!existsSync(exhibit)) {
        problems.push({
          file: `course/${mod}/README.md`,
          ref: `museum/${m[1]}`,
          target: exhibit.replace(repoRoot + '/', ''),
        });
      }
    }
  }
}

if (problems.length === 0) {
  console.log(
    `check:museum — ${files.length} files, ${checked} local refs and ` +
      `${lessonLinks} lesson→museum links all resolve ✓`,
  );
  process.exit(0);
}

console.error(`check:museum — ${problems.length} broken reference(s):`);
for (const p of problems) {
  console.error(`  [${p.file}]  ${p.ref}  ->  ${p.target} (missing)`);
}
process.exit(1);
