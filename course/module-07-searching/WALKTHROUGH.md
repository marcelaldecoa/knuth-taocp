# Walkthrough — Module 07: Searching

Read this AFTER a stage is green — it explains how the reference solution is
built and why.

## Stage 1: Binary search

The reference keeps the plain `binary_search` a one-liner over
`binary_search_comparisons`, so the counted and uncounted versions can never
disagree — the same idiom used throughout the course. It follows Knuth's 1-based
indices literally (`K_i` is `a[i-1]`), which keeps the code aligned with the
book's step labels and, more subtly, sidesteps the notorious `(l + u)` overflow
bug: the comment notes that an `i64` slice can hold fewer than `2^61` elements,
so `l + u` cannot overflow `usize`. The three-way comparison is genuinely
three-way — `<`, `>`, then equal — so a hit is detected at the earliest possible
moment rather than after the range collapses. The prettiest touch is the failure
return: when the loop exits at `u < l`, everything below `l` is `< key` and
everything above is `> key`, so `l - 1` (0-based) is exactly the sorted
insertion point, matching `<[T]>::binary_search`. That enrichment costs nothing
and is what makes the miss case as useful as the hit.

## Stage 2: Binary search trees

The `Bst` uses an index-based arena — `Vec<BstNode>` with `usize` links and
`usize::MAX` as the null sentinel Λ — which is both faithful to Knuth's MIX link
fields and more idiomatic (and cache-friendly) than `Rc<RefCell<...>>`. Deleted
slots are pushed onto a `free` list and reused by `alloc`, so long insert/delete
streams don't leak arena slots. The delicate code is `delete`: it first locates
the doomed node `T` *and* the parent link pointing at it (parent + side, or
"root" when parent is null), then handles the three structural cases, the hard
one being two children — replaced by the symmetric successor found by descending
left from the right child. Writing deletion against the arena (rehooking `usize`
indices) rather than owned subtrees is what makes all four shapes uniform. Both
`inorder` and `height` use an **explicit stack**, not recursion: the tests build
sorted-input vines thousands of nodes deep, and a recursive traversal would blow
the call stack — a real bug the design avoids. Height is measured in edges, so a
lone node is height 0 and the sorted-input vine of 100 keys is height 99.

## Stage 3: Balanced trees (AVL)

This is the module's hardest reference, and its single best idea is the
`link`/`set_link` pair parameterized by `a ∈ {−1, +1}`: writing every rotation in
terms of `LINK(a, ·)` and `LINK(−a, ·)` collapses Knuth's four mirror-image cases
(LL/RR single, LR/RL double) into just two code paths, with `−a` producing the
mirror for free. Algorithm A walks down tracking `S` (the deepest node with a
nonzero balance factor — the only place rebalancing can be needed, because
everything below it was perfectly balanced) and `T` (its parent), so after
inserting it rebalances in `O(lg n)` with at most one rotation and no recursion.
The balance-factor updates along the path from `S` to the new leaf rely on those
nodes all having had factor 0 (by the choice of `S`), and the A9 double-rotation
sets the new factors from `B(P)` before the rotation via the three-way case
`(−a,0)/(0,0)/(0,a)`. The correctness net is `is_balanced`, which ignores the
stored `bal` fields entirely and recomputes every subtree height from scratch in
an iterative post-order, checking both the AVL invariant *and* that each stored
factor matches reality — so a wrong rotation or a stale balance factor is caught
immediately instead of silently corrupting later inserts.

## Stage 4: Hashing with open addressing

Both tables use `Vec<Option<u64>>` and enforce `N <= M − 1` (one slot always
empty), which is what guarantees an unsuccessful search hits a `None` and stops
rather than looping forever — a correctness property, not a tuning knob. The
constructors `assert!` that `M` is a prime `>= 3`, because the division method
`h(K) = K mod M` inherits the structure of `M`'s factors: a composite modulus
lets arithmetic-progression keys collide onto `M/gcd` slots, so the reference
refuses one with a pinned "prime" message. `LinearProbe` advances with the
decreasing sequence `i-1 mod M`; its `probes_for` returns Knuth's quantity `C`,
counting the final probe (the hit, or the empty slot proving absence).
`DoubleHash` computes a *key-dependent* step `c = 1 + (K mod (M−2))` with
`1 <= c <= M−2`; because `M` is prime, `gcd(c, M) = 1`, so the probe sequence
visits every slot and two keys sharing `h1` almost never march in lockstep —
that is what breaks the primary clustering that makes linear probing's
unsuccessful search blow up like `1/(1−α)²`. The three shared methods (`insert`,
`contains`, `probes_for`) walk the identical probe sequence, so a search always
retraces the path insertion took.
