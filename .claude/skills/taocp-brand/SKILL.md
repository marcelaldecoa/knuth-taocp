---
name: taocp-brand
description: The TAOCP "digital editions" brand system and math-notation standard for the knuth-taocp repo. Palette (parchment/ink/plum + per-volume inks), the three typefaces, signature devices, the WCAG contrast law, and — critically — how to write math, formulas, and symbols so they render consistently in the accent, museum-style (KaTeX `$…$`, never raw Unicode or backtick pseudo-math). Use when creating or editing any course lesson, museum exhibit, website page, or stylesheet, or whenever styling math/formulas/symbols.
---

# TAOCP brand & notation

One coherent identity across the museum, the course website, and every page
descended from Knuth's masterwork. Make a screen feel like an **object off a
shelf** — a scholarly press setting a canonical text, not a web app in serifs.
Three principles: **restraint** (quiet parchment and ink; colour only where it
means something), **provenance** (colour, banners, volume blocks, and the
tracked author line are lifted from the printed covers), **precision** (a fixed
type scale, hairline rules, contrast that never drops below legibility).

The living sources this distills — keep them in sync if you change tokens:
- Visual guide: `website/static/brand/taocp-brand-guide.html`
- Site tokens & rules: `website/src/css/custom.css` (`--taocp-*`, `--v1…--v4c`)
- Vendored fonts: `website/static/museum/fonts/*.woff2`

---

## 1. Palette

**Neutrals** (the ground everything sits on — one warm parchment, one warm ink):

| Token | Hex | Role |
|---|---|---|
| Parchment | `#F4EDDD` | page ground |
| Ivory | `#FBF7EA` | raised surfaces · cards |
| Ink | `#1F1B16` | primary text · titles (14.7:1) |
| Muted | `#6B5D4F` | captions · metadata — never lighter (5.5:1) |

**Master accent:** Plum `#5C1D2A` — navbars, chrome, global links, the default
prose accent. One accent per view.

**Volume inks** — each "wing" carries the ink of its book, so a reader is
oriented by colour before reading a word. Two are *display-only* (too light for
text) and switch to a darker text-safe sibling:

| Wing / Vol. | Display | Text-safe | Token |
|---|---|---|---|
| I · Fundamental (1) | Petrol `#0F3642` | (same, 11:1) | `--v1` |
| II · Seminumerical (2) | Magenta `#7C024C` | (same, 9:1) | `--v2` |
| III · Sorting/Searching (3) | Orange `#E36D1B` ⚠ | Amber-ink `#A34A08` | `--v3` |
| IV · Combinatorial (4) | Green `#00634E` | (same, 6.2:1) | `--v4` |
| → 4C | Teal `#5AA8A4` ⚠ | Deep-teal `#245C58` | `--v4c` |

**Gold `#C6A664`** — decorative only (rules, foil edges, tags *on plum*). Never
text on parchment.

Dark mode ("cloth") lightens each ink to a pastel; the tokens in `custom.css`
already carry both modes. Colour-key by token (`var(--v2)`), never by raw hex,
so both themes follow.

### The contrast law (non-negotiable, WCAG AA vs parchment)

- Body & titles: **Ink** only.
- Captions/metadata: never lighter than **Muted**.
- Petrol, magenta, green: safe as text. **Orange and teal are forbidden as
  text** — use amber-ink / deep-teal. **Gold is never text** on parchment.
- Links: plum vs body is only 1.36:1, so **colour is never the only signal** —
  underline every in-prose link (`text-underline-offset: 3px`).

---

## 2. Typography

Three faces, three jobs. Vendored as woff2 — never link a font CDN.

- **Display** — *Cormorant Garamond* (700, italic for emphasis). Large and
  sparing: heroes, section heads, drop caps. `--taocp-serif` / `--display`.
- **Body** — *Spectral* (400/600, 400 italic). The reading voice, ~17px,
  line-height ~1.6, measure ≤ 70ch.
- **Utility** — *IBM Plex Mono* (400/500). Labels, metadata, code, data;
  small-caps eyebrows are mono + `letter-spacing: .1–.24em; text-transform:
  uppercase`.

Type scale is fixed; don't invent sizes. Give headings `text-wrap: balance`.

---

## 3. Signature devices

Reuse these and any page instantly reads as part of the work:

- **Classic-work banner** — a small tracked mono line between two hairline
  rules, tinted with the volume ink. For heroes and section openers.
- **Volume block** — tracked volume label → hairline → subtitle → italic
  edition line. The bibliographic tell.
- **Drop cap** — Cormorant, plum, on the opening paragraph of a section.
- **Small-caps eyebrow** — mono, tracked, volume-inked, above a heading.
- **Author line & colophon** — wide-tracked caps for attribution; a small
  colophon mark (❦) anchors a corner. Use your own mark, never a publisher's.

Prefer **hairline rules over boxes**; ivory cards with a 1px `--hair` border and
a volume-inked top edge when a container is needed.

---

## 4. Math, formulas & symbols — the notation standard

This is where course pages drift most. The website already renders math
beautifully — `remark-math` + `rehype-katex` are wired, and `custom.css` styles
**inline math in the plum accent** (the museum "pop") while keeping **display
math in ink** for readability. The job is to *use* it, consistently.

### The rule

1. **Real mathematics → KaTeX.** Inline with `$…$`; display (its own line,
   blank line before and after) with `$$…$$`. Never write mathematics as raw
   Unicode (`∑ ⌊ ≈ π`) or as backtick pseudo-code (`` `H_n` ``, `` `C(n,k)` ``)
   — those render as plain prose or as literal monospace, unhighlighted and
   without proper sub/superscripts.
2. **Code things → backticks.** Identifiers, function names, types, file paths,
   CLI commands, DIMACS/opcodes, literal keystrokes: `` `euclid_e` ``,
   `` `./grade 6` ``, `` `Vec<Node>` ``. These get the plum code chip.
3. **Algorithm step labels** (E1, E2, …) stay as **bold** in prose (`**E3**`)
   and verbatim in traces — they are Knuth's labels, not math.
4. **ASCII step-traces, algorithm listings, and multi-line derivations** stay in
   fenced ```text code blocks. Do **not** KaTeX-ify a trace table; its alignment
   is the point.
5. **Escape literal dollars** as `\$` so `remark-math` doesn't read them as
   delimiters — MMIX registers (`\$X`, `\$255`), shell vars, and the `\$2.56`
   reward. (Module 18 already does this; match it.)
6. **Don't over-convert.** A lone variable in flowing prose may be `$n$` for
   consistency, but don't wrap plain English words. Taste over zeal.

### Conversion cheat-sheet

| Prose / Unicode / backtick | KaTeX |
|---|---|
| `H_n`, `F_n`, `a_i` | `$H_n$`, `$F_n$`, `$a_i$` |
| `C(n,k)` / "n choose k" | `$\binom{n}{k}$` |
| `x^2`, `2^n` | `$x^2$`, `$2^n$` |
| `O(n log n)`, `Θ(n²)`, `Ω(n)` | `$O(n \log n)$`, `$\Theta(n^2)$`, `$\Omega(n)$` |
| `lg n`, `ln n`, `log_b` | `$\lg n$`, `$\ln n$`, `$\log_b$` |
| `⌊x⌋`, `⌈x⌉` | `$\lfloor x \rfloor$`, `$\lceil x \rceil$` |
| `∑_{k=1}^n`, `∏` | `$\sum_{k=1}^{n}$`, `$\prod$` |
| `√x`, `x²` | `$\sqrt{x}$`, `$x^2$` |
| `≤ ≥ ≠ ≈ ≡` | `\le \ge \ne \approx \equiv` |
| `∈ ∉ ⊆ ∪ ∩` | `\in \notin \subseteq \cup \cap` |
| `· × →` | `\cdot \times \to` |
| `α β γ φ π λ` | `\alpha \beta \gamma \varphi \pi \lambda` |
| `⋯` (mid-line dots) | `\cdots` (or `\ldots` on the baseline) |
| `a ≡ b (mod m)` | `$a \equiv b \pmod{m}$` |
| gcd, lcm, mod ops | `$\gcd$`, `$\operatorname{lcm}$`, `$a \bmod n$` |

Display example (note blank lines and `$$`):

```markdown
By Bézout's identity there exist integers $a,b$ with

$$a\,m + b\,n = \gcd(m,n).$$

When $\gcd(m,n)=1$ this reads $a\,m \equiv 1 \pmod n$.
```

### Verifying math renders

`rehype-katex` uses `throwOnError: false`, so a malformed expression **won't
fail the build** — it renders as red error text instead. Always verify:

```bash
cd website && npm run build
grep -rl 'katex-error' build/ && echo "FIX THESE" || echo "math clean"
```

A zero from that grep is the gate. Also eyeball a math-heavy page (`npm run
serve`) — KaTeX can render nonsense without erroring.

---

## 5. Do / Don't

**Do:** let plum lead and volume inks orient (one accent per view); reach for
amber-ink / deep-teal wherever text is coloured; keep parchment and ink warm;
hairline rules over boxes; underline links; write real math in `$…$`.

**Don't:** set orange, teal, or gold as body text; run several creams across the
properties; leave cool Docusaurus greys on a warm page; reproduce a publisher's
trademark; let colour be a link's only signal; write math as Unicode or in
backticks.
