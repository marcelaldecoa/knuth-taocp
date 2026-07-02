# Module 07 — Searching

> **Source:** *The Art of Computer Programming*, Vol. 3, 2nd ed., Ch. 6:
> §6.2.1 (binary search), §6.2.2 (binary search trees), §6.2.3 (balanced
> trees), §6.4 (hashing).
> **Lab:** `labs/module-07-searching` · **Grade it:** `./grade 7`
>
> This lesson is self-contained: you can complete the module without the book.
> If you own Vol. 3, read §6.2.1–6.2.3 and §6.4 alongside it.

Searching is the other half of the "sorting and searching" universe: given a
collection of records identified by *keys*, find the record with a given key
(or prove it absent). Knuth organises the whole subject around one number —
the **average number of comparisons (or probes) per search** — and shows how
each data structure trades memory, insert cost, and search cost against it.
By the end of this module you will have built four searchers, from the humble
sorted array up to open-addressing hash tables, and reproduced the analyses
that rank them: the `floor(lg N) + 1` bound for binary search, the `1.386 lg n`
average depth of a random binary search tree, the Fibonacci-tree proof that
AVL trees stay `~1.44 lg n` tall, and Knuth's 1962 linear-probing formula.

---

## 1. The landscape

Fix a set of `N` records with distinct keys `K_1, ..., K_N`. A **search**
takes an *argument* `K` and returns the matching record, or a signal that no
record has key `K`. The cost we track is the number of key comparisons `K : K_i`
(for comparison-based methods) or table probes (for hashing).

| Structure | Search | Insert | Space | Notes |
|---|---|---|---|---|
| Unsorted array — sequential | Θ(N) | O(1) | N | §6.1 |
| Sorted array — **binary** | Θ(lg N) | Θ(N) | N | §6.2.1; insert is dear |
| **Binary search tree** | Θ(lg N) avg, Θ(N) worst | Θ(depth) | N | §6.2.2 dynamic |
| **AVL (balanced) tree** | Θ(lg N) worst | Θ(lg N) | N | §6.2.3 guaranteed |
| **Hash table** | Θ(1) avg | Θ(1) avg | ~N/α | §6.4 no order |

Two themes recur. First, *comparisons cannot go below `lg N`* for a
comparison-based search: with `N` possible answers a decision tree needs
`>= lg N` levels (§6.2.1, and the same information-theoretic bound as sorting).
Second, *hashing escapes that bound* by computing an address from the key
rather than comparing keys — at the price of losing sorted order and of
worst-case (rare) linear behaviour.

### Sequential search, for contrast

```text
S1. [Initialize.] i <- 1.
S2. [Compare.]    If K = K_i, terminate successfully.
S3. [Advance.]    i <- i + 1.
S4. [End?]        If i <= N, go to S2; else terminate unsuccessfully.
```

A successful search averages `(N+1)/2` comparisons, an unsuccessful one `N`.
Linear in `N` — fine for a handful of records, hopeless for millions. Every
structure below is a way to beat it.

---

## 2. Binary search (§6.2.1)

If the array is **sorted**, one comparison can eliminate *half* the remaining
candidates. That is the whole idea.

### Algorithm B (Binary search)

Search the sorted array `K_1 <= K_2 <= ... <= K_N` for argument `K`.

```text
B1. [Initialize.]   Set l <- 1, u <- N.
B2. [Get midpoint.] If u < l, the algorithm terminates unsuccessfully.
                    Otherwise set i <- floor((l + u)/2).
B3. [Compare.]      If K < K_i, go to B4; if K > K_i, go to B5;
                    if K = K_i, the algorithm terminates successfully.
B4. [Adjust u.]     Set u <- i - 1 and return to B2.
B5. [Adjust l.]     Set l <- i + 1 and return to B2.
```

The invariant is: *if `K` is present it lies in `K_l .. K_u`.* Each pass halves
`u - l + 1`, so the loop runs at most about `lg N` times.

**Worked trace** (Knuth's sixteen keys, searching for `K = 503`):

```
index:  1   2    3    4    5    6    7    8    9   10   11   12   13   14   15   16
key:   61  87  154  170  275  426  503  509  512  612  653  677  703  765  897  908
```

| pass | l | u | i | K_i | K : K_i |
|---|---|---|---|---|---|
| 1 | 1 | 16 | 8 | 509 | 503 < 509 → B4 |
| 2 | 1 | 7 | 4 | 170 | 503 > 170 → B5 |
| 3 | 5 | 7 | 6 | 426 | 503 > 426 → B5 |
| 4 | 7 | 7 | 7 | 503 | **equal** ✓ |

Four comparisons. Searching for the absent `400` also takes four (509, 170,
426, 275) and then `u < l` fires, reporting "not here." In the lab we enrich
the failure with an **insertion point**, exactly as Rust's
`<[T]>::binary_search` does: `Err(p)` where `p` is the number of keys `< K`, so
inserting `K` at index `p` keeps the array sorted. When the loop exits at
`u < l`, that point is `l - 1` (0-based).

### Theorem B — the comparison bound

> **Theorem B.** Algorithm B never makes more than `floor(lg N) + 1`
> comparisons, whether the search succeeds or fails.

*Proof (comparison tree).* Draw the tree of comparisons Algorithm B can make.
The root compares against `K_{floor((1+N)/2)}`; its children handle the two
halves, and so on. This is a binary tree with `N` internal nodes (one per key)
and `N + 1` external nodes (the gaps where an absent key ends up). A binary
tree with `N` internal nodes has height at most... well, Algorithm B always
splits as evenly as possible, so every internal node is on level
`<= floor(lg N)` (counting the root as level 0). A successful search stops at an
internal node, costing at most `floor(lg N) + 1` comparisons; an unsuccessful
one stops at an external node one level deeper, but its *last* comparison was at
the internal parent, so the count is the same. ∎

For `N = 16`, `floor(lg 16) + 1 = 5`; our trace used 4, and no key needs more
than 5. The average successful search costs about `lg N - 1` comparisons — you
save essentially nothing over the worst case, which is the price of the
perfectly balanced comparison tree.

The catch: keeping the array sorted under insertions costs `Θ(N)` per insert
(shift everything over). Binary search is unbeatable for *static* tables and
useless for rapidly changing ones. That tension is what trees resolve.

---

## 3. Binary search trees (§6.2.2)

A **binary search tree** (BST) stores keys in nodes so that, for every node
`P`, all keys in `P`'s left subtree are `< KEY(P)` and all keys in its right
subtree are `> KEY(P)`. The consequence you will lean on constantly:

> **A symmetric (inorder) traversal of a BST visits the keys in sorted order.**

Search is "binary search made dynamic": at each node go left or right. Insertion
searches for the key and, on falling off the tree, hangs a new leaf where the
search stopped.

### Algorithm T (Tree search and insertion)

```text
T1. [Initialize.] Set P <- ROOT.  (Empty tree: insert at the root.)
T2. [Compare.]    If K < KEY(P) go to T3; if K > KEY(P) go to T4;
                  otherwise the search is successful — a duplicate.
T3. [Move left.]  If LLINK(P) != Λ set P <- LLINK(P), go to T2; else go to T5.
T4. [Move right.] If RLINK(P) != Λ set P <- RLINK(P), go to T2; else go to T5.
T5. [Insert.]     Allocate Q; KEY(Q) <- K, LLINK(Q) <- RLINK(Q) <- Λ; hang Q
                  off P on the side the search fell off.
```

Inserting `503, 87, 512, 61, 908, 170, ...` builds a tree whose inorder is the
sorted list, no matter what order the keys arrive. We store nodes in an
**index-based arena** (a `Vec<Node>` with `usize` links, `usize::MAX` = Λ),
which is both faithful to Knuth's MIX link fields and idiomatic Rust — no
`Rc<RefCell<...>>` needed.

### How tall is a random BST? (the 1.386 lg n theorem)

The shape depends entirely on the insertion order. Worst case — keys inserted
in sorted order — the tree degenerates to a *vine* of height `N - 1`, and search
is back to sequential. But over all `N!` insertion orders (each equally likely),
the tree is short:

> **Theorem.** The average number of comparisons in a *successful* search of a
> BST built from `n` random distinct keys is
>
>     C_n = 2(1 + 1/n) H_n - 3 ≈ 1.386 lg n,
>
> where `H_n = 1 + 1/2 + ... + 1/n` is the `n`-th harmonic number.

*Proof sketch — the same recurrence as quicksort!* Let `C_n` be the average
**internal path length** (sum of depths of all nodes) divided by `n`, so `C_n`
counts the average comparisons to reach a node. The first key inserted becomes
the root; it splits the remaining `n - 1` keys into a left subtree of size `k`
and a right subtree of size `n - 1 - k`, with `k` uniform on `{0, ..., n-1}`.
Every node below the root is one comparison deeper, contributing `n - 1` extra
comparisons total, so the total internal path length `S_n` satisfies

     S_n = (n - 1) + (1/n) Σ_{k=0}^{n-1} (S_k + S_{n-1-k}).

This is *exactly* the recurrence for the average number of comparisons in
quicksort (Module 06) — and no wonder: **building a random BST does the same
comparisons quicksort does**, the root playing the role of the pivot. Solving it
(multiply by `n`, subtract consecutive instances, telescope) gives
`S_n = 2(n+1)H_n - 4n`, hence `C_n = S_n/n + 1 = 2(1 + 1/n)H_n - 3`. Since
`H_n ≈ ln n`, and `ln n = lg n · ln 2`, we get `C_n ≈ 2 ln 2 · lg n ≈ 1.386 lg n`.
∎

So a random BST costs only about 39% more comparisons than the perfectly
balanced tree (`lg n`) — for free, with no balancing logic. The danger is that
"random" is an assumption about the *input*, which an adversary (or sorted data)
can violate. That is what §6.2.3 fixes.

### Deletion (Algorithm D) and the non-randomness surprise

Deleting a node with zero or one child is trivial — splice it out. A node with
*two* children is replaced by its **symmetric successor**: the smallest key in
its right subtree (the leftmost node there), which slots in without violating
order. This is the **Hibbard deletion** scheme.

```text
D1. [RLINK null?]   If RLINK(T) = Λ, set Q <- LLINK(T); go to D4.
D2. [Find succ.]    R <- RLINK(T). If LLINK(R) = Λ, LLINK(R) <- LLINK(T),
                    Q <- R; go to D4.
D3. [Descend left.] Walk S down LLINKs from R to the successor, keeping its
                    parent; unhook S and let it inherit T's two subtrees; Q <- S.
D4. [Anchor.]       Make T's parent (or ROOT) point to Q; recycle T's slot.
```

Here is a subtlety Knuth flags (and exercise 6.2.2-15 explores): **repeated
Hibbard deletions do not preserve randomness.** Deleting always via the *right*
subtree's successor biases the tree leftward; after many random insert/delete
pairs the expected search cost drifts *up*, toward `Θ(√n)` average path length,
not the `Θ(lg n)` you started with. The fix in practice is to alternate
predecessor/successor, or to use a self-balancing tree — which is our next stop.

### The lab arena

The stage-2 stub gives you the arena and constructor; you write `insert`,
`contains`, `delete`, `inorder`, and `height`. Two Rust notes: use an **explicit
stack** for `inorder`/`height` (the tests build vines thousands deep, which
would overflow a recursive traversal), and measure **height in edges** (a single
node has height 0).

---

## 4. Balanced trees: AVL (§6.2.3)

Adelson-Velsky and Landis (1962) gave the first structure that guarantees
`O(lg n)` search *worst case*, by keeping every subtree nearly balanced.

> **Definition.** A binary tree is **height-balanced** (AVL) if, for every
> node, the heights of its left and right subtrees differ by at most 1. The
> node's **balance factor** is `B(P) = height(right) − height(left) ∈ {−1, 0, +1}`.

### Why balance implies `~1.44 lg n` height — the Fibonacci tree

How tall can an AVL tree of `n` nodes be? Equivalently: what is the *fewest*
nodes `A_h` in an AVL tree of height `h` (edges)? To be as tall as possible with
few nodes, make one subtree height `h−1`, the other `h−2` (the max difference
the invariant allows):

     A_0 = 1,  A_1 = 2,  A_h = A_{h-1} + A_{h-2} + 1.

Add 1 to both sides: `A_h + 1 = (A_{h-1}+1) + (A_{h-2}+1)`. So `A_h + 1` obeys
the Fibonacci recurrence, and one checks `A_h = F_{h+3} − 1`, where
`F_1 = F_2 = 1`. The sparsest AVL trees are **Fibonacci trees**:

```text
 h=1 (A=2)      h=2 (A=4)          h=3 (A=7)
    b             c                    d
   /             / \                  / \
  a             b   e (single       c   f
               /     leaf)         /|   |
              a                   b e   g
                                 /
                                a
```

Since `F_k ≈ φ^k / √5` with `φ = (1+√5)/2`, we invert `n >= A_h = F_{h+3} − 1`:

     h + 3 <= log_φ(√5 (n + 1)),   so   h <= 1.4404 lg(n+1) + const.

Knuth's clean statement:

> **Theorem.** An AVL tree with `n` internal nodes has height at most
> `1.4405 lg(n + 2) − 0.3277`.

That is only ~44% worse than a perfect tree (`lg n`), *guaranteed* — never the
random-BST hope, always the fact. For `n = 1000`, the bound is under 14; for
`n = 10^4`, under 19. The lab tests enforce exactly these.

### Keeping balance: the four rotations

An insertion can push one balance factor to `±2`. We repair it at `S`, the
**deepest ancestor of the new leaf whose balance factor was already nonzero**
(above `S`, heights are unchanged; below `S`, everything was balanced). There
are four cases, mirror-images in pairs. Let `a = +1` if the insertion went into
`S`'s right subtree, `−1` if left, and let `R = LINK(a, S)` be the child on that
side.

**Case RR (left-heavy on the right of the right — single left rotation).**
Insert `1, 2, 3`:

```text
   1                         2
    \                       / \
     2      rotate left    1   3
      \    ───────────►
       3
   B(1)=+2 (illegal)      balanced, height 1
```

Single rotation A8: `R` rises, `S` becomes `R`'s left child, `R`'s old left
subtree hangs on `S`'s right. Both balance factors reset to 0.

```text
        S                      R
       / \                    / \
      α   R      LEFT        S   γ
         / \    ──────►     / \
        β   γ              α   β
   (α,β,γ subtrees; β moves from R's left to S's right)
```

**Case LL (mirror image — single right rotation).** Insert `3, 2, 1`:

```text
     3                        2
    /                        / \
   2      rotate right      1   3
  /      ────────────►
 1
```

**Case LR (double rotation, left-then-right).** Insert `3, 1, 2` — the new key
zig-zags, so a single rotation won't fix it. Rotate the *lower* pair first, then
the upper:

```text
     3            3                2
    /            /                / \
   1     ──►    2      ──►       1   3
    \          /
     2        1
   step 1: left-rotate (1,2)   step 2: right-rotate (3,2)
```

The general A9 double rotation promotes `P = LINK(−a, R)` (the "middle"
grandchild) two levels; its two subtrees split between `S` and `R`, and the new
balance factors depend on `B(P)` before the rotation:
`(B(S), B(R)) = (−a, 0)` if `B(P) = a`, `(0, 0)` if `B(P) = 0`, `(0, a)` if
`B(P) = −a`; then `B(P) <- 0`.

```text
        S                          P
       / \                       /   \
      α   R        double       S     R
         / \      ──────►      / \   / \
        P   δ                 α  β  γ   δ
       / \
      β   γ         (P rises; β→S.right, γ→R.left)
```

**Case RL (mirror of LR, right-then-left).** Insert `1, 3, 2`.

### Algorithm A (Balanced tree search and insertion)

```text
A1. [Initialize.] T <- Λ (parent of S), S <- P <- ROOT.
A2. [Compare.]    K < KEY(P) → A3; K > KEY(P) → A4; else duplicate.
A3. [Move left.]  Q <- LLINK(P). If Λ: allocate, link, go A5. If B(Q)!=0:
                  T <- P, S <- Q. P <- Q, go A2.
A4. [Move right.] Mirror of A3 with RLINK.
A5. [Insert.]     New node Q: key K, null links, B(Q) <- 0.
A6. [Adjust.]     a <- sign(K − KEY(S)); walk from LINK(a,S) to Q setting each
                  balance factor along the path (all were 0). Remember R.
A7. [Balance.]    B(S)=0  → B(S)<-a, tree grew, done.
                  B(S)=−a → B(S)<-0, tree balanced, done.
                  B(S)=a  → rotate: A8 if B(R)=a (single), A9 if B(R)=−a (double).
A8. [Single.]     as above; B(S)<-B(R)<-0.        A10.
A9. [Double.]     as above.                        A10.
A10.[Finish.]     New subtree root replaces S under T (or becomes ROOT).
```

The beauty of Algorithm A is that it does **one** rotation (single or double)
per insertion and touches only balance factors on the search path — `O(lg n)`
work, no recursion. The lab's `is_balanced` recomputes every subtree height
*from scratch* and checks each stored `bal` against it, so a wrong rotation or a
stale balance factor is caught immediately.

---

## 5. Hashing (§6.4)

Comparison-based search is stuck at `Ω(lg n)`. **Hashing** sidesteps the bound:
compute a table address `h(K)` directly from the key, and look there. With a
good hash function and a table not too full, a search touches `O(1)` slots on
average — independent of `n`.

### The division method

Knuth's workhorse hash for integer keys:

     h(K) = K mod M,    M a prime.

Why prime? A composite `M` inherits the arithmetic structure of its factors.
If `M` is even, `h(K)` has the parity of `K` — half the table is unreachable
from even keys. If `M = 2^p`, `h(K)` is just the low `p` bits of `K`, ignoring
everything else. Worst of all, keys often arrive in arithmetic progressions
(record numbers, addresses spaced by a struct size `s`); if `gcd(s, M) = d > 1`,
those keys collide onto only `M/d` slots. A **prime** `M` (not close to a power
of 2) shares no factor with typical strides, spreading keys evenly. Pick `M`
prime and comfortably larger than the number of keys.

### Collisions and open addressing

Two keys can hash to the same slot — a **collision**. With **open addressing**
we resolve it by probing a sequence of slots until we find the key or an empty
slot. The **load factor** `α = N/M` (fraction full) governs everything.

### Algorithm L (Linear probing)

The simplest probe sequence: on collision, step to the adjacent slot.

```text
L1. [Hash.]            i <- h(K) = K mod M.
L2. [Compare.]         If TABLE[i] empty, go to L4. If it holds K, found
                       (here: duplicate).
L3. [Advance.]         i <- i − 1; if i < 0, i <- i + M. Back to L2.
L4. [Insert.]          If N = M − 1, overflow. Else N <- N + 1, TABLE[i] <- K.
```

(We keep one slot always empty, `N <= M − 1`, so an unsuccessful search is
guaranteed to hit an empty slot and stop.) **Worked example**, `M = 7`,
`h(K) = K mod 7`:

```text
insert 12: 12 mod 7 = 5           slot 5
insert 19: 19 mod 7 = 5 (taken)   → slot 4
insert  5:  5 mod 7 = 5,4 (taken) → slot 3

 slot:  0    1    2    3    4    5    6
 key:   .    .    .    5   19   12    .
probes: 12→1,  19→2,  5→3
```

### Knuth's 1962 linear-probing analysis

Linear probing suffers from **primary clustering**: an occupied run tends to
grow at both ends, because any key hashing anywhere into the run must probe to
its end. Long runs beget longer runs. Knuth's celebrated 1962 result quantifies
the average number of probes:

> **Successful search:**   `C ≈ (1/2)(1 + 1/(1 − α))`.
> **Unsuccessful search:** `C' ≈ (1/2)(1 + 1/(1 − α)^2)`.

At `α = 0.5`: successful `≈ 1.5` probes, unsuccessful `≈ 2.5`. At `α = 0.9`:
successful `≈ 5.5`, unsuccessful `≈ 50.5`(!). The unsuccessful cost blows up
quadratically as `α → 1` — the signature of clustering. This is the analysis the
lab measures empirically: fill a 1009-slot table to `α = 0.5` and confirm the
average successful search is under 2 probes; fill to `α = 0.9` and watch it climb
past 5.

### Algorithm D (Double hashing)

To break up clusters, make the *step size* depend on the key too:

     h1(K) = K mod M,    c = h2(K) = 1 + (K mod (M − 2)),    probe i <- i − c (mod M).

Since `M` is prime and `1 <= c <= M − 2`, we have `gcd(c, M) = 1`, so the probe
sequence visits **all** `M` slots before repeating. Two keys with the same
`h1` almost always have different steps `c`, so they don't march in lockstep:
double hashing eliminates primary clustering. (It still has mild **secondary
clustering** in some variants, but with two hash functions distinct keys follow
distinct sequences.) It behaves almost like the idealised **uniform hashing**
model, whose averages are

> **Successful:**   `(1/α) ln(1/(1 − α))`.
> **Unsuccessful:** `1/(1 − α)`.

At `α = 0.9`: successful `≈ 2.56` probes vs. linear probing's `5.5`, and
unsuccessful `≈ 10` vs. `50`. The lab inserts the *same* key set into both
tables at `α = 0.9` and confirms double hashing wins decisively.

```text
double hashing, M = 7, c = 1 + (K mod 5):
insert 12:                       slot 5
insert 19: slot 5 taken, c=1+4=5, 5−5 = 0  → slot 0   (not slot 4!)
insert  5: slot 5 taken, c=1+0=1, 5−1 = 4  → slot 4
```

Same three keys as linear probing, but 19 lands far away at slot 0 instead of
adjacent slot 4 — the cluster never forms.

---

## 6. Stage-by-stage lab guide

Open `labs/module-07-searching/src/lab.rs`; run `./grade 7`. Stages run in
order, stopping at the first failure.

### Stage 1 — `binary_search`, `binary_search_comparisons` (Algorithm 6.2.1B)

Implement Algorithm B with Knuth's 1-based indices (so `K_i` is `a[i-1]`).
Return `Ok(i)` on a hit (any matching index if there are duplicates) and
`Err(p)` on a miss, where `p = l − 1` is the sorted insertion point. The
instrumented version counts executions of step B3. Tests check membership and
insertion points against a linear scan over 100+ element arrays, empty/singleton
cases, duplicate membership, and Theorem B's `floor(lg N) + 1` bound across many
sizes (tight on `N = 2^k − 1`).

### Stage 2 — `Bst` (Algorithms 6.2.2T, 6.2.2D)

Fill in `insert`, `contains`, `delete`, `inorder`, `height` on the arena.
Deletion is Hibbard's symmetric-successor scheme — get all four shapes right
(leaf, one child either side, two children, and the root). Use an **explicit
stack** for traversals. Tests: inorder sorted after LCG inserts, a 20 000-op
insert/contains/delete stream matched against a `BTreeSet`, all deletion cases,
`height == n − 1` for sorted input, and `height < 5 lg n` for 10 000 random keys.

### Stage 3 — `AvlTree` (Algorithm 6.2.3A)

The hardest stage: balance factors and the four rotations. The `LINK(a, ·)`
trick collapses the mirror-image cases into one code path parameterised by
`a = ±1`. Implement `is_balanced` by recomputing heights from scratch — do not
trust the stored `bal` fields. Tests: the four rotation shapes on tiny
sequences, ascending `1..=1000` and descending with `height <= 14`, 10 000
random keys `height <= 18`, duplicate rejection, and a `BTreeSet` model.

### Stage 4 — `LinearProbe`, `DoubleHash` (Algorithms 6.4L, 6.4D)

Division hashing `h(K) = K mod M`, decreasing probe sequences, full at
`N = M − 1`. `probes_for` counts slots examined including the final one. Tests
anchor on the `M = 7` worked examples, then verify no false positives/negatives
against a `HashSet` at `α = 0.5` and `0.9`, and check the *analysis*: linear
probing averages under 2 probes at `α = 0.5`, climbs past 2.5 at `α = 0.9`, and
double hashing beats linear probing on the same key set at `α = 0.9`.

---

## 7. Check your understanding

1. Why does binary search need the array **sorted**, and what breaks if it isn't?
   (The invariant "K is in `K_l..K_u`" — a wrong branch discards the wrong half.)
2. A BST built from already-sorted input has what height, and what does search
   cost then? (`n − 1`; `Θ(n)` — no better than sequential.)
3. Where does the `1.386` in the random-BST average come from? (`2 ln 2`,
   because the internal-path recurrence is quicksort's.)
4. An AVL tree has 100 nodes. Can its height be 10? 20? (No to both: the max is
   `1.4405 lg 102 − 0.328 ≈ 9.3`, so height `<= 9`; 10 and 20 are impossible.)
5. Why must a hash table's modulus `M` be prime (and not near a power of 2)?
6. At `α = 0.9`, why is linear probing's *unsuccessful* search (~50 probes) so
   much worse than double hashing's (~10)? (Primary clustering: runs grow
   quadratically in `1/(1−α)`.)

## 8. Exercises from the text

Ratings use Knuth's scale: 00 immediate · 10 a minute · 20 fifteen minutes to an
hour · 30 hours · 40 term project · 50 open research. ▶ marks especially
instructive exercises. Log attempts in
`course/module-07-searching/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 6.2.1-1 | 10 | Trace Algorithm B searching Knuth's 16 keys for a given argument; count comparisons. |
| ▶6.2.1-3 | 22 | Prove Theorem B: at most `floor(lg N) + 1` comparisons, successful or not. |
| 6.2.2-5 | 20 | Show an inorder traversal of a BST yields the keys in increasing order. |
| ▶6.2.2-9 | 26 | Derive the average successful-search cost `2(1 + 1/n)H_n − 3` for a random BST. |
| 6.2.2-15 | 30 | Show that repeated Hibbard deletion+insertion degrades expected search cost. |
| ▶6.2.3-? | 25 | Prove the AVL height bound via Fibonacci trees; show it is tight. |
| 6.4-8 | 22 | Analyse linear probing: derive `(1/2)(1 + 1/(1−α))` for successful search. |
| ▶6.4-? | 28 | Compare double hashing to uniform hashing; explain secondary clustering. |

## 9. Where this leads

- **Comparison lower bounds** (`lg N`) tie back to Module 06's sorting bound —
  same decision-tree argument, same `Ω(n lg n)` / `Ω(lg n)` walls.
- **The quicksort ↔ BST correspondence** is one of the prettiest facts in the
  book: the same recurrence, hence the same `1.386 lg n` constant, governs both.
- **Balanced trees** generalise beyond AVL to red-black trees, B-trees (§6.2.4,
  for external memory), and beyond — all descendants of Algorithm A's idea of
  rebalancing on the search path.
- **Hashing** and its analysis (the `1/(1−α)` and `ln(1/(1−α))` laws) reappear
  wherever "amortised `O(1)`" is claimed — hash maps, sets, memoisation,
  Bloom filters, and the cuckoo/Robin-Hood variants that tame the tail.
