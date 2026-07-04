# Module 03 — Information Structures

> **Source:** *The Art of Computer Programming*, Vol. 1, 3rd ed., §2.2.1–2.2.3
> and §2.3.1.
> **Lab:** `labs/module-03-structures` · **Grade it:** `./grade 3`
>
> This lesson is self-contained: you can complete the module without the book.
> If you own Vol. 1, read §2.2 and §2.3.1 alongside it.

Chapter 2 is where Knuth teaches how to *represent* data, not just compute with
it. Everything in this module is built from one primitive — a **link**, an
address of another datum — and one discipline for handing out storage. By the
end you will have built stacks, queues, singly linked lists with a free-cell
recycler, a topological sorter, and binary trees in three guises (ordinary,
reconstructed-from-traversals, and threaded), all on the same memory model.

> **Companion exhibit — _Ariadne's Thread_.** The linked list of stage 2 has a
> visual twin in the [Museum of Algorithms](https://marcelaldecoa.github.io/knuth-taocp/museum/exhibit-1.2-ariadnes-thread.html):
> the _Memory Untangler_, where you tie pointer-threads from HEAD through
> scattered cells, then cut a node and watch the list heal itself in a single
> pointer write as the freed cell returns to the AVAIL pool. Open it in a second
> window — or reach it any time from the **Museum** tab in the top navigation.

---

## 1. Two ways to store a sequence

Suppose you must store the sequence $(x_1, x_2, \ldots, x_n)$ and support insertion and
deletion in the middle. There are exactly two classical representations, and
the whole chapter turns on their trade-off.

**Sequential allocation (§2.2.2).** Put the items in consecutive locations:
`X[1], X[2], …`. The successor of `X[i]` is *computed* — it is `X[i+1]`, no
storage needed to say so. Random access is $O(1)$. But inserting into the middle
must shift everything after it: $O(n)$ work, and the block has a fixed size fixed
in advance.

**Linked allocation (§2.2.3).** Give each item its own *node* carrying the item
(`INFO`) and the address of the next node (`LINK`). The successor is *stored*,
costing one extra field per node. Now insertion and deletion in the middle are
$O(1)$ pointer surgery — but you lose $O(1)$ random access (you must walk the
links), and you pay the memory for the links.

Knuth's slogan: **sequential storage trades flexibility for density; linked
storage trades density for flexibility.** Neither is "better"; the access
pattern decides. This module implements both and lets you feel the difference.

### The memory model we use everywhere

Knuth writes his structures for MIX, where a node is a machine word and a link
is an address. The faithful — and idiomatic — Rust translation is an
**index-based arena**:

- a node "lives at address `p`" means it is stored at index `p` of a `Vec`;
- a link is a plain `usize`, an index into that `Vec`;
- the **null link $\Lambda$** (Knuth's "points nowhere") is a reserved value. We use
  `LAMBDA = usize::MAX`, which is never a valid index, so index 0 stays an
  ordinary usable cell. (MIX used 0 = Λ and thereby wasted location 0.)

No `Rc`, no `RefCell`, no `unsafe`. A link is a number, exactly as in the book,
and the borrow checker never fights you because there is only one owner — the
arena's `Vec`. When Knuth writes `INFO(P)` and `LINK(P)`, we write `info[p]`
and `link[p]`.

---

## 2. Stacks and queues in sequential storage (§2.2.1–2.2.2)

A **stack** is a sequence with insertion and deletion only at one end (the
*top*): last-in, first-out (LIFO). A **queue** restricts insertion to one end
(the *rear*) and deletion to the other (the *front*): first-in, first-out
(FIFO). A **deque** allows both ends; we don't need it here.

### The stack, and the OVERFLOW/UNDERFLOW contract

Knuth keeps a pointer `T` into a block of `M` cells. Insertion and deletion are
two lines each (§2.2.2, eq. (2)–(3)):

```text
X <= stack (insert Y):    T <- T + 1;  if T > M then OVERFLOW;  X[T] <- Y.
stack => X (delete):      if T = 0 then UNDERFLOW;  Y <- X[T];  T <- T - 1.
```

Two boundary conditions have names because they matter:

- **OVERFLOW** — inserting into a full structure (`T > M`).
- **UNDERFLOW** — deleting from an empty one (`T = 0`).

A subtle but important design point Knuth stresses: *the structure does not
decide what to do about them.* UNDERFLOW usually means "no more work" — a
perfectly normal signal to stop — and OVERFLOW might mean "grow the block" or
"abort" depending on the caller. So our API **reports** these conditions
instead of panicking:

```rust
pub fn push(&mut self, x: T) -> Result<(), Overflow>;   // Err(Overflow) when full
pub fn pop(&mut self)          -> Result<T, Underflow>;  // Err(Underflow) when empty
```

Nothing panics; the caller reads the `Result` and applies *its* policy.

### The queue, and why a circular buffer loses a cell

A queue in `M` sequential cells is naturally *circular*: two pointers `R`
(rear) and `F` (front) chase each other around the ring, each advancing modulo
`M` (§2.2.2, eq. (6)–(7)):

```text
X <= queue:   R <- (R + 1) mod M;  if R = F then OVERFLOW;  X[R] <- Y.
queue => X:   if F = R then UNDERFLOW;  F <- (F + 1) mod M;  Y <- X[F].
```

There is a classic snag (§2.2.2, exercise 1): with only the two pointers,
`R = F` means *both* "empty" and "full" — you cannot tell them apart, so you
must sacrifice one cell and cap the queue at `M − 1` items. The standard cure,
which our `ArrayQueue` uses, is to store an explicit **length** alongside the
pointers; then all `M` cells are usable and full/empty are distinguished by
`len == M` vs `len == 0`. The rear cell is always at `(F + len) mod M`.

### Railway shunting and stack-permutable permutations (§2.2.1)

Knuth opens the chapter with a picture: railway cars 1, 2, …, n arrive in that
order, and a single **siding** (a stack) can hold cars temporarily before they
leave onto the main line. Which output orders are achievable?

The simulation is greedy and provably optimal. To produce a wanted car next:
push arriving cars until the wanted one has entered the siding; it must then be
exactly on top, so pop it. If it isn't on top, the order is impossible.

```rust
fn stack_permutable(perm: &[usize]) -> bool {
    let mut stack = Vec::new();
    let mut next = 1;                 // next car still on the input track
    for &want in perm {
        while next <= want { stack.push(next); next += 1; }
        if stack.pop() != Some(want) { return false; }
    }
    true
}
```

**Theorem (§2.2.1).** A permutation is obtainable through one stack **iff** it
avoids the pattern **3-1-2**: there are no positions $i < j < k$ with
`perm[j] < perm[k] < perm[i]`.

*Why the pattern is fatal.* Say `perm[i]` is the large value `c`, and later two
smaller values `a < b` appear in the order a-then-b (that is `perm[j]=a`,
`perm[k]=b`, with `a < b < c`). To output `c` before `a` and `b`, all three
must have entered the siding, and `c` entered *first* (it has the earliest
position but the input arrives in increasing value, so… — careful: positions
are output order). The clean statement is the one the tests use, and the
smallest instance is $n = 3$: of the 6 orders, exactly **312** is unobtainable.

**Counting.** The number of obtainable permutations of $n$ cars is the **Catalan
number**

$$C_n = \frac{1}{n+1} \binom{2n}{n}, \qquad C_0, C_1, \ldots = 1, 1, 2, 5, 14, 42, 132, \ldots$$

For $n = 3$ that is $C_3 = 5$ (all but 312); for $n = 4$, $C_4 = 14$ of the 24 orders.
The lab verifies this count exhaustively for $n \le 6$. Catalan numbers count a
staggering number of things — and one of them, the number of binary trees on $n$
nodes, returns in §5.

---

## 3. Linked allocation and the AVAIL list (§2.2.3)

Now give each list node two fields, `INFO` and `LINK`, and let a list be a
chain of nodes ending in $\Lambda$. The operations that made linked storage worth its
overhead:

```text
insert after P:   Q <= AVAIL;  INFO(Q) <- Y;  LINK(Q) <- LINK(P);  LINK(P) <- Q.
delete after P:   Q <- LINK(P);  LINK(P) <- LINK(Q);  AVAIL <= Q.
```

Both are $O(1)$. The mysterious `AVAIL` is the heart of §2.2.3.

### Where do nodes come from? The AVAIL stack

Rather than ask the operating system for memory on every insert, Knuth keeps a
**free list**: a stack of currently-unused nodes, threaded through their own
`LINK` fields, with head pointer `AVAIL`. Allocation and deallocation are just
stack operations on it (eq. (4)–(5)):

```text
P <= AVAIL   (allocate):   if AVAIL = Λ then OVERFLOW (or draw fresh memory);
                           P <- AVAIL;  AVAIL <- LINK(AVAIL).
AVAIL <= P   (free):       LINK(P) <- AVAIL;  AVAIL <- P.
```

The elegance: a freed node goes straight back onto `AVAIL` and is the very next
node handed out. In our `LinkedArena`, fresh cells are drawn from the backing
`Vec` **only when `AVAIL` is empty**. Consequence — and this is the property
the lab checks hardest:

> `cells_in_memory()`, the number of cells ever drawn from the pool, is bounded
> by the **peak** number of simultaneously live nodes, no matter how many total
> allocate/free operations occur.

That is exactly what a real allocator's free list buys you: churn is free. A
program that repeatedly deletes one node and inserts another never grows.

`info[p]` is `Some` while cell `p` is in use and `None` while it sits on
`AVAIL` — a cheap, self-checking invariant.

---

## 4. Topological sorting — Algorithm 2.2.3T (§2.2.3)

A **partial order** on objects $\{1, \ldots, n\}$ is a set of relations $j \prec k$
("$j$ precedes $k$") that is transitive and has no cycles. A **topological sort**
(a *linear extension*) arranges all $n$ objects in a line so that every relation
points forward: if $j \prec k$ then $j$ appears before $k$. Knuth's application is
sorting the entries of a glossary, or the modules of a program, so each is
defined before it is used.

### Algorithm T

The idea: an object with **no predecessors** (in-count zero) may be output now;
outputting it erases its outgoing relations, possibly freeing others.

```text
T1. [Initialize.]      COUNT[k] <- 0 and an empty successor list, for 1<=k<=n.
T2. [Next relation.]   For each input pair (j, k):
T3. [Record it.]         COUNT[k] <- COUNT[k] + 1;  append k to SUC(j).
T4. [Scan for zeros.]  Put every k with COUNT[k] = 0 into a queue, scanning
                       k = 1, 2, …, n in increasing order (front F, rear R).
T5. [Output front.]    While the queue is nonempty: remove the front F, output it.
T6. [Erase relations.] For each successor k of F, in recorded order:
                         COUNT[k] <- COUNT[k] - 1;
T7. [Zero?]              if COUNT[k] = 0, append k at the rear R.
T8. [End.]             If all n objects were output, that is a topological
                       order; otherwise the survivors lie on a cycle → no order.
```

Knuth threads the queue *through the COUNT array itself* (the QLINK trick: a
zero-count object's slot is reused as its queue link), so the whole algorithm
runs in $O(n + m)$ time and $O(n)$ space with no extra allocation, where $m$ is the
number of relations. Our version uses a `VecDeque` of indices — the same FIFO,
spelled more plainly.

**Determinism.** The output is fixed by two conventions: initial zeros enter
the queue in increasing index order (T4), and each object's successors are
scanned in the order the input listed them (T6). FIFO does the rest.

### Hand trace on a 7-element partial order

Take objects 1..7 with these eight relations, listed in this input order:

$$1 \prec 2, \quad 1 \prec 3, \quad 2 \prec 4, \quad 3 \prec 4, \quad 4 \prec 5, \quad 4 \prec 6, \quad 5 \prec 7, \quad 6 \prec 7$$

pictured as a DAG (a diamond that splits at 1, rejoins at 4, splits again,
rejoins at 7):

```text
        2       5
      /   \   /   \
    1       4       7
      \   /   \   /
        3       6
```

**T1–T3 (build COUNT and successor lists):**

| k | COUNT[k] | SUC(k) |
|---|----------|--------|
| 1 | 0 | 2, 3 |
| 2 | 1 | 4 |
| 3 | 1 | 4 |
| 4 | 2 | 5, 6 |
| 5 | 1 | 7 |
| 6 | 1 | 7 |
| 7 | 2 | — |

**T4:** scan 1..7 for COUNT = 0 → only object 1. Queue = [1].

**T5–T7 loop** (output F, decrement each successor, enqueue new zeros at rear):

| output F | successors decremented | new zeros → rear | queue after | output so far |
|----------|------------------------|------------------|-------------|---------------|
| 1 | 2→0, 3→0 | 2, 3 | [2, 3] | 1 |
| 2 | 4→1 | — | [3] | 1 2 |
| 3 | 4→0 | 4 | [4] | 1 2 3 |
| 4 | 5→0, 6→0 | 5, 6 | [5, 6] | 1 2 3 4 |
| 5 | 7→1 | — | [6] | 1 2 3 4 5 |
| 6 | 7→0 | 7 | [7] | 1 2 3 4 5 6 |
| 7 | — | — | [] | 1 2 3 4 5 6 7 |

**T8:** all 7 objects were output → the topological order is
**1 2 3 4 5 6 7**. Verify by eye: every one of the eight relations points
left-to-right in that line. ✓

Had we added the relation $7 \prec 1$, object 1 would start with COUNT 1, no object
would have count zero, the queue would be empty at T4, nothing is output, and
T8 reports a cycle → `None`. The lab checks exactly this: a valid extension
always satisfies every relation, and any cycle (including a self-loop $j \prec j$)
yields `None`.

---

## 5. Binary trees and their traversal (§2.3.1)

A **binary tree** is either empty, or a root node with a *left subtree* and a
*right subtree*, each itself a binary tree. The order matters and the two
subtrees are distinguished — this is *not* the same as an ordinary tree.
Represent it with three fields per node: `INFO`, `LLINK`, `RLINK`, using Λ for
an empty subtree. In our arena, nodes are appended with `add_node(info, l, r)`
and the root is set explicitly.

### Counting binary trees: Catalan again

**Theorem.** The number of distinct binary trees with $n$ nodes is the Catalan
number $C_n$.

*Proof.* Let $b_n$ be that number, $b_0 = 1$. A nonempty tree splits into a root, a
left subtree of some size $k$, and a right subtree of size $n-1-k$. Summing over $k$,

$$b_n = \sum_{k=0}^{n-1} b_k \cdot b_{n-1-k},$$

the Catalan recurrence. Its solution is $C_n = \binom{2n}{n}/(n+1)$. ∎

So the very count that governed stack-permutable permutations governs tree
shapes — no coincidence: a stack trace of a traversal *is* a way of encoding
the tree, and the encodings are the stack-realizable sequences.

### Three traversals

Each visits every node once; they differ in *when* the root is visited relative
to its subtrees.

- **Preorder** — root, then left subtree, then right subtree.
- **Inorder** — left subtree, then root, then right subtree.
- **Postorder** — left subtree, then right subtree, then root.

For the expression tree of `a*(b − c) + d/e`

```text
          +
        /   \
       *     /
      / \   / \
     a   - d   e
        / \
       b   c
```

they give the **prefix**, **infix**, and **postfix (Polish)** forms:

| order | result |
|-------|--------|
| preorder | `+ * a - b c / d e` |
| inorder | `a * b - c + d / e` |
| postorder | `a b c - * d e / +` |

### Algorithm T: inorder without recursion, and the stack invariant

Recursion hides a stack in the call frames; Knuth makes it explicit
(§2.3.1, Algorithm 2.3.1T), which is the *point* of this stage:

```text
T1. [Initialize.] Stack A empty;  P <- root.
T2. [P = Λ?]      If P = Λ, go to T4.
T3. [Stack <= P.] Push P;  P <- LLINK(P);  return to T2.
T4. [P <= Stack.] If A empty, terminate;  else pop P from A.
T5. [Visit P.]    Visit NODE(P);  P <- RLINK(P);  return to T2.
```

**The stack invariant.** At every arrival at T2, stack A holds — from top to
bottom — exactly the chain of ancestors of the current position `P` *whose
left subtrees we have entered but whose own value we have not yet visited*.
T3 descends left, stacking each node we pass so we can return to visit it; T4
pops the nearest such node when the left descent bottoms out (`P = Λ`); T5
visits it and turns right. Because each node is pushed once and popped once, the
traversal is $O(n)$ time, and the stack's maximum depth equals the tree's height.

`preorder` is the same skeleton but visits a node *when pushing* it (T3), not
when popping. `postorder` can be done with a two-stack trick: a "root, right,
left" preorder reversed is exactly postorder. (Both are exercises 2.3.1-12/13.)

### Reconstructing a tree from two traversals

**Theorem.** If all keys are distinct, the pair (preorder, inorder) determines
the tree uniquely.

*Construction (`from_traversals`).* The first preorder element is the root.
Find it in the inorder sequence; everything to its left is the inorder of the
left subtree, everything to its right the inorder of the right subtree — and
their sizes tell you how to split the *rest* of the preorder correspondingly.
Recurse on the two halves. The lab round-trips: build a tree, take its preorder
and inorder, rebuild, and check all three traversals match the original.

---

## 6. Threaded binary trees — Algorithm 2.3.1S (§2.3.1)

Look at an $n$-node binary tree: it has $2n$ links, of which $n - 1$ are used to bind
the tree together, leaving **$n + 1$ null links** wasted. A. J. Perlis and
C. Thornton's idea (§2.3.1): replace each null link with a **thread** — a
tagged link to where a stackless traversal would want to go next.

- A null `LLINK` becomes a *left thread* to the node's **inorder predecessor**.
- A null `RLINK` becomes a *right thread* to the node's **inorder successor**.

A tag bit per link (`LTAG`, `RTAG`; true = "this is a thread") tells threads
from real child links. A **list head** node (index 0, no INFO) anchors the
tree: `LLINK(HEAD)` is the root, `RLINK(HEAD) = HEAD`, and it plays the role of
"one past the end" of the inorder sequence — the first node's left thread and
the last node's right thread both point to it. The empty tree is just the head,
left-threaded to itself.

### Algorithm S: the inorder successor, stackless

```text
S1. [RLINK a thread?]  Q <- RLINK(P);  if RTAG(P) = 1, terminate: Q is it.
S2. [Search to left.]  While LTAG(Q) = 0:  Q <- LLINK(Q).
S3. [Done.]            Q is the inorder successor of P.
```

Why it works: if P's right link is a thread, it points *straight at* the
successor — done. Otherwise the successor is the leftmost node of P's right
subtree, and S2 walks left links (real ones only; a left *thread* stops it)
down to it. No stack, no recursion, $O(1)$ extra space — the promise threads were
invented to keep.

Iterating S from the head until it returns to the head reproduces the ordinary
inorder walk. And it is genuinely **$O(n)$** overall even though one `successor`
call may cost $O(\text{height})$: across a full traversal, each of the $n + 1$ successor
calls follows `RLINK` once, and each *real* left link is descended exactly once,
so the total number of link-follows is at most **$2n + 2$**. The lab measures
this count directly (15 follows for the 9-node expression tree) — a concrete,
checkable proof of linearity.

Insertion keeps the threads consistent (Knuth's Algorithm I, eq. (12)). To
insert N as the right child of P, N inherits P's old right link/tag, N's left
link becomes a thread back to P (P is N's inorder predecessor), P's right link
now points at N; and if N acquired a real right subtree, the leftmost node of
that subtree had a left thread aimed at P that must be re-aimed at N. The lab's
`insert_left` is the mirror image, so `insert_left(head, x)` appends x at the
*end* of the inorder sequence.

---

## 7. Stage-by-stage lab guide

Open `labs/module-03-structures/src/lab.rs`. Run `./grade 3`; the grader takes
the five stages in order, stopping at the first failure.

### Stage 1 — `ArrayStack`, `ArrayQueue`, `stack_permutable` (§2.2.1–2.2.2)

Sequential stack and circular-buffer queue. Honour the contract: `push` /
`enqueue` return `Err(Overflow)` when full, `pop` / `dequeue` return
`Err(Underflow)` when empty — never panic. Keep an explicit length in the queue
so all M cells are usable and wraparound works. `stack_permutable` is the
greedy siding simulation of §2; the tests confirm it splits the permutations of
4 exactly on pattern 312 and that realizable counts are Catalan numbers.

### Stage 2 — `LinkedArena` (§2.2.3)

The INFO/LINK arena with an AVAIL free stack. Implement `allocate` (`P <=
AVAIL`, growing the pool only when AVAIL = Λ), `free` (`AVAIL <= P`),
`push_front`, `delete_after`, and `to_vec`. The decisive test: under a long
churn of delete+insert, `cells_in_memory()` must stay pinned at the peak live
count — freed cells *must* be reused before fresh memory is drawn, and AVAIL is
LIFO.

### Stage 3 — `topological_sort` (Algorithm 2.2.3T)

Implement Algorithm T with a FIFO queue. Match the queue discipline exactly
(initial zeros in increasing index order, successors in input order) so the
output is deterministic; the lab pins Knuth's 9-object worked example to
`1 9 3 2 7 5 4 8 6`. Return `None` on any cycle (including j ≺ j); panic only if
a relation names an object outside 1..=n. The property test feeds random
*acyclic* relation sets and checks the output is a genuine linear extension.

### Stage 4 — `BinaryTree` traversals + `from_traversals` (Algorithm 2.3.1T)

Implement `inorder` with the **explicit stack** (not recursion), plus
`preorder` and `postorder`, and `from_traversals`. Anchor: the expression tree
gives prefix/infix/postfix. The property tests build random binary *search*
trees (whose inorder must be sorted) and round-trip them through
`from_traversals`.

### Stage 5 — `ThreadedTree` (Algorithm 2.3.1S)

The fully threaded tree with a list head. Implement `insert_left`,
`insert_right`, the stackless `successor` (Algorithm S), and the two inorder
walkers. `inorder_via_threads` must equal an ordinary inorder walk, and
`inorder_via_threads_counting` must report ≤ 2n + 2 link-follows — the proof
the walk is O(n).

---

## 8. Check your understanding

1. Why must a two-pointer circular queue waste one cell, and how does an
   explicit length fix it? (Empty and full both show `R = F`.)
2. After 10⁶ interleaved `push_front`/`delete_after` calls on a list that never
   exceeds 5 live nodes, how large is `cells_in_memory()`? Why? (5 — AVAIL
   reuse.)
3. In Algorithm T, what does it *mean* if the queue empties before all n
   objects are output? (A cycle: every survivor still has a predecessor.)
4. State the stack invariant of Algorithm 2.3.1T at step T2 in one sentence.
5. An n-node binary tree has how many null links, and what does threading do
   with them? (n + 1; turns each into a thread to an inorder neighbour.)
6. Why is iterating Algorithm S over the whole tree O(n) even though one call
   can be O(height)? (Amortization: each real left link is descended once.)

## 9. Exercises from the text

Ratings use Knuth's scale: 00 immediate · 10 a minute · 20 fifteen minutes to
an hour · 30 hours · 40 term project · 50 open research problem. ▶ marks
especially instructive ones. Log attempts in
`course/module-03-structures/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| ▶2.2.1-2 | 20 | Show a permutation is obtainable through a stack iff it avoids the pattern 3-1-2. |
| 2.2.1-3 | 22 | How many permutations of n elements are obtainable through one stack? (Catalan C_n.) |
| 2.2.1-5 | 22 | Same question for a single queue; for a single deque. |
| 2.2.2-1 | 10 | Why can a circular queue with only two pointers hold at most M − 1 items? |
| ▶2.2.3-6 | 22 | Give the details of the QLINK trick that threads Algorithm T's queue through the COUNT array. |
| 2.2.3-7 | 25 | Prove Algorithm T outputs a valid topological order and detects every cycle. |
| ▶2.3.1-12 | 20 | Modify Algorithm T to produce preorder; then postorder. |
| 2.3.1-13 | 22 | Design a stack-based postorder traversal and prove it visits each node once. |
| 2.3.1-21 | 22 | Prove Algorithm S returns the inorder successor for every node, including the last. |
| ▶2.3.1-23 | 24 | Write the threaded insertion Algorithm I and prove it preserves all threads. |

## Why it's done this way

Knuth's MIX-era memory model — INFO and LINK fields in flat storage, an
AVAIL list of free cells, Λ for "no link" — looks dated until you write a
linked structure in Rust. Then it turns out to be the *idiomatic* answer:
a `Vec<Node>` arena with `usize` links sidesteps the borrow checker's
hostility to pointer cycles, keeps nodes cache-adjacent, and makes every
"pointer" printable and testable. We use it in every tree module that
follows, not out of nostalgia but because the 1968 memory model and the
2020s performance model agree.

## In the real world

Topological sort *is* your build system: cargo, make, and every CI pipeline
run Algorithm 2.2.3T's in-degree-and-queue idea over dependency graphs, and
spreadsheets recompute cells with it. Arena allocation is how rustc itself
stores its ASTs, how game engines lay out entities, and how databases
manage pages. The threaded tree's central trick — a structure whose unused
null links are recycled to encode extra navigation for free — resurfaces in
succinct data structures and in the parent-pointer compression tricks of
modern B-tree implementations. And the stack-realizability question of
stage 1 is a bona fide interview classic with a Catalan-number answer.

## Proof techniques you practiced

- **Data-structure invariants** — each operation preserves a stated shape
  property (queue wraparound, AVAIL-list integrity, thread correctness).
- **Counting via bijection** — binary trees ↔ balanced parenthesis strings
  ↔ stack-realizable outputs: three faces of the Catalan numbers.
- **Structural induction on trees** — traversal correctness and the
  preorder+inorder reconstruction argument.
- **Conservation arguments** — topological sort terminates having output
  everything iff no cycle exists: count what enters and leaves the queue.

## 10. Where this leads

- The **AVAIL free list** is the seed of dynamic storage allocation (§2.5) and
  of every real allocator you will ever use. When a program *forgets* to return
  cells, the machine must reclaim them itself — garbage collection (§2.3.5). See
  it run in the Museum's [_The Specter of Forgetting_](https://marcelaldecoa.github.io/knuth-taocp/museum/exhibit-1.3-specter-of-forgetting.html):
  allocate until the heap chokes, then a mark-and-sweep radar traces the live
  objects from the roots and burns the forgotten ones to dust.
- **Topological sort** underlies build systems, spreadsheet recalculation, and
  instruction scheduling; the DAG machinery returns throughout Vol. 1 §2.3.4.
- **Binary trees** are the backbone of Vol. 3: binary search trees, balanced
  trees, heaps, and the tree-based sorts all build on the traversal and
  reconstruction skills from Stage 4.
- **Threaded trees** foreshadow the space-saving representations of §2.3.2–2.3.5
  and the general principle that *wasted structure is opportunity*.
- **Catalan numbers** reappear whenever a structure decomposes recursively into
  a root plus independent parts — a theme of Vol. 4's combinatorics.
