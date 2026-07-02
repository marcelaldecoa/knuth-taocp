//! Module 07 — Searching (TAOCP Vol. 3, Ch. 6).
//!
//! **Scaffolding tier — Module 05 and up:** the stub states the algorithm and
//! the contract and trusts you to translate it to Rust; the guided-tour aids of
//! Modules 01–04 are gone by design. The nets remain for every stage — the
//! lesson, three graduated hints (`--hint`), the reference, and the walkthrough.
//! (The taper is described in docs/for-newcomers.md §5.)
//!
//! YOUR WORKSPACE. Replace each `todo!()` with an implementation, then run
//! `./grade 7` from the repository root. Work the stages in order; the lesson
//! in `course/module-07-searching/README.md` develops all the theory.
//!
//! Conventions for this module (see CONVENTIONS.md):
//! - Trees use **index-based arenas**: nodes live in a `Vec`, links are
//!   `usize` indices, and `usize::MAX` plays the role of Knuth's null link Λ.
//!   The arenas and constructors are already set up for you.
//! - All tree keys are `i64`; hash keys are `u64`.
//! - Keep Knuth's step labels (B1, T1, A1, L1, D1, ...) as comments.

// ===========================================================================
// Stage 1 — Algorithm 6.2.1B: binary search
// ===========================================================================

/// Stage 1 — Algorithm 6.2.1B (Binary search).
///
/// Search the sorted slice `a` for `key`. Knuth's algorithm merely reports
/// success or failure; we enrich the result the way `<[T]>::binary_search`
/// in std does:
/// - `Ok(i)`  — `a[i] == key` (if `key` occurs more than once, returning
///   *any* matching index is correct);
/// - `Err(p)` — `key` is absent; inserting it at index `p` keeps `a` sorted
///   (`p` = number of elements `< key`).
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
///
/// Hints: Knuth's indices are 1-based (`K_i` is `a[i - 1]`); with 1-based
/// `l` and `u`, `u = i - 1` never underflows a `usize`, and `l + u` cannot
/// overflow because a slice of `i64` holds fewer than 2^61 elements. When
/// the loop falls out at `u < l`, the 0-based insertion point is `l - 1`.
pub fn binary_search(a: &[i64], key: i64) -> Result<usize, usize> {
    let _ = (a, key);
    todo!("implement Algorithm 6.2.1B")
}

/// Stage 1 — Algorithm 6.2.1B, instrumented.
///
/// Same search, but also return C = the number of executions of step B3
/// (each three-way comparison `K : K_i` counts **once**, however you write
/// it in Rust). Theorem 6.2.1B, which the tests enforce: every search makes
/// at most `floor(lg N) + 1` comparisons.
///
/// Easiest plan: implement the algorithm here, and let `binary_search`
/// delegate to this function, discarding the count.
pub fn binary_search_comparisons(a: &[i64], key: i64) -> (Result<usize, usize>, u32) {
    let _ = (a, key);
    todo!("implement Algorithm 6.2.1B with a comparison counter")
}

// ===========================================================================
// Stage 2 — Algorithm 6.2.2T (tree search and insertion) + Algorithm 6.2.2D
// ===========================================================================

/// Null link sentinel (Knuth's Λ) for the arenas in this module.
const NULL: usize = usize::MAX;

/// One node of the (unbalanced) binary search tree.
#[allow(dead_code)] // read once you implement the methods below
struct BstNode {
    key: i64,
    left: usize,  // LLINK, NULL = Λ
    right: usize, // RLINK, NULL = Λ
}

/// Stage 2 — a binary search tree over `i64` keys (TAOCP Vol. 3, §6.2.2).
///
/// Arena representation: `nodes[i]` is node `i`; `root == NULL` means the
/// tree is empty. `free` recycles the slots of deleted nodes (push a slot
/// on deletion, pop one on insertion before growing `nodes`).
pub struct Bst {
    #[allow(dead_code)] // read once you implement the methods below
    nodes: Vec<BstNode>,
    #[allow(dead_code)]
    root: usize,
    #[allow(dead_code)]
    free: Vec<usize>,
}

impl Bst {
    /// An empty tree. (Done for you.)
    pub fn new() -> Self {
        Bst { nodes: Vec::new(), root: NULL, free: Vec::new() }
    }

    /// Algorithm 6.2.2T (Tree search and insertion), insertion half.
    /// Return `false` — leaving the tree unchanged — if `key` is already
    /// present; `true` if a node was inserted.
    ///
    /// ```text
    /// T1. [Initialize.] Set P <- ROOT.  (Empty tree: insert at the root.)
    /// T2. [Compare.]    If K < KEY(P) go to T3; if K > KEY(P) go to T4;
    ///                   otherwise the search is successful — a duplicate.
    /// T3. [Move left.]  If LLINK(P) != Λ, set P <- LLINK(P) and go to T2;
    ///                   otherwise go to T5.
    /// T4. [Move right.] If RLINK(P) != Λ, set P <- RLINK(P) and go to T2;
    ///                   otherwise go to T5.
    /// T5. [Insert.]     Q <= AVAIL (allocate a node); KEY(Q) <- K,
    ///                   LLINK(Q) <- RLINK(Q) <- Λ; and set LLINK(P) <- Q or
    ///                   RLINK(P) <- Q according to which side the search
    ///                   fell off.
    /// ```
    pub fn insert(&mut self, key: i64) -> bool {
        let _ = key;
        todo!("implement Algorithm 6.2.2T (insertion)")
    }

    /// The search half of Algorithm 6.2.2T: steps T1-T4, with T5 replaced
    /// by "terminate unsuccessfully".
    pub fn contains(&self, key: i64) -> bool {
        let _ = key;
        todo!("implement Algorithm 6.2.2T (search)")
    }

    /// Algorithm 6.2.2D (Tree deletion) — the Hibbard/Knuth scheme: a node
    /// with two children is replaced by its **symmetric successor** (the
    /// leftmost node of its right subtree). Return `false` if `key` is
    /// absent.
    ///
    /// First find the doomed node T *and the link pointing at it* (its
    /// parent and side; the root has no parent). Then, with Q the subtree
    /// that will take T's place:
    ///
    /// ```text
    /// D1. [Is RLINK null?]   If RLINK(T) = Λ, set Q <- LLINK(T); go to D4.
    /// D2. [Find successor.]  Set R <- RLINK(T). If LLINK(R) = Λ, set
    ///                        LLINK(R) <- LLINK(T), Q <- R; go to D4.
    /// D3. [Find null LLINK.] Set S <- LLINK(R); while LLINK(S) != Λ, set
    ///                        R <- S, S <- LLINK(S).  (S is T's successor,
    ///                        R its parent.)  Then set LLINK(S) <- LLINK(T),
    ///                        LLINK(R) <- RLINK(S), RLINK(S) <- RLINK(T),
    ///                        Q <- S.
    /// D4. [Anchor.]          Make T's parent point at Q instead of T (or
    ///                        ROOT <- Q if T was the root). Recycle T's slot.
    /// ```
    pub fn delete(&mut self, key: i64) -> bool {
        let _ = key;
        todo!("implement Algorithm 6.2.2D (deletion by symmetric successor)")
    }

    /// The keys in symmetric (inorder) order — sorted, if your tree is a
    /// correct BST. Use an explicit stack, not recursion: stage tests build
    /// degenerate trees thousands of nodes deep.
    pub fn inorder(&self) -> Vec<i64> {
        todo!("inorder traversal with an explicit stack")
    }

    /// Height in **edges**: the length of the longest root-to-leaf path.
    /// Single-node tree: 0. Empty tree: 0 (by convention). Again, explicit
    /// stack (e.g. of `(node, depth)` pairs), not recursion.
    pub fn height(&self) -> usize {
        todo!("compute tree height in edges")
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

/// One node of the AVL tree. `bal` is Knuth's balance factor
/// B(P) = height(right subtree) - height(left subtree), always in
/// {-1, 0, +1} when the tree is balanced.
#[allow(dead_code)] // read once you implement the methods below
struct AvlNode {
    key: i64,
    left: usize,  // LLINK
    right: usize, // RLINK
    bal: i8,      // B(P)
}

/// Stage 3 — a height-balanced (AVL) tree (TAOCP Vol. 3, §6.2.3).
/// Insertion is Algorithm 6.2.3A; there is **no deletion** in this stage
/// (Knuth doesn't give one either).
pub struct AvlTree {
    #[allow(dead_code)] // read once you implement the methods below
    nodes: Vec<AvlNode>,
    #[allow(dead_code)]
    root: usize,
}

impl AvlTree {
    /// An empty tree. (Done for you.)
    pub fn new() -> Self {
        AvlTree { nodes: Vec::new(), root: NULL }
    }

    /// Algorithm 6.2.3A (Balanced tree search and insertion). Return
    /// `false` (tree unchanged) if `key` is already present.
    ///
    /// Cast of pointers: P walks the search path; S is the deepest node on
    /// the path with B(S) != 0 — the only node where rebalancing might be
    /// needed; T is S's parent (Knuth uses a header node; use your NULL
    /// sentinel to mean "S is the root"). LINK(a, P) means LLINK(P) if
    /// a = -1 and RLINK(P) if a = +1.
    ///
    /// ```text
    /// A1. [Initialize.] T <- Λ, S <- P <- ROOT. (Empty tree: just insert.)
    /// A2. [Compare.]    If K < KEY(P) go to A3; if K > KEY(P) go to A4;
    ///                   otherwise K is already in the tree.
    /// A3. [Move left.]  Q <- LLINK(P). If Q = Λ: allocate Q,
    ///                   LLINK(P) <- Q, go to A5. Otherwise, if B(Q) != 0,
    ///                   set T <- P, S <- Q. Set P <- Q, go to A2.
    /// A4. [Move right.] Mirror image of A3 with RLINK.
    /// A5. [Insert.]     KEY(Q) <- K, LLINK(Q) <- RLINK(Q) <- Λ, B(Q) <- 0.
    /// A6. [Adjust balance factors.] Set a <- -1 if K < KEY(S), else +1.
    ///                   Then walk R <- P <- LINK(a, S) toward Q, setting
    ///                   each intermediate node's balance factor to -1 or +1
    ///                   according to the side the path takes (they were all
    ///                   0, by the choice of S). Keep hold of R.
    /// A7. [Balancing act.]
    ///       (i)   B(S) = 0:  B(S) <- a; the whole tree grew taller. Done.
    ///       (ii)  B(S) = -a: B(S) <- 0; the tree got *more* balanced. Done.
    ///       (iii) B(S) = a:  out of balance at S — go to A8 if B(R) = a
    ///                        (single rotation) or A9 if B(R) = -a (double).
    /// A8. [Single rotation.] P <- R; LINK(a, S) <- LINK(-a, R);
    ///                   LINK(-a, R) <- S; B(S) <- B(R) <- 0. Go to A10.
    /// A9. [Double rotation.] P <- LINK(-a, R); LINK(-a, R) <- LINK(a, P);
    ///                   LINK(a, P) <- R; LINK(a, S) <- LINK(-a, P);
    ///                   LINK(-a, P) <- S. Then set
    ///                   (B(S), B(R)) <- (-a, 0) if B(P) = a,
    ///                                   ( 0, 0) if B(P) = 0,
    ///                                   ( 0, a) if B(P) = -a;
    ///                   and finally B(P) <- 0.
    /// A10. [Finishing touch.] P is the new root of the rebalanced subtree:
    ///                   if T = Λ set ROOT <- P; else set RLINK(T) <- P if
    ///                   S was RLINK(T), LLINK(T) <- P otherwise.
    /// ```
    pub fn insert(&mut self, key: i64) -> bool {
        let _ = key;
        todo!("implement Algorithm 6.2.3A")
    }

    /// Ordinary tree search — balance factors don't change how you search.
    pub fn contains(&self, key: i64) -> bool {
        let _ = key;
        todo!("tree search (steps T1-T4)")
    }

    /// The keys in symmetric (inorder) order.
    pub fn inorder(&self) -> Vec<i64> {
        todo!("inorder traversal")
    }

    /// Height in **edges** (empty and single-node trees: 0).
    pub fn height(&self) -> usize {
        todo!("compute tree height in edges")
    }

    /// Check the AVL invariant *from scratch*: recompute every subtree
    /// height (do NOT trust the stored balance factors) and verify that
    /// each node's `bal` field equals height(right) - height(left) AND lies
    /// in {-1, 0, +1}. Return `true` for the empty tree.
    pub fn is_balanced(&self) -> bool {
        todo!("recompute heights; verify every balance factor")
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

/// Trial-division primality test — enough for validating table sizes.
/// (Done for you.)
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

/// Stage 4 — open addressing with **linear probing**, Algorithm 6.4L.
///
/// Division hashing h(K) = K mod M, M prime. The probe sequence *decreases*
/// cyclically, as in the text: h(K), h(K)-1, ..., 0, M-1, ... The table is
/// declared full at N = M - 1 occupied slots — one slot always stays empty
/// so unsuccessful searches are guaranteed to terminate.
pub struct LinearProbe {
    #[allow(dead_code)] // read once you implement the methods below
    table: Vec<Option<u64>>,
    #[allow(dead_code)]
    m: usize,
    #[allow(dead_code)]
    n: usize, // occupied slots, kept < M
}

impl LinearProbe {
    /// An empty table of size `m`; panics unless `m` is a prime >= 3.
    /// (Done for you.)
    pub fn new(m: usize) -> Self {
        assert!(m >= 3 && is_prime(m), "table size M must be a prime >= 3");
        LinearProbe { table: vec![None; m], m, n: 0 }
    }

    /// Algorithm 6.4L (Linear probing and insertion). Return `false` —
    /// table unchanged — if `key` is already present (duplicate) **or**
    /// the table is full (N = M - 1).
    ///
    /// ```text
    /// L1. [Hash.]            Set i <- h(K) = K mod M.  (0 <= i < M.)
    /// L2. [Compare.]         If TABLE[i] is empty, go to L4. If it holds
    ///                        K, the search is successful — a duplicate.
    /// L3. [Advance to next.] Set i <- i - 1; if now i < 0, set i <- i + M.
    ///                        Go back to L2.
    /// L4. [Insert.]          If N = M - 1, terminate with overflow (full).
    ///                        Otherwise set N <- N + 1, TABLE[i] <- K.
    /// ```
    pub fn insert(&mut self, key: u64) -> bool {
        let _ = key;
        todo!("implement Algorithm 6.4L")
    }

    /// The search half of Algorithm 6.4L: follow the probe sequence until
    /// you find `key` (true) or an empty slot (false).
    pub fn contains(&self, key: u64) -> bool {
        let _ = key;
        todo!("linear-probing search")
    }

    /// How many probes (table slots examined) a search for `key` makes,
    /// counting the final probe — the one that finds `key`, or the empty
    /// slot proving it absent. A key found in its home slot costs 1 probe.
    /// This is the C of Knuth's 1962 analysis (see the lesson).
    pub fn probes_for(&self, key: u64) -> u32 {
        let _ = key;
        todo!("count probes of a search")
    }
}

/// Stage 4 — open addressing with **double hashing**, Algorithm 6.4D.
///
/// h1(K) = K mod M; after the first collision the probe *decrement* is
/// c = h2(K) = 1 + (K mod (M - 2)), so 1 <= c <= M - 2. M prime makes
/// gcd(c, M) = 1, hence the probe sequence visits every slot. Same
/// full-at-N-=-M-1 rule as `LinearProbe`.
pub struct DoubleHash {
    #[allow(dead_code)] // read once you implement the methods below
    table: Vec<Option<u64>>,
    #[allow(dead_code)]
    m: usize,
    #[allow(dead_code)]
    n: usize,
}

impl DoubleHash {
    /// An empty table of size `m`; panics unless `m` is a prime >= 3.
    /// (Done for you.)
    pub fn new(m: usize) -> Self {
        assert!(m >= 3 && is_prime(m), "table size M must be a prime >= 3");
        DoubleHash { table: vec![None; m], m, n: 0 }
    }

    /// Algorithm 6.4D (Open addressing with double hashing). Return `false`
    /// on duplicate or full table.
    ///
    /// ```text
    /// D1. [First hash.]      Set i <- h1(K) = K mod M.
    /// D2. [First probe.]     If TABLE[i] is empty, go to D6. If it holds
    ///                        K, successful — a duplicate.
    /// D3. [Second hash.]     Set c <- h2(K) = 1 + (K mod (M - 2)).
    /// D4. [Advance to next.] Set i <- i - c; if now i < 0, set i <- i + M.
    /// D5. [Compare.]         If TABLE[i] is empty, go to D6. If it holds
    ///                        K, successful. Otherwise go back to D4.
    /// D6. [Insert.]          If N = M - 1, overflow. Otherwise N <- N + 1,
    ///                        TABLE[i] <- K.
    /// ```
    pub fn insert(&mut self, key: u64) -> bool {
        let _ = key;
        todo!("implement Algorithm 6.4D")
    }

    /// The search half of Algorithm 6.4D.
    pub fn contains(&self, key: u64) -> bool {
        let _ = key;
        todo!("double-hashing search")
    }

    /// Probe count of a search for `key` — same counting rule as
    /// `LinearProbe::probes_for`.
    pub fn probes_for(&self, key: u64) -> u32 {
        let _ = key;
        todo!("count probes of a search")
    }
}
