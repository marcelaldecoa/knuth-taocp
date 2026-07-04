# Exercises — Module 07 (Searching)

Self-contained problems on this module's material — binary search and its
comparison bound, binary search trees and their random-input analysis, Hibbard
deletion, AVL height via Fibonacci trees, and the hashing analyses (linear
probing vs. uniform/double hashing). You can work every one **without the
books**: each states the problem in full, gives a **hint** to peek at when stuck,
and a worked **answer sketch** to check against after you try. Computational
answers here are reproduced by the code you write in the lab (or a few lines at a
REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise its number is noted for
readers who own Volume 3.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Hand-trace Algorithm B and count comparisons | 10 | ⬜ |
| 2 | ▶ Prove Theorem B (comparison-tree bound) | 22 | ⬜ |
| 3 | An inorder traversal of a BST is sorted | 20 | ⬜ |
| 4 | ▶ Random-BST average search cost $2(1+1/n)H_n - 3$ | 26 | ⬜ |
| 5 | Hibbard deletion is not randomness-preserving | 30 | ⬜ |
| 6 | ▶ AVL height bound via Fibonacci trees; tightness | 25 | ⬜ |
| 7 | Linear-probing successful-search analysis | 22 | ⬜ |
| 8 | ▶ Double vs. uniform hashing; secondary clustering | 28 | ⬜ |

## Problems

### 1. Hand-trace Algorithm B and count comparisons (rating 10 · cf. 6.2.1-1)

**Problem.** The sixteen keys, sorted, are
$$61\ 87\ 154\ 170\ 275\ 426\ 503\ 509\ 512\ 612\ 653\ 677\ 703\ 765\ 897\ 908.$$
Trace Algorithm B (binary search: $l = 1$, $u = N$; repeatedly compare against
$K_i$ with $i = \lfloor (l+u)/2 \rfloor$, narrowing to the correct half) searching
for $K = 503$. List the successive $(l, u, i, K_i)$ and count the comparisons.
Then do the same for the *absent* key $400$.

**Hint.** Each pass makes one three-way comparison $K : K_i$ and moves either $u
\leftarrow i-1$ or $l \leftarrow i+1$. The search fails when $u < l$.

**Answer sketch.** For $K = 503$: pass 1 $(l,u,i) = (1,16,8)$, $K_8 = 509$,
$503 < 509$; pass 2 $(1,7,4)$, $K_4 = 170$, $503 > 170$; pass 3 $(5,7,6)$, $K_6 =
426$, $503 > 426$; pass 4 $(7,7,7)$, $K_7 = 503$, **equal** — found at index 7 in
**4 comparisons**. For $K = 400$: it compares against $509, 170, 426, 275$ (also
**4 comparisons**), then $u < l$ fires and it reports "absent" with insertion
point $5$ (four keys are $< 400$). (Verified by direct simulation: 503 → 4
comparisons at index 7; 400 → 4 comparisons, not found.) Both stay within
Theorem B's ceiling $\lfloor \lg 16 \rfloor + 1 = 5$.

### 2. ▶ Prove Theorem B (comparison-tree bound) (rating 22 · cf. 6.2.1-3)

**Problem.** Prove **Theorem B**: Algorithm B makes at most
$\lfloor \lg N \rfloor + 1$ comparisons on an array of $N$ keys, whether the
search succeeds or fails.

**Hint.** Model the algorithm as a binary comparison tree: $N$ internal nodes
(one per key, where a successful search stops) and $N+1$ external nodes (the gaps
where an absent key lands). Because Algorithm B always halves the live range, how
deep can an internal node be?

**Answer sketch.** Draw the tree of comparisons: the root compares against
$K_{\lfloor(1+N)/2\rfloor}$, its children handle the two halves, recursively. This
binary tree has $N$ internal nodes and $N+1$ external nodes. Since each pass makes
the range $[l, u]$ as even as possible, halving $u - l + 1$ every step, an
internal node at "level $t$" (root = level 0) governs a subrange of size at most
$\lfloor N/2^t \rfloor$; the deepest internal node has level $\le \lfloor \lg N
\rfloor$. A **successful** search stops at an internal node, costing at most
$\lfloor \lg N \rfloor + 1$ comparisons (levels $0$ through $\lfloor \lg N
\rfloor$). An **unsuccessful** search descends to an external node one level
deeper, but its last actual comparison happened at the internal parent, so the
count is the same bound. Hence $\le \lfloor \lg N \rfloor + 1$ either way. (Check:
$N = 16 \Rightarrow$ bound $5$, and the traces of Problem 1 used $4$.) $\blacksquare$

### 3. An inorder traversal of a BST is sorted (rating 20 · cf. 6.2.2-5)

**Problem.** A binary search tree stores keys so that for every node $P$, all
keys in $P$'s left subtree are $< \operatorname{KEY}(P)$ and all in its right
subtree are $> \operatorname{KEY}(P)$. Prove that a symmetric (inorder) traversal
— left subtree, then node, then right subtree — visits the keys in strictly
increasing order.

**Hint.** Induct on the number of nodes. Assume both subtrees' inorder outputs
are already sorted; where does $\operatorname{KEY}(P)$ fall relative to each?

**Answer sketch.** Induct on tree size $n$. *Base:* the empty tree emits nothing
(vacuously sorted); a single node emits one key. *Step:* let $P$ be the root with
left subtree $L$ and right subtree $R$, each of size $< n$. By the induction
hypothesis, inorder of $L$ is a sorted list of keys, every one $< \operatorname{KEY}(P)$
(the BST property applies to *all* of $L$, not just $P$'s child), and inorder of
$R$ is sorted with every key $> \operatorname{KEY}(P)$. The inorder traversal
concatenates: $[\text{sorted } L] \cdot \operatorname{KEY}(P) \cdot [\text{sorted } R]$.
The first block is $< \operatorname{KEY}(P) <$ the last block, and each block is
internally sorted, so the whole sequence is strictly increasing. $\blacksquare$
(This is why the lab checks correctness by asserting `inorder()` equals the sorted
key list after arbitrary insertion orders.)

### 4. ▶ Random-BST average search cost $2(1+1/n)H_n - 3$ (rating 26 · cf. 6.2.2-9)

**Problem.** Build a BST by inserting $n$ distinct keys in a uniformly random
order (all $n!$ orders equally likely). Show that the average number of
comparisons in a *successful* search is
$$C_n = 2\!\left(1 + \tfrac1n\right) H_n - 3 \approx 1.386 \lg n,$$
and explain why the same constant $1.386$ governs quicksort.

**Hint.** Let $S_n$ be the expected *internal path length* (sum of node depths).
The first inserted key is the root; it splits the other $n-1$ keys into subtrees
of sizes $k$ and $n-1-k$ with $k$ uniform on $\{0,\ldots,n-1\}$, and every node
below the root is one comparison deeper. Set up the recurrence for $S_n$ and note
it is *identical* to quicksort's comparison recurrence.

**Answer sketch.** With the root splitting the remaining keys, and $n-1$ nodes
each one level deeper,
$$S_n = (n-1) + \frac1n \sum_{k=0}^{n-1}\big(S_k + S_{n-1-k}\big)
= (n-1) + \frac2n \sum_{k=0}^{n-1} S_k.$$
This is *exactly* quicksort's recurrence (Module 06), because building a random
BST performs the same comparisons quicksort does, the root playing the pivot.
Solving (multiply by $n$, subtract the $n-1$ instance, telescope) gives
$S_n = 2(n+1)H_n - 4n$, and since the average search visits path length plus one
node, $C_n = S_n/n + 1 = 2(1 + 1/n)H_n - 3$. (Verified against the closed form:
$n = 10 \to 3.4437$, $n = 100 \to 7.4785$, with $C_n$ and $S_n/n + 1$ agreeing
exactly.) As $H_n \approx \ln n$ and $\ln n = \lg n \cdot \ln 2$,
$$C_n \approx 2\ln 2 \cdot \lg n \approx 1.386 \lg n$$
— only about 39% above the perfectly balanced $\lg n$, with no balancing logic.
The shared $1.386 = 2\ln 2$ is the quicksort ↔ BST correspondence.

### 5. Hibbard deletion is not randomness-preserving (rating 30 · cf. 6.2.2-15)

**Problem.** Hibbard deletion removes a two-child node by replacing it with its
**symmetric successor** (the minimum of its right subtree). Show that repeatedly
deleting and re-inserting keys does *not* preserve the "random BST" distribution
— so the clean $1.386 \lg n$ average of Problem 4 is not maintained under
updates. What is the qualitative effect on expected search cost?

**Hint.** The successor is *always* drawn from the **right** subtree. What
asymmetry does that build in over many deletions? Compare the distribution of
tree shapes a fixed number of random insertions produces against what you get
after a delete/insert cycle.

**Answer sketch.** The random-insertion model gives a specific, non-uniform
distribution over tree shapes — for $n = 3$ keys, the balanced tree (root the
median) occurs with probability $\tfrac13$ and each of the four lopsided shapes
with probability $\tfrac16$ (verified by enumerating all $3! = 6$ insertion
orders). Hibbard deletion breaks this: because the replacement is *always* the
right subtree's minimum, deletions systematically pull structure from the right
and leave the tree **biased leftward**, so the post-deletion shape distribution is
no longer the random-insertion one. Since the $C_n = 2(1+1/n)H_n - 3$ result of
Problem 4 depends on that exact distribution, it no longer applies after
updates. Knuth's analysis (and Eppinger's later experiments, cited in §3) show
the expected internal path length **drifts upward** under long streams of
random delete/insert pairs — toward $\Theta(\sqrt n)$ rather than staying at
$\Theta(\lg n)$ — the asymmetry accumulating over many operations. The practical
fixes are to alternate predecessor/successor replacement (restoring symmetry) or
to switch to a self-balancing tree (Problem 6), which guarantees the bound
regardless of the update history.

### 6. ▶ AVL height bound via Fibonacci trees; tightness (rating 25 · cf. §6.2.3)

**Problem.** An AVL tree is height-balanced: at every node the two subtree heights
differ by at most $1$. Let $A_h$ be the *minimum* number of nodes in an AVL tree
of height $h$ (edges). Derive a recurrence for $A_h$, solve it in terms of
Fibonacci numbers, and conclude that an $n$-node AVL tree has height
$O(\lg n)$ — specifically $\le 1.4405 \lg(n+2) - 0.3277$. Show the bound is tight
by exhibiting the extremal trees.

**Hint.** To be as tall as possible with as few nodes as possible, make one
subtree height $h-1$ and the other $h-2$ (the largest gap the invariant allows).
That gives $A_h = A_{h-1} + A_{h-2} + 1$; add $1$ to both sides to reveal the
Fibonacci recurrence.

**Answer sketch.** The sparsest height-$h$ AVL tree has a root whose subtrees are
sparsest AVL trees of heights $h-1$ and $h-2$, so
$$A_0 = 1,\quad A_1 = 2,\quad A_h = A_{h-1} + A_{h-2} + 1.$$
Adding $1$: $A_h + 1 = (A_{h-1}+1) + (A_{h-2}+1)$, so $A_h + 1$ satisfies the
Fibonacci recurrence, giving $A_h = F_{h+3} - 1$ (with $F_1 = F_2 = 1$). (Verified:
$A_0,\dots,A_4 = 1, 2, 4, 7, 12 = F_3{-}1,\dots,F_7{-}1$.) The extremal trees are
the **Fibonacci trees** — recursively, height-$h$ tree = root with a height-$(h-1)$
and a height-$(h-2)$ Fibonacci subtree — and they *attain* $A_h$, so the bound is
tight. Since $F_k \approx \varphi^k/\sqrt5$ with $\varphi = (1+\sqrt5)/2$, inverting
$n \ge A_h = F_{h+3} - 1$ gives $h \le \log_\varphi(\sqrt5\,(n+1)) - 3 =
1.4405 \lg(n+2) - 0.3277$. (Verified: $n = 1000 \to$ bound $\approx 14.03$ so
height $\le 14$; $n = 10^4 \to$ bound $\approx 18.81$ so height $\le 18$ — matching
the lab's enforced ceilings.) An AVL tree is thus at most ~44% taller than a
perfect tree, *guaranteed*.

### 7. Linear-probing successful-search analysis (rating 22 · cf. 6.4-8)

**Problem.** In an open-addressing hash table with linear probing (on collision,
step to the adjacent slot), let $\alpha = N/M$ be the load factor. Knuth's 1962
result gives the average number of probes for a **successful** search as
$$C \approx \tfrac12\!\left(1 + \frac{1}{1 - \alpha}\right).$$
Explain the mechanism (primary clustering) that drives this formula, evaluate it
and the unsuccessful-search formula
$C' \approx \tfrac12\big(1 + 1/(1-\alpha)^2\big)$ at $\alpha = 0.5$ and
$\alpha = 0.9$, and say what the comparison reveals.

**Hint.** A successful search retraces the probe path taken when the key was
inserted, so its cost is the average insertion cost so far. Occupied runs
("clusters") lengthen at both ends because any key hashing anywhere into a run
must probe to the run's end — long runs beget longer runs.

**Answer sketch.** *Primary clustering:* linear probing places colliding keys into
contiguous runs, and a run of length $\ell$ captures *every* key hashing to any of
its $\ell$ slots, so runs grow super-linearly and a search's cost is dominated by
how long a run it lands in. Averaging insertion costs over the fill history yields
Knuth's $C \approx \tfrac12(1 + 1/(1-\alpha))$ for a hit and $C' \approx \tfrac12(1
+ 1/(1-\alpha)^2)$ for a miss. Evaluating (verified):
$$\alpha = 0.5:\ C \approx 1.5,\ C' \approx 2.5; \qquad
\alpha = 0.9:\ C \approx 5.5,\ C' \approx 50.5.$$
The reveal: the **unsuccessful** cost blows up *quadratically* in $1/(1-\alpha)$
— from $2.5$ to $50.5$ as the table fills from half to 90% — while the successful
cost only grows linearly. That runaway miss cost is the fingerprint of clustering
and the reason open-addressing tables resize well before $\alpha \to 1$. (The lab
fills a $1009$-slot table and confirms $< 2$ probes at $\alpha = 0.5$ and a climb
past $\sim 5$ at $\alpha = 0.9$.)

### 8. ▶ Double vs. uniform hashing; secondary clustering (rating 28 · cf. §6.4)

**Problem.** Double hashing resolves collisions with a key-dependent step
$c = h_2(K) = 1 + (K \bmod (M-2))$ (with $M$ prime, so $\gcd(c, M) = 1$). Explain
why this removes the *primary* clustering of linear probing, why it nonetheless
approximates the idealized **uniform hashing** model, and what "secondary
clustering" means. Compare the average probes at $\alpha = 0.9$ against linear
probing.

**Hint.** Under uniform hashing every key's probe sequence is an independent
random permutation of the slots, giving successful cost
$\frac1\alpha \ln\frac{1}{1-\alpha}$ and unsuccessful $\frac1{1-\alpha}$. Ask how
close double hashing's per-key sequence comes to "independent random," and where
it can still fall short (keys sharing a probe sequence).

**Answer sketch.** Two keys colliding at $h_1$ almost always get *different* steps
$c$, so they do not march in lockstep down the table — the runs that define
primary clustering never form. Because $\gcd(c, M) = 1$ the step sequence visits
all $M$ slots, and with two independent-ish hash values each key follows an
essentially distinct pseudo-random probe path, so double hashing tracks the
uniform-hashing averages closely. *Secondary clustering* is the residual effect:
keys that collide in *both* $h_1$ and $h_2$ (or, in weaker schemes like quadratic
probing, keys sharing only $h_1$) follow the *same* probe sequence and thus still
pile up — a milder, second-order clustering. Numerically at $\alpha = 0.9$
(verified against the uniform-hashing formulas):
$$\text{successful } \tfrac1\alpha\ln\tfrac1{1-\alpha} \approx 2.56 \text{ probes},
\qquad \text{unsuccessful } \tfrac1{1-\alpha} = 10,$$
versus linear probing's $5.5$ and $50.5$ (Problem 7) — a decisive win, especially
on the unsuccessful search where clustering hurt most. (The lab inserts the same
key set into both tables at $\alpha = 0.9$ and confirms double hashing wins.)

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
