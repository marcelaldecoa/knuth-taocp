# Hints — Module 17: ZDDs and Exact Covering with Colors

Graduated hints, three per stage, gentlest first. Read only as far as you
need; after a stage is green, read `WALKTHROUGH.md`.

## Stage 1: the `Zdd` arena: sinks, `single`, queries, `union`

1. This is Module 13's BDD arena with exactly *one* reduction rule changed:
   zero-suppression (never build a node with HI = ⊥) replaces the BDD's
   "elide LO = HI." Canonicity then makes family equality equal `Ref`
   equality — and the two sinks are as different as 0 and 1: ⊥ = ∅ has
   zero member sets, ⊤ = {∅} has one (the empty set).
2. Keep the arena a `Vec<Node>` with the two sinks first (sentinel var
   `u32::MAX`) and a unique table `HashMap`. `mk(v, lo, hi)` returns `lo`
   when `hi == ⊥`, otherwise hash-conses; do *not* collapse `lo == hi`.
   `single(v) = mk(v, ⊥, ⊤)`. `count_sets` is c(⊥)=0, c(⊤)=1,
   c=c(LO)+c(HI) memoized. `contains_set` walks from the root taking HI on
   a wanted variable, LO otherwise, and returns false the moment it skips
   *past* a wanted element. `union` is the gentlest recursion.
3. `fn mk(&mut self, v, lo, hi) { if hi == Ref(0) { return lo } … }`.
   `union`: trivial cases ∅∪g=g, f∪f=f; expand on the smaller top
   variable — if only f tests v, `mk(v, union(f_lo, g), f_hi)` because no
   member of g contains v; symmetric for g; if both, `mk(v, union(f_lo,
   g_lo), union(f_hi, g_hi))`.

## Stage 2: the family algebra

1. Each of `intersect`, `diff`, `join` is a memoized top-variable
   recursion; write out the split f = f₀ ∪ ({{v}} ⊔ f₁) and derive the
   base cases *before* coding. The subtle cases are when only one argument
   tests the top variable v — and diff is **not** commutative, so its memo
   key must keep (f, g) in order.
2. §2's table is the answer key. When only f tests v: intersect drops f's
   HI wholesale (`intersect(f_lo, g)`, since no set of g contains v); diff
   keeps it (`mk(v, diff(f_lo, g), f_hi)`, g can't cancel v-sets). When
   only g tests v: intersect is `intersect(f, g_lo)`, diff is
   `diff(f, g_lo)` (g's HI is idle). Join is the four-way distribution.
3. Join, both testing v: `LO = join(f_lo, g_lo)` and
   `HI = union(union(join(f_hi, g_hi), join(f_hi, g_lo)), join(f_lo, g_hi))`
   — v enters a union if either side supplies it. When only f tests v it
   collapses to `mk(v, join(f_lo, g), join(f_hi, g))`. Share one memo
   `HashMap` keyed by an `Op` tag; normalize the key for commutative ops,
   keep order for diff.

## Stage 3: `matchings_zdd`, `independent_sets_zdd`

1. Both are pure clients of stages 1–2: "count subsets containing no
   conflicting pair." Only the encoding differs — for matchings the ZDD
   variables are the *edge indices* and two edges conflict when they share
   an endpoint; for independent sets the variables are the *vertices* and
   each edge is one conflicting pair.
2. The G1–G3 recipe: G1 build the power set P as the join over all
   variables of ({∅} ∪ {{v}}); G2 for each conflicting pair (u, v) do
   F ← F \ (P ⊔ {{u, v}}), which deletes exactly the members containing
   both; G3 return `count_sets(F)`. Trace P₃ (edges 01, 12, one conflict)
   by hand first — the answer is 3.
3. Power-set factor: `union(unit(), single(v))`. The bad family for a
   pair: `bad = join(P, join(single(u), single(v)))`, then
   `fam = diff(fam, bad)`. Build the conflict list first (matchings: nested
   loop over edge pairs testing shared endpoints; independent sets: the
   edges themselves), then feed `(n_vars, conflicts)` to the shared engine.

## Stage 4: `Xcc`

1. Algorithm 7.2.2.1C is Algorithm X (Module 09) with one new idea:
   secondary items covered *at most* once, compatible iff they share the
   same color. The −1 "purified" marker together with strict LIFO undo is
   what keeps commit and uncommit exact inverses — reversibility *is* the
   algorithm.
2. Make four changes to your Module 09 solver, in order: (1) the
   constructor rings only the *primary* headers with the root, so the
   choose-item loop never sees a secondary item; (2) every node gets a
   `color` field (0 primary / c+1 secondary / −1 purified) and
   `add_option` takes `(&[primary], &[(secondary, color)])`; (3) hide and
   unhide gain a `color >= 0` guard; (4) write purify/unpurify and
   commit/uncommit and substitute commit/uncommit for cover/uncover in the
   search's C5/C6.
3. `purify(p)` with c = color[p]: walk item i = col[p]'s vertical list
   downward — `if color[q] == c { if q != p { color[q] = -1 } } else {
   hide(q) }`. `commit(j)`: match color[j] — `0 => cover(col[j])`,
   `c if c>0 => purify(j)`, `_ => {}` (purified: nothing to do or undo).
   In C5 commit the row walking rightward (`j = right[r]`); in C6 uncommit
   walking leftward (`j = left[r]`) so undo mirrors do; unpurify walks the
   item list the opposite direction from purify, restoring −1 to c and
   unhiding the others.
