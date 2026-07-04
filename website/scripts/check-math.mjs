// Validates every KaTeX expression in the course/handbook Markdown.
//
// `rehype-katex` runs with `throwOnError: false`, so a malformed formula does
// not fail the Docusaurus build — it renders as red error text that's easy to
// miss. This lint parses the same `$…$` / `$$…$$` math out of the Markdown and
// actually renders each one with KaTeX (`throwOnError: true`), so a broken
// expression fails loudly. Fast (no site build); pairs with the build's own
// `onBrokenLinks` gate.
//
// Usage:
//   node scripts/check-math.mjs                       # all course + handbook docs
//   node scripts/check-math.mjs ../course/module-02-math/README.md   # one file
//
// Run via `npm run check:math`.
import katex from 'katex';
import {readFileSync, existsSync} from 'node:fs';
import {globSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {dirname, resolve, relative} from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, '../..');

const targets =
  process.argv.length > 2
    ? process.argv.slice(2)
    : [
        ...globSync('course/*/README.md', {cwd: repoRoot}),
        ...globSync('course/*/{WALKTHROUGH,exercises,hints}.md', {cwd: repoRoot}),
        ...globSync('docs/*.md', {cwd: repoRoot}),
      ].map((p) => resolve(repoRoot, p));

// Remove regions where `$` is literal, replacing them with equal-length blanks
// so line/column offsets are preserved for reporting.
function blank(match) {
  return match.replace(/[^\n]/g, ' ');
}
function stripLiterals(src) {
  return src
    .replace(/```[\s\S]*?```/g, blank) // fenced code blocks
    .replace(/~~~[\s\S]*?~~~/g, blank)
    .replace(/`[^`\n]*`/g, blank); // inline code spans
}

// Extract math spans. Display `$$…$$` first (may span lines), then inline
// `$…$` on the remaining single lines. Escaped `\$` is not a delimiter.
function extractMath(src) {
  const spans = [];
  let s = stripLiterals(src);
  s = s.replace(/\$\$([\s\S]+?)\$\$/g, (m, body, idx) => {
    spans.push({tex: body, display: true, index: idx});
    return blank(m);
  });
  const inline = /(^|[^\\$])\$([^$\n]+?)\$/g;
  let m;
  while ((m = inline.exec(s)) !== null) {
    spans.push({tex: m[2], display: false, index: m.index});
  }
  return spans;
}

function lineOf(src, index) {
  return src.slice(0, index).split('\n').length;
}

let filesChecked = 0;
let exprChecked = 0;
const problems = [];

for (const file of targets) {
  if (!existsSync(file)) {
    console.error(`check:math — no such file: ${file}`);
    process.exit(1);
  }
  const src = readFileSync(file, 'utf8');
  filesChecked++;
  for (const {tex, display, index} of extractMath(src)) {
    exprChecked++;
    try {
      katex.renderToString(tex, {displayMode: display, throwOnError: true, strict: false});
    } catch (e) {
      problems.push({
        file: relative(repoRoot, file),
        line: lineOf(src, index),
        tex: tex.trim().slice(0, 80),
        msg: String(e.message || e).replace(/^KaTeX parse error:\s*/, ''),
      });
    }
  }
}

if (problems.length === 0) {
  console.log(
    `check:math — ${exprChecked} KaTeX expressions across ${filesChecked} files all render ✓`,
  );
  process.exit(0);
}

console.error(`check:math — ${problems.length} broken KaTeX expression(s):`);
for (const p of problems) {
  console.error(`  ${p.file}:${p.line}  $${p.tex}$\n      ${p.msg}`);
}
process.exit(1);
