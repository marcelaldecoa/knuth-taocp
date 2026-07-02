# Hints — Module 13: Bitwise Tricks and Binary Decision Diagrams

## Stage 1: Ruler function, sideways addition, Gosper's hack

1. Every trick here rests on one two's-complement identity: `x & (-x)` isolates the rightmost 1-bit, because `-x = ~x + 1` flips all bits below-and-including the lowest 1 back into agreement. `smear_right` is its dual (`x | (x-1)`), and both `ruler` and Gosper's hack start by isolating that bit. You are forbidden from calling `trailing_zeros`/`count_ones` — the point is to re-derive them.
2. For `ruler`, multiply the isolated bit `2^a` by a de Bruijn constant so the top 6 bits become the `a`-th window of the de Bruijn cycle, then a 64-entry table inverts the window. For `sideways_addition`, use the SWAR divide-and-conquer: count within 2-bit fields, then fold pairs into nibbles, nibbles into bytes, and sum the bytes with one multiply. Gosper: isolate, carry, redistribute the vacated low ones.
3. `ruler`: `b = x & x.wrapping_neg(); TABLE[(b.wrapping_mul(0x03f79d71b4cb0a89) >> 58) as usize]`, building `TABLE` so entry `(C << a) >> 58` equals `a`; panic on `x==0` with "undefined". `sideways_addition` is Knuth's four masks (`0x5555.../0x3333.../0x0f0f...` then `*0x0101... >> 56`). Gosper: `u = x&-x; v = x+u; v | (((x^v)/u) >> 2)`; panic on `x==0` with "weight 0".

## Stage 2: Building reduced ordered BDDs

1. A BDD is *reduced* iff it obeys two invariants at every moment: no node with `lo == hi` (redundant test), and no two nodes sharing a `(var, lo, hi)` triple (hash-consing). Enforce them in one place — a node-maker `mk` — and canonicity follows: two functions are equal iff they are the *same* `Ref`. Boolean ops are memoized Shannon expansion (`apply`).
2. Store nodes in a `Vec<Node>` arena with `Ref(u32)` indices, sinks at 0 and 1 with a sentinel `var = u32::MAX` (so sinks sit *below* every real variable in the ordering). Keep a `unique: HashMap<(u32,Ref,Ref),Ref>` for hash-consing and a `memo: HashMap<(Op,Ref,Ref),Ref>` for apply. Give the three ops a shared `apply(op, f, g)` that differs only in its trivial cases.
3. `mk(v,lo,hi)`: `if lo==hi { return lo }`; else return the cached `Ref` or push a new node and record it. `apply` steps: (A1) handle sink identities per op; (A2) look up a *commutativity-normalized* key `if f.0<=g.0 {(op,f,g)} else {(op,g,f)}`; (A3) `v = min(level(f),level(g))`, split each operand into `(lo,hi)` if it tests `v` else `(f,f)`, recurse on the two halves, combine with `mk`; (A4) memoize. `not(f) = xor(f, one)`.

## Stage 3: Model counting and the ordering problem

1. Counting satisfying assignments is a single bottom-up pass over the DAG (Algorithm 7.1.4C), but a reduced BDD *skips* variables an edge doesn't test — and every skipped variable is free to be 0 or 1, so it doubles the count. The whole subtlety is paying for those skipped levels with the right power of two.
2. Recurse with memoization on `Ref`: a sink contributes 0 (⊥) or 1 (⊤); a node at level `v` with children `l, h` contributes `2^(level(l)-v-1)·c(l) + 2^(level(h)-v-1)·c(h)`, where a sink's level counts as `n_vars`. At the end multiply by `2^level(f)` to account for variables above the root. Use `u128` — counts can reach `2^n`.
3. Pin sink level to `n_vars` in a `level_at` helper. `count_rec(f)`: return 0/1 at sinks; else `c = (c_lo << (level_at(lo) - v - 1)) + (c_hi << (level_at(hi) - v - 1))`, memoized in a local `HashMap<Ref,u128>`. Public `count_models`: `count_rec(f) << level_at(f, n_vars)`. To *feel* the ordering problem, build the same OR-of-pairs function under adjacent vs. interleaved orders and compare `node_count`.

## Stage 4: BDDs at work: independent sets and queens

1. Both applications are the same recipe: express the constraint as a conjunction of tiny boolean functions, AND them into one BDD, then call your stage-3 `count_models`. An independent set forbids both endpoints of any edge; the n-queens board forbids any attacking pair and requires a queen in each row.
2. `independent_set_count`: fold `f = f AND not(x_u AND x_v)` over every edge, then `count_models(f, n)`. Sanity anchors: path `P_n` gives Fibonacci `F_{n+2}`, cycle `C_n` gives Lucas `L_n`, empty graph gives `2^n`, complete `K_n` gives `n+1`.
3. `queens_bdd_count(n)`: cell variable `idx(r,c) = r*n + c`; for every pair of distinct cells that attack (same row/col, or `|r1-r2| == |c1-c2|`) AND in `not(a AND b)`; for every row AND in an OR over its cells (at least one queen). Then `count_models(f, n*n)`. The BDD blows up fast — the tests stop at n=6 (answers 2, 10, 4 for n=4,5,6).
