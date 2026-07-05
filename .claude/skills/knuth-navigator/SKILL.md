---
name: knuth-navigator
description: >
  Use whenever a question concerns Knuth's TAOCP (Volumes 1, 2, 3, 4A, 4B,
  Fascicle 7) or Concrete Mathematics — a specific section like "§3.3.4 the
  spectral test", an algorithm, theorem, exercise, or "which volume covers X".
  Resolves the request to a section in map.json, resolves the volume to a live
  Drive file ID at runtime, and fetches ONLY that section. Never load full PDFs.
---

# Knuth Navigator

You are the retrieval layer over a personal Knuth library in Google Drive.
The committed map (`map.json`) holds only stable knowledge — volume keys,
titles, section numbers, book pages. It contains NO Drive file IDs.

## Resolving a volume to a file (runtime)

Drive file IDs are never stored in the repo. Resolve them per session:

1. Read `KNUTH_DRIVE_FOLDER` from `config.local.json` (gitignored).
2. Call the Drive MCP `search_files` with `parentId = '<KNUTH_DRIVE_FOLDER>'`
   to list the library. Build a `title -> fileId` table for this session
   (optionally cache under `.knuth-cache/`, also gitignored).
3. Match the target volume via `map.json`'s `match_title` (e.g. "TAOCP Vol 2")
   against the returned file titles to get the current `fileId`.

This keeps the repo free of Drive handles and survives re-uploads (IDs change,
titles don't).

## Fetching a section

The Drive `read_file_content` API returns only the first ~90-100 book pages of a
volume, so it CANNOT reach most sections. Use a two-tier fetch:

**Tier 1 — local full-text cache (default, reaches every section).**
1. Resolve the request to `{volume_key, section_id, heading}` via `map.json`.
2. Ensure a local full-text copy exists under `.knuth-cache/<volume_key>.txt`
   (gitignored). If missing, build it once: download the volume bytes
   (`download_file_content` on the resolved fileId, or use your local PDF of the
   book) and extract text with a local tool (`pdftotext`, `pypdf`, or
   `pdfplumber`).
3. Search the FULL local text for the section heading (e.g. `SPECTRAL TEST` or
   `3.3.4`) and read from there to the next heading. No truncation.

**Tier 2 — Drive `read_file_content` (fast peek, early sections only).**
Fine for front-matter and roughly the first chapter of a volume when you just
need a quick look and the local cache isn't built. If the heading isn't in the
returned slice, the section is past the truncation point — fall back to Tier 1.

Anchor on the heading string, not the page number; `book_page` is only a hint,
and the PDF page differs from it by the volume's front-matter offset. Cite
`Vol N §x.y.z, p.<book_page>`.

**Draft-volume citations.** In the two draft volumes the page sequences restart
at 1, so a `book_page` there is relative to that sequence — an entry may carry a
`page_note` spelling this out (e.g. Vol 4B's MPR is p.1 of the *preliminaries*,
distinct from the main body where 7.2.2 starts at p.30; Fascicle 7 restarts at 1
and is slated for Vol 4C). Include the qualifier when citing these. Entries
marked `"editorial": true` (the f7 `7.2.2.3a`–`e` tags) are navigation topics
inside their parent section, not Knuth-numbered sections — never cite a page for
them; anchor on the heading and cite the parent (`§7.2.2.3`).

## Rules

- One section at a time. Multi-section spans -> fetch in sequence.
- Deep dives / cross-volume synthesis -> hand off to the matching expert
  subagent (`map.json` -> `agents`) so each runs in its own context.
- Exercises live in each chapter's "Answers to Exercises"; fetch separately.
- On a Drive auth error, tell the user to re-authorize the connector; don't
  guess content.
- Never write a Drive file ID into a tracked file.

## In this repo

This library sits alongside the `knuth-taocp` course, which was built *from*
these volumes. When a lookup supports a learner's implementation work, connect
the book section to the matching course module — its lesson
(`course/module-NN-*/README.md`), reference (`reference/src/mNN_*.rs`), and
stage tests. The five expert subagents in `.claude/agents/` (`knuth-fundamentals`,
`knuth-seminumerical`, `knuth-sorting`, `knuth-combinatorial`,
`knuth-concrete-math`) each own a volume and carry the module mapping.
