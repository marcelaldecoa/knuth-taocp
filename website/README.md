# Website

This website is built using [Docusaurus](https://docusaurus.io/), a modern static website generator.

## Installation

```bash
yarn
```

## Local Development

```bash
yarn start
```

This command starts a local development server and opens up a browser window. Most changes are reflected live without having to restart the server.

## Build

```bash
yarn build
```

This command generates static content into the `build` directory and can be served using any static contents hosting service.

## Checks

The Docusaurus build enforces `onBrokenLinks: 'throw'`, so `npm run build`
already fails on any broken link or anchor in a rendered page. The hand-written
**Museum of Algorithms** exhibits under `static/` are copied verbatim and are
*not* covered by that check, so a separate guard validates them:

```bash
npm run check:museum   # validate museum links/assets against build/ (needs a build first)
npm run verify:site    # build, then run the museum check — the full local gate
```

`check:museum` resolves every `href`/`src` in the museum and brand-guide HTML
against the built tree (so cross-links into generated course pages count too),
and confirms every `museum/exhibit-*.html` link inside a course lesson points at
a real exhibit. The Pages workflow runs it after the build, before deploying.

## Deployment

Using SSH:

```bash
USE_SSH=true yarn deploy
```

Not using SSH:

```bash
GIT_USER=<Your GitHub username> yarn deploy
```

If you are using GitHub pages for hosting, this command is a convenient way to build the website and push to the `gh-pages` branch.
