//! Module 03 — Information Structures.
//! Source: TAOCP Vol. 1, 3rd ed., Ch. 2 (§2.2.1–2.2.3 and §2.3.1).
//!
//! Memory model used throughout this module: Knuth's MIX-era linked memory
//! maps onto **index-based arenas** — a `Vec` of cells addressed by `usize`
//! indices, with the null link Λ represented by [`LAMBDA`] (= `usize::MAX`).
//! No `Rc`/`RefCell`: a link is just a number, exactly as in the book.

/// The null link Λ. We use `usize::MAX` (never a valid `Vec` index) so that
/// index 0 remains an ordinary addressable cell, unlike the "0 = Λ" MIX
/// convention which silently wastes location 0.
pub const LAMBDA: usize = usize::MAX;

/// Error value reported when an insertion hits a full structure (§2.2.2's
/// OVERFLOW condition). Reported, never panicked: whether to grow, purge or
/// abort is the *caller's* policy decision, not the structure's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overflow;

/// Error value reported when a deletion finds the structure empty (§2.2.2's
/// UNDERFLOW condition). Usually meaningful — "no more work to do" — which
/// is why it too must be reported, not panicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Underflow;

// ---------------------------------------------------------------------------
// §2.2.1–2.2.2 — Stacks and queues in sequential storage
// ---------------------------------------------------------------------------

/// A fixed-capacity stack in sequential storage (§2.2.2, eq. (2)–(3)).
///
/// Knuth's picture: a base address and a pointer T; insertion is
/// `T <- T + 1; X[T] <- Y` with an OVERFLOW test, deletion is
/// `Y <- X[T]; T <- T - 1` with an UNDERFLOW test. Here the `Vec`'s length
/// plays the role of T.
///
/// Error policy (documented contract): `push` returns `Err(Overflow)` when
/// full, `pop` returns `Err(Underflow)` when empty. Nothing panics.
pub struct ArrayStack<T> {
    items: Vec<T>,
    capacity: usize,
}

impl<T> ArrayStack<T> {
    /// An empty stack that can hold at most `capacity` items.
    pub fn new(capacity: usize) -> Self {
        ArrayStack { items: Vec::with_capacity(capacity), capacity }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.items.len() == self.capacity
    }

    /// X ⇐ stack insertion, eq. (2): `T <- T+1`; OVERFLOW if T > M.
    pub fn push(&mut self, x: T) -> Result<(), Overflow> {
        if self.items.len() == self.capacity {
            return Err(Overflow);
        }
        self.items.push(x);
        Ok(())
    }

    /// stack ⇒ X deletion, eq. (3): UNDERFLOW if T = 0, else `T <- T-1`.
    pub fn pop(&mut self) -> Result<T, Underflow> {
        self.items.pop().ok_or(Underflow)
    }

    /// The item that `pop` would return, if any.
    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }
}

/// A fixed-capacity queue in a circular buffer (§2.2.2, eq. (6)–(7)).
///
/// Knuth keeps two pointers R (rear) and F (front) that move around a circle
/// of M cells: `R <- R+1 mod M` on insert, `F <- F+1 mod M` on delete. With
/// only the two pointers, a full queue is indistinguishable from an empty
/// one, so at most M − 1 of the M cells are usable (§2.2.2 exercise 1).
/// We store an explicit length instead — the standard cure — so all
/// `capacity` cells hold items.
///
/// Error policy: `enqueue` returns `Err(Overflow)` when full, `dequeue`
/// returns `Err(Underflow)` when empty. Nothing panics.
pub struct ArrayQueue<T> {
    buf: Vec<Option<T>>,
    front: usize, // index of the front item (F)
    len: usize,   // number of items stored
}

impl<T> ArrayQueue<T> {
    /// An empty queue that can hold at most `capacity` items.
    pub fn new(capacity: usize) -> Self {
        let mut buf = Vec::with_capacity(capacity);
        buf.resize_with(capacity, || None);
        ArrayQueue { buf, front: 0, len: 0 }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.buf.len()
    }

    /// X ⇐ queue insertion, eq. (6): advance R cyclically; OVERFLOW if the
    /// rear catches up with the front.
    pub fn enqueue(&mut self, x: T) -> Result<(), Overflow> {
        if self.is_full() {
            return Err(Overflow);
        }
        // R = (F + len) mod M is where the new rear item goes.
        let r = (self.front + self.len) % self.buf.len();
        self.buf[r] = Some(x);
        self.len += 1;
        Ok(())
    }

    /// queue ⇒ X deletion, eq. (7): UNDERFLOW if F = R, else advance F
    /// cyclically and yield the cell it passed.
    pub fn dequeue(&mut self) -> Result<T, Underflow> {
        if self.len == 0 {
            return Err(Underflow);
        }
        let x = self.buf[self.front].take().expect("front cell occupied");
        self.front = (self.front + 1) % self.buf.len();
        self.len -= 1;
        Ok(x)
    }

    /// The item that `dequeue` would return, if any.
    pub fn front(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            self.buf[self.front].as_ref()
        }
    }
}

/// §2.2.1 railway shunting: can the permutation `perm` of {1, 2, …, n} be
/// produced by passing the cars 1, 2, …, n (arriving in that order) through
/// a single stack (siding)?
///
/// Greedy simulation — provably optimal: when the required next output car
/// is on top of the stack we *must* pop it now or bury it forever; when it
/// has not yet arrived we must keep pushing arrivals.
///
/// Returns `false` for any sequence that is not a permutation of 1..=n
/// (such a sequence certainly cannot be produced).
///
/// Theorem (§2.2.1, exercises 2–5): `perm` is obtainable iff it contains no
/// three positions i < j < k with `perm[k] < perm[j] < perm[i]`... more
/// precisely no pattern 3-1-2: indices i < j < k with
/// `perm[j] < perm[k] < perm[i]`. The number of obtainable permutations of
/// n cars is the Catalan number C(n).
pub fn stack_permutable(perm: &[usize]) -> bool {
    let n = perm.len();
    let mut stack: Vec<usize> = Vec::new();
    let mut next_arrival = 1; // the next car waiting on the input track
    for &want in perm {
        if want == 0 || want > n {
            return false; // not a permutation of 1..=n
        }
        // Push arrivals until the wanted car has entered the siding.
        while next_arrival <= want {
            stack.push(next_arrival);
            next_arrival += 1;
        }
        // The wanted car must now be on top of the siding.
        if stack.pop() != Some(want) {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// §2.2.3 — Linked allocation with an AVAIL list
// ---------------------------------------------------------------------------

/// An arena of one-word nodes, each with an INFO field and a LINK field,
/// plus the AVAIL stack of free cells (§2.2.3, eq. (4)–(5)).
///
/// The two parallel vectors mirror the two fields of Knuth's nodes.
/// `avail` heads a singly linked stack of free cells threaded through their
/// LINK fields; `allocate` is Knuth's `P <= AVAIL` and `free` is
/// `AVAIL <= P`. Fresh memory is obtained from the `Vec` (the analogue of
/// the SEQMIN/"pool grows toward the list" trick) only when AVAIL is empty,
/// so freed cells are always reused first.
pub struct LinkedArena<T> {
    info: Vec<Option<T>>, // INFO field; None while the cell sits on AVAIL
    link: Vec<usize>,     // LINK field
    avail: usize,         // head of the AVAIL stack, LAMBDA when empty
}

impl<T> LinkedArena<T> {
    /// A new arena with no cells drawn from memory yet.
    pub fn new() -> Self {
        LinkedArena { info: Vec::new(), link: Vec::new(), avail: LAMBDA }
    }

    /// `P <= AVAIL` (eq. (4)): grab a free cell, or fresh memory if the
    /// AVAIL stack is empty. The cell's LINK is reset to Λ.
    pub fn allocate(&mut self, info: T) -> usize {
        if self.avail != LAMBDA {
            // P <- AVAIL, AVAIL <- LINK(AVAIL).
            let p = self.avail;
            self.avail = self.link[p];
            self.info[p] = Some(info);
            self.link[p] = LAMBDA;
            p
        } else {
            // AVAIL empty: draw a brand-new cell from the memory pool.
            self.info.push(Some(info));
            self.link.push(LAMBDA);
            self.info.len() - 1
        }
    }

    /// `AVAIL <= P` (eq. (5)): return cell `p` to the free stack and yield
    /// its INFO. Panics if `p` is not a currently allocated cell.
    pub fn free(&mut self, p: usize) -> T {
        let x = self.info[p].take().expect("free of a cell not in use");
        // LINK(P) <- AVAIL, AVAIL <- P.
        self.link[p] = self.avail;
        self.avail = p;
        x
    }

    /// How many cells have ever been drawn from the memory pool (allocated
    /// nodes + nodes on the AVAIL stack). If freeing works, this stays
    /// bounded by the *peak* number of simultaneously live nodes.
    pub fn cells_in_memory(&self) -> usize {
        self.info.len()
    }

    /// INFO(p). Panics if `p` is not allocated.
    pub fn info(&self, p: usize) -> &T {
        self.info[p].as_ref().expect("INFO of a cell not in use")
    }

    /// LINK(p).
    pub fn link(&self, p: usize) -> usize {
        self.link[p]
    }

    /// LINK(p) <- target.
    pub fn set_link(&mut self, p: usize, target: usize) {
        self.link[p] = target;
    }

    /// Insert a new node at the front of the list headed by `first`
    /// (§2.2.3, eq. (8)): `P <= AVAIL; INFO(P) <- info; LINK(P) <- first`.
    /// Returns the new first node.
    pub fn push_front(&mut self, first: usize, info: T) -> usize {
        let p = self.allocate(info);
        self.link[p] = first;
        p
    }

    /// Delete the node *after* `p` (§2.2.3: deleting a known node's
    /// successor is the O(1) deletion a singly linked list supports):
    /// `Q <- LINK(P); LINK(P) <- LINK(Q); AVAIL <= Q`.
    /// Returns its INFO, or `None` if `p` was the last node.
    pub fn delete_after(&mut self, p: usize) -> Option<T> {
        let q = self.link[p];
        if q == LAMBDA {
            return None;
        }
        self.link[p] = self.link[q];
        Some(self.free(q))
    }

    /// Walk the list headed by `first`, collecting the INFO fields in order.
    pub fn to_vec(&self, first: usize) -> Vec<T>
    where
        T: Clone,
    {
        let mut out = Vec::new();
        let mut p = first;
        while p != LAMBDA {
            out.push(self.info(p).clone());
            p = self.link[p];
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Algorithm 2.2.3T — Topological sorting
// ---------------------------------------------------------------------------

/// Algorithm 2.2.3T (Topological sort), step-faithful.
///
/// Objects are numbered 1..=n; each pair `(j, k)` in `relations` declares
/// j ≺ k (j must precede k). Returns a linear arrangement of 1..=n
/// consistent with every relation, or `None` if the relations contain a
/// cycle (in particular, any pair with j = k). Panics if a pair mentions an
/// object outside 1..=n.
///
/// Queue discipline (this makes the output deterministic): successors of a
/// node are kept in the order the input pairs listed them; step T4 scans
/// k = 1, 2, …, n and enqueues the initially-unconstrained objects in
/// increasing order; the queue is FIFO (new ready items are appended at the
/// rear R, exactly as Knuth's QLINK trick does).
pub fn topological_sort(n: usize, relations: &[(usize, usize)]) -> Option<Vec<usize>> {
    // T1. [Initialize.] COUNT[k] <- 0 and the successor list of k empty,
    //     for 1 <= k <= n. (Index 0 is unused, as in the book.)
    let mut count = vec![0usize; n + 1];
    let mut suc: Vec<Vec<usize>> = vec![Vec::new(); n + 1];

    // T2. [Next relation.] / T3. [Record the relation.] For each input pair
    //     (j, k): increase COUNT[k] by one and add k to j's successor list.
    for &(j, k) in relations {
        assert!(
            (1..=n).contains(&j) && (1..=n).contains(&k),
            "relation ({j}, {k}) mentions an object outside 1..={n}"
        );
        if j == k {
            return None; // j ≺ j is already a cycle
        }
        count[k] += 1;
        suc[j].push(k);
    }

    // T4. [Scan for zeros.] Link every object with COUNT = 0 into a queue,
    //     front F, rear R. (Knuth threads the queue through the COUNT/QLINK
    //     array; a VecDeque-style index queue is the same thing.)
    let mut queue: std::collections::VecDeque<usize> =
        (1..=n).filter(|&k| count[k] == 0).collect();

    let mut output = Vec::with_capacity(n);
    // T5. [Output front of queue.] While the queue is nonempty:
    while let Some(f) = queue.pop_front() {
        output.push(f);
        // T6. [Erase relations.] For every successor of F, decrease its
        //     count; T7. any count that reaches zero joins the queue at the
        //     rear.
        for &k in &suc[f] {
            count[k] -= 1;
            if count[k] == 0 {
                queue.push_back(k);
            }
        }
    }

    // T8. [End of process.] If every object was output, we have a
    //     topological order; otherwise the leftover objects all have
    //     positive counts — they lie on or beyond a cycle.
    if output.len() == n {
        Some(output)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// §2.3.1 — Binary trees and Algorithm 2.3.1T
// ---------------------------------------------------------------------------

/// A binary tree in an index arena. Each node has an INFO field and two
/// links LLINK, RLINK; Λ = [`LAMBDA`]. Nodes are created bottom-up with
/// [`BinaryTree::add_node`] and the root is set explicitly.
pub struct BinaryTree<T> {
    info: Vec<T>,
    llink: Vec<usize>,
    rlink: Vec<usize>,
    root: usize,
}

impl<T> BinaryTree<T> {
    /// The empty binary tree (root = Λ).
    pub fn new() -> Self {
        BinaryTree { info: Vec::new(), llink: Vec::new(), rlink: Vec::new(), root: LAMBDA }
    }

    /// Add a node with the given INFO and children (use `LAMBDA` for an
    /// empty subtree); returns its index. Does not change the root.
    pub fn add_node(&mut self, info: T, llink: usize, rlink: usize) -> usize {
        self.info.push(info);
        self.llink.push(llink);
        self.rlink.push(rlink);
        self.info.len() - 1
    }

    pub fn root(&self) -> usize {
        self.root
    }

    pub fn set_root(&mut self, p: usize) {
        self.root = p;
    }

    pub fn len(&self) -> usize {
        self.info.len()
    }

    pub fn is_empty(&self) -> bool {
        self.info.is_empty()
    }

    pub fn info(&self, p: usize) -> &T {
        &self.info[p]
    }

    pub fn llink(&self, p: usize) -> usize {
        self.llink[p]
    }

    pub fn rlink(&self, p: usize) -> usize {
        self.rlink[p]
    }

    pub fn set_llink(&mut self, p: usize, q: usize) {
        self.llink[p] = q;
    }

    pub fn set_rlink(&mut self, p: usize, q: usize) {
        self.rlink[p] = q;
    }

    /// Algorithm 2.3.1T (Traverse binary tree in inorder), step-faithful:
    /// the explicit-stack formulation, **not** recursion.
    pub fn inorder(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut out = Vec::with_capacity(self.info.len());
        // T1. [Initialize.] Set stack A empty, P <- T (the root).
        let mut a: Vec<usize> = Vec::new();
        let mut p = self.root;
        loop {
            // T2. [P = Λ?] If P = Λ, go to T4.
            while p != LAMBDA {
                // T3. [Stack <= P.] Set A <= P (push), P <- LLINK(P), back
                //     to T2.
                a.push(p);
                p = self.llink[p];
            }
            // T4. [P <= Stack.] If stack A is empty, the algorithm
            //     terminates; otherwise set P <= A (pop).
            match a.pop() {
                None => return out,
                Some(q) => {
                    // T5. [Visit P.] Visit NODE(P), then set P <- RLINK(P)
                    //     and return to T2.
                    out.push(self.info[q].clone());
                    p = self.rlink[q];
                }
            }
        }
    }

    /// Preorder traversal (visit, then left subtree, then right subtree) —
    /// exercise 2.3.1-12's variant of Algorithm T: visit at the moment of
    /// *pushing* rather than popping.
    pub fn preorder(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut out = Vec::with_capacity(self.info.len());
        let mut a: Vec<usize> = Vec::new();
        let mut p = self.root;
        loop {
            while p != LAMBDA {
                out.push(self.info[p].clone()); // visit on the way down
                a.push(p);
                p = self.llink[p];
            }
            match a.pop() {
                None => return out,
                Some(q) => p = self.rlink[q],
            }
        }
    }

    /// Postorder traversal (left subtree, right subtree, then visit) —
    /// exercise 2.3.1-13. Two-stack formulation: a reversed "visit root
    /// before children, right before left" preorder is exactly postorder.
    pub fn postorder(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut out = Vec::with_capacity(self.info.len());
        if self.root == LAMBDA {
            return out;
        }
        let mut a = vec![self.root];
        while let Some(p) = a.pop() {
            out.push(self.info[p].clone());
            if self.llink[p] != LAMBDA {
                a.push(self.llink[p]);
            }
            if self.rlink[p] != LAMBDA {
                a.push(self.rlink[p]);
            }
        }
        out.reverse();
        out
    }
}

/// Rebuild the unique binary tree having the given `preorder` and `inorder`
/// sequences (all elements must be distinct — then the pair of traversals
/// determines the tree, §2.3.1 exercise 7 territory).
///
/// The first preorder element is the root; it splits the inorder sequence
/// into left and right subtrees; recurse. Panics if the two sequences are
/// inconsistent.
pub fn from_traversals<T: Clone + PartialEq>(preorder: &[T], inorder: &[T]) -> BinaryTree<T> {
    assert_eq!(preorder.len(), inorder.len(), "traversals must have equal length");
    fn build<T: Clone + PartialEq>(
        tree: &mut BinaryTree<T>,
        pre: &[T],
        ino: &[T],
    ) -> usize {
        if pre.is_empty() {
            return LAMBDA;
        }
        let root = &pre[0];
        let split = ino
            .iter()
            .position(|x| x == root)
            .expect("preorder and inorder disagree");
        let left = build(tree, &pre[1..=split], &ino[..split]);
        let right = build(tree, &pre[split + 1..], &ino[split + 1..]);
        tree.add_node(root.clone(), left, right)
    }
    let mut tree = BinaryTree::new();
    let root = build(&mut tree, preorder, inorder);
    tree.set_root(root);
    tree
}

// ---------------------------------------------------------------------------
// §2.3.1 — Threaded binary trees and Algorithm 2.3.1S
// ---------------------------------------------------------------------------

/// A **fully threaded** binary tree with a list head (§2.3.1, Fig. 24 and
/// eq. (8)–(10)). Every would-be Λ link is replaced by a *thread*: a tagged
/// link to the node's inorder predecessor (left threads) or inorder
/// successor (right threads). The leftmost left thread and the last right
/// thread point to the list head, node 0, whose LLINK is the root of the
/// tree; RLINK(HEAD) = HEAD always. The empty tree has LLINK(HEAD) = HEAD
/// with LTAG = thread.
///
/// Because threads make every node's successor computable in place,
/// traversal needs **no stack and no recursion** — Algorithm S below.
struct ThreadedNode<T> {
    info: Option<T>, // None only for the head
    llink: usize,
    ltag: bool, // true = LLINK is a thread (points to inorder predecessor)
    rlink: usize,
    rtag: bool, // true = RLINK is a thread (points to inorder successor)
}

pub struct ThreadedTree<T> {
    nodes: Vec<ThreadedNode<T>>,
}

impl<T> ThreadedTree<T> {
    /// The empty threaded tree: just the list head, threaded to itself.
    pub fn new() -> Self {
        ThreadedTree {
            nodes: vec![ThreadedNode {
                info: None,
                llink: 0,
                ltag: true, // empty tree: LLINK(HEAD) is a thread to HEAD
                rlink: 0,
                rtag: false, // RLINK(HEAD) = HEAD, always a "real" link
            }],
        }
    }

    /// The list head's index (always 0). The head is not a tree node: it
    /// stores no INFO, and it acts as the "one past the end" position of
    /// the inorder sequence.
    pub fn head(&self) -> usize {
        0
    }

    /// Number of real nodes (the head does not count).
    pub fn len(&self) -> usize {
        self.nodes.len() - 1
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.len() == 1
    }

    /// INFO(p). Panics if `p` is the head.
    pub fn info(&self, p: usize) -> &T {
        self.nodes[p].info.as_ref().expect("the list head stores no INFO")
    }

    /// Insert a new node as the **left child** of `p`, taking over `p`'s
    /// old left subtree; the new node becomes the immediate inorder
    /// *predecessor* of `p`. `insert_left(head, x)` therefore appends `x`
    /// at the *end* of the inorder sequence (the whole tree becomes the new
    /// node's left subtree). Returns the new node's index.
    ///
    /// This is the left-hand mirror of Knuth's threaded insertion (§2.3.1,
    /// eq. (12) / exercise 23):
    ///
    /// ```text
    /// I1'. LLINK(N) <- LLINK(P), LTAG(N) <- LTAG(P);
    ///      RLINK(N) <- P, RTAG(N) <- 1  (thread: successor of N is P).
    /// I2'. LLINK(P) <- N, LTAG(P) <- 0.
    /// I3'. If LTAG(N) = 0: the inorder predecessor of N (the rightmost
    ///      node of its new left subtree) had a right thread to P; retarget
    ///      it to N.
    /// ```
    pub fn insert_left(&mut self, p: usize, info: T) -> usize {
        let n = self.nodes.len();
        // I1'. New node inherits P's left link/tag; its right link is a
        //      thread to P.
        self.nodes.push(ThreadedNode {
            info: Some(info),
            llink: self.nodes[p].llink,
            ltag: self.nodes[p].ltag,
            rlink: p,
            rtag: true,
        });
        // I2'. N becomes the left child of P.
        self.nodes[p].llink = n;
        self.nodes[p].ltag = false;
        // I3'. Fix the thread from N's predecessor (was aimed at P).
        if !self.nodes[n].ltag {
            let mut q = self.nodes[n].llink;
            while !self.nodes[q].rtag {
                q = self.nodes[q].rlink;
            }
            self.nodes[q].rlink = n;
        }
        n
    }

    /// Insert a new node as the **right child** of `p` (a real node, not
    /// the head), taking over `p`'s old right subtree; the new node becomes
    /// the immediate inorder *successor* of `p`. Returns its index.
    ///
    /// Knuth's Algorithm I (§2.3.1, eq. (12)):
    ///
    /// ```text
    /// I1. RLINK(N) <- RLINK(P), RTAG(N) <- RTAG(P);
    ///     LLINK(N) <- P, LTAG(N) <- 1  (thread: predecessor of N is P).
    /// I2. RLINK(P) <- N, RTAG(P) <- 0.
    /// I3. If RTAG(N) = 0: the inorder successor of N (the leftmost node of
    ///     its new right subtree) had a left thread to P; retarget it to N.
    /// ```
    pub fn insert_right(&mut self, p: usize, info: T) -> usize {
        assert!(p != self.head(), "insert_right at the head is undefined; use insert_left");
        let n = self.nodes.len();
        // I1. New node inherits P's right link/tag; its left link is a
        //     thread to P.
        self.nodes.push(ThreadedNode {
            info: Some(info),
            llink: p,
            ltag: true,
            rlink: self.nodes[p].rlink,
            rtag: self.nodes[p].rtag,
        });
        // I2. N becomes the right child of P.
        self.nodes[p].rlink = n;
        self.nodes[p].rtag = false;
        // I3. Fix the thread from N's successor (was aimed at P).
        if !self.nodes[n].rtag {
            let mut q = self.nodes[n].rlink;
            while !self.nodes[q].ltag {
                q = self.nodes[q].llink;
            }
            self.nodes[q].llink = n;
        }
        n
    }

    /// Algorithm 2.3.1S (Inorder successor in a threaded binary tree),
    /// step-faithful. The successor of the last node — and of the head in
    /// an empty tree — is the head itself; the successor of the head is the
    /// first node in inorder. **No stack**: threads carry all the state.
    pub fn successor(&self, p: usize) -> usize {
        // S1. [RLINK(P) a thread?] Set Q <- RLINK(P). If RTAG(P) = 1,
        //     terminate: the thread points straight at the successor.
        let mut q = self.nodes[p].rlink;
        if self.nodes[p].rtag {
            return q;
        }
        // S2. [Search to left.] While LTAG(Q) = 0, set Q <- LLINK(Q):
        //     descend to the leftmost node of the right subtree.
        while !self.nodes[q].ltag {
            q = self.nodes[q].llink;
        }
        // S3. [Terminate.] Q is the answer.
        q
    }

    /// The full inorder sequence obtained by iterating Algorithm S from the
    /// head until it returns to the head — no stack, no recursion, O(1)
    /// extra space.
    pub fn inorder_via_threads(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.inorder_via_threads_counting().0
    }

    /// Same traversal, also counting every link the algorithm follows
    /// (each `Q <- RLINK(...)` and each `Q <- LLINK(Q)`). A full traversal
    /// of n nodes makes n + 1 successor calls, each following RLINK once,
    /// and descends each *real* left link (at most n + 1 of them, counting
    /// the head's) exactly once — so the count is at most 2n + 2. The whole
    /// traversal is O(n) even though a single `successor` call may cost
    /// O(height).
    pub fn inorder_via_threads_counting(&self) -> (Vec<T>, usize)
    where
        T: Clone,
    {
        let mut out = Vec::with_capacity(self.len());
        let mut follows = 0usize;
        let mut p = self.head();
        loop {
            // Inline Algorithm S with link-follow accounting.
            let mut q = self.nodes[p].rlink; // S1
            follows += 1;
            if !self.nodes[p].rtag {
                while !self.nodes[q].ltag {
                    q = self.nodes[q].llink; // S2
                    follows += 1;
                }
            }
            if q == self.head() {
                return (out, follows);
            }
            out.push(self.info(q).clone());
            p = q;
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests: worked examples from the text
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_is_lifo_and_reports_conditions() {
        let mut s = ArrayStack::new(3);
        assert_eq!(s.pop(), Err(Underflow));
        assert_eq!(s.push('a'), Ok(()));
        assert_eq!(s.push('b'), Ok(()));
        assert_eq!(s.push('c'), Ok(()));
        assert!(s.is_full());
        assert_eq!(s.push('d'), Err(Overflow));
        assert_eq!(s.pop(), Ok('c'));
        assert_eq!(s.peek(), Some(&'b'));
        assert_eq!(s.pop(), Ok('b'));
        assert_eq!(s.pop(), Ok('a'));
        assert_eq!(s.pop(), Err(Underflow));
    }

    #[test]
    fn queue_wraps_around() {
        let mut q = ArrayQueue::new(3);
        assert_eq!(q.dequeue(), Err(Underflow));
        for round in 0..10 {
            assert_eq!(q.enqueue(round), Ok(()));
            assert_eq!(q.enqueue(round + 100), Ok(()));
            assert_eq!(q.dequeue(), Ok(round));
            assert_eq!(q.dequeue(), Ok(round + 100));
        }
        q.enqueue(1).unwrap();
        q.enqueue(2).unwrap();
        q.enqueue(3).unwrap();
        assert_eq!(q.enqueue(4), Err(Overflow));
        assert_eq!(q.dequeue(), Ok(1));
    }

    #[test]
    fn shunting_examples_from_2_2_1() {
        // §2.2.1: with three cars, 312 is the single unobtainable order.
        assert!(stack_permutable(&[1, 2, 3]));
        assert!(stack_permutable(&[3, 2, 1]));
        assert!(stack_permutable(&[2, 1, 3]));
        assert!(!stack_permutable(&[3, 1, 2]));
        // Not permutations at all:
        assert!(!stack_permutable(&[1, 1, 2]));
        assert!(!stack_permutable(&[0, 1, 2]));
        assert!(stack_permutable(&[])); // vacuously
    }

    #[test]
    fn avail_list_reuses_cells() {
        let mut arena: LinkedArena<u32> = LinkedArena::new();
        let mut first = LAMBDA;
        for x in [3, 2, 1] {
            first = arena.push_front(first, x);
        }
        assert_eq!(arena.to_vec(first), vec![1, 2, 3]);
        assert_eq!(arena.delete_after(first), Some(2));
        assert_eq!(arena.to_vec(first), vec![1, 3]);
        // The freed cell must be reused before new memory is drawn.
        let before = arena.cells_in_memory();
        arena.push_front(first, 9);
        assert_eq!(arena.cells_in_memory(), before);
    }

    #[test]
    fn knuth_toposort_example() {
        // §2.2.3's worked example: nine objects, input relations
        // 9≺2, 3≺7, 7≺5, 5≺8, 8≺6, 4≺6, 1≺3, 7≺4, 9≺5, 2≺8.
        // With the FIFO queue discipline the algorithm outputs
        // 1 9 3 2 7 5 4 8 6.
        let rel = [(9, 2), (3, 7), (7, 5), (5, 8), (8, 6), (4, 6), (1, 3), (7, 4), (9, 5), (2, 8)];
        assert_eq!(
            topological_sort(9, &rel),
            Some(vec![1, 9, 3, 2, 7, 5, 4, 8, 6])
        );
        // A cycle is reported as None.
        assert_eq!(topological_sort(3, &[(1, 2), (2, 3), (3, 1)]), None);
    }

    /// The expression tree for a*(b - c) + d/e used across the tree tests.
    fn expression_tree() -> BinaryTree<char> {
        let mut t = BinaryTree::new();
        let a = t.add_node('a', LAMBDA, LAMBDA);
        let b = t.add_node('b', LAMBDA, LAMBDA);
        let c = t.add_node('c', LAMBDA, LAMBDA);
        let minus = t.add_node('-', b, c);
        let times = t.add_node('*', a, minus);
        let d = t.add_node('d', LAMBDA, LAMBDA);
        let e = t.add_node('e', LAMBDA, LAMBDA);
        let div = t.add_node('/', d, e);
        let plus = t.add_node('+', times, div);
        t.set_root(plus);
        t
    }

    #[test]
    fn traversals_of_the_expression_tree() {
        let t = expression_tree();
        assert_eq!(t.preorder(), "+*a-bc/de".chars().collect::<Vec<_>>());
        assert_eq!(t.inorder(), "a*b-c+d/e".chars().collect::<Vec<_>>());
        assert_eq!(t.postorder(), "abc-*de/+".chars().collect::<Vec<_>>());
    }

    #[test]
    fn rebuild_from_traversals() {
        let t = expression_tree();
        let rebuilt = from_traversals(&t.preorder(), &t.inorder());
        assert_eq!(rebuilt.preorder(), t.preorder());
        assert_eq!(rebuilt.inorder(), t.inorder());
        assert_eq!(rebuilt.postorder(), t.postorder());
    }

    #[test]
    fn threads_traverse_without_a_stack() {
        // Build a*(b-c)+d/e in the threaded representation by inorder
        // insertions, then check Algorithm S reproduces the inorder walk.
        let mut t = ThreadedTree::new();
        let plus = t.insert_left(t.head(), '+'); // whole tree so far
        let times = t.insert_left(plus, '*');
        let div = t.insert_right(plus, '/');
        let a = t.insert_left(times, 'a');
        let minus = t.insert_right(times, '-');
        t.insert_left(minus, 'b');
        t.insert_right(minus, 'c');
        t.insert_left(div, 'd');
        t.insert_right(div, 'e');
        let _ = a;
        assert_eq!(t.inorder_via_threads(), "a*b-c+d/e".chars().collect::<Vec<_>>());
        let (seq, follows) = t.inorder_via_threads_counting();
        assert_eq!(seq.len(), 9);
        // 10 successor calls follow RLINK once each; the 5 real left links
        // (head→+, +→*, *→a, -→b, /→d) are each descended once: 15 total.
        assert_eq!(follows, 15);
        assert!(follows <= 2 * t.len() + 2);
    }
}
