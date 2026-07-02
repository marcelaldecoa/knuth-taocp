//! Module 07 — Searching.
//! Source: TAOCP Vol. 3, 2nd ed., Ch. 6: §6.2.1 (binary search), §6.2.2
//! (binary search trees), §6.2.3 (balanced trees), §6.4 (hashing).

// ===========================================================================
// Stage 1 — Algorithm 6.2.1B: binary search
// ===========================================================================

/// Algorithm 6.2.1B (Binary search), step-faithful.
///
/// Searches the sorted slice `a` for `key`. Knuth's algorithm reports only
/// "successful at position i" or "unsuccessful"; following the Rust standard
/// library convention we enrich the unsuccessful outcome with the insertion
/// point: `Ok(i)` means `a[i] == key` (if `key` occurs several times, *any*
/// matching index may be returned), and `Err(p)` means `key` is absent and
/// inserting it at index `p` keeps `a` sorted.
///
/// ```text
/// B1. [Initialize.]   Set l <- 1, u <- N.
/// B2. [Get midpoint.] If u < l, the algorithm terminates unsuccessfully.
///                     Otherwise set i <- floor((l + u)/2).
/// B3. [Compare.]      If K < K_i, go to B4; if K > K_i, go to B5;
///                     if K = K_i, the algorithm terminates successfully.
/// B4. [Adjust u.]     Set u <- i - 1 and return to B2.
/// B5. [Adjust l.]     Set l <- i + 1 and return to B2.
/// ```
pub fn binary_search(a: &[i64], key: i64) -> Result<usize, usize> {
    binary_search_comparisons(a, key).0
}

/// Algorithm 6.2.1B, instrumented: also returns C, the number of executions
/// of step B3 (each three-way comparison K : K_i counts once).
///
/// Theorem 6.2.1B: C <= floor(lg N) + 1 for every search, successful or not
/// (and C = 0 only for N = 0).
pub fn binary_search_comparisons(a: &[i64], key: i64) -> (Result<usize, usize>, u32) {
    // B1. [Initialize.] l <- 1, u <- N.  (1-based indices, as in the text;
    //     l + u cannot overflow because a slice of i64 holds < 2^61 elements.)
    let mut l: usize = 1;
    let mut u: usize = a.len();
    let mut c: u32 = 0;
    loop {
        // B2. [Get midpoint.] If u < l, terminate unsuccessfully.
        if u < l {
            // Everything at 1-based positions < l is < key, everything at
            // positions > u = l - 1 is > key: 0-based insertion point l - 1.
            return (Err(l - 1), c);
        }
        let i = (l + u) / 2; // i <- floor((l + u)/2)

        // B3. [Compare.]
        c += 1;
        if key < a[i - 1] {
            // B4. [Adjust u.] u <- i - 1.
            u = i - 1;
        } else if key > a[i - 1] {
            // B5. [Adjust l.] l <- i + 1.
            l = i + 1;
        } else {
            return (Ok(i - 1), c);
        }
    }
}

// ===========================================================================
// Stage 2 — Algorithm 6.2.2T (tree search and insertion) + Algorithm 6.2.2D
// (Hibbard-style deletion by symmetric successor)
// ===========================================================================

/// Null link sentinel for the index-based arenas in this module (MIX's Λ).
const NULL: usize = usize::MAX;

#[derive(Clone)]
struct BstNode {
    key: i64,
    left: usize,  // LLINK, NULL = Λ
    right: usize, // RLINK, NULL = Λ
}

/// An (unbalanced) binary search tree over `i64` keys, TAOCP Vol. 3, §6.2.2.
///
/// Nodes live in an index-based arena (`Vec<BstNode>` with `usize` links;
/// `usize::MAX` is the null sentinel Λ). Deleted slots are recycled through
/// a free list.
pub struct Bst {
    nodes: Vec<BstNode>,
    root: usize,
    free: Vec<usize>,
}

impl Bst {
    /// An empty tree.
    pub fn new() -> Self {
        Bst { nodes: Vec::new(), root: NULL, free: Vec::new() }
    }

    fn alloc(&mut self, key: i64) -> usize {
        let node = BstNode { key, left: NULL, right: NULL };
        match self.free.pop() {
            Some(q) => {
                self.nodes[q] = node;
                q
            }
            None => {
                self.nodes.push(node);
                self.nodes.len() - 1
            }
        }
    }

    /// Algorithm 6.2.2T (Tree search and insertion), insertion half.
    /// Returns `false` (tree unchanged) if `key` is already present.
    ///
    /// ```text
    /// T1. [Initialize.] Set P <- ROOT.  (Empty tree: insert at the root.)
    /// T2. [Compare.]    If K < KEY(P) go to T3; if K > KEY(P) go to T4;
    ///                   otherwise the search is successful (duplicate).
    /// T3. [Move left.]  If LLINK(P) != Λ set P <- LLINK(P), go to T2;
    ///                   else go to T5.
    /// T4. [Move right.] Same with RLINK(P).
    /// T5. [Insert.]     Q <= AVAIL; KEY(Q) <- K, LLINK(Q) <- RLINK(Q) <- Λ;
    ///                   hang Q off P on the side the search fell off.
    /// ```
    pub fn insert(&mut self, key: i64) -> bool {
        // T1. [Initialize.]
        if self.root == NULL {
            self.root = self.alloc(key);
            return true;
        }
        let mut p = self.root;
        loop {
            // T2. [Compare.]
            if key < self.nodes[p].key {
                // T3. [Move left.]
                if self.nodes[p].left != NULL {
                    p = self.nodes[p].left;
                } else {
                    // T5. [Insert into tree.]
                    let q = self.alloc(key);
                    self.nodes[p].left = q;
                    return true;
                }
            } else if key > self.nodes[p].key {
                // T4. [Move right.]
                if self.nodes[p].right != NULL {
                    p = self.nodes[p].right;
                } else {
                    // T5. [Insert into tree.]
                    let q = self.alloc(key);
                    self.nodes[p].right = q;
                    return true;
                }
            } else {
                return false; // duplicate: the search was successful
            }
        }
    }

    /// Algorithm 6.2.2T, search half (steps T1-T4 with T5 replaced by
    /// "terminate unsuccessfully").
    pub fn contains(&self, key: i64) -> bool {
        let mut p = self.root;
        while p != NULL {
            if key < self.nodes[p].key {
                p = self.nodes[p].left;
            } else if key > self.nodes[p].key {
                p = self.nodes[p].right;
            } else {
                return true;
            }
        }
        false
    }

    /// Algorithm 6.2.2D (Tree deletion), the Hibbard/Knuth scheme: a node
    /// with two children is replaced by its symmetric (inorder) successor.
    /// Returns `false` (tree unchanged) if `key` is absent.
    ///
    /// With T the doomed node and Q the link that will replace it:
    /// ```text
    /// D1. [Is RLINK null?]   If RLINK(T) = Λ, set Q <- LLINK(T); go to D4.
    /// D2. [Find successor.]  Set R <- RLINK(T). If LLINK(R) = Λ, set
    ///                        LLINK(R) <- LLINK(T), Q <- R; go to D4.
    /// D3. [Find null LLINK.] Set S <- LLINK(R); while LLINK(S) != Λ set
    ///                        R <- S, S <- LLINK(S).  (S = successor of T.)
    ///                        Then LLINK(S) <- LLINK(T), LLINK(R) <- RLINK(S),
    ///                        RLINK(S) <- RLINK(T), Q <- S.
    /// D4. [Anchor.]          Replace the parent's link to T by Q.
    /// ```
    pub fn delete(&mut self, key: i64) -> bool {
        // Find T and the link pointing to it (parent + side; parent = NULL
        // means T is the root).
        let mut parent = NULL;
        let mut left_side = false;
        let mut t = self.root;
        while t != NULL {
            if key < self.nodes[t].key {
                parent = t;
                left_side = true;
                t = self.nodes[t].left;
            } else if key > self.nodes[t].key {
                parent = t;
                left_side = false;
                t = self.nodes[t].right;
            } else {
                break;
            }
        }
        if t == NULL {
            return false;
        }

        let q; // the subtree that replaces T
        if self.nodes[t].right == NULL {
            // D1. [Is RLINK null?]
            q = self.nodes[t].left;
        } else {
            // D2. [Find successor.]
            let r = self.nodes[t].right;
            if self.nodes[r].left == NULL {
                self.nodes[r].left = self.nodes[t].left;
                q = r;
            } else {
                // D3. [Find null LLINK.]
                let mut r2 = r;
                let mut s = self.nodes[r2].left;
                while self.nodes[s].left != NULL {
                    r2 = s;
                    s = self.nodes[s].left;
                }
                self.nodes[s].left = self.nodes[t].left;
                self.nodes[r2].left = self.nodes[s].right;
                self.nodes[s].right = self.nodes[t].right;
                q = s;
            }
        }

        // D4. [Anchor.]
        if parent == NULL {
            self.root = q;
        } else if left_side {
            self.nodes[parent].left = q;
        } else {
            self.nodes[parent].right = q;
        }
        self.free.push(t);
        true
    }

    /// The keys in symmetric (inorder) order — always sorted, by the BST
    /// property. Iterative (explicit stack), so degenerate trees are fine.
    pub fn inorder(&self) -> Vec<i64> {
        let mut out = Vec::new();
        let mut stack = Vec::new();
        let mut p = self.root;
        while p != NULL || !stack.is_empty() {
            while p != NULL {
                stack.push(p);
                p = self.nodes[p].left;
            }
            let q = stack.pop().expect("stack nonempty");
            out.push(self.nodes[q].key);
            p = self.nodes[q].right;
        }
        out
    }

    /// Height in **edges**: the length of the longest root-to-leaf path.
    /// A single-node tree has height 0; so does the empty tree (by fiat).
    pub fn height(&self) -> usize {
        if self.root == NULL {
            return 0;
        }
        let mut h = 0;
        let mut stack = vec![(self.root, 0usize)];
        while let Some((p, d)) = stack.pop() {
            h = h.max(d);
            let node = &self.nodes[p];
            if node.left != NULL {
                stack.push((node.left, d + 1));
            }
            if node.right != NULL {
                stack.push((node.right, d + 1));
            }
        }
        h
    }
}

impl Default for Bst {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Stage 3 — Algorithm 6.2.3A: balanced (AVL) tree insertion
// ===========================================================================

#[derive(Clone)]
struct AvlNode {
    key: i64,
    left: usize,  // LLINK
    right: usize, // RLINK
    bal: i8,      // B(P) = height(right subtree) - height(left subtree)
}

/// A height-balanced (AVL) binary search tree, TAOCP Vol. 3, §6.2.3:
/// every node's two subtree heights differ by at most 1. Insertion is
/// Algorithm 6.2.3A with balance factors; no deletion (neither does Knuth).
pub struct AvlTree {
    nodes: Vec<AvlNode>,
    root: usize,
}

impl AvlTree {
    /// An empty tree.
    pub fn new() -> Self {
        AvlTree { nodes: Vec::new(), root: NULL }
    }

    fn alloc(&mut self, key: i64) -> usize {
        self.nodes.push(AvlNode { key, left: NULL, right: NULL, bal: 0 });
        self.nodes.len() - 1
    }

    /// LINK(a, P): the left link if a = -1, the right link if a = +1.
    fn link(&self, a: i8, p: usize) -> usize {
        if a < 0 {
            self.nodes[p].left
        } else {
            self.nodes[p].right
        }
    }

    fn set_link(&mut self, a: i8, p: usize, q: usize) {
        if a < 0 {
            self.nodes[p].left = q;
        } else {
            self.nodes[p].right = q;
        }
    }

    /// Algorithm 6.2.3A (Balanced tree search and insertion).
    /// Returns `false` (tree unchanged) if `key` is already present.
    ///
    /// Pointer cast: P walks the search path; S is the deepest node on the
    /// path with B(S) != 0 (the only place rebalancing can be needed);
    /// T is S's parent (Knuth's header node stands in for "S is the root").
    ///
    /// ```text
    /// A1. [Initialize.] T <- HEAD, S <- P <- ROOT.  (Empty tree: just insert.)
    /// A2. [Compare.]    If K < KEY(P) go to A3; if K > KEY(P) go to A4;
    ///                   otherwise the key is already there.
    /// A3. [Move left.]  Q <- LLINK(P). If Q = Λ, allocate Q, LLINK(P) <- Q,
    ///                   go to A5. If B(Q) != 0, T <- P, S <- Q. P <- Q, to A2.
    /// A4. [Move right.] Mirror image of A3.
    /// A5. [Insert.]     KEY(Q) <- K, LLINK(Q) <- RLINK(Q) <- Λ, B(Q) <- 0.
    /// A6. [Adjust balance factors.] a <- -1 if K < KEY(S), else +1. Walk
    ///                   R <- P <- LINK(a, S) toward Q, setting each node's
    ///                   balance factor to -1 or +1 as the path goes.
    /// A7. [Balancing act.] (i) B(S) = 0: B(S) <- a, whole tree grew, done.
    ///                   (ii) B(S) = -a: B(S) <- 0, tree got more balanced.
    ///                   (iii) B(S) = a: rebalance! Go to A8 if B(R) = a,
    ///                   to A9 if B(R) = -a.
    /// A8. [Single rotation.] P <- R, LINK(a,S) <- LINK(-a,R),
    ///                   LINK(-a,R) <- S, B(S) <- B(R) <- 0. Go to A10.
    /// A9. [Double rotation.] P <- LINK(-a,R), LINK(-a,R) <- LINK(a,P),
    ///                   LINK(a,P) <- R, LINK(a,S) <- LINK(-a,P),
    ///                   LINK(-a,P) <- S; then set
    ///                   (B(S),B(R)) <- (-a,0) if B(P) = a, (0,0) if B(P) = 0,
    ///                   (0,a) if B(P) = -a; and B(P) <- 0.
    /// A10. [Finishing touch.] The subtree that hung from T where S was now
    ///                   hangs P (the new subtree root).
    /// ```
    pub fn insert(&mut self, key: i64) -> bool {
        // A1. [Initialize.]
        if self.root == NULL {
            self.root = self.alloc(key);
            return true;
        }
        let mut t = NULL; // parent of S; NULL = S is the root
        let mut s = self.root;
        let mut p = self.root;
        let q;
        loop {
            // A2. [Compare.]
            if key < self.nodes[p].key {
                // A3. [Move left.]
                let c = self.nodes[p].left;
                if c == NULL {
                    // A5. [Insert.]
                    q = self.alloc(key);
                    self.nodes[p].left = q;
                    break;
                }
                if self.nodes[c].bal != 0 {
                    t = p;
                    s = c;
                }
                p = c;
            } else if key > self.nodes[p].key {
                // A4. [Move right.]
                let c = self.nodes[p].right;
                if c == NULL {
                    // A5. [Insert.]
                    q = self.alloc(key);
                    self.nodes[p].right = q;
                    break;
                }
                if self.nodes[c].bal != 0 {
                    t = p;
                    s = c;
                }
                p = c;
            } else {
                return false; // duplicate
            }
        }

        // A6. [Adjust balance factors.] Every node strictly between S and Q
        // had balance factor 0 (by choice of S) and now leans toward Q.
        let a: i8 = if key < self.nodes[s].key { -1 } else { 1 };
        let r = self.link(a, s);
        let mut w = r;
        while w != q {
            if key < self.nodes[w].key {
                self.nodes[w].bal = -1;
                w = self.nodes[w].left;
            } else {
                self.nodes[w].bal = 1;
                w = self.nodes[w].right;
            }
        }

        // A7. [Balancing act.]
        if self.nodes[s].bal == 0 {
            // (i) The whole tree has grown one level taller.
            self.nodes[s].bal = a;
            return true;
        }
        if self.nodes[s].bal == -a {
            // (ii) The tree has gotten more balanced.
            self.nodes[s].bal = 0;
            return true;
        }
        // (iii) B(S) = a: the tree is out of balance at S.
        let new_root_of_subtree;
        if self.nodes[r].bal == a {
            // A8. [Single rotation.]
            new_root_of_subtree = r;
            let inner = self.link(-a, r);
            self.set_link(a, s, inner);
            self.set_link(-a, r, s);
            self.nodes[s].bal = 0;
            self.nodes[r].bal = 0;
        } else {
            // A9. [Double rotation.]
            let p2 = self.link(-a, r);
            let x = self.link(a, p2);
            self.set_link(-a, r, x);
            self.set_link(a, p2, r);
            let y = self.link(-a, p2);
            self.set_link(a, s, y);
            self.set_link(-a, p2, s);
            let bp = self.nodes[p2].bal;
            let (bs, br) = if bp == a {
                (-a, 0)
            } else if bp == 0 {
                (0, 0)
            } else {
                (0, a)
            };
            self.nodes[s].bal = bs;
            self.nodes[r].bal = br;
            self.nodes[p2].bal = 0;
            new_root_of_subtree = p2;
        }

        // A10. [Finishing touch.]
        if t == NULL {
            self.root = new_root_of_subtree;
        } else if s == self.nodes[t].right {
            self.nodes[t].right = new_root_of_subtree;
        } else {
            self.nodes[t].left = new_root_of_subtree;
        }
        true
    }

    /// Tree search (Algorithm 6.2.2T's search half — balance factors don't
    /// change the search).
    pub fn contains(&self, key: i64) -> bool {
        let mut p = self.root;
        while p != NULL {
            if key < self.nodes[p].key {
                p = self.nodes[p].left;
            } else if key > self.nodes[p].key {
                p = self.nodes[p].right;
            } else {
                return true;
            }
        }
        false
    }

    /// The keys in symmetric (inorder) order.
    pub fn inorder(&self) -> Vec<i64> {
        let mut out = Vec::new();
        let mut stack = Vec::new();
        let mut p = self.root;
        while p != NULL || !stack.is_empty() {
            while p != NULL {
                stack.push(p);
                p = self.nodes[p].left;
            }
            let q = stack.pop().expect("stack nonempty");
            out.push(self.nodes[q].key);
            p = self.nodes[q].right;
        }
        out
    }

    /// Height in **edges** (empty and single-node trees both have height 0).
    pub fn height(&self) -> usize {
        if self.root == NULL {
            return 0;
        }
        let mut h = 0;
        let mut stack = vec![(self.root, 0usize)];
        while let Some((p, d)) = stack.pop() {
            h = h.max(d);
            let node = &self.nodes[p];
            if node.left != NULL {
                stack.push((node.left, d + 1));
            }
            if node.right != NULL {
                stack.push((node.right, d + 1));
            }
        }
        h
    }

    /// The AVL invariant, checked from scratch: recompute every subtree
    /// height and verify that each node's stored balance factor equals
    /// height(right) - height(left) *and* lies in {-1, 0, +1}.
    pub fn is_balanced(&self) -> bool {
        if self.root == NULL {
            return true;
        }
        // Iterative post-order; height[p] = height of p's subtree in edges.
        let mut height = vec![0i64; self.nodes.len()];
        let mut stack = vec![(self.root, false)];
        let mut ok = true;
        while let Some((p, children_done)) = stack.pop() {
            let (l, r) = (self.nodes[p].left, self.nodes[p].right);
            if !children_done {
                stack.push((p, true));
                if l != NULL {
                    stack.push((l, false));
                }
                if r != NULL {
                    stack.push((r, false));
                }
            } else {
                let lh = if l == NULL { -1 } else { height[l] };
                let rh = if r == NULL { -1 } else { height[r] };
                height[p] = 1 + lh.max(rh);
                let b = rh - lh;
                if b.abs() > 1 || b != i64::from(self.nodes[p].bal) {
                    ok = false;
                }
            }
        }
        ok
    }
}

impl Default for AvlTree {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Stage 4 — §6.4: hashing with open addressing
// (Algorithm 6.4L linear probing, Algorithm 6.4D double hashing)
// ===========================================================================

/// Trial-division primality test, adequate for table sizes.
fn is_prime(m: usize) -> bool {
    if m < 2 {
        return false;
    }
    if m % 2 == 0 {
        return m == 2;
    }
    let mut d = 3;
    while d * d <= m {
        if m % d == 0 {
            return false;
        }
        d += 2;
    }
    true
}

/// Open-addressing hash table with **linear probing** — Algorithm 6.4L.
///
/// Division hashing h(K) = K mod M with M prime; the probe sequence
/// decreases cyclically: h(K), h(K)-1, ..., 0, M-1, ... As in the text the
/// table is declared full at N = M - 1 (one slot always stays empty so that
/// unsuccessful searches terminate).
pub struct LinearProbe {
    table: Vec<Option<u64>>,
    m: usize,
    n: usize, // number of occupied slots, N < M
}

impl LinearProbe {
    /// An empty table of size `m`. Panics unless `m` is a prime >= 3
    /// (the division method wants a prime modulus — see the lesson).
    pub fn new(m: usize) -> Self {
        assert!(m >= 3 && is_prime(m), "table size M must be a prime >= 3");
        LinearProbe { table: vec![None; m], m, n: 0 }
    }

    /// Algorithm 6.4L (Linear probing and insertion). Returns `false` and
    /// leaves the table unchanged if `key` is already present *or* the
    /// table is full (N = M - 1).
    ///
    /// ```text
    /// L1. [Hash.]            i <- h(K).  (0 <= i < M.)
    /// L2. [Compare.]         If TABLE[i] is empty, go to L4. If it holds K,
    ///                        the search is successful (here: duplicate).
    /// L3. [Advance to next.] i <- i - 1; if i < 0, i <- i + M. Back to L2.
    /// L4. [Insert.]          If N = M - 1, overflow. Else N <- N + 1,
    ///                        TABLE[i] <- K.
    /// ```
    pub fn insert(&mut self, key: u64) -> bool {
        // L1. [Hash.]
        let mut i = (key % self.m as u64) as usize;
        loop {
            // L2. [Compare.]
            match self.table[i] {
                None => break,
                Some(k) if k == key => return false, // duplicate
                _ => {
                    // L3. [Advance to next.]
                    i = if i == 0 { self.m - 1 } else { i - 1 };
                }
            }
        }
        // L4. [Insert.]
        if self.n == self.m - 1 {
            return false; // overflow: the table is full
        }
        self.n += 1;
        self.table[i] = Some(key);
        true
    }

    /// The search half of Algorithm 6.4L.
    pub fn contains(&self, key: u64) -> bool {
        let mut i = (key % self.m as u64) as usize;
        loop {
            match self.table[i] {
                None => return false,
                Some(k) if k == key => return true,
                _ => i = if i == 0 { self.m - 1 } else { i - 1 },
            }
        }
    }

    /// The number of probes (table slots examined) a search for `key`
    /// makes, counting the final probe — the one that finds `key`, or the
    /// empty slot that proves it absent. This is the quantity C in Knuth's
    /// 1962 analysis (see the lesson).
    pub fn probes_for(&self, key: u64) -> u32 {
        let mut i = (key % self.m as u64) as usize;
        let mut c = 1u32;
        loop {
            match self.table[i] {
                None => return c,
                Some(k) if k == key => return c,
                _ => {
                    i = if i == 0 { self.m - 1 } else { i - 1 };
                    c += 1;
                }
            }
        }
    }
}

/// Open-addressing hash table with **double hashing** — Algorithm 6.4D.
///
/// h1(K) = K mod M, and after the first collision the probe decrement is
/// c = h2(K) = 1 + (K mod (M-2)), so 1 <= c <= M-2. Since M is prime, c is
/// relatively prime to M and the probe sequence visits every slot.
pub struct DoubleHash {
    table: Vec<Option<u64>>,
    m: usize,
    n: usize,
}

impl DoubleHash {
    /// An empty table of size `m`. Panics unless `m` is a prime >= 3.
    /// (Choosing M so that M and M-2 are twin primes is Knuth's refinement;
    /// any prime M works.)
    pub fn new(m: usize) -> Self {
        assert!(m >= 3 && is_prime(m), "table size M must be a prime >= 3");
        DoubleHash { table: vec![None; m], m, n: 0 }
    }

    fn h2(&self, key: u64) -> usize {
        1 + (key % (self.m as u64 - 2)) as usize
    }

    /// Algorithm 6.4D (Open addressing with double hashing). Returns
    /// `false` if `key` is a duplicate or the table is full (N = M - 1).
    ///
    /// ```text
    /// D1. [First hash.]      i <- h1(K).
    /// D2. [First probe.]     If TABLE[i] is empty, go to D6. If it holds K,
    ///                        successful (duplicate).
    /// D3. [Second hash.]     c <- h2(K).
    /// D4. [Advance to next.] i <- i - c; if i < 0, i <- i + M.
    /// D5. [Compare.]         If TABLE[i] is empty, go to D6. If it holds K,
    ///                        successful. Otherwise back to D4.
    /// D6. [Insert.]          If N = M - 1, overflow. Else N <- N + 1,
    ///                        TABLE[i] <- K.
    /// ```
    pub fn insert(&mut self, key: u64) -> bool {
        // D1. [First hash.]
        let mut i = (key % self.m as u64) as usize;
        // D2. [First probe.]
        let full_or_dup = loop {
            match self.table[i] {
                None => break false,
                Some(k) if k == key => break true,
                _ => {
                    // D3. [Second hash.] / D4. [Advance to next.] / D5. [Compare.]
                    let c = self.h2(key);
                    i = (i + self.m - c) % self.m;
                }
            }
        };
        if full_or_dup {
            return false; // duplicate
        }
        // D6. [Insert.]
        if self.n == self.m - 1 {
            return false; // overflow
        }
        self.n += 1;
        self.table[i] = Some(key);
        true
    }

    /// The search half of Algorithm 6.4D.
    pub fn contains(&self, key: u64) -> bool {
        let mut i = (key % self.m as u64) as usize;
        loop {
            match self.table[i] {
                None => return false,
                Some(k) if k == key => return true,
                _ => {
                    let c = self.h2(key);
                    i = (i + self.m - c) % self.m;
                }
            }
        }
    }

    /// Probe count of a search for `key` (see `LinearProbe::probes_for`).
    pub fn probes_for(&self, key: u64) -> u32 {
        let mut i = (key % self.m as u64) as usize;
        let mut count = 1u32;
        loop {
            match self.table[i] {
                None => return count,
                Some(k) if k == key => return count,
                _ => {
                    let c = self.h2(key);
                    i = (i + self.m - c) % self.m;
                    count += 1;
                }
            }
        }
    }
}

// ===========================================================================
// Unit tests: worked examples from the text
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Knuth's running example table in §6.2.1: 16 sorted keys.
    const SIXTEEN: [i64; 16] = [
        61, 87, 154, 170, 275, 426, 503, 509, 512, 612, 653, 677, 703, 765, 897, 908,
    ];

    #[test]
    fn binary_search_worked_example_6_2_1() {
        // §6.2.1 traces the search for K = 503: probes 509, 170, 426, 503.
        let (res, c) = binary_search_comparisons(&SIXTEEN, 503);
        assert_eq!(res, Ok(6));
        assert_eq!(c, 4);
        // Unsuccessful search for 400: probes 509, 170, 426, 275, then u < l.
        let (res, c) = binary_search_comparisons(&SIXTEEN, 400);
        assert_eq!(res, Err(5)); // five keys are < 400
        assert_eq!(c, 4);
        // Theorem B: at most floor(lg 16) + 1 = 5 comparisons, any key.
        for key in 0..1000 {
            let (res, c) = binary_search_comparisons(&SIXTEEN, key);
            assert!(c <= 5);
            assert_eq!(res, binary_search(&SIXTEEN, key));
        }
        assert_eq!(binary_search(&[], 42), Err(0));
    }

    #[test]
    fn bst_insertion_and_deletion_6_2_2() {
        // Insert the sixteen keys in Knuth's "input order" for Fig. 10-style
        // trees; inorder must come out sorted whatever the input order.
        let order = [503, 87, 512, 61, 908, 170, 897, 275, 653, 426, 154, 509, 612, 677, 765, 703];
        let mut t = Bst::new();
        for &k in &order {
            assert!(t.insert(k));
            assert!(!t.insert(k), "duplicate {k} must be rejected");
        }
        let mut sorted = SIXTEEN.to_vec();
        sorted.sort_unstable();
        assert_eq!(t.inorder(), sorted);
        // Delete the root (two children): its symmetric successor 509 takes
        // its place; inorder stays sorted minus 503.
        assert!(t.delete(503));
        assert!(!t.contains(503));
        assert!(!t.delete(503));
        let expect: Vec<i64> = sorted.iter().copied().filter(|&k| k != 503).collect();
        assert_eq!(t.inorder(), expect);
    }

    #[test]
    fn bst_degenerate_and_height() {
        let mut t = Bst::new();
        for k in 0..100 {
            t.insert(k);
        }
        assert_eq!(t.height(), 99); // sorted insertion: a 99-edge vine
        let mut t = Bst::new();
        t.insert(7);
        assert_eq!(t.height(), 0);
    }

    #[test]
    fn avl_stays_balanced_6_2_3() {
        // Ascending insertions: the classic single-rotation workout.
        let mut t = AvlTree::new();
        for k in 1..=7 {
            assert!(t.insert(k));
            assert!(t.is_balanced(), "unbalanced after inserting {k}");
        }
        assert_eq!(t.inorder(), (1..=7).collect::<Vec<_>>());
        assert_eq!(t.height(), 2); // 7 nodes -> perfect tree of height 2
        // The four rotation shapes on three keys.
        for order in [[1, 2, 3], [3, 2, 1], [3, 1, 2], [1, 3, 2]] {
            let mut t = AvlTree::new();
            for k in order {
                t.insert(k);
            }
            assert!(t.is_balanced());
            assert_eq!(t.height(), 1);
            assert_eq!(t.inorder(), vec![1, 2, 3]);
        }
    }

    #[test]
    fn avl_random_10k() {
        let mut t = AvlTree::new();
        let mut x: u64 = 2463534242;
        let mut n = 0u32;
        for _ in 0..10_000 {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            if t.insert((x >> 16) as i64) {
                n += 1;
            }
        }
        assert!(t.is_balanced());
        let ino = t.inorder();
        assert_eq!(ino.len(), n as usize);
        assert!(ino.windows(2).all(|w| w[0] < w[1]));
        assert!(t.height() <= 18); // 1.4405 lg(10002) - 0.3277 < 19 (edges)
    }

    #[test]
    fn hashing_algorithms_l_and_d() {
        // Tiny table, M = 7: h(K) = K mod 7. Insert 12, 19 (collision at 5),
        // 5 (collision at 5, then 4, then 3).
        let mut lp = LinearProbe::new(7);
        assert!(lp.insert(12)); // slot 5
        assert!(lp.insert(19)); // 19 mod 7 = 5, taken -> slot 4
        assert!(lp.insert(5)); //  5 mod 7 = 5, then 4, -> slot 3
        assert_eq!(lp.probes_for(12), 1);
        assert_eq!(lp.probes_for(19), 2);
        assert_eq!(lp.probes_for(5), 3);
        assert!(lp.contains(19) && !lp.contains(26));
        assert!(!lp.insert(19), "duplicate");
        // Fill to N = M - 1 = 6, then overflow.
        assert!(lp.insert(0) && lp.insert(1) && lp.insert(2));
        assert!(!lp.insert(100), "table full");
        assert!(lp.contains(2));

        // Double hashing spreads the same colliding keys: c = 1 + K mod 5.
        let mut dh = DoubleHash::new(7);
        assert!(dh.insert(12)); // slot 5
        assert!(dh.insert(19)); // slot 5 taken, c = 1 + 4 = 5 -> slot 0
        assert!(dh.insert(5)); //  slot 5 taken, c = 1 + 0 = 1 -> slot 4
        assert_eq!(dh.probes_for(12), 1);
        assert_eq!(dh.probes_for(19), 2);
        assert_eq!(dh.probes_for(5), 2);
        assert!(dh.contains(5) && !dh.contains(26));
    }

    #[test]
    #[should_panic(expected = "prime")]
    fn hashing_rejects_composite_m() {
        LinearProbe::new(100);
    }
}
