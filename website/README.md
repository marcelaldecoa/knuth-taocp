# Website

This website is built using [Docusaurus](https://docusaurus.io/), a modern static website generator.

## Installation

```bash
npm ci
```

## Local Development

```bash
npm run start
```

This command starts a local development server and opens up a browser window. Most changes are reflected live without having to restart the server.

## Build

```bash
npm run build
```

This command generates static content into the `build` directory and can be served using any static contents hosting service.

## Checks

The Docusaurus build enforces `onBrokenLinks: 'throw'`, so `npm run build`
already fails on any broken link or anchor in a rendered page. The hand-written
**Museum of Algorithms** exhibits under `static/` are copied verbatim and are
*not* covered by that check, so a separate guard validates them:

```bash
npm run check:museum   # validate museum links/assets against build/ (needs a build first)
npm run verify:site    # build, then the museum and built-math checks — the full local gate
```

`check:museum` resolves every `href`/`src` in the museum and brand-guide HTML
against the built tree (so cross-links into generated course pages count too),
and confirms every `museum/exhibit-*.html` link inside a course lesson points at
a real exhibit. The Pages workflow runs it after the build, before deploying.
`check:built-math` greps the built pages for KaTeX render errors; CI
additionally runs `check:math`, a fast pre-build lint of every `$…$`
expression.

## Progress bridge (course map)

The course map on the landing page can display your real grading record. Open
**"track your progress from ./grade"** above the map and paste the contents of
`.taocp/progress` — the plain-text file in the repo root that `./grade` appends
to each time a stage passes (one `<lab_crate>/<test_target>` line, e.g.
`lab-06-sorting/stage_02_quicksort`). Recognized lines light up their stage
pips and module rings and show an "N of 98 stages recorded" count; unknown
lines are ignored. Everything is parsed client-side in
`src/components/CourseMap/` and stored in `localStorage` under
`taocp.progress` — no network requests, nothing leaves the browser. "clear
imported record" removes it (your on-disk `.taocp/progress` is never touched).

## Deployment

Deployment is automatic: the Pages workflow
(`.github/workflows/pages.yml`) builds the site, runs the math and museum
checks, and publishes to GitHub Pages on every push to `main` (or via
`workflow_dispatch`). There is no manual deploy step.
