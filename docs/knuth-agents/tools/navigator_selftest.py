#!/usr/bin/env python3
"""Offline self-test and lookup demo for the knuth-navigator skill.

The navigator's live step (resolving a volume title to a Drive file ID and
fetching a section) needs the Google Drive connector and therefore an
interactive Claude Code session. Everything *up to* that network hop is
deterministic and lives in map.json — so this script validates that part and
demonstrates the full resolution path, without calling Drive.

What it does:
  1. INTEGRITY  — schema, unique section ids, agent coverage, page sanity.
  2. RESOLVE    — turn a query ("3.3.4", "spectral test", "which volume has
                  dancing links") into {volume, section, book_page, expert,
                  and the Drive title the navigator would search for next}.

Usage:
    python3 navigator_selftest.py            # integrity checks + demo queries
    python3 navigator_selftest.py "5.2.2"    # resolve a single query
    python3 navigator_selftest.py --quiet    # integrity only, exit code is the result

Exit code is non-zero if any integrity check fails, so it doubles as CI glue.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

MAP_PATH = Path(__file__).resolve().parent.parent.parent.parent / ".claude" / "skills" / "knuth-navigator" / "map.json"

SECTION_ID = re.compile(r"^\d+(?:\.\d+)*[a-z]?$|^MPR$", re.IGNORECASE)


def load_map() -> dict:
    with MAP_PATH.open(encoding="utf-8") as fh:
        return json.load(fh)


# ---------------------------------------------------------------------------
# 1. Integrity
# ---------------------------------------------------------------------------
def check_integrity(m: dict) -> list[str]:
    errors: list[str] = []
    warnings: list[str] = []

    volumes = m.get("volumes", {})
    agents = m.get("agents", {})
    if not volumes:
        errors.append("no volumes in map")
        return errors

    # every volume is well formed
    for key, vol in volumes.items():
        for field in ("title", "match_title", "sections"):
            if not vol.get(field):
                errors.append(f"volume '{key}' missing '{field}'")
        seen: set[str] = set()
        last_page = 0
        for sec in vol.get("sections", []):
            sid = sec.get("id")
            if not sid:
                errors.append(f"volume '{key}' has a section with no id")
                continue
            if sid in seen:
                errors.append(f"volume '{key}' duplicate section id '{sid}'")
            seen.add(sid)
            page = sec.get("book_page")
            if page is not None:
                if not isinstance(page, int) or page <= 0:
                    errors.append(f"{key} §{sid}: book_page must be a positive int, got {page!r}")
                elif page < last_page:
                    warnings.append(f"{key} §{sid}: book_page {page} < previous {last_page} (out of reading order?)")
                else:
                    last_page = page

    # agent coverage: every volume owned by exactly one agent, no dangling refs
    owned: dict[str, list[str]] = {}
    for agent, spec in agents.items():
        for vk in spec.get("vols", []):
            if vk not in volumes:
                errors.append(f"agent '{agent}' references unknown volume '{vk}'")
            owned.setdefault(vk, []).append(agent)
    for vk in volumes:
        holders = owned.get(vk, [])
        if not holders:
            errors.append(f"volume '{vk}' has no expert agent")
        elif len(holders) > 1:
            errors.append(f"volume '{vk}' claimed by multiple agents: {holders}")

    for w in warnings:
        print(f"  ! warning: {w}")
    return errors


# ---------------------------------------------------------------------------
# 2. Resolution
# ---------------------------------------------------------------------------
def agent_for_volume(m: dict, vk: str) -> str | None:
    for agent, spec in m.get("agents", {}).items():
        if vk in spec.get("vols", []):
            return agent
    return None


def resolve(m: dict, query: str) -> list[dict]:
    """Resolve a query to zero or more sections, richest match first."""
    q = query.strip()
    hits: list[dict] = []

    def record(vk: str, sec: dict, why: str) -> None:
        hits.append(
            {
                "volume_key": vk,
                "match_title": m["volumes"][vk]["match_title"],
                "volume_title": m["volumes"][vk]["title"],
                "id": sec["id"],
                "title": sec["title"],
                "book_page": sec.get("book_page"),
                "expert": agent_for_volume(m, vk),
                "why": why,
            }
        )

    # exact section-id match (strip a leading § if present)
    qid = q.lstrip("§ ").strip()
    if SECTION_ID.match(qid):
        for vk, vol in m["volumes"].items():
            for sec in vol["sections"]:
                if sec["id"].lower() == qid.lower():
                    record(vk, sec, f"exact section id {sec['id']}")
        if hits:
            return hits

    # substring match on section titles, then volume titles
    ql = q.lower()
    for vk, vol in m["volumes"].items():
        for sec in vol["sections"]:
            if ql in sec["title"].lower():
                record(vk, sec, f"title contains '{q}'")
    if hits:
        return hits
    for vk, vol in m["volumes"].items():
        if ql in vol["title"].lower():
            # point at the volume's first (top) section as the anchor
            record(vk, vol["sections"][0], f"volume title contains '{q}'")
    return hits


def show(hit: dict) -> None:
    page = f"p.{hit['book_page']}" if hit["book_page"] is not None else "p.— (anchor on heading)"
    print(f"  §{hit['id']}  {hit['title']}")
    print(f"     volume : {hit['volume_key']}  ({hit['volume_title']})")
    print(f"     cite   : Vol/{hit['volume_key']} §{hit['id']}, {page}")
    print(f"     expert : {hit['expert']}")
    print(f"     → navigator next step: search KNUTH_DRIVE_FOLDER for a file whose")
    print(f"       title matches \"{hit['match_title']}\", then anchor on the heading")
    print(f"       \"{hit['title']}\" in .knuth-cache/{hit['volume_key']}.txt")
    print(f"     (matched: {hit['why']})")


DEMO_QUERIES = [
    "3.3.4",              # exact id  -> spectral test
    "spectral test",      # title substring
    "dancing links",      # -> Vol 4B, expert handoff
    "5.2.2",              # quicksort's home
    "Bernoulli Numbers",  # Concrete Mathematics
    "7.2.2.3c",           # a Fascicle 7 sub-section (null page, heading anchor)
]


def main(argv: list[str]) -> int:
    quiet = "--quiet" in argv
    argv = [a for a in argv if a != "--quiet"]
    m = load_map()

    print(f"map: {MAP_PATH}")
    n = sum(len(v["sections"]) for v in m["volumes"].values())
    print(f"volumes: {len(m['volumes'])}   sections: {n}   experts: {len(m.get('agents', {}))}\n")

    print("== integrity ==")
    errors = check_integrity(m)
    if errors:
        for e in errors:
            print(f"  FAIL: {e}")
        print(f"\n{len(errors)} integrity error(s).")
        return 1
    print("  OK: schema, unique ids, agent coverage, page sanity all pass.\n")

    if quiet:
        return 0

    queries = argv[1:] if len(argv) > 1 else DEMO_QUERIES
    print("== resolution ==")
    exit_code = 0
    for q in queries:
        print(f"\nquery: {q!r}")
        hits = resolve(m, q)
        if not hits:
            print("  (no match)")
            if len(argv) > 1:
                exit_code = 2
            continue
        for h in hits[:4]:
            show(h)
        if len(hits) > 4:
            print(f"  … and {len(hits) - 4} more")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
