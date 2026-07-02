# Walkthrough — Module 17: ZDDs and Exact Covering with Colors

Read this AFTER a stage is green — it explains how the reference solution is
built and why.

## Stage 1: the `Zdd` arena: sinks, `single`, queries, `union`

The whole ZDD lives in one place: `mk`. Its first line, `if hi == Ref(0) {
return lo }`, is the entire difference from a BDD — and note what it does
*not* do: there is no `if lo == hi { return lo }`. A ZDD node with LO = HI is
kept, because it means "v is optional." Everything downstream (canonicity,
sparsity, the no-2^skip counting) is a consequence of that one asymmetry.
The arena seeds the two sinks with a closure so both get the sentinel var
`u32::MAX`, which is what lets `level` compare any node against a sink as
"sinks sit below every variable" — the top-variable recursions rely on that
ordering without a special case.

The queries are each shaped by the semantics. `count_sets` is the naked
c(LO)+c(HI) with *no* 2^skip factor, because a skipped variable in a ZDD is
absent (one possibility), not free (two) — the memo makes it O(Z(f)).
`contains_set` sorts and dedups the input, then walks: the decisive line is
`if i < s.len() && s[i] < n.var { return false }` — a wanted element smaller
than the current node's variable was zero-suppressed away, so no member below
contains it, answered in O(1). `union`'s design is the template for stage 2:
trivial cases first (∅ ∪ g = g, f ∪ f = f via the `f == g` check), a
commutative memo key (`if f.0 <= g.0`), then expand on the smaller top
variable — and when only f tests v, the recursion is `mk(v, union(f_lo, g),
f_hi)` precisely because g contributes nothing to the v-level.

## Stage 2: the family algebra

All three operations share the shape of `union` but diverge exactly where the
set semantics diverge, and the reference makes each divergence a one-liner
you can read against §2's table. `intersect`: when only f tests v, the answer
is `intersect(f_lo, g)` — f's entire HI branch dies, because no set of g
contains v so nothing in f's "contains v" side can survive the intersection.
`diff`: the mirror asymmetry — when only f tests v, `mk(v, diff(f_lo, g),
f_hi)` keeps f's HI (g cannot cancel a set g doesn't reach), but when only g
tests v it is `diff(f, g_lo)` (g's HI is idle). The load-bearing detail is the
memo key: union/intersect/join normalize `(f, g)` because they are
commutative, but `diff`'s key is `(Op::Diff, f, g)` in order — a commutative
key here would return f\g when you asked for g\f, and the semiring-law tests
would fail while the brute-force mirror still passed (the classic bug the
lesson warns about).

`join` is the richest and the one idiom most worth internalizing. Both-test-v
expands to four sub-joins distributed over the splits, and because v·v = v the
three that contribute a v collapse into one HI via two `union` calls:
`h = union(union(join(f_hi,g_hi), join(f_hi,g_lo)), join(f_lo,g_hi))`. That
`join` *calls* `union` is why the memo is keyed by an `Op` tag and shared
across operations — the recursion is genuinely mutually recursive, and one
table serves all of it.

## Stage 3: `matchings_zdd`, `independent_sets_zdd`

Both public functions are thin adapters over one private engine,
`conflict_free_count(n_vars, conflicts)`, which is the G1–G3 recipe verbatim:
build the power set P by folding `join(P, union(unit, single(v)))` over the
variables, then for each conflict fold `diff(fam, join(P, {{u,v}}))`, then
`count_sets`. The design win is that the two counters differ only in how they
compute `conflicts` and what `n_vars` means — `matchings_zdd` runs a nested
loop over edge *pairs* pushing a conflict when they share an endpoint (and
asserts each edge joins two distinct in-range vertices), while
`independent_sets_zdd` passes the edges themselves as the conflict list with
`n_vars = n_vertices`. Everything hard already happened in stage 2; this stage
is proof that a proved algebra lets you assemble a graph counter out of four
calls and get Fibonacci, Lucas, and telephone numbers for free.

The subtle correctness point the reference gets right: the "bad" family for a
conflict is `join(P, {{u,v}})`, i.e. *every* subset containing both u and v,
not just `{{u,v}}` itself — subtracting only the singleton pair would leave
larger violating sets in place. Using the full power set P as the multiplier
is what makes one `diff` per conflict remove *all* members violating it.

## Stage 4: `Xcc`

The reference is Module 09's four-way-ring dancing links with the color
machinery layered on so that the O(1)-undo discipline survives bit for bit.
The constructor encodes the primary/secondary split structurally: the root
ring threads only headers `1..=n_primary` (secondary headers are
horizontally self-linked, reached only through their vertical lists), so
`choose_item` physically cannot propose a secondary item — that is Knuth's N₁
boundary, realized as a ring topology instead of an index comparison. Each
node carries `color`: 0 for primary, c+1 for a secondary node with user color
c, −1 once purified.

The heart is the pair `hide`/`purify` and their inverses. `hide` and `unhide`
both guard with `if self.color[q] >= 0`, skipping purified nodes so that a
node marked −1 by a `purify` stays linked through an enclosing hide/unhide —
this is what fact 3 of §4 (LIFO exactness) needs to be true. `purify(p)` fixes
the item to color c: same-color nodes become −1 *but stay linked* (their
options are compatible and must remain choosable), different-color nodes are
`hide`d wholesale (unlinking their rows from every primary list so C5 can
never pick them again). `commit` dispatches on color — `cover` a primary,
`purify` a live secondary, do *nothing* on −1 (an equal-color commitment
already fixed the item, so there is nothing to undo either). The search is
Algorithm X with commit/uncommit substituted for cover/uncover, committing
rightward in C5 and uncommitting leftward in C6 so undo exactly reverses do;
`solve_all` restores the structure completely, which is why nearly every
stage-4 test solves twice and demands identical output.
