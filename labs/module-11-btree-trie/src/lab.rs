//! Module 11 — Multiway Trees and Digital Searching (TAOCP Vol. 3,
//! §6.2.4 and §6.3).
//!
//! **Scaffolding tier — Module 05 and up:** the stub states the algorithm and
//! the contract and trusts you to translate it to Rust; the guided-tour aids of
//! Modules 01–04 are gone by design. The nets remain for every stage — the
//! lesson, three graduated hints (`--hint`), the reference, and the walkthrough.
//! (The taper is described in docs/for-newcomers.md §5.)
//!
//! YOUR WORKSPACE. Replace each `todo!()` with an implementation, then run
//! `./grade 11` from the repository root. Work the stages in order; the
//! lesson in `course/module-11-btree-trie/README.md` teaches everything the
//! stages need (B-tree splitting, the height bound, tries, Patricia).
//!
//! Memory model, used throughout: each structure is an **arena** — a
//! `Vec` of nodes addressed by `usize` — with the null link Λ spelled
//! `NULL == usize::MAX`. That is the faithful, idiomatic translation of
//! Knuth's MIX-era link fields; no `Rc<RefCell<..>>` anywhere. The arenas
//! and constructors are already set up for you; the internals are private,
//! so you may reshape them if you prefer, as long as the public API keeps
//! these exact names and signatures.

/// Null link sentinel (Knuth's Λ) for every arena in this module.
/// `usize::MAX` can never be a valid `Vec` index on a real machine, so a
/// link either addresses a live node or is exactly `NULL`.
const NULL: usize = usize::MAX;

// ===========================================================================
// Stages 1 & 2 — B-trees (TAOCP Vol. 3, §6.2.4)
// ===========================================================================

/// One node of a B-tree of order `m`.
///
/// Keep `keys` strictly increasing. `children` must be empty **iff** the
/// node is a leaf; otherwise it holds exactly `keys.len() + 1` links, with
/// `children[i]` leading to the keys strictly between `keys[i-1]` and
/// `keys[i]` (open ends at the extremes).
#[allow(dead_code)] // read once you implement the methods below
struct BNode {
    keys: Vec<i64>,
    children: Vec<usize>,
}

/// Stages 1 & 2 — a B-tree of order `m` over `i64` keys (§6.2.4).
///
/// Definition — a B-tree of order `m` satisfies:
///
/// 1. every node has at most `m` children (so at most `m - 1` keys);
/// 2. every node except the root and the leaves has at least `⌈m/2⌉`
///    children (so at least `⌈m/2⌉ - 1` keys);
/// 3. the root, if not a leaf, has at least 2 children;
/// 4. all leaves appear on the same level;
/// 5. keys inside a node are strictly increasing, and subtree `children[i]`
///    holds exactly the keys between the neighbouring keys.
///
/// Arena representation: `nodes[i]` is node `i`; `root == NULL` means the
/// tree is empty. You never need to free slots — this module has no B-tree
/// deletion, so split-off siblings and new roots are simply pushed.
pub struct BTree {
    #[allow(dead_code)] // read once you implement the methods below
    order: usize,
    #[allow(dead_code)]
    nodes: Vec<BNode>,
    #[allow(dead_code)]
    root: usize,
    #[allow(dead_code)]
    len: usize,
}

impl BTree {
    /// An empty B-tree of order `m`. (Done for you.)
    ///
    /// Orders below 3 are rejected: with `m = 2` a split would create
    /// children with zero keys, violating invariant 2.
    pub fn new(order: usize) -> Self {
        assert!(order >= 3, "B-tree order must be at least 3");
        BTree { order, nodes: Vec::new(), root: NULL, len: 0 }
    }

    /// Stage 1 — B-tree insertion with node splitting. Return `false`
    /// (leaving the tree unchanged) if `key` is already present.
    ///
    /// ```text
    /// B1. [Empty?]    If ROOT = Λ, make a one-key leaf the root; done.
    /// B2. [Descend.]  Search for K in the current node; a hit is a
    ///                 duplicate. Otherwise follow the child whose interval
    ///                 contains K, until a leaf is reached.
    /// B3. [Insert.]   Put K into the leaf at its sorted position.
    /// B4. [Overflow?] If a node now has m keys (one too many), split it:
    ///                 with mid = ⌊m/2⌋, keys[mid] moves UP into the parent,
    ///                 keys[..mid] stay put, keys[mid+1..] (and, at internal
    ///                 nodes, children[mid+1..]) become a NEW right sibling.
    ///                 Both halves end with >= ⌈m/2⌉ - 1 keys. The parent
    ///                 gained a key: repeat B4 there.
    /// B5. [New root?] If the root itself split, create a new root holding
    ///                 just the promoted median and the two halves as its
    ///                 children. This is the only way the tree grows taller
    ///                 — which is why all leaves stay on one level forever.
    /// ```
    ///
    /// Hint: a recursive helper returning "duplicate / done / split(median,
    /// right-sibling index)" keeps the borrow checker happy in an arena.
    pub fn insert(&mut self, key: i64) -> bool {
        let _ = key;
        todo!("implement B-tree insertion with splitting (§6.2.4)")
    }

    /// Stage 1 — B-tree search: binary-search the keys of each node; on a
    /// miss at an internal node, descend into the child whose open interval
    /// contains `key`; a miss at a leaf is an unsuccessful search.
    pub fn contains(&self, key: i64) -> bool {
        let _ = key;
        todo!("implement B-tree search")
    }

    /// Stage 1 — all keys in symmetric (inorder) order: child 0, key 0,
    /// child 1, key 1, …, last child. Sorted iff your tree is correct.
    pub fn keys_inorder(&self) -> Vec<i64> {
        todo!("inorder traversal of the B-tree")
    }

    /// Stage 2 — height in **levels**: 0 for the empty tree, 1 for a lone
    /// root leaf. All leaves sit on one level, so following any downward
    /// path (say, the leftmost spine) measures it.
    pub fn height(&self) -> usize {
        todo!("compute the number of levels")
    }

    /// Stage 2 — check ALL the B-tree invariants (the numbered list on
    /// [`BTree`]): per-node key-count bounds (min `⌈m/2⌉ - 1`, root exempt
    /// — it needs only 1 key, i.e. 2 children if internal; max `m - 1`),
    /// strict in-node sortedness, separator consistency across levels
    /// (every key in `children[i]` lies strictly between the neighbouring
    /// keys), `children.len() == keys.len() + 1` at internal nodes, and all
    /// leaves on the same level. The empty tree is valid.
    ///
    /// Hint: recurse with open-interval bounds `(Option<i64>, Option<i64>)`
    /// and record the first leaf level you meet; every other leaf must
    /// match it.
    pub fn is_valid(&self) -> bool {
        todo!("verify every B-tree invariant")
    }
}

// ===========================================================================
// Stage 3 — Binary tries (TAOCP Vol. 3, §6.3: Algorithm T, with the "M-ary
// characters" specialised to the bits of a u32, most significant bit first)
// ===========================================================================

/// One node of the binary trie. `link[b]` follows bit value `b`
/// (`NULL` = absent); a node reached after consuming all 32 bits of some
/// inserted key has `is_key = true`.
#[allow(dead_code)] // read once you implement the methods below
struct TrieNode {
    link: [usize; 2],
    is_key: bool,
}

/// Stage 3 — a binary trie over `u32` keys (§6.3).
///
/// Branching consumes the key's bits **most significant bit first**:
/// bit `i` of `k` is `(k >> (31 - i)) & 1`, for `i = 0, 1, …, 31`. A key is
/// present iff the depth-32 node its bits spell out exists and is marked.
///
/// Arena representation: `nodes[0]` is the root, which always exists (an
/// empty trie is just an unmarked, childless root). `free` recycles the
/// slots that `remove`'s pruning detaches; `len` counts the stored keys.
pub struct BinaryTrie {
    #[allow(dead_code)] // read once you implement the methods below
    nodes: Vec<TrieNode>,
    #[allow(dead_code)]
    free: Vec<usize>,
    #[allow(dead_code)]
    len: usize,
}

impl BinaryTrie {
    /// An empty trie: just an unmarked root. (Done for you.)
    pub fn new() -> Self {
        BinaryTrie {
            nodes: vec![TrieNode { link: [NULL, NULL], is_key: false }],
            free: Vec::new(),
            len: 0,
        }
    }

    /// Trie insertion (Algorithm 6.3T, insertion half). Return `false` if
    /// `key` was already present.
    ///
    /// ```text
    /// T1. [Initialize.] Set P <- ROOT, i <- 0.
    /// T2. [Branch.]     b <- bit i of K (MSB first). If LINK_b(P) = Λ,
    ///                   create a fresh empty node there.
    /// T3. [Advance.]    P <- LINK_b(P), i <- i + 1; if i < 32 go to T2.
    /// T4. [Mark.]       All 32 bits consumed: if P is already marked, K is
    ///                   a duplicate; otherwise mark P and count the key.
    /// ```
    pub fn insert(&mut self, key: u32) -> bool {
        let _ = key;
        todo!("implement trie insertion (Algorithm 6.3T)")
    }

    /// Trie search (Algorithm 6.3T, search half): follow one bit per level;
    /// hitting Λ means absent, and after 32 bits the mark decides.
    pub fn contains(&self, key: u32) -> bool {
        let _ = key;
        todo!("implement trie search (Algorithm 6.3T)")
    }

    /// Remove `key`; return `false` if it was absent.
    ///
    /// After unmarking the depth-32 node, **prune**: walking back up the
    /// search path (record it on the way down), detach every node that is
    /// now unmarked and childless from its parent — never the root — and
    /// recycle its slot through `free`. Pruning is what lets a
    /// remove-then-reinsert cycle not leak structure.
    pub fn remove(&mut self, key: u32) -> bool {
        let _ = key;
        todo!("implement trie deletion with pruning")
    }

    /// How many keys the trie currently holds.
    pub fn count(&self) -> usize {
        todo!("return the number of stored keys")
    }
}

impl Default for BinaryTrie {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Stage 4 — Patricia (TAOCP Vol. 3, §6.3, Algorithm P), compressed form
// ===========================================================================

/// A Patricia node: either a leaf carrying a full key, or a branch that
/// tests exactly one bit position.
///
/// Invariant: along any root-to-leaf path, `bit` indices are strictly
/// increasing. `link[b]` holds every key of the subtree whose tested bit
/// equals `b`.
#[allow(dead_code)] // read once you implement the methods below
enum PatNode {
    Leaf { key: u64 },
    Branch { bit: u32, link: [usize; 2] },
}

/// Stage 4 — Patricia over fixed 64-bit keys (§6.3, Algorithm P essence).
///
/// The two Patricia ideas your implementation must keep:
///
/// 1. **Skip the boring bits.** A branch stores the index of the next bit
///    that actually *distinguishes* keys in its subtree; bits on which all
///    those keys agree are never tested (no one-way branching).
/// 2. **One branch per key boundary.** `n` keys need exactly `n - 1`
///    branch nodes — a full binary tree with `n` leaves — independent of
///    key length. (A plain trie can burn 64 nodes on two keys.)
///
/// **Honest note.** Knuth's own Algorithm P stores each key *inside* a
/// branch node, reached via a tagged back-pointer, using exactly `n` nodes
/// for `n` keys; that exact form is exercise material in §6.3. Here we
/// build the *compressed radix trie* form — separate leaves, `n - 1`
/// branches — which keeps both ideas and is what production Patricia/
/// crit-bit trees ship.
///
/// Bit `i` (0 = most significant) of `k` is `(k >> (63 - i)) & 1`.
///
/// Arena representation: `root == NULL` iff empty; there is no removal, so
/// every slot ever pushed stays live.
pub struct Patricia {
    #[allow(dead_code)] // read once you implement the methods below
    nodes: Vec<PatNode>,
    #[allow(dead_code)]
    root: usize,
    #[allow(dead_code)]
    len: usize,
}

impl Patricia {
    /// An empty Patricia tree. (Done for you.)
    pub fn new() -> Self {
        Patricia { nodes: Vec::new(), root: NULL, len: 0 }
    }

    /// Patricia search. Branches only say *which* bits to test — nothing
    /// vouches for the skipped bits — so descend "blindly" to a leaf, then
    /// make **one full key comparison** there.
    ///
    /// ```text
    /// P1. [Empty?]   If ROOT = Λ, the search fails.
    /// P2. [Descend.] While at a branch testing bit j, follow
    ///                LINK[bit j of K].
    /// P3. [Compare.] At the leaf: K is present iff it equals the leaf key.
    /// ```
    pub fn contains(&self, key: u64) -> bool {
        let _ = key;
        todo!("implement Patricia search")
    }

    /// Patricia insertion. Return `false` on a duplicate.
    ///
    /// ```text
    /// I1. [Empty?]      If ROOT = Λ, a new leaf for K becomes the root.
    /// I2. [Blind hunt.] Search as in `contains`, reaching some leaf key L.
    ///                   If L = K: duplicate, stop.
    /// I3. [Crit bit.]   d <- index of the most significant bit where K and
    ///                   L differ. (`(k ^ l).leading_zeros()` is exactly
    ///                   that index in MSB-first numbering.)
    /// I4. [Re-descend.] Walk from ROOT again, stopping at the first node
    ///                   that is a leaf or a branch testing a bit > d
    ///                   (remember the parent link you came through).
    /// I5. [Splice.]     Replace that node with a new branch testing bit d:
    ///                   the bit-d-of-K side is a fresh leaf for K, the
    ///                   other side is the displaced node. Bit indices stay
    ///                   strictly increasing along every path.
    /// ```
    pub fn insert(&mut self, key: u64) -> bool {
        let _ = key;
        todo!("implement Patricia insertion")
    }

    /// Total number of live nodes, leaves and branches together. With no
    /// removal this is just how many nodes you have ever created; a correct
    /// Patricia holding `n >= 1` keys has exactly `n` leaves and `n - 1`
    /// branches, i.e. `2n - 1` nodes — the stage tests hold you to the
    /// bound `node_count() <= 2 * keys - 1`.
    pub fn node_count(&self) -> usize {
        todo!("return the number of live nodes")
    }
}

impl Default for Patricia {
    fn default() -> Self {
        Self::new()
    }
}
