# Hints — Module 11: Multiway Trees and Digital Searching

Graduated hints, three per stage. Reach for the next one only when the
previous did not unstick you.

## Stage 1: B-tree search and insertion with node splitting

1. The defining discipline of a B-tree is that it grows *upward*, never
   downward: a node that overflows splits and pushes its median key up to
   its parent, and the only way a new level appears is when the root itself
   splits. Keep that one invariant in view and correctness follows — there
   is nothing to rebalance.
2. Search down to a leaf, insert there, then let overflow propagate back up.
   The natural shape in an arena is a recursive helper on a node index that
   returns one of three outcomes to its caller: the key was a duplicate, the
   insertion was absorbed with no overflow, or the child split and here is
   the promoted median plus the index of the new right sibling.
3. Use `keys.binary_search(&key)`: `Ok(_)` is a duplicate, `Err(p)` is *both*
   the child index to descend into and the sorted slot to insert at. Split
   when `keys.len() == order` (m keys, one too many): with `mid = order/2`,
   the median is `keys[mid]`, the left node keeps `keys[..mid]`, the right
   sibling takes `keys[mid+1..]` (and the matching `children` slice if
   internal). `keys_inorder` interleaves child 0, key 0, child 1, key 1, …,
   last child.

## Stage 2: B-tree invariants and the height bound

1. `is_valid` should be written as if it distrusts your Stage-1 code: it
   independently re-derives every clause of the B-tree definition, so a bug
   in insertion shows up as a broken invariant rather than a wrong answer.
   `height` exploits the fact that all leaves share a level (Property 4).
2. Check the invariants with a recursion that carries the open key interval
   `(lo, hi)` each subtree must lie inside, as `(Option<i64>, Option<i64>)`
   with `None` meaning ±∞. Track a single "leaf level" cell: the first leaf
   you reach sets it, every other leaf must match it. For `height`, walk the
   leftmost child spine and count levels.
3. Per-node bounds: `min_keys = if is_root { 1 } else { (m+1)/2 - 1 }` and
   `max = m - 1`; also require strictly increasing keys, `children.len() ==
   keys.len() + 1` at internal nodes, and `keys[i-1] < child keys < keys[i]`
   via the passed-down `(lo, hi)`. `height` is `0` for an empty tree, else
   `1 + (levels down the leftmost spine)`.

## Stage 3: Digital searching: tries

1. A trie branches on the *bits* of the key, not on comparisons: the path
   from the root spells the key out, one bit per level, so there is no
   ordering logic at all — just "which link do I follow next?" Fixed 32-bit
   keys, most-significant bit first, give 32 levels and keep left/right order
   equal to numeric order.
2. Store two links per node, indexed by the bit value 0 or 1, plus an
   `is_key` mark set on the node reached after all 32 bits are consumed.
   Insertion creates missing nodes as it descends; search fails the moment a
   link is null. `remove` must *prune*: after unmarking, walk back up and
   detach every node that is now unmarked and childless, or insert/remove
   cycles leak a chain per key.
3. Extract bit `i` as `(key >> (31 - i)) & 1`. Insert loop: for `i in 0..32`,
   if `link[b]` is `NULL` allocate a fresh node there, then advance
   `p = link[b]`; finally, `is_key` already set means duplicate. To prune,
   record the path as `(parent, bit)` pairs on the way down, then iterate it
   in reverse breaking as soon as a node is still marked or still has a
   child; recycle detached slots on a `free` list.

## Stage 4: Patricia: compressed binary tries

1. Patricia removes one-way branching by storing, in each branch node, *the
   index of the next bit that actually distinguishes* the keys below it —
   agreed-upon bits are skipped entirely. The price is that a branch can
   route you but cannot vouch for the bits it skipped, so a search ends with
   exactly one full key comparison at a leaf. The invariant that makes it all
   work: bit indices strictly increase along every root-to-leaf path.
2. Two node kinds: a `Leaf { key }` and a `Branch { bit, link: [_; 2] }`.
   Search descends blindly, following `link[bit-of-key-at-branch.bit]` until
   it hits a leaf, then compares. Insertion is a blind hunt to a leaf `L`,
   then find the critical bit where the new key first differs from `L`, then
   re-descend to splice a new branch at that bit.
3. The critical bit is `d = (key ^ found).leading_zeros()` (MSB-first index
   of the first differing bit). Re-descend from the root following branches
   whose `bit < d`, tracking `(parent, side)`, and stop at the first node
   that is a leaf or a branch testing a bit `> d`; splice a new `Branch { bit:
   d, .. }` there whose two links are the fresh leaf and the displaced node,
   ordered by `bit64(key, d)`. Correct code holds `node_count() ≤ 2·keys - 1`.
