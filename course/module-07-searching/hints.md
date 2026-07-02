# Hints — Module 07: Searching

## Stage 1: Binary search

1. If the array is sorted, one three-way comparison halves the candidate range.
   Algorithm 6.2.1B keeps the invariant "if `key` is present it lies in
   `K_l .. K_u`"; Theorem B guarantees it never makes more than
   `floor(lg N) + 1` comparisons. Following the Rust convention, return `Ok(i)`
   on a hit and `Err(p)` on a miss, where `p` is the number of keys `< key`.
2. Track `l` and `u` as 1-based bounds (so `K_i` is `a[i-1]`). Each pass sets
   `i = (l+u)/2`, compares, and moves the far bound inward: on `key < K_i` set
   `u = i-1`, on `key > K_i` set `l = i+1`, on equal you are done. When `u < l`
   the loop ends unsuccessfully and the 0-based insertion point is `l-1`.
3. Skeleton: `let (mut l, mut u) = (1, a.len()); loop { if u < l { return
   (Err(l-1), c); } let i = (l+u)/2; c += 1; if key < a[i-1] { u = i-1; } else
   if key > a[i-1] { l = i+1; } else { return (Ok(i-1), c); } }`. Have the plain
   `binary_search` call the instrumented one and drop the count.

## Stage 2: Binary search trees

1. A BST keeps every left-subtree key `< KEY(P)` and every right-subtree key
   `> KEY(P)`, so an inorder traversal yields sorted keys. Search and insertion
   are Algorithm 6.2.2T; deletion of a two-child node uses the Hibbard scheme —
   replace it with its symmetric (inorder) successor, the leftmost node of its
   right subtree.
2. Store nodes in the given index arena (`Vec<Node>` + `usize` links,
   `usize::MAX` = null), recycling deleted slots through a free list. For
   `insert`, walk down comparing until you fall off a null link, then hang the
   new node there (reject duplicates). Crucially, use an *explicit stack* for
   `inorder` and `height` — the tests build vines thousands deep that would
   overflow a recursive traversal. Measure height in *edges* (a single node is 0).
3. Deletion has four shapes; handle them by the successor rule. If `RLINK(T)`
   is null, splice in `LLINK(T)`. Else find `R = RLINK(T)`: if `LLINK(R)` is
   null, `R` inherits `LLINK(T)` and replaces `T`; otherwise descend left from
   `R` to the successor `S`, keeping its parent, then let `S` inherit both of
   `T`'s subtrees and rehook its parent's left link to `S`'s old right child.
   Finally repoint `T`'s parent (or the root) at the replacement and free `T`.

## Stage 3: Balanced trees (AVL)

1. An AVL tree keeps every node's two subtree heights within 1; the balance
   factor `B(P) = height(right) − height(left) ∈ {−1,0,+1}`. The Fibonacci-tree
   argument bounds the height at `~1.4405 lg(n+2) − 0.3277`, so search is
   `O(lg n)` *worst case*. Algorithm 6.2.3A does one rotation (single or double)
   per insertion and touches only balance factors on the search path.
2. Track `S`, the deepest node on the path whose balance factor is already
   nonzero (rebalancing can only be needed there), and `T`, its parent. After
   inserting, adjust the balance factors from `S`'s relevant child down to the
   new leaf, then apply the §6.2.3 case analysis at `S`. The `LINK(a, ·)` helper
   (left if `a=−1`, right if `a=+1`) collapses the mirror-image cases into one
   code path. Implement `is_balanced` by recomputing heights from scratch — do
   not trust the stored `bal`.
3. Let `a = +1` if the key went right of `S`, else `−1`, and `R = LINK(a, S)`.
   If `B(S) = 0`, set `B(S) = a` (tree grew) — done; if `B(S) = −a`, set
   `B(S) = 0` (more balanced) — done; if `B(S) = a`, rotate: single (A8) when
   `B(R) = a`, double (A9) when `B(R) = −a`. After rotating, repoint `T`'s link
   (or the root) at the new subtree root. Write the rotations with `link`/
   `set_link` so `−a` gives the mirror automatically.

## Stage 4: Hashing with open addressing

1. Hashing computes an address from the key (division method `h(K) = K mod M`,
   `M` prime) instead of comparing keys, so average search is `O(1)`. `M` must
   be prime and not near a power of two, or arithmetic-progression keys collide
   onto few slots. Linear probing (6.4L) steps to the adjacent slot on
   collision; double hashing (6.4D) uses a key-dependent step to break clusters.
2. Back each table with `Vec<Option<u64>>` of size `M`, keeping a count `N` and
   declaring the table full at `N = M − 1` (one slot always empty, so
   unsuccessful search terminates). `insert` probes from `h(K)` until it finds
   the key (duplicate → false) or an empty slot; `probes_for` counts slots
   examined including the final one. Constructors should `assert!` `M` is a
   prime `>= 3` (message containing "prime").
3. Linear probe advance: `i = if i == 0 { m - 1 } else { i - 1 };`. Double
   hashing uses `c = 1 + (key % (m-2))` (so `1 <= c <= M−2`, coprime to the
   prime `M`) and advances `i = (i + m - c) % m`. Insert flow: probe to an empty
   slot or duplicate; if duplicate or `n == m-1` return false; else store and
   bump `n`.
