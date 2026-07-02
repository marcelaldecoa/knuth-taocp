# Walkthrough — Module 11: Multiway Trees and Digital Searching

Read this AFTER a stage is green — it explains how the reference solution is
built and why.

## Stage 1: B-tree search and insertion with node splitting

The reference splits insertion into a thin public `insert` and a recursive
worker `insert_at(x, key) -> Ins`, where `Ins` is a three-variant enum:
`Duplicate`, `Done`, and `Split { median, right }`. This enum *is* the design:
it lets overflow information travel back up the call stack exactly one level at
a time, so each parent decides locally whether to absorb the promoted median
(`keys.insert(pos, median); children.insert(pos+1, right)`) or to overflow in
turn. The one step that cannot be handled by any parent — the root splitting —
is handled by `insert` itself (step B5): it allocates a new root
`{ keys: vec![median], children: vec![old_root, right] }`. That is the single
place the tree gains height, which is precisely why all leaves stay on one
level with no rebalancing code anywhere.

The idiom worth stealing is `keys.binary_search(&key)`: its `Ok`/`Err(p)`
result serves triple duty — `Ok` is the duplicate check, and `Err(p)` is
simultaneously the child link to descend (`children[pos]`) and the sorted
insertion slot (`keys.insert(pos, ..)`). `split` uses `Vec::split_off(mid+1)`
plus `pop()` to carve the node in three (left keeps `[..mid]`, median is
`keys[mid]`, right gets `[mid+1..]`) with the children sliced the same way only
when the node is internal. The naive alternative — rebalancing with rotations
like an AVL tree — is avoided entirely: a split touches three nodes and never
re-hangs a subtree, which on disk means three page writes instead of a scatter
of them.

## Stage 2: B-tree invariants and the height bound

`is_valid` is deliberately written as an *independent* re-derivation of the
B-tree definition, not a shortcut that trusts Stage 1. The recursive `check`
carries down an open interval `(lo, hi)` — as `(Option<i64>, Option<i64>)`,
`None` meaning ±∞ — and each child `i` is checked against the tightened bounds
`(keys[i-1], keys[i])`. This is what verifies the separator/interval property
(Invariant 5) across levels, catching a key that is locally sorted but globally
misplaced. Two shared mutable cells thread through the recursion: a `leaf_level`
`Option` that the first leaf sets and every later leaf must equal (Invariant 4,
uniform depth), and a `counted` total that must match `self.len` at the end, so
a lost or duplicated key cannot hide.

The one subtlety is the root exemption: `min_keys = if is_root { 1 } else {
(m+1)/2 - 1 }`. The `(m+1)/2` is `⌈m/2⌉` written in integer arithmetic, and the
root is allowed to be nearly empty (down to one key) because a freshly split
root legitimately holds a single median. `height` then leans on the invariant
you just proved: since every leaf is at the same level, it simply walks the
leftmost child spine (`children.first()`), counting one per level and stopping
at `NULL` — no full traversal needed. This is the mechanical counterpart to
Theorem B: the bench in `examples/bench.rs` watches this height stay near
`log_t n` as `n` climbs by decades.

## Stage 3: Digital searching: tries

The `BinaryTrie` is an arena of `TrieNode { link: [usize; 2], is_key: bool }`
where the two links are indexed *directly by the bit value* — `link[b]` with
`b = bit32(key, i)` — so branching is an array index, never a comparison. Node
`0` is a permanent unmarked root (present even in an empty trie), which removes
the empty-tree special case from `insert` and `contains`. The `insert` loop is a
faithful transcription of Algorithm T's steps T1–T4: initialize `p = 0`, for
each of 32 bits allocate a child if `link[b] == NULL` then advance, and finally
either report a duplicate (`is_key` already set) or mark and count.

The detail that separates a correct trie from a leaky one is that `remove`
truly inverts `insert`. It records the descent path as `(parent, bit)` pairs,
unmarks the terminal node, then prunes bottom-up: iterating the path in reverse,
it breaks the moment it meets a node that is still marked or still has a child,
and otherwise detaches the node (`nodes[parent].link[b] = NULL`) and pushes its
slot onto a `free` list for `alloc` to reuse. Without this pruning, an
insert/remove cycle would leave a 32-node corridor per departed key and the
structure would stop mirroring its contents — the model-checker test against a
`HashSet` over 20 000 mixed ops is exactly what catches that.

## Stage 4: Patricia: compressed binary tries

`Patricia` stores a `PatNode` enum — `Leaf { key }` or `Branch { bit, link:
[usize; 2] }` — and its entire correctness rests on one invariant: bit indices
strictly increase along every root-to-leaf path. `contains` descends "blindly"
through branches (following `link[bit64(key, bit)]`) because a branch only knows
*which* bit distinguishes its subtrees, not the bits it skipped; the truth is
settled by the *single* full comparison `*k == key` at the leaf. That one
deferred comparison is what lets every branch node skip runs of agreed-upon
bits, giving the `2n - 1` node budget regardless of key length.

`insert` is the elegant part. The blind hunt reaches a leaf key `found`; the
critical bit is computed in one instruction as `d = (key ^ found).leading_zeros()`
— the MSB-first index of the first differing bit, which is where `key` parts
company with the *whole* tree, not just that one leaf. The re-descent then walks
from the root through branches with `bit < d`, remembering `(parent, pside)`,
and stops at the first node testing a bit `> d` (or a leaf); the new
`Branch { bit: d, .. }` is spliced there with its links ordered by
`bit64(key, d)`. Because the splice point is fixed by the *set* of keys and not
the insertion history, the same keys always build the same shape — the
order-independence test inserts a key set forwards and backwards and checks the
node counts and answers match, a property a plain BST conspicuously lacks.
