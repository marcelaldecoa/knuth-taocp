# Exercises — Module 03 (Information Structures, §2.2.1–2.2.3, §2.3.1)

Self-contained problems on this module's material — stacks and queues in
sequential storage, the AVAIL free list, topological sorting (Algorithm T),
binary-tree traversal, and threaded trees (Algorithm S). You can work every one
**without the book**: each states the problem in full, gives a **hint** to peek
at when stuck, and a worked **answer sketch** to check against. Counts below are
reproduced by exhaustive enumeration (or the code you write in the lab).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise, its number is noted for
readers who own Volume 1.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | ▶ Stack-realizable $\iff$ avoids the pattern 3-1-2 | 20 | ⬜ |
| 2 | Count of stack-realizable permutations is $C_n$ | 22 | ⬜ |
| 3 | Permutations through one queue? one deque? | 22 | ⬜ |
| 4 | Why a two-pointer circular queue caps at $M-1$ | 10 | ⬜ |
| 5 | ▶ The QLINK trick: thread the queue through COUNT | 22 | ⬜ |
| 6 | Algorithm T is correct and detects every cycle | 25 | ⬜ |
| 7 | ▶ Preorder and postorder variants of the stack walk | 20 | ⬜ |
| 8 | Stack-based postorder visits each node once | 22 | ⬜ |
| 9 | Algorithm S returns the successor for every node | 22 | ⬜ |
| 10 | ▶ Threaded insertion (Algorithm I) preserves threads | 24 | ⬜ |

## Problems

### 1. ▶ Stack-realizable $\iff$ avoids the pattern 3-1-2 (rating 20 · cf. 2.2.1–2)

**Problem.** Railway cars $1, 2, \dots, n$ arrive in that order; a single siding
(a stack) may hold cars before they leave onto the main line. Prove that an
output permutation is achievable through the stack **iff** it avoids the pattern
**3-1-2**: there are no positions $i < j < k$ with
$\text{perm}[j] < \text{perm}[k] < \text{perm}[i]$ (a large value output first,
then two smaller values in increasing order).

**Hint.** Cars arrive in *increasing value* order, so any car with a smaller
value has already arrived. For necessity, look at the moment the large value is
output and ask where the two smaller ones must be. For sufficiency, argue the
contrapositive: whenever the greedy siding simulation fails, it exposes a 3-1-2.

**Answer sketch.** *Necessity (a 3-1-2 is fatal).* Suppose the output has
$\text{perm}[i] = c$, $\text{perm}[j] = a$, $\text{perm}[k] = b$ with
$i < j < k$ and $a < b < c$. When $c$ is output (earliest, at position $i$), cars
$a$ and $b$ have already arrived (they have smaller values, and arrival is
increasing), yet neither has been output (they leave later). So both sit in the
stack when $c$ is popped. Now $a$ leaves before $b$ (position $j < k$); by LIFO
that forces $a$ to be *above* $b$, i.e. $a$ was pushed after $b$, i.e. $a$ arrived
after $b$, i.e. $a > b$ — contradicting $a < b$. So no stack can produce the
pattern.

*Sufficiency (avoiding it suffices), by contrapositive.* Run the greedy
simulation from the lesson (push arriving cars until the wanted one is on top,
then pop it); it is optimal, so if it *fails* the permutation is unrealizable —
and we show any failure exhibits a 3-1-2. Under greedy the stack grows in
increasing value (each pushed car is larger than all below it), so the top is
always the largest car currently held. Suppose greedy fails while trying to
output $w = \text{perm}[k]$: then $w$ is buried, and the top is some $c > w$ that
lies above $w$. Since $c$ is still in the stack, it is output later, at some
position $k' > k$, so $\text{perm}[k'] = c$. Car $c$ was pushed on top of $w$
during an earlier fetch — the fetch of $\text{perm}[i]$ for some output position
$i < k$ — and that fetch only pushes cars of value $\le \text{perm}[i]$, with
$c \ne \text{perm}[i]$ (that car left at step $i$); hence $c < \text{perm}[i]$.
Collecting the three positions $i < k < k'$ with values
$\text{perm}[k] = w < \text{perm}[k'] = c < \text{perm}[i]$ gives exactly the
forbidden pattern. So "avoids 3-1-2" implies greedy never blocks, i.e. the
permutation is realizable. $\blacksquare$ (The smallest instance is $n = 3$: of the
six orders exactly **312** is unobtainable, matching the lesson.)

### 2. Count of stack-realizable permutations is $C_n$ (rating 22 · cf. 2.2.1–3)

**Problem.** How many of the $n!$ permutations of $1, 2, \dots, n$ are obtainable
through a single stack? Derive the count and give its closed form.

**Hint.** Condition on **when car 1 is output**. Say car $n' $... simpler: let
$b_n$ be the count; think about the position of the largest car, or set up the
recurrence by splitting on where a chosen car leaves. The answer is a number you
have already met counting binary trees.

**Answer sketch.** Let $b_n$ be the number of stack-realizable permutations,
$b_0 = 1$. Consider the moment car $1$ (the first to arrive) leaves the stack: at
that instant some set of cars has arrived and the rest have not. If exactly $k$
cars are output before... the standard decomposition splits the output into the
block produced "under" a given car and the block produced after it, and the two
blocks are independently realizable of sizes $k$ and $n-1-k$. Summing over the
split point,

$$
b_n = \sum_{k=0}^{n-1} b_k\,b_{n-1-k},
$$

the **Catalan recurrence** — the very recurrence the lesson derives for the
number of binary trees on $n$ nodes. Its solution is the Catalan number

$$
b_n = C_n = \frac{1}{n+1}\binom{2n}{n}, \qquad C_0, C_1, \dots = 1, 1, 2, 5, 14, 42, 132, \dots
$$

So $C_3 = 5$ (all orders of $3$ except $312$) and $C_4 = 14$ of the $24$ orders.
Exhaustive enumeration confirms $1, 2, 5, 14, 42, 132$ for $n = 1, \dots, 6$. $\blacksquare$

### 3. Permutations through one queue? one deque? (rating 22 · cf. 2.2.1–5)

**Problem.** Repeat the counting question of Problem 2 for two other structures:
how many permutations of $1, \dots, n$ can be produced (a) through a single
**queue** (FIFO), and (b) through a single **deque** (insertion and deletion
allowed at *both* ends)?

**Hint.** A queue never reorders anything. A deque is far more powerful — try
small $n$ by hand or by search before guessing a pattern.

**Answer sketch.** *(a) Queue.* A queue is first-in-first-out and the input
arrives in order $1, 2, \dots, n$, so cars leave in exactly the order they
entered: the **only** obtainable permutation is the identity $1\,2\cdots n$.
Count $= 1$ for every $n$.

*(b) Deque.* A deque can push each arriving car onto the front or the back and
pop from either end, so it realizes far more orders. Exhaustive search gives the
counts $1, 2, 6, 24, 116$ for $n = 1, 2, 3, 4, 5$: every permutation is reachable
through $n = 4$, and the first shortfall is at $n = 5$, where $116$ of the $120$
orders are realizable (four are not). A general deque is therefore strictly more
powerful than a stack (whose counts are the Catalan $1, 2, 5, 14, \dots$) but not
omnipotent. *(A closed form for the deque count is genuine but is not derived in
this lesson; the small values above are the honest, verified takeaway.)*

### 4. Why a two-pointer circular queue caps at $M-1$ (rating 10 · cf. 2.2.2–1)

**Problem.** A queue implemented in $M$ sequential cells as a circular buffer
uses a front pointer $F$ and a rear pointer $R$, each advancing modulo $M$.
Explain why, with only these two pointers, the queue can hold at most $M - 1$
items, and how storing an explicit length removes the restriction.

**Hint.** Both the empty state and the completely full state make $F$ and $R$
land on the same relationship. What does $R = F$ mean?

**Answer sketch.** With two pointers and the lesson's update rules, the condition
$R = F$ arises in **two** different situations: when the queue is *empty* (nothing
between $F$ and $R$) and when it is *full* (the rear has wrapped all the way
around to meet the front). The two pointers alone cannot distinguish these cases,
so to keep the test unambiguous you must forbid the "full wraps to meet front"
state — i.e. leave one cell always unused, capping capacity at $M - 1$. Storing
an explicit **length** alongside $F$ and $R$ fixes it: empty is `len == 0`, full
is `len == M`, they are now distinct, and all $M$ cells become usable (the rear
cell sits at $(F + \text{len}) \bmod M$). That is exactly what the lesson's
`ArrayQueue` does. $\blacksquare$

### 5. ▶ The QLINK trick: thread the queue through COUNT (rating 22 · cf. 2.2.3–6)

**Problem.** Algorithm T (topological sort) maintains a FIFO queue of the objects
whose predecessor-count has reached zero. Give the details of Knuth's space trick
that stores this queue *inside* the `COUNT` array — no separate queue storage —
and explain why it is safe.

**Hint.** Once `COUNT[k]` hits zero, that slot's job (counting predecessors) is
finished forever. What else could the slot hold?

**Answer sketch.** When `COUNT[k]` reaches $0$, object $k$ is ready to be output
and its count field is henceforth useless — so **reuse it as the queue link**.
Rename it `QLINK[k]`: it will hold the index of the *next* object in the queue
(or a null marker for the tail). Keep two scalars, a front $F$ and a rear $R$.

- *Enqueue $k$* (when its count drops to $0$): set `QLINK[R] <- k`, then
  `R <- k`. (Initialize by scanning $k = 1, \dots, n$ and chaining every
  initially-zero object in increasing order.)
- *Dequeue*: take $F$'s object as output, then advance `F <- QLINK[F]`.

The queue is thus a singly linked list threaded through the very array cells
whose counts have expired. It is safe because a slot is only overwritten *after*
`COUNT[k] = 0`, and a zero-count object is never decremented again (all its
predecessors are gone). The result is $O(n + m)$ time and $O(n)$ space with **no
extra allocation** — the lesson's `VecDeque` is the same FIFO spelled more
plainly. $\blacksquare$

### 6. Algorithm T is correct and detects every cycle (rating 25 · cf. 2.2.3–7)

**Problem.** Prove that Algorithm T (a) outputs a valid topological order whenever
the input relation is acyclic, and (b) outputs fewer than $n$ objects — signalling
"cycle" — whenever the relation contains a cycle.

**Hint.** For validity, ask what must be true of an object's count at the instant
it is output. For cycle detection, use a conservation argument: an object enters
the queue exactly when its count hits zero; count what can prevent that.

**Answer sketch.** *(a) Valid order.* An object $k$ is output only after
`COUNT[k]` has fallen to $0$, and `COUNT[k]` is decremented exactly once for each
predecessor $j \prec k$ **as $j$ is output**. So $k$ is output only after *all* its
predecessors have been output — precisely the topological-order condition
$j \prec k \Rightarrow j$ before $k$. Since every relation is honoured, the output
(if it contains all $n$ objects) is a genuine linear extension.

*(b) Cycle $\Rightarrow$ short output.* Every object that is output had its count
reach zero; each output object decrements its successors' counts by the number of
edges into them. If all $n$ were output, then every relation $j \prec k$ was
"used" exactly once to decrement `COUNT[k]` from its initial value down to $0$, so
the used edges form the whole relation and admit the total order just built —
impossible if a cycle exists (a cycle $c_1 \prec c_2 \prec \cdots \prec c_1$ has no
consistent linear order). Concretely, no object on a cycle can ever reach count
$0$: each waits on a predecessor also on the cycle, so the queue empties with the
cycle's objects never enqueued, and the count of outputs is $< n$. Algorithm T
reports this (returns `None`). $\blacksquare$ (The lesson's example: adding
$7 \prec 1$ to the diamond leaves object $1$ with count $1$, the queue starts
empty, nothing is output.)

### 7. ▶ Preorder and postorder variants of the stack walk (rating 20 · cf. 2.3.1–12)

**Problem.** Algorithm 2.3.1T does an **inorder** traversal with an explicit
stack (push on the way down-left, visit on pop, then go right). Modify it to
produce **preorder**, and then **postorder**.

**Hint.** Inorder visits a node *when it is popped*. Preorder differs only in
*when* the visit happens relative to the descent. Postorder is the awkward one —
a node must wait until *both* its subtrees are done; a two-stack reversal trick
sidesteps the bookkeeping.

**Answer sketch.** *Preorder (root, left, right).* Same skeleton, but **visit the
node when you push it** (during the left-descent, step T3) instead of when you pop
it. Then the root is emitted before descending its left subtree, and each right
subtree is handled when the node is later popped — giving root, left, right.

*Postorder (left, right, root).* The clean trick: run a *modified preorder* that
visits **root, then right, then left** (push left child before right so right is
processed first), collecting nodes into a list; then **reverse** the list. Since
"root, right, left" reversed is "left, right, root", the reversal is exactly
postorder. Equivalently, use two stacks: pop from stack 1 pushing each node to
stack 2 and its children (left then right) back to stack 1; stack 2, emptied at
the end, yields postorder. Each node is pushed and popped a constant number of
times, so both variants are $O(n)$. $\blacksquare$ (The lesson lists these as
exercises 2.3.1-12/13 and pins the expression-tree outputs
`+ * a - b c / d e` (pre), `a b c - * d e / +` (post).)

### 8. Stack-based postorder visits each node once (rating 22 · cf. 2.3.1–13)

**Problem.** Design a stack-based (non-recursive) postorder traversal and prove
that it visits each node exactly once, in postorder, in $O(n)$ time.

**Hint.** Take the two-stack scheme from Problem 7. Argue by a simple counting
invariant how many times each node can enter and leave each stack.

**Answer sketch.** Use two stacks. Push the root on $S_1$. Repeat until $S_1$ is
empty: pop $P$ from $S_1$, push $P$ onto $S_2$, then push $P$'s left child and
right child (if non-null) onto $S_1$. When $S_1$ empties, pop $S_2$ entirely — that
sequence is the postorder.

*Each node visited once.* Every node is pushed onto $S_1$ exactly once: the root
by initialization, every other node by its unique parent (each node has exactly
one parent, and the parent is processed once). So each node is popped from $S_1$
once and pushed onto $S_2$ once, hence popped from $S_2$ (visited) exactly once.
Total pushes/pops $\le 4n$, giving $O(n)$.

*Correct order.* Processing $S_1$ emits nodes in "root, right, left" order into
$S_2$ (right child pushed before left is popped first... arrange the child pushes
so that draining $S_2$ reverses to left-right-root). Since $S_2$ is LIFO, popping
it reverses the $S_1$-emission order, turning "root, right, left" into "left,
right, root" — postorder. $\blacksquare$

### 9. Algorithm S returns the successor for every node (rating 22 · cf. 2.3.1–21)

**Problem.** In a threaded binary tree, Algorithm S computes the inorder
successor of a node $P$: if `RLINK(P)` is a thread, return it; otherwise follow
`RLINK(P)` once and then descend left links (real ones) until a left thread stops
you. Prove S returns the correct inorder successor for **every** node, including
the last node in inorder (whose successor is the list head).

**Hint.** Split into two cases on `RTAG(P)`. Recall the definitions: a right
thread points *at* the inorder successor; the head plays "one past the end" of the
inorder sequence.

**Answer sketch.** Two cases.

*Case RTAG(P) = 1 (right link is a thread).* By definition, a right thread points
directly at $P$'s inorder successor, so returning `RLINK(P)` = $Q$ is immediately
correct. For the **last** inorder node, its right thread points at the list head —
which is exactly the designated "one past the end", so S correctly returns the
head, terminating a full traversal.

*Case RTAG(P) = 0 (real right subtree).* In inorder (left, root, right), the
successor of $P$ is the first node visited in $P$'s right subtree, which is that
subtree's **leftmost** node. Step S1 moves to $Q = $ right child; step S2 walks
`Q <- LLINK(Q)` while `LTAG(Q) = 0`, descending real left links, and stops at the
node whose left link is a *thread* — that node has no left child, so it is the
leftmost node of the right subtree, i.e. the successor. Correct.

Both cases cover every node, so S is total and correct. Iterating S from the head
reproduces the ordinary inorder walk, and across a full traversal each real left
link is descended once and each `RLINK` followed once — at most $2n + 2$
link-follows, the $O(n)$ bound the lab measures ($15$ follows for the $9$-node
expression tree). $\blacksquare$

### 10. ▶ Threaded insertion (Algorithm I) preserves threads (rating 24 · cf. 2.3.1–23)

**Problem.** Write the steps to insert a new node $N$ as the **right child** of a
node $P$ in a threaded binary tree, and prove the insertion leaves all threads
correct (every null-link-turned-thread still points to the right inorder
neighbour).

**Hint.** Before insertion, `RLINK(P)` is either a thread (to $P$'s old successor)
or a real subtree. In inorder, $N$ will sit *immediately after* $P$. Handle the
sub-case where $N$ later gains a real right subtree carefully: some node's left
thread used to aim at $P$.

**Answer sketch.** *Algorithm I (right insertion).* To make $N$ the right child of
$P$:

1. $N$ inherits $P$'s old right link and tag: `RLINK(N) <- RLINK(P)`,
   `RTAG(N) <- RTAG(P)`.
2. $N$'s left link becomes a **thread back to $P$**: `LLINK(N) <- P`,
   `LTAG(N) <- 1` (thread) — because $P$ is $N$'s inorder predecessor.
3. $P$ now points at $N$ with a real link: `RLINK(P) <- N`, `RTAG(P) <- 0`.
4. If $N$ acquired a **real** right subtree (RTAG(N) = 0, the old case where
   `RLINK(P)` was a subtree), the leftmost node of that subtree previously had a
   left thread aimed at $P$; re-aim it at $N$ (find it by Algorithm S's left
   descent and set its `LLINK` to $N$).

*Correctness.* In inorder, inserting $N$ as $P$'s right child places $N$ exactly
between $P$ and $P$'s former successor. Step 2 gives $N$ the correct inorder
predecessor thread ($P$); step 1 gives $N$ the correct successor thread when $N$
is a leaf on the right (it inherits $P$'s old successor); step 3 makes $P \to N$ a
real child link so $P$'s successor is now found *through* $N$. The only thread
that could go stale is the left thread that used to point at $P$ from $P$'s old
inorder successor — step 4 redirects it to $N$, which is now that node's inorder
predecessor. Every null link is again a thread to the correct neighbour, so the
threading invariant is preserved. $\blacksquare$ (`insert_left` is the mirror
image; the lesson's `insert_left(head, x)` appends $x$ at the *end* of the inorder
sequence.)

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
