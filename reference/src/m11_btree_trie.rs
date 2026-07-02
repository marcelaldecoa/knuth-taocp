//! Module 11 — Multiway Trees and Digital Searching.
//! Source: TAOCP Vol. 3, 2nd ed., §6.2.4 (multiway trees / B-trees) and
//! §6.3 (digital searching: tries and Patricia).
//!
//! Memory model: every structure here is an **arena** — a `Vec` of nodes
//! addressed by `usize` indices, exactly the index-based reading of Knuth's
//! MIX-era link fields. The null link Λ is the sentinel `NULL == usize::MAX`
//! (no `Option<usize>`, no `Rc<RefCell<..>>`).

/// Null link sentinel (Knuth's Λ) for every arena in this module.
/// `usize::MAX` can never be a valid `Vec` index on a real machine, so a
/// link either addresses a live node or is exactly `NULL`.
const NULL: usize = usize::MAX;

// ===========================================================================
// Stages 1 & 2 — B-trees (TAOCP Vol. 3, §6.2.4)
// ===========================================================================

/// One node of a B-tree of order `m`.
///
/// `keys` are strictly increasing; `children` is empty **iff** the node is a
/// leaf, and otherwise holds exactly `keys.len() + 1` links: `children[i]`
/// leads to the keys strictly between `keys[i-1]` and `keys[i]` (with the
/// obvious open ends).
struct BNode {
    keys: Vec<i64>,
    children: Vec<usize>,
}

/// A B-tree of order `m` over `i64` keys (Bayer & McCreight 1972; TAOCP
/// Vol. 3, §6.2.4). Definition — a B-tree of order `m` satisfies:
///
/// 1. every node has at most `m` children (so at most `m - 1` keys);
/// 2. every node except the root and the leaves has at least `⌈m/2⌉`
///    children (so at least `⌈m/2⌉ - 1` keys);
/// 3. the root, if not a leaf, has at least 2 children;
/// 4. all leaves appear on the same level;
/// 5. within a node keys are strictly increasing, and subtree `children[i]`
///    contains exactly the keys falling in the open interval between the
///    neighbouring keys.
///
/// Arena representation: `nodes[i]` is node `i`; `root == NULL` means empty.
/// Split-off right halves and new roots are pushed onto the arena; slots are
/// never freed (this module has no B-tree deletion).
pub struct BTree {
    order: usize,
    nodes: Vec<BNode>,
    root: usize,
    len: usize,
}

/// What a recursive insertion step reports to its caller.
enum Ins {
    /// Key already present; nothing changed.
    Duplicate,
    /// Inserted; the subtree absorbed it without overflowing.
    Done,
    /// Inserted, and the child split: `median` and the link to the new
    /// right sibling must be added to the caller's node.
    Split { median: i64, right: usize },
}

impl BTree {
    /// An empty B-tree of order `m` (maximum `m` children per node).
    ///
    /// Panics unless `order >= 3`: with `m = 2` a "split" would have to
    /// produce children with zero keys, violating invariant 2.
    pub fn new(order: usize) -> Self {
        assert!(order >= 3, "B-tree order must be at least 3");
        BTree { order, nodes: Vec::new(), root: NULL, len: 0 }
    }

    /// B-tree insertion (§6.2.4): search to a leaf, insert there, and split
    /// overfull nodes on the way back up. Returns `false` (tree unchanged)
    /// if `key` is already present.
    ///
    /// ```text
    /// B1. [Empty?]    If ROOT = Λ, make a one-key leaf the root; done.
    /// B2. [Descend.]  Search for K in the current node; a hit is a
    ///                 duplicate. Otherwise follow the child whose interval
    ///                 contains K, until a leaf is reached.
    /// B3. [Insert.]   Put K into the leaf at its sorted position.
    /// B4. [Overflow?] If a node now has m keys, split it: the median key
    ///                 moves UP into the parent, the keys below it stay, the
    ///                 keys above it become a new right sibling. Repeat B4
    ///                 at the parent.
    /// B5. [New root?] If the root itself split, a new root holding just
    ///                 the median is created — the only way the tree grows
    ///                 taller, which is why all leaves stay on one level.
    /// ```
    pub fn insert(&mut self, key: i64) -> bool {
        // B1. [Empty?]
        if self.root == NULL {
            self.nodes.push(BNode { keys: vec![key], children: Vec::new() });
            self.root = self.nodes.len() - 1;
            self.len = 1;
            return true;
        }
        match self.insert_at(self.root, key) {
            Ins::Duplicate => false,
            Ins::Done => {
                self.len += 1;
                true
            }
            // B5. [New root?] The tree grows at the TOP, never at the leaves.
            Ins::Split { median, right } => {
                let old_root = self.root;
                self.nodes.push(BNode { keys: vec![median], children: vec![old_root, right] });
                self.root = self.nodes.len() - 1;
                self.len += 1;
                true
            }
        }
    }

    /// Recursive worker for steps B2–B4 on the subtree rooted at `x`.
    fn insert_at(&mut self, x: usize, key: i64) -> Ins {
        // B2. [Descend.] Binary search inside the node.
        let pos = match self.nodes[x].keys.binary_search(&key) {
            Ok(_) => return Ins::Duplicate,
            Err(p) => p,
        };
        if self.nodes[x].children.is_empty() {
            // B3. [Insert.] We are at a leaf.
            self.nodes[x].keys.insert(pos, key);
        } else {
            let child = self.nodes[x].children[pos];
            match self.insert_at(child, key) {
                Ins::Duplicate => return Ins::Duplicate,
                Ins::Done => return Ins::Done,
                Ins::Split { median, right } => {
                    // Absorb the promoted median and the new right sibling.
                    self.nodes[x].keys.insert(pos, median);
                    self.nodes[x].children.insert(pos + 1, right);
                }
            }
        }
        // B4. [Overflow?] A node may hold at most m - 1 keys.
        if self.nodes[x].keys.len() == self.order {
            let (median, right) = self.split(x);
            Ins::Split { median, right }
        } else {
            Ins::Done
        }
    }

    /// Split a node holding exactly `m` keys (one too many). With
    /// `mid = ⌊m/2⌋`, the left node keeps `keys[..mid]` (that is ⌊m/2⌋
    /// keys), `keys[mid]` is promoted, and the new right sibling gets the
    /// remaining `⌈m/2⌉ - 1` keys — both halves meet the minimum
    /// `⌈m/2⌉ - 1`, which is exactly why splitting only *overfull* nodes
    /// works for every order, odd or even.
    fn split(&mut self, x: usize) -> (i64, usize) {
        let mid = self.order / 2;
        let right_keys = self.nodes[x].keys.split_off(mid + 1);
        let median = self.nodes[x].keys.pop().expect("overfull node has a median");
        let right_children = if self.nodes[x].children.is_empty() {
            Vec::new()
        } else {
            self.nodes[x].children.split_off(mid + 1)
        };
        self.nodes.push(BNode { keys: right_keys, children: right_children });
        (median, self.nodes.len() - 1)
    }

    /// B-tree search: at each node binary-search the keys; on a miss at an
    /// internal node follow the child whose open interval contains `key`.
    pub fn contains(&self, key: i64) -> bool {
        let mut x = self.root;
        while x != NULL {
            let node = &self.nodes[x];
            match node.keys.binary_search(&key) {
                Ok(_) => return true,
                Err(p) => {
                    if node.children.is_empty() {
                        return false;
                    }
                    x = node.children[p];
                }
            }
        }
        false
    }

    /// All keys in symmetric (inorder) order: child 0, key 0, child 1,
    /// key 1, …, last child. Sorted iff the tree is a valid B-tree.
    pub fn keys_inorder(&self) -> Vec<i64> {
        let mut out = Vec::with_capacity(self.len);
        if self.root != NULL {
            self.collect(self.root, &mut out);
        }
        out
    }

    fn collect(&self, x: usize, out: &mut Vec<i64>) {
        let node = &self.nodes[x];
        if node.children.is_empty() {
            out.extend_from_slice(&node.keys);
        } else {
            for i in 0..node.keys.len() {
                self.collect(node.children[i], out);
                out.push(node.keys[i]);
            }
            self.collect(*node.children.last().expect("internal node has children"), out);
        }
    }

    /// Height in **levels**: 0 for the empty tree, 1 for a lone root leaf.
    /// Because all leaves live on the same level, walking the leftmost
    /// spine measures it.
    pub fn height(&self) -> usize {
        let mut h = 0;
        let mut x = self.root;
        while x != NULL {
            h += 1;
            x = *self.nodes[x].children.first().unwrap_or(&NULL);
        }
        h
    }

    /// Check **all** the B-tree invariants (the numbered list on
    /// [`BTree`]): per-node key-count bounds (root exempt from the
    /// minimum), strict in-node sortedness, separator/interval consistency
    /// across levels, `children.len() == keys.len() + 1` at internal nodes,
    /// and all leaves on the same level. The empty tree is valid.
    pub fn is_valid(&self) -> bool {
        if self.root == NULL {
            return self.len == 0;
        }
        let mut leaf_level = None;
        let mut counted = 0usize;
        self.check(self.root, true, None, None, 1, &mut leaf_level, &mut counted)
            && counted == self.len
    }

    /// Recursive invariant check for the subtree at `x`, whose keys must
    /// lie strictly inside the open interval `(lo, hi)` (`None` = ±∞).
    fn check(
        &self,
        x: usize,
        is_root: bool,
        lo: Option<i64>,
        hi: Option<i64>,
        level: usize,
        leaf_level: &mut Option<usize>,
        counted: &mut usize,
    ) -> bool {
        if x >= self.nodes.len() {
            return false;
        }
        let node = &self.nodes[x];
        let m = self.order;
        // Invariants 1–3: key-count bounds. min = ⌈m/2⌉ - 1 except at the
        // root, which needs only 1 key (equivalently: ≥ 2 children when it
        // is internal, since children = keys + 1).
        let min_keys = if is_root { 1 } else { (m + 1) / 2 - 1 };
        if node.keys.len() < min_keys || node.keys.len() > m - 1 {
            return false;
        }
        // Invariant 5: strictly increasing and inside (lo, hi).
        if node.keys.windows(2).any(|w| w[0] >= w[1]) {
            return false;
        }
        if let Some(l) = lo {
            if node.keys[0] <= l {
                return false;
            }
        }
        if let Some(h) = hi {
            if *node.keys.last().expect("nonempty") >= h {
                return false;
            }
        }
        *counted += node.keys.len();
        if node.children.is_empty() {
            // Invariant 4: every leaf on one level.
            match *leaf_level {
                None => {
                    *leaf_level = Some(level);
                    true
                }
                Some(d) => d == level,
            }
        } else {
            if node.children.len() != node.keys.len() + 1 {
                return false;
            }
            for i in 0..node.children.len() {
                let clo = if i == 0 { lo } else { Some(node.keys[i - 1]) };
                let chi = if i == node.keys.len() { hi } else { Some(node.keys[i]) };
                if !self.check(node.children[i], false, clo, chi, level + 1, leaf_level, counted) {
                    return false;
                }
            }
            true
        }
    }
}

// ===========================================================================
// Stage 3 — Binary tries (TAOCP Vol. 3, §6.3: Algorithm T with the "M-ary
// characters" specialised to the bits of a u32, most significant bit first)
// ===========================================================================

/// One node of the binary trie. `link[b]` follows bit value `b`; a node
/// reached after consuming all 32 bits of a key has `is_key = true`.
struct TrieNode {
    link: [usize; 2],
    is_key: bool,
}

/// A binary trie over `u32` keys (§6.3). Branching consumes the bits of the
/// key **most significant bit first** — bit `i` of `k` is
/// `(k >> (31 - i)) & 1` — so left/right order in the trie is numeric order
/// of the keys, just as a MIX character trie's fan-out order is
/// alphabetical.
///
/// Arena representation: `nodes[0]` is the root (always present, even in an
/// empty trie); absent links are `NULL`. `remove` detaches childless
/// unmarked nodes on the search path and recycles their slots via `free`.
pub struct BinaryTrie {
    nodes: Vec<TrieNode>,
    free: Vec<usize>,
    len: usize,
}

/// Bit `i` (0 = most significant) of a 32-bit key.
fn bit32(key: u32, i: u32) -> usize {
    ((key >> (31 - i)) & 1) as usize
}

impl BinaryTrie {
    /// An empty trie: just an unmarked root.
    pub fn new() -> Self {
        BinaryTrie {
            nodes: vec![TrieNode { link: [NULL, NULL], is_key: false }],
            free: Vec::new(),
            len: 0,
        }
    }

    /// Allocate a fresh empty node, reusing a freed slot if possible.
    fn alloc(&mut self) -> usize {
        match self.free.pop() {
            Some(i) => {
                self.nodes[i] = TrieNode { link: [NULL, NULL], is_key: false };
                i
            }
            None => {
                self.nodes.push(TrieNode { link: [NULL, NULL], is_key: false });
                self.nodes.len() - 1
            }
        }
    }

    /// Trie insertion (Algorithm 6.3T, insertion half, binary alphabet).
    /// Returns `false` if `key` was already present.
    ///
    /// ```text
    /// T1. [Initialize.] Set P <- ROOT, i <- 0.
    /// T2. [Branch.]     b <- bit i of K (MSB first). If LINK_b(P) = Λ,
    ///                   create a fresh empty node there.
    /// T3. [Advance.]    P <- LINK_b(P), i <- i + 1; if i < 32 go to T2.
    /// T4. [Mark.]       All 32 bits consumed: if P is already marked, K is
    ///                   a duplicate; otherwise mark P as holding a key.
    /// ```
    pub fn insert(&mut self, key: u32) -> bool {
        // T1. [Initialize.]
        let mut p = 0usize;
        for i in 0..32 {
            // T2. [Branch.]
            let b = bit32(key, i);
            if self.nodes[p].link[b] == NULL {
                let q = self.alloc();
                self.nodes[p].link[b] = q;
            }
            // T3. [Advance.]
            p = self.nodes[p].link[b];
        }
        // T4. [Mark.]
        if self.nodes[p].is_key {
            false
        } else {
            self.nodes[p].is_key = true;
            self.len += 1;
            true
        }
    }

    /// Trie search (Algorithm 6.3T, search half): follow one bit per level;
    /// a null link means absent, and after 32 bits the mark decides.
    pub fn contains(&self, key: u32) -> bool {
        let mut p = 0usize;
        for i in 0..32 {
            let b = bit32(key, i);
            p = self.nodes[p].link[b];
            if p == NULL {
                return false;
            }
        }
        self.nodes[p].is_key
    }

    /// Remove `key`; returns `false` if it was absent. After unmarking the
    /// terminal node, prune upward: every node on the path that is now
    /// unmarked and childless is detached from its parent and its slot
    /// recycled — a remove really undoes the structure an insert built.
    pub fn remove(&mut self, key: u32) -> bool {
        // Walk down, recording the path as (parent, bit taken).
        let mut path = Vec::with_capacity(32);
        let mut p = 0usize;
        for i in 0..32 {
            let b = bit32(key, i);
            let q = self.nodes[p].link[b];
            if q == NULL {
                return false;
            }
            path.push((p, b));
            p = q;
        }
        if !self.nodes[p].is_key {
            return false;
        }
        self.nodes[p].is_key = false;
        self.len -= 1;
        // Prune childless, unmarked nodes bottom-up (never the root).
        let mut cur = p;
        for &(parent, b) in path.iter().rev() {
            let n = &self.nodes[cur];
            if n.is_key || n.link[0] != NULL || n.link[1] != NULL {
                break;
            }
            self.nodes[parent].link[b] = NULL;
            self.free.push(cur);
            cur = parent;
        }
        true
    }

    /// How many keys the trie currently holds.
    pub fn count(&self) -> usize {
        self.len
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
enum PatNode {
    Leaf { key: u64 },
    /// Test bit `bit` (0 = MSB of the u64); `link[b]` holds every key of
    /// the subtree whose tested bit equals `b`. Along any root-to-leaf path
    /// the `bit` fields are strictly increasing — that is the whole
    /// structural invariant.
    Branch { bit: u32, link: [usize; 2] },
}

/// Patricia over fixed 64-bit keys (§6.3). "Practical Algorithm To Retrieve
/// Information Coded In Alphanumeric" — Morrison 1968; Knuth's Algorithm P.
///
/// The two Patricia ideas, both kept here:
///
/// 1. **Skip the boring bits.** A branch node stores the index of the next
///    bit that actually *distinguishes* keys in its subtree; runs of bits on
///    which all those keys agree are never tested. No one-way branching —
///    a search inspects at most `min(64, n - 1)` bits for `n` keys.
/// 2. **One branch per key boundary.** `n` keys need exactly `n - 1` branch
///    nodes (a full binary tree with `n` leaves), independent of key length
///    — a plain binary trie can burn 64 nodes on two keys.
///
/// **Honest note.** Knuth's own Algorithm P is subtler still: it has *no*
/// leaf nodes at all — each key lives inside a branch node and is reached by
/// a link that points back *up* the tree (a tagged back pointer), so `n`
/// keys use exactly `n` nodes; reconstructing that form is exercise material
/// in §6.3. We implement the *compressed radix trie* form — separate leaves,
/// `n - 1` branches — which keeps both ideas above and is what production
/// "Patricia/crit-bit" trees actually ship.
///
/// Arena representation: `nodes[i]` is a `PatNode`; `root == NULL` iff the
/// tree is empty. There is no removal, so every arena slot stays live.
pub struct Patricia {
    nodes: Vec<PatNode>,
    root: usize,
    len: usize,
}

/// Bit `i` (0 = most significant) of a 64-bit key.
fn bit64(key: u64, i: u32) -> usize {
    ((key >> (63 - i)) & 1) as usize
}

impl Patricia {
    /// An empty Patricia tree.
    pub fn new() -> Self {
        Patricia { nodes: Vec::new(), root: NULL, len: 0 }
    }

    /// Patricia search. Branch nodes only say *which* bits to test — they
    /// cannot vouch for the skipped bits — so the search "blindly" descends
    /// to a leaf and then makes **one full key comparison** there.
    ///
    /// ```text
    /// P1. [Empty?]   If ROOT = Λ, the search fails.
    /// P2. [Descend.] While the current node is a branch testing bit j,
    ///                follow LINK[bit j of K].
    /// P3. [Compare.] At the leaf: K is present iff it equals the leaf key.
    /// ```
    pub fn contains(&self, key: u64) -> bool {
        // P1. [Empty?]
        if self.root == NULL {
            return false;
        }
        // P2. [Descend.]
        let mut x = self.root;
        loop {
            match &self.nodes[x] {
                // P3. [Compare.]
                PatNode::Leaf { key: k } => return *k == key,
                PatNode::Branch { bit, link } => x = link[bit64(key, *bit)],
            }
        }
    }

    /// Patricia insertion. Returns `false` on a duplicate.
    ///
    /// ```text
    /// I1. [Empty?]      If ROOT = Λ, a new leaf for K becomes the root.
    /// I2. [Blind hunt.] Search for K as in `contains`, reaching some leaf
    ///                   key L. If L = K: duplicate, stop.
    /// I3. [Crit bit.]   Let d = index of the first (most significant) bit
    ///                   where K and L differ. Every key in the tree that
    ///                   the hunt's path could reach agrees with L on all
    ///                   bits before d that were tested, so d is where K
    ///                   parts company with the tree.
    /// I4. [Re-descend.] Walk from ROOT again, stopping at the first node
    ///                   that is a leaf or a branch testing a bit > d.
    /// I5. [Splice.]     Replace that node with a new branch testing bit d:
    ///                   its bit-d-of-K side is a fresh leaf for K, its
    ///                   other side is the displaced node. Bit indices stay
    ///                   strictly increasing along every path.
    /// ```
    pub fn insert(&mut self, key: u64) -> bool {
        // I1. [Empty?]
        if self.root == NULL {
            self.nodes.push(PatNode::Leaf { key });
            self.root = 0;
            self.len = 1;
            return true;
        }
        // I2. [Blind hunt.]
        let mut x = self.root;
        let found = loop {
            match &self.nodes[x] {
                PatNode::Leaf { key: k } => break *k,
                PatNode::Branch { bit, link } => x = link[bit64(key, *bit)],
            }
        };
        if found == key {
            return false;
        }
        // I3. [Crit bit.] leading_zeros of the XOR is exactly the MSB-first
        // index of the first differing bit.
        let d = (key ^ found).leading_zeros();
        // I4. [Re-descend.]
        let mut parent = NULL;
        let mut pside = 0usize;
        let mut cur = self.root;
        loop {
            match &self.nodes[cur] {
                PatNode::Branch { bit, link } if *bit < d => {
                    parent = cur;
                    pside = bit64(key, *bit);
                    cur = link[pside];
                }
                _ => break,
            }
        }
        // I5. [Splice.]
        self.nodes.push(PatNode::Leaf { key });
        let leaf = self.nodes.len() - 1;
        let side = bit64(key, d);
        let mut link = [NULL, NULL];
        link[side] = leaf;
        link[1 - side] = cur;
        self.nodes.push(PatNode::Branch { bit: d, link });
        let branch = self.nodes.len() - 1;
        if parent == NULL {
            self.root = branch;
        } else if let PatNode::Branch { link, .. } = &mut self.nodes[parent] {
            link[pside] = branch;
        }
        self.len += 1;
        true
    }

    /// Total number of live nodes (leaves + branches). Since this tree only
    /// grows, that is the arena length; a correct Patricia holds `n ≥ 1`
    /// keys in exactly `n` leaves plus `n - 1` branches = `2n - 1` nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for Patricia {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Unit tests = worked examples from the lesson
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// The lesson's hand-traced order-3 insertion, eight keys:
    /// 50 30 70 10 40 60 20 80.
    #[test]
    fn btree_lesson_trace_m3() {
        let mut t = BTree::new(3);
        for &k in &[50, 30, 70] {
            assert!(t.insert(k));
        }
        // The third insert overflowed the root leaf: 50 was promoted.
        assert_eq!(t.height(), 2);
        assert!(t.is_valid());
        for &k in &[10, 40, 60, 20] {
            assert!(t.insert(k));
        }
        assert_eq!(t.height(), 2, "40 split a leaf but the root absorbed 30");
        // 80 overflows [60,70,80] -> 70 up -> root [30,50,70] overflows ->
        // 50 up: the tree grows at the top.
        assert!(t.insert(80));
        assert_eq!(t.height(), 3);
        assert!(t.is_valid());
        assert_eq!(t.keys_inorder(), vec![10, 20, 30, 40, 50, 60, 70, 80]);
        for &k in &[10, 20, 30, 40, 50, 60, 70, 80] {
            assert!(t.contains(k));
        }
        assert!(!t.contains(55));
        assert!(!t.insert(40), "duplicate must be rejected");
        assert_eq!(t.keys_inorder().len(), 8);
    }

    #[test]
    #[should_panic(expected = "at least 3")]
    fn btree_order_two_rejected() {
        BTree::new(2);
    }

    #[test]
    fn btree_height_bound_small() {
        // n keys, order m: levels <= 1 + log_{⌈m/2⌉}((n+1)/2).
        for &m in &[3usize, 4, 7] {
            let mut t = BTree::new(m);
            let n = 2000u64;
            for i in 0..n {
                // multiply by an odd constant: a bijection mod 2^64,
                // so all n keys are distinct.
                assert!(t.insert(i.wrapping_mul(0x9E37_79B9_7F4A_7C15) as i64));
            }
            assert!(t.is_valid());
            assert_eq!(t.keys_inorder().len(), n as usize);
            let half = ((m + 1) / 2) as f64;
            let bound = 1.0 + (((n + 1) as f64) / 2.0).ln() / half.ln();
            assert!((t.height() as f64) <= bound + 1e-9, "m={m} h={}", t.height());
        }
    }

    #[test]
    fn trie_shared_prefix_example() {
        // Keys sharing 24 bits: the trie stores the common spine once.
        let mut t = BinaryTrie::new();
        assert_eq!(t.count(), 0);
        assert!(!t.contains(0));
        for i in 0..8u32 {
            assert!(t.insert(0xABCD_EF00 | i));
        }
        assert!(!t.insert(0xABCD_EF03), "duplicate");
        assert_eq!(t.count(), 8);
        assert!(t.contains(0xABCD_EF05));
        assert!(!t.contains(0xABCD_EF08));
        assert!(t.remove(0xABCD_EF05));
        assert!(!t.remove(0xABCD_EF05));
        assert!(!t.contains(0xABCD_EF05));
        assert!(t.insert(0xABCD_EF05), "reinsert after remove");
        assert_eq!(t.count(), 8);
        assert!(t.insert(0) && t.insert(u32::MAX));
        assert_eq!(t.count(), 10);
    }

    #[test]
    fn patricia_worked_example() {
        // The lesson diagram: 8 = ...1000, 11 = ...1011, 10 = ...1010.
        let mut p = Patricia::new();
        assert!(!p.contains(7));
        assert!(p.insert(0b1000));
        assert_eq!(p.node_count(), 1);
        assert!(p.insert(0b1011)); // first differing bit vs 8: bit 62 (value 2)
        assert_eq!(p.node_count(), 3);
        assert!(p.insert(0b1010)); // then bit 63 (value 1) splits 10 from 11
        assert_eq!(p.node_count(), 5);
        assert!(!p.insert(0b1010), "duplicate");
        for k in 0..16u64 {
            assert_eq!(p.contains(k), k == 8 || k == 10 || k == 11, "key {k}");
        }
    }

    #[test]
    fn patricia_order_independence() {
        let keys: Vec<u64> = (0..64).map(|i| 1u64 << i).chain([0, u64::MAX]).collect();
        let mut a = Patricia::new();
        let mut b = Patricia::new();
        for &k in &keys {
            assert!(a.insert(k));
        }
        for &k in keys.iter().rev() {
            assert!(b.insert(k));
        }
        assert_eq!(a.node_count(), 2 * keys.len() - 1);
        assert_eq!(a.node_count(), b.node_count());
        for probe in 0..2000u64 {
            let x = probe.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            assert_eq!(a.contains(x), b.contains(x));
            assert_eq!(a.contains(x), keys.contains(&x));
        }
    }
}
