# Knuth expert agents

A retrieval layer over your **personal Knuth library** (the TAOCP PDFs and
*Concrete Mathematics* in Google Drive), wired into this repo as one skill plus
five volume-specialist subagents. It answers "§3.3.4, the spectral test", "which
volume covers dancing links", "prove Theorem 5.2.1H", etc. by fetching **only
the relevant section** — never a whole PDF — and it does so **without committing
any Drive file IDs or credentials**.

This is optional tooling: the course itself is self-contained. These agents are
for when you want to consult the source books alongside your implementation work.

## What is / isn't committed

- **Committed (safe knowledge):** volume keys, titles, section numbers, book
  pages, the navigator skill, the five expert agents. No file IDs, no folder ID,
  no credentials.
- **Local only (gitignored):** `config.local.json` (your Drive folder ID), the
  active `.mcp.json` (self-hosted server), the Drive OAuth credentials, and any
  `.knuth-cache/` of resolved IDs / extracted text.

## Layout (already in place in this repo)

```
knuth-taocp/
├── .claude/
│   ├── agents/
│   │   ├── knuth-fundamentals.md      # Vol 1
│   │   ├── knuth-seminumerical.md     # Vol 2
│   │   ├── knuth-sorting.md           # Vol 3
│   │   ├── knuth-combinatorial.md     # Vol 4A / 4B / Fascicle 7
│   │   └── knuth-concrete-math.md     # Concrete Mathematics
│   └── skills/knuth-navigator/
│       ├── SKILL.md                   # router: resolves title -> file ID at runtime
│       └── map.json                   # sections + book pages only, NO file IDs
├── config.local.example.json          # copy -> config.local.json, add your folder ID
├── .mcp.json.example                  # copy -> .mcp.json only for Option B (self-hosted)
├── githooks/{pre-commit, pre-commit.ps1}   # block committing IDs / credentials
└── docs/knuth-agents/{README.md, SETUP.md, INDEX.md}
```

## How file resolution works

`map.json` holds no Drive handles. At session start the navigator:

1. reads `KNUTH_DRIVE_FOLDER` from `config.local.json`,
2. calls the Drive MCP `search_files(parentId = <folder>)` to list the library,
3. matches each volume by **title** → its current file ID, cached for the session.

Benefits: nothing sensitive in git; survives re-uploads (IDs change, titles
don't); portable to anyone who has the same books in their own Drive.

The Drive `read_file_content` API only returns roughly the first ~90–100 book
pages, so deep sections are read from a local full-text cache
(`.knuth-cache/<volume_key>.txt`, gitignored) — see the navigator `SKILL.md`.

Everything up to the Drive network hop is deterministic and lives in `map.json`.
`tools/navigator_selftest.py` validates that part offline — map integrity plus
the full resolution path (query → volume → section → book page → expert handoff
→ the Drive title to search) — without a connector:

```
python3 docs/knuth-agents/tools/navigator_selftest.py            # checks + demo
python3 docs/knuth-agents/tools/navigator_selftest.py "3.3.4"    # resolve one query
```

The live fetch (resolve title → Drive file ID, read the section) then runs from
an interactive session where the connector can be authorized.

## Activate (three steps — the files are already here)

1. `cp config.local.example.json config.local.json` and paste your Drive folder ID.
2. Connect Google Drive (managed connector, recommended — see `SETUP.md`).
3. Turn on the safety hook: `git config core.hooksPath githooks`.

Full platform steps (macOS / Windows) and *where each value comes from* are in
[`SETUP.md`](SETUP.md); a human-readable section index is in [`INDEX.md`](INDEX.md).

## Volume → course-module map

Each expert also knows which course modules were built from its volume, so a
lookup can connect a book section to the module lesson, `reference/src/mNN_*.rs`,
and stage tests:

| Expert | Volume(s) | Course modules |
|---|---|---|
| `knuth-fundamentals` | Vol 1 | 01, 02, 03, 18 |
| `knuth-seminumerical` | Vol 2 | 04, 05, 12, 16, 19 |
| `knuth-sorting` | Vol 3 | 06, 07, 11, 15, 20 |
| `knuth-combinatorial` | Vol 4A / 4B / F7 | 08, 09, 10, 13, 14, 17, 21, 22 |
| `knuth-concrete-math` | Concrete Mathematics | companion (`docs/concrete-mathematics.md`) |

## Notes / verify

- Anchor fetches on the **section heading**, not the page number (the PDF page
  differs from `book_page` by the volume's front-matter offset).
- Skill/subagent frontmatter and `.mcp.json` schema drift over time — verify
  against current Claude Code docs if something doesn't load.
- `map.json` carries book-page hints for every volume except a few page-less
  entries in Vol 4B (the `MPR` preliminaries) and Fascicle 7 (the 7.2.2.3
  constraint-satisfaction sub-sections), where `book_page` is `null` — the
  navigator anchors on the heading anyway, so the hint is optional.
