//! Module 03 — Information Structures (TAOCP Vol. 1, Ch. 2).
//!
//! YOUR WORKSPACE. Replace each `todo!()` with an implementation, then run
//! `./grade 3` from the repository root. Work the stages in order — each
//! test file `tests/stage_NN_*.rs` corresponds to one stage, and the lesson
//! in `course/module-03-structures/README.md` walks you through the theory.
//!
//! **Memory model for the whole module** (see the lesson, §2): Knuth's
//! linked memory becomes an index-based arena — a `Vec` of cells addressed
//! by `usize` indices, with the null link Λ represented by [`LAMBDA`]
//! (= `usize::MAX`). Links are plain numbers; no `Rc`, no `RefCell`.
//!
//! The private fields suggested inside each struct are one workable layout;
//! you may change them freely — only the *public* names and signatures are
//! graded.

#![allow(dead_code)] // stub fields are unread until you implement the methods

/// The null link Λ. We use `usize::MAX` (never a valid `Vec` index) so that
/// index 0 remains an ordinary addressable cell.
pub const LAMBDA: usize = usize::MAX;

/// Reported when an insertion hits a full structure (§2.2.2 OVERFLOW).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overflow;

/// Reported when a deletion finds the structure empty (§2.2.2 UNDERFLOW).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Underflow;

// ---------------------------------------------------------------------------
// Stage 1 — §2.2.1–2.2.2: stacks and queues in sequential storage
// ---------------------------------------------------------------------------

/// A fixed-capacity stack in sequential storage (§2.2.2, eq. (2)–(3)).
///
/// Knuth's picture: a pointer T into a block of M cells; insertion is
/// `T <- T + 1; X[T] <- Y` (OVERFLOW if T > M), deletion is
/// `Y <- X[T]; T <- T - 1` (UNDERFLOW if T = 0).
///
/// **Error policy (part of the contract):** `push` returns `Err(Overflow)`
/// when full, `pop` returns `Err(Underflow)` when empty. Nothing panics —
/// the *caller* decides what an overflow means.
pub struct ArrayStack<T> {
    items: Vec<T>,
    capacity: usize,
}

impl<T> ArrayStack<T> {
    /// An empty stack that can hold at most `capacity` items.
    pub fn new(capacity: usize) -> Self {
        let _ = capacity;
        todo!("create an empty fixed-capacity stack")
    }

    pub fn capacity(&self) -> usize {
        todo!("report M, the fixed capacity")
    }

    pub fn len(&self) -> usize {
        todo!("report T, the number of items stored")
    }

    pub fn is_empty(&self) -> bool {
        todo!("T = 0?")
    }

    pub fn is_full(&self) -> bool {
        todo!("T = M?")
    }

    /// X ⇐ stack (eq. (2)): OVERFLOW if T = M, else store at the top.
    pub fn push(&mut self, x: T) -> Result<(), Overflow> {
        let _ = x;
        todo!("push, reporting Overflow when full")
    }

    /// stack ⇒ X (eq. (3)): UNDERFLOW if T = 0, else remove the top.
    pub fn pop(&mut self) -> Result<T, Underflow> {
        todo!("pop, reporting Underflow when empty")
    }

    /// The item `pop` would return, if any.
    pub fn peek(&self) -> Option<&T> {
        todo!("borrow the top item")
    }
}

/// A fixed-capacity queue in a circular buffer (§2.2.2, eq. (6)–(7)).
///
/// Knuth moves two pointers R (rear) and F (front) around a circle of M
/// cells: `R <- R+1 mod M` on insert, `F <- F+1 mod M` on delete. With
/// pointers alone, "full" and "empty" both look like R = F, so only M − 1
/// cells are usable (§2.2.2 exercise 1); keep an explicit length as well —
/// the standard cure — and all `capacity` cells hold items.
///
/// **Error policy:** `enqueue` returns `Err(Overflow)` when full, `dequeue`
/// returns `Err(Underflow)` when empty. Nothing panics.
pub struct ArrayQueue<T> {
    buf: Vec<Option<T>>,
    front: usize, // F: index of the front item
    len: usize,   // number of items stored
}

impl<T> ArrayQueue<T> {
    /// An empty queue that can hold at most `capacity` items.
    pub fn new(capacity: usize) -> Self {
        let _ = capacity;
        todo!("create an empty circular-buffer queue")
    }

    pub fn capacity(&self) -> usize {
        todo!("report M")
    }

    pub fn len(&self) -> usize {
        todo!("number of items currently stored")
    }

    pub fn is_empty(&self) -> bool {
        todo!("length zero?")
    }

    pub fn is_full(&self) -> bool {
        todo!("length = M?")
    }

    /// X ⇐ queue (eq. (6)): advance R cyclically; OVERFLOW if the rear
    /// would catch the front. The rear cell is at (F + len) mod M.
    pub fn enqueue(&mut self, x: T) -> Result<(), Overflow> {
        let _ = x;
        todo!("insert at the rear, reporting Overflow when full")
    }

    /// queue ⇒ X (eq. (7)): UNDERFLOW if empty, else take the front item
    /// and advance F cyclically — this is where wraparound must work.
    pub fn dequeue(&mut self) -> Result<T, Underflow> {
        todo!("remove from the front, reporting Underflow when empty")
    }

    /// The item `dequeue` would return, if any.
    pub fn front(&self) -> Option<&T> {
        todo!("borrow the front item")
    }
}

/// §2.2.1 railway shunting: can `perm`, a permutation of {1, …, n}, be
/// produced by running the cars 1, 2, …, n (arriving in that order) through
/// one stack?
///
/// Greedy simulation is optimal: keep a `next_arrival` counter starting at
/// 1; for each wanted output car, push arrivals until that car is in the
/// stack, then it must be exactly on top — pop it. Any mismatch means the
/// permutation is unobtainable.
///
/// Return `false` for input that is not a permutation of 1..=n.
///
/// (Theory, §2.2.1 exercises 2–5: obtainable ⇔ no pattern 3-1-2, i.e. no
/// i < j < k with `perm[j] < perm[k] < perm[i]`; the count of obtainable
/// permutations is the Catalan number C(n). The tests check both.)
pub fn stack_permutable(perm: &[usize]) -> bool {
    let _ = perm;
    todo!("simulate the siding greedily")
}

// ---------------------------------------------------------------------------
// Stage 2 — §2.2.3: linked allocation with an AVAIL list
// ---------------------------------------------------------------------------

/// An arena of one-word nodes, each with an INFO field and a LINK field,
/// plus the AVAIL stack of free cells (§2.2.3, eq. (4)–(5)).
///
/// Layout suggestion (mirrors the two fields of Knuth's nodes): two
/// parallel vectors `info` and `link`, and `avail`, the head of a singly
/// linked stack of free cells threaded through their LINK fields
/// (`LAMBDA` when empty). `info[p]` is `None` exactly while cell `p` sits
/// on the AVAIL stack.
pub struct LinkedArena<T> {
    info: Vec<Option<T>>,
    link: Vec<usize>,
    avail: usize,
}

impl<T> LinkedArena<T> {
    /// A new arena with no cells drawn from memory yet.
    pub fn new() -> Self {
        todo!("empty arena, AVAIL = Λ")
    }

    /// `P <= AVAIL` (eq. (4)): if AVAIL ≠ Λ, pop a cell from the free
    /// stack (`P <- AVAIL; AVAIL <- LINK(AVAIL)`); only if AVAIL = Λ draw
    /// a brand-new cell from the memory pool. Reset the cell's LINK to Λ,
    /// store `info`, return the cell's index.
    pub fn allocate(&mut self, info: T) -> usize {
        let _ = info;
        todo!("pop AVAIL, or grow the pool if AVAIL = Λ")
    }

    /// `AVAIL <= P` (eq. (5)): `LINK(P) <- AVAIL; AVAIL <- P`. Yield the
    /// cell's INFO. May panic if `p` is not currently allocated.
    pub fn free(&mut self, p: usize) -> T {
        let _ = p;
        todo!("push p onto the AVAIL stack, returning its INFO")
    }

    /// How many cells have ever been drawn from the memory pool (allocated
    /// + free). If `free` works, this stays bounded by the *peak* number of
    /// simultaneously live nodes — the tests check exactly that.
    pub fn cells_in_memory(&self) -> usize {
        todo!("size of the pool")
    }

    /// INFO(p). May panic if `p` is not allocated.
    pub fn info(&self, p: usize) -> &T {
        let _ = p;
        todo!("borrow INFO(p)")
    }

    /// LINK(p).
    pub fn link(&self, p: usize) -> usize {
        let _ = p;
        todo!("return LINK(p)")
    }

    /// LINK(p) <- target.
    pub fn set_link(&mut self, p: usize, target: usize) {
        let _ = (p, target);
        todo!("set LINK(p)")
    }

    /// Insert a new node at the front of the list headed by `first`
    /// (§2.2.3, eq. (8)): `P <= AVAIL; INFO(P) <- info; LINK(P) <- first`.
    /// Returns the new first node.
    pub fn push_front(&mut self, first: usize, info: T) -> usize {
        let _ = (first, info);
        todo!("allocate and link in front")
    }

    /// Delete the node *after* `p` — the O(1) deletion a singly linked
    /// list supports: `Q <- LINK(P); LINK(P) <- LINK(Q); AVAIL <= Q`.
    /// Return its INFO, or `None` if `p` has no successor.
    pub fn delete_after(&mut self, p: usize) -> Option<T> {
        let _ = p;
        todo!("unlink LINK(p) and free it")
    }

    /// Walk the list headed by `first` (follow LINK until Λ), collecting
    /// the INFO fields in order.
    pub fn to_vec(&self, first: usize) -> Vec<T>
    where
        T: Clone,
    {
        let _ = first;
        todo!("walk the LINK chain")
    }
}

// ---------------------------------------------------------------------------
// Stage 3 — Algorithm 2.2.3T: topological sorting
// ---------------------------------------------------------------------------

/// Algorithm 2.2.3T (Topological sort).
///
/// Objects are numbered 1..=n; each pair `(j, k)` in `relations` declares
/// j ≺ k. Return a linear arrangement of 1..=n consistent with every
/// relation, or `None` if the relations contain a cycle (a pair with j = k
/// counts as a cycle). Panic if a pair mentions an object outside 1..=n.
///
/// ```text
/// T1. [Initialize.]      COUNT[k] <- 0 and empty successor list, 1 <= k <= n.
/// T2. [Next relation.]   For each input pair (j, k):
/// T3. [Record it.]         COUNT[k] <- COUNT[k] + 1; append k to j's list.
/// T4. [Scan for zeros.]  Put every k with COUNT[k] = 0 into a queue,
///                        scanning k = 1, 2, ..., n in increasing order.
/// T5. [Output front.]    While the queue is nonempty: remove the front F
///                        and output it.
/// T6. [Erase relations.] For each successor k of F (in recorded order):
///                        COUNT[k] <- COUNT[k] - 1;
/// T7. [Zero?]              if COUNT[k] = 0, append k at the REAR.
/// T8. [End.]             If n objects were output, that is the answer;
///                        otherwise a cycle remains: return None.
/// ```
///
/// **Queue discipline (fixes the output deterministically):** FIFO, with
/// the initial zeros enqueued in increasing index order and successors
/// scanned in input order — exactly what Knuth's QLINK trick produces.
pub fn topological_sort(n: usize, relations: &[(usize, usize)]) -> Option<Vec<usize>> {
    let _ = (n, relations);
    todo!("implement Algorithm 2.2.3T")
}

// ---------------------------------------------------------------------------
// Stage 4 — §2.3.1, Algorithm 2.3.1T: binary trees and their traversal
// ---------------------------------------------------------------------------

/// A binary tree in an index arena: each node has INFO, LLINK, RLINK, with
/// Λ = [`LAMBDA`]. Build nodes bottom-up with [`BinaryTree::add_node`] and
/// set the root explicitly.
pub struct BinaryTree<T> {
    info: Vec<T>,
    llink: Vec<usize>,
    rlink: Vec<usize>,
    root: usize,
}

impl<T> BinaryTree<T> {
    /// The empty binary tree (root = Λ).
    pub fn new() -> Self {
        todo!("empty tree, root = Λ")
    }

    /// Add a node with the given INFO and children (`LAMBDA` for an empty
    /// subtree); return its index. Does not change the root.
    pub fn add_node(&mut self, info: T, llink: usize, rlink: usize) -> usize {
        let _ = (info, llink, rlink);
        todo!("append a node to the arena")
    }

    pub fn root(&self) -> usize {
        todo!("current root, Λ if empty")
    }

    pub fn set_root(&mut self, p: usize) {
        let _ = p;
        todo!("set the root pointer")
    }

    pub fn len(&self) -> usize {
        todo!("number of nodes in the arena")
    }

    pub fn is_empty(&self) -> bool {
        todo!("no nodes?")
    }

    pub fn info(&self, p: usize) -> &T {
        let _ = p;
        todo!("borrow INFO(p)")
    }

    pub fn llink(&self, p: usize) -> usize {
        let _ = p;
        todo!("LLINK(p)")
    }

    pub fn rlink(&self, p: usize) -> usize {
        let _ = p;
        todo!("RLINK(p)")
    }

    pub fn set_llink(&mut self, p: usize, q: usize) {
        let _ = (p, q);
        todo!("LLINK(p) <- q")
    }

    pub fn set_rlink(&mut self, p: usize, q: usize) {
        let _ = (p, q);
        todo!("RLINK(p) <- q")
    }

    /// Algorithm 2.3.1T (Traverse binary tree in inorder) — implement the
    /// **explicit-stack** formulation, NOT recursion; the stack invariant
    /// is the point of this stage (lesson §5.2):
    ///
    /// ```text
    /// T1. [Initialize.] Set stack A empty; P <- T (the root).
    /// T2. [P = Λ?]      If P = Λ, go to T4.
    /// T3. [Stack <= P.] Push P onto A; P <- LLINK(P); back to T2.
    /// T4. [P <= Stack.] If A is empty, terminate; else pop P from A.
    /// T5. [Visit P.]    Visit NODE(P); P <- RLINK(P); back to T2.
    /// ```
    pub fn inorder(&self) -> Vec<T>
    where
        T: Clone,
    {
        todo!("Algorithm T with an explicit stack")
    }

    /// Preorder: visit, then left subtree, then right subtree (exercise
    /// 2.3.1-12: same as Algorithm T but visit when *pushing*).
    pub fn preorder(&self) -> Vec<T>
    where
        T: Clone,
    {
        todo!("preorder traversal")
    }

    /// Postorder: left subtree, right subtree, then visit (exercise
    /// 2.3.1-13). Hint: reversing a "root, right, left" preorder gives
    /// postorder — or adapt Algorithm T with a second visit flag.
    pub fn postorder(&self) -> Vec<T>
    where
        T: Clone,
    {
        todo!("postorder traversal")
    }
}

/// Rebuild the unique binary tree having the given `preorder` and `inorder`
/// sequences (elements must be distinct — then the two traversals determine
/// the tree). The first preorder element is the root; find it in the
/// inorder sequence; the parts before/after it are the left/right subtrees;
/// recurse. May panic if the sequences are inconsistent.
pub fn from_traversals<T: Clone + PartialEq>(preorder: &[T], inorder: &[T]) -> BinaryTree<T> {
    let _ = (preorder, inorder);
    todo!("split on the root and recurse")
}

// ---------------------------------------------------------------------------
// Stage 5 — §2.3.1, Algorithm 2.3.1S: threaded binary trees
// ---------------------------------------------------------------------------

/// A **fully threaded** binary tree with a list head (§2.3.1). Every
/// would-be Λ link is a *thread*: a tagged link to the node's inorder
/// predecessor (left threads) or successor (right threads). Node 0 is the
/// list head: LLINK(HEAD) is the root (a thread to HEAD itself when the
/// tree is empty), RLINK(HEAD) = HEAD always, and the head stores no INFO —
/// it acts as the "one past the end" position of the inorder sequence.
///
/// Suggested node layout: `{ info: Option<T>, llink, ltag, rlink, rtag }`
/// with `tag = true` meaning "this link is a thread".
pub struct ThreadedTree<T> {
    nodes: Vec<ThreadedNode<T>>,
}

struct ThreadedNode<T> {
    info: Option<T>, // None only for the head
    llink: usize,
    ltag: bool,
    rlink: usize,
    rtag: bool,
}

impl<T> ThreadedTree<T> {
    /// The empty threaded tree: just the head, left-threaded to itself
    /// (LLINK = 0, LTAG = thread; RLINK = 0, RTAG = link).
    pub fn new() -> Self {
        todo!("head node only")
    }

    /// The list head's index (always 0).
    pub fn head(&self) -> usize {
        todo!("index of the head")
    }

    /// Number of real nodes (the head does not count).
    pub fn len(&self) -> usize {
        todo!("node count excluding the head")
    }

    pub fn is_empty(&self) -> bool {
        todo!("only the head present?")
    }

    /// INFO(p). May panic if `p` is the head.
    pub fn info(&self, p: usize) -> &T {
        let _ = p;
        todo!("borrow INFO(p)")
    }

    /// Insert a new node as the **left child** of `p`, taking over `p`'s
    /// old left subtree; the new node becomes the immediate inorder
    /// *predecessor* of `p`. `insert_left(head, x)` appends `x` at the END
    /// of the inorder sequence. Returns the new node's index.
    ///
    /// Mirror of Knuth's threaded insertion (§2.3.1, eq. (12)):
    ///
    /// ```text
    /// I1'. LLINK(N) <- LLINK(P), LTAG(N) <- LTAG(P);
    ///      RLINK(N) <- P, RTAG(N) <- 1   (N's successor is P).
    /// I2'. LLINK(P) <- N, LTAG(P) <- 0.
    /// I3'. If LTAG(N) = 0, the rightmost node of N's new left subtree has
    ///      a right thread aimed at P; retarget it to N.
    /// ```
    pub fn insert_left(&mut self, p: usize, info: T) -> usize {
        let _ = (p, info);
        todo!("threaded insertion to the left")
    }

    /// Insert a new node as the **right child** of `p` (a real node, not
    /// the head), taking over `p`'s old right subtree; the new node becomes
    /// the immediate inorder *successor* of `p`. Returns its index.
    ///
    /// Knuth's Algorithm I (§2.3.1, eq. (12)):
    ///
    /// ```text
    /// I1. RLINK(N) <- RLINK(P), RTAG(N) <- RTAG(P);
    ///     LLINK(N) <- P, LTAG(N) <- 1   (N's predecessor is P).
    /// I2. RLINK(P) <- N, RTAG(P) <- 0.
    /// I3. If RTAG(N) = 0, the leftmost node of N's new right subtree has
    ///     a left thread aimed at P; retarget it to N.
    /// ```
    pub fn insert_right(&mut self, p: usize, info: T) -> usize {
        let _ = (p, info);
        todo!("Algorithm I: threaded insertion to the right")
    }

    /// Algorithm 2.3.1S (Inorder successor in a threaded binary tree).
    /// The successor of the last node is the head; the successor of the
    /// head is the first node in inorder. **No stack.**
    ///
    /// ```text
    /// S1. [RLINK a thread?] Q <- RLINK(P); if RTAG(P) = 1, terminate.
    /// S2. [Search left.]    While LTAG(Q) = 0, Q <- LLINK(Q).
    /// S3. [Done.]           Q is the successor.
    /// ```
    pub fn successor(&self, p: usize) -> usize {
        let _ = p;
        todo!("implement Algorithm 2.3.1S")
    }

    /// The inorder sequence obtained by iterating Algorithm S from the
    /// head until it comes back to the head — no stack, no recursion,
    /// O(1) extra space.
    pub fn inorder_via_threads(&self) -> Vec<T>
    where
        T: Clone,
    {
        todo!("iterate successor() from the head")
    }

    /// Same traversal, additionally counting every link the algorithm
    /// follows (each `Q <- RLINK(...)` and each `Q <- LLINK(Q)`). A full
    /// traversal of n nodes follows at most 2n + 2 links — that count is
    /// the proof the traversal is O(n).
    pub fn inorder_via_threads_counting(&self) -> (Vec<T>, usize)
    where
        T: Clone,
    {
        todo!("traverse and count link-follows")
    }
}
