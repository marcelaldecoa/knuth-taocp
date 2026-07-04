# Module 11 — Multiway Trees and Digital Searching

> **Source:** *The Art of Computer Programming*, Vol. 3, 2nd ed., §6.2.4
> (multiway trees / B-trees) and §6.3 (digital searching: tries and Patricia).
> **Lab:** `labs/module-11-btree-trie` · **Grade it:** `./grade 11`
>
> This lesson is self-contained: you can complete the module without the book.
> If you own Vol. 3, read §6.2.4 and §6.3 first — the lesson tells you where
> to look.

Module 07 gave you comparison trees that live in RAM: BSTs, AVL trees, hash
tables. This module answers two questions those structures dodge:

1. **What if the keys don't fit in memory?** One disk seek costs as much as
   hundreds of thousands of comparisons, so the shape of the tree must be
   redesigned around the *page*, not the pointer. The answer is the
   **B-tree** — arguably the most commercially important data structure ever
   published.
2. **What if we stop comparing keys and start *indexing* on their bits?**
   Keys are not opaque tokens; they are digit strings. Branching on digits
   instead of comparisons gives the **trie**, and squeezing the waste out of
   a trie gives **Patricia** — the structure inside your kernel's IP routing
   table right now.

> **Companion exhibit — _The Tree That Holds the World_.** Watch the B-tree of
> §6.2.4 build itself in the Museum's
> [Database Simulator](https://marcelaldecoa.github.io/knuth-taocp/museum/exhibit-3.2-tree-that-holds-the-world.html):
> inject records and see pages fill, overflow, and split — the median rising
> into the parent and, when the root splits, the whole tree growing a level
> taller so every leaf stays at one depth. It reports the height as disk reads
> per lookup, the number that actually matters when the data lives on a page,
> not a pointer.

---

## 1. Why disks changed the game

Every structure in Module 07 was tuned to minimize *comparisons*, because in
RAM a comparison and a pointer-chase cost about the same. External storage
breaks that assumption violently. A random access to a disk (or even the
page-granular cost of SSDs and CPU cache lines) has two parts:

- **latency** — getting to the data: milliseconds on spinning rust,
  microseconds on SSD, ~100 ns for a RAM cache miss;
- **transfer** — once you are there, reading a *block* of hundreds or
  thousands of bytes costs barely more than reading one byte.

So the honest cost model for big data is: **you pay per node visited, and a
node may be as large as a page** (4096 bytes is typical). An AVL tree with a
million keys makes ~20 page accesses per search — 20 payments of the
dominant cost, with each page carrying a laughable 1 key of useful payload.

Turn the knob the other way: make one node hold $m - 1$ keys and $m$
children, where $m$ is chosen so a node exactly fills a page (for 8-byte
keys and 8-byte links, a 4 KB page gives $m$ in the hundreds). Now each
payment buys a fan-out of $m$, and the search visits $\log_m n$ nodes instead
of $\log_2 n$. For $n = 10^9$ and $m = 256$: **4 page reads** instead of 30.
That factor — $\log_2 m$ fewer seeks — is the entire reason B-trees exist,
and why every serious database and filesystem is built on one.

The problem Bayer and McCreight solved in 1972 (and Knuth analyzes in
§6.2.4) is keeping such a tree **balanced under insertion** without ever
moving much data. Their answer is beautifully simple: let nodes be elastic
between "half full" and "full", and when one overflows, *split it in half*.

## 2. B-trees: the definition

**Definition.** A **B-tree of order $m$** is a multiway search tree that is
either empty or satisfies:

1. every node has at most $m$ children (hence at most $m - 1$ keys);
2. every node except the root and the leaves has at least $\lceil m/2 \rceil$ children
   (hence at least $\lceil m/2 \rceil - 1$ keys);
3. the root has at least 2 children — unless it is a leaf, in which case it
   may hold as little as 1 key;
4. **all leaves appear on the same level**;
5. a node with keys $K_1 < K_2 < \cdots < K_j$ and children $P_0, P_1, \ldots, P_j$
   is a search tree: subtree $P_i$ holds exactly the keys strictly between
   $K_i$ and $K_{i+1}$ (with open ends $-\infty$ and $+\infty$ at the extremes).

Throughout, write $t = \lceil m/2 \rceil$ for the minimum fan-out. Order $m = 3$ gives
$t = 2$: nodes hold 1–2 keys — the classic **2-3 tree**. Property 4 is the
striking one: a B-tree is *perfectly height-balanced by definition*. The
insertion algorithm doesn't approximate balance, it preserves it exactly.

### Algorithm B (B-tree search and insertion)

Given a B-tree of order `m` and a key `K`, find `K` or insert it.

```text
B1. [Empty?]    If the tree is empty, create a leaf containing K as the
                root and terminate.
B2. [Descend.]  Beginning at the root: search for K among the keys of the
                current node (binary search — the node is sorted). If found,
                the search is successful (an insertion stops: duplicate).
                Otherwise, if the node is internal, move to the child P_i
                whose interval contains K and repeat B2. If it is a leaf,
                the search is unsuccessful; to insert, go on to B3.
B3. [Insert.]   Insert K into the leaf at its sorted position.
                (The leaf may now hold m keys — one too many.)
B4. [Overflow?] If the current node holds m keys K_1 < … < K_m, split it:
                with mid = ⌊m/2⌋, keys K_1..K_mid stay in place, key
                K_{mid+1} moves UP into the parent, and keys K_{mid+2}..K_m
                (with their children, if any) move into a brand-new right
                sibling whose link is inserted next to the promoted key.
                The parent has gained one key: repeat B4 there.
B5. [New root?] If the root itself split, create a new root containing only
                the promoted key, with the two halves as its children.
```

Convince yourself of the arithmetic in B4: the overfull node has $m$ keys;
after removing the median, $\lfloor m/2 \rfloor$ keys stay left and $\lceil m/2 \rceil - 1$ go right,
and **both quantities are $\ge \lceil m/2 \rceil - 1$** whether $m$ is odd or even. That is
why "split only overfull nodes" never manufactures an illegally empty node —
and why order 2 is impossible (the halves would have 0 keys; the lab
constructor rejects $m < 3$).

### Hand trace: order 3, eight keys

Insert `50 30 70 10 40 60 20 80` into an empty B-tree of order 3 (max 2 keys
per node). Work this on paper before coding — the stage-1 tests replay it.

```text
insert 50:   [50]                    insert 30:   [30 50]

insert 70:   [30 50 70] overflows -> split, median 50 rises:

                  [50]
                 /    \
             [30]      [70]

insert 10:        [50]               insert 40:  [10 30 40] overflows ->
                 /    \                          median 30 rises into root:
          [10 30]      [70]
                                                [30 50]
                                               /   |   \
                                           [10]  [40]  [70]

insert 60:      [30 50]              insert 20:     [30 50]
               /   |   \                           /    |    \
           [10]  [40]  [60 70]              [10 20]   [40]   [60 70]

insert 80:  [60 70 80] overflows -> median 70 rises -> root [30 50 70]
            overflows -> median 50 rises -> NEW ROOT:

                       [50]
                      /    \
                  [30]      [70]
                 /    \    /    \
            [10 20] [40] [60]  [80]
```

Final height: 3 levels; every leaf on level 3; every non-root node holds
$\ge 1 = \lceil 3/2 \rceil - 1$ keys. Notice *when* the tree got taller: only at the two
root splits (after 70, after 80). It never grew at the bottom.

## 3. Theorem B: the height bound, with proof

Let the **height** $h$ be the number of levels (0 for the empty tree, 1 for
a lone root leaf). Searching costs one node access per level, so $h$ *is*
the seek count — bounding it is the whole analysis.

**Theorem B (Bayer–McCreight; TAOCP §6.2.4).** A B-tree of order $m$
containing $n \ge 1$ keys has height

$$h \le 1 + \log_t\!\left( \frac{n + 1}{2} \right), \qquad t = \lceil m/2 \rceil.$$

*Proof — count the minimum number of keys a tree of height $h$ can carry.*
Make every node as empty as the definition allows and count.

- Level 1: the root — at least 1 key, hence (if internal) at least 2
  children.
- Level 2: at least 2 nodes. Every deeper node has at least $t$ children
  (property 2), so level 3 has $\ge 2t$ nodes, level 4 has $\ge 2t^2$, …, and in
  general level $j \ge 2$ has at least $2t^{j-2}$ nodes.
- Every node below the root carries at least $t - 1$ keys (property 2
  again; leaves included).

Summing over a tree of height $h$:

$$
\begin{aligned}
n &\ge 1 + (t - 1)\cdot\left( 2 + 2t + 2t^2 + \cdots + 2t^{h-2} \right) \\
  &= 1 + 2(t - 1)\cdot\frac{t^{h-1} - 1}{t - 1} \\
  &= 2\cdot t^{h-1} - 1.
\end{aligned}
$$

So $n + 1 \ge 2\cdot t^{h-1}$; take $\log_t$ and rearrange. ∎

The proof is an **extremal counting argument**: exhibit the *stingiest*
legal tree of each height and show even it must hold $2t^{h-1} - 1$ keys.
It is the same shape as Lamé's theorem in Module 01 (the slowest-shrinking
inputs to Euclid are Fibonacci pairs) — pin down the extremal instance, and
every instance is bounded. Knuth phrases the count in terms of the $n + 1$
*external* nodes (failure positions) on level $h + 1$; the arithmetic is
identical.

Plug in real numbers: $m = 256$, $n = 10^9$ gives
$h \le 1 + \log_{128}(5\cdot 10^8) \approx 5.13$ — five page reads, worst case, guaranteed,
with the root and second level invariably cached in practice. Stage 2 makes
you verify the bound mechanically at $m = 8$, $n = 10\,000$ ($h \le 7$) and in
the degenerate 2-3-tree case.

### Why B-trees need no rotations

An AVL tree restores balance by *rotating*: locally re-hanging subtrees,
with careful case analysis (Module 07). A B-tree never rotates, because of
one structural fact worth internalizing:

> **The tree grows only at the root.** A split adds a *sibling* at the same
> level, never a child at a new level; the only step that adds a level is
> B5, which pushes a new root above *everything*, moving every leaf down by
> one in lock-step.

Property 4 ("all leaves on the same level") is therefore preserved by
construction — there is nothing to rebalance, ever. Deletion (not in this
lab; see the exercises) mirrors the story: underfull nodes borrow from or
merge with a sibling, and the tree shrinks only by deleting the root.

Contrast the failure mode of a BST: sorted input drives it to a linked
list. Feed a B-tree sorted input (stage 1 and 2 do) and it hums along —
the rightmost node keeps splitting, and the height obeys Theorem B
regardless of insertion order.

## 4. Digital searching: bits instead of comparisons

§6.3 turns the entire searching problem sideways. A comparison-based search
extracts at most one bit of information per comparison — that is why
$\lg n! \approx n \lg n$ is the sorting floor (Module 06) and $\lg n$ the search
floor. But keys *are already bit strings*. Why interrogate them politely
("are you bigger?") when you can dissect them ("what is your next bit?")?

A **trie** (Fredkin, 1960; from re*trie*val — Knuth pronounces it "try")
over binary keys is a tree where the *path* spells the key: at depth `i`,
branch on bit `i`. No comparisons at all until (possibly) the very end. We
use fixed 32-bit keys, **most significant bit first**, so bit `i` of `k` is
`(k >> (31 − i)) & 1` — MSB-first keeps the trie's left-to-right order
identical to numeric order, and makes "subtree" mean "prefix range", which
is exactly the semantics IP routing needs.

### Algorithm T (trie search and insertion, binary alphabet)

```text
T1. [Initialize.] Set P <- ROOT, i <- 0.
T2. [Branch.]     Set b <- bit i of K. If LINK_b(P) = Λ: the search is
                  unsuccessful (an insertion creates a fresh node there).
T3. [Advance.]    Set P <- LINK_b(P), i <- i + 1. If i < 32, go to T2.
T4. [Done.]       All 32 bits consumed: K is present iff node P is marked.
                  (An insertion marks P; already marked = duplicate.)
```

Deletion is Algorithm T run backwards: unmark, then **prune** — walking
back up the path, detach every node that is now unmarked and childless.
Pruning matters: without it, insert/remove cycles leak a 32-node chain per
key, and the structure stops mirroring its contents.

### How deep is a trie? (The ~lg n sketch)

Worst case, 32 levels. But the *expected* depth for `n` random keys is far
better, and the argument is a first-moment computation you can do in your
head. Fix one key; the probability that another random key agrees with it
on the first $k$ bits is $2^{-k}$. So the expected number of the other
$n - 1$ keys sharing its $k$-bit prefix is $(n - 1)/2^k$ — which drops
below 1 as soon as $k > \lg n$. A key's search path only stays "crowded"
while other keys share its prefix, so typical branching stops near depth
$\lg n$: a random trie is about as deep as a *perfectly balanced* BST,
with no balancing code. Knuth's exact analysis (§6.3, one of the book's
tour-de-force passages) gives $\lg n + 1.33\ldots$ plus a tiny oscillating term
for the average depth of a random binary trie.

The catch: *adversarial* keys ignore expectation. 256 keys sharing a 24-bit
prefix (stage 3 builds exactly this) cost a 24-node one-way corridor before
the first real decision. Which brings us to Patricia.

## 5. Patricia: test only the bits that matter

Patricia — *Practical Algorithm To Retrieve Information Coded In
Alphanumeric* (Morrison 1968; Knuth's Algorithm P in §6.3) — rests on two
ideas:

1. **Skip the boring bits.** If every key in a subtree agrees on bits
   5..17, testing them one at a time is pure waste. Store, in each branch
   node, *the index of the next bit that actually distinguishes* its keys,
   and jump straight to it. One-way branching disappears.
2. **Don't spend nodes on key length.** With idea 1, every branch node has
   two real children, so a tree with $n$ leaves has exactly $n - 1$
   branches: **$2n - 1$ nodes total, independent of how long the keys
   are.** (A plain binary trie can burn 64 nodes on two keys.)

The price of skipping: a branch can *route* you but cannot *vouch* for the
bits it skipped. So a Patricia search descends "blindly" and then performs
**one full key comparison at the leaf** — a single comparison, at the end,
catching every skipped bit at once.

### Worked diagram

Pretend keys are 4 bits (the lab uses 64; the picture is the same). Insert
`1000`, `1011`, `1010` (bits indexed 0..3, MSB first):

```text
after 1000:      (leaf 1000)                     1 node

after 1011:  1000 vs 1011 first differ at bit 2
                 ┌────────┐
                 │ bit 2? │                      3 nodes
                 └─┬────┬─┘
                 0/      \1
          (leaf 1000)  (leaf 1011)

after 1010:  blind hunt for 1010 -> bit 2 of 1010 is 1 -> reaches 1011;
             1010 vs 1011 first differ at bit 3; splice a bit-3 branch
             where the hunt's path first tests a bit > 3 (the leaf):
                 ┌────────┐
                 │ bit 2? │
                 └─┬────┬─┘                      5 nodes = 2·3 − 1
                 0/      \1
          (leaf 1000)  ┌────────┐
                       │ bit 3? │
                       └─┬────┬─┘
                       0/      \1
                (leaf 1010)  (leaf 1011)
```

Note what is *not* tested anywhere: bits 0 and 1, on which all three keys
agree, and bit 3 on the left side, where only one key lives. Search for
`1001`: bit 2 is 0 → leaf `1000` → compare: $1001 \ne 1000$ → absent. The
final comparison is what catches the difference at bit 3, which no branch
ever examined.

The structural invariant that makes insertion work: **bit indices strictly
increase along every root-to-leaf path.** To insert `K`: hunt blindly to a
leaf `L`; the first differing bit `d = leading_zeros(K ⊕ L)` is where `K`
parts company with the *whole* tree (every key reachable along that path
agrees with `L` on the tested bits above `d`); re-descend and splice a new
bit-`d` branch just above the first node testing a bit `> d`. Because the
splice point is determined by the *set of keys*, not the insertion history,
**the same key set always produces the same tree shape** — stage 4 tests
insert-order independence explicitly, a property BSTs conspicuously lack.

**Honest note.** Knuth's own Algorithm P is one twist cleverer: it has *no
leaf nodes at all*. Each key is stored in a branch node and found via a
link that points back *up* the tree (a "tagged" back pointer), so $n$ keys
use exactly $n$ nodes. Reconstructing that form is exercise material in
§6.3; the lab implements the *compressed radix trie* form — separate
leaves, $n - 1$ branches — which keeps both Patricia ideas and is the form
production systems actually ship (crit-bit trees, kernel radix trees).

### Connection to radix sorting (Module 06)

A trie is *frozen radix sort*. MSD radix exchange (Algorithm 5.2.2R)
partitions keys by their leading bit, then recurses on each half exactly as
a trie branches on that bit: the recursion tree of radix exchange **is** the
trie of the keys, and the depth analyses match term for term. Patricia is,
in the same picture, radix exchange that skips partitioning steps where all
keys agree. If you traced Module 06's radix exchange, you have already
drawn tries without knowing it.

---

## 6. Why it's done this way

Design rationale, collected in one place:

- **Node = page.** A B-tree node is sized to the block device: one node,
  one I/O. The order $m$ is not a tuning whim, it is
  `page_size / (key + link size)`. Everything else — elastic occupancy,
  splitting, the height bound — follows from wanting balance *without*
  moving page-sized data around.
- **Half-full minimum.** Property 2 is the compromise between space
  ($\ge 50\%$ of every page is useful payload) and time (fan-out $\ge \lceil m/2 \rceil$
  gives the $\log_t$ in Theorem B). B*-trees (exercises) push the floor to
  2/3 at the cost of more elaborate splitting.
- **Splits instead of rotations.** Rotations re-hang subtrees; on disk that
  means writing parent pages scattered across the file, and reasoning about
  balance factors. A split touches exactly three pages (node, new sibling,
  parent), is local, and preserves "all leaves level" *by construction*.
  This locality is also why B-trees took over concurrent workloads — lock
  the three pages, done (Postgres's Lehman–Yao variant even avoids that).
- **Arena + `usize` links, `NULL = usize::MAX`.** Knuth's MIX links are
  integers into memory; a database's child links are *page numbers* —
  integers into a file. `Vec<Node>` + `usize` is the faithful model of
  both, cheaper and safer than `Rc<RefCell<…>>`, and the sentinel mirrors
  $\Lambda$ exactly. (Documented at the top of `src/lab.rs`.)
- **MSB-first bits.** Both digital structures branch on the most
  significant bit first so that in-order = numeric order and subtrees =
  prefix ranges ("all keys starting `10110…`") — the semantics longest-
  prefix routing and range scans need. LSB-first tries exist (hash-array
  mapped tries) but trade away ordering.
- **Fixed-width keys.** Real tries handle variable-length strings with
  end-markers; fixed 32/64-bit keys keep the lab focused on the branching
  logic itself, and match the router use case (IPv4/IPv6 addresses are
  fixed-width bit strings).

## 7. Stage-by-stage lab guide

Open `labs/module-11-btree-trie/src/lab.rs`. Arenas and constructors are
provided; the internals are private, so you may reorganize them freely as
long as the public names and signatures stay put. Run `./grade 11`.

### Stage 1 — `BTree`: insertion with splitting (`stage_01_btree.rs`)

Implement `insert`, `contains`, `keys_inorder` per Algorithm B. Duplicates
return `false` and change nothing. Practical notes:

- In an arena, the pleasant shape for insertion is a recursive helper
  `fn insert_at(&mut self, x: usize, key: i64) -> …` returning a small enum:
  *duplicate* / *done* / *split: here is the promoted median and the index
  of the new right sibling*. The parent then absorbs the median or, if the
  root split, `insert` wraps both halves in a fresh root (step B5).
- `Vec::binary_search` gives you both "found" and "which child" in one
  call: `Err(p)` is exactly the child index to descend into and the slot to
  insert at.
- `keys_inorder` interleaves children and keys: child 0, key 0, child 1,
  key 1, …, last child.

The tests replay the hand trace above, drive orders 3, 4, 7, 32 with LCG
keys, model-check long op sequences against `std::collections::BTreeSet`,
and hammer ascending/descending insertion (the BST-killer patterns).

### Stage 2 — invariants and the height bound (`stage_02_btree_analysis.rs`)

Implement `is_valid` and `height`.

- `is_valid` must check *everything*: key-count bounds per node (root
  exempt from the minimum), strict in-node sortedness, the separator
  intervals (recurse with `(Option<i64>, Option<i64>)` open bounds),
  `children = keys + 1` at internal nodes, and uniform leaf depth (record
  the first leaf level met; all others must equal it). Write it as if you
  distrust stage 1 — that is its job.
- `height` counts levels; since leaves are level-uniform (you just
  verified it), walking the leftmost spine suffices.

The tests checkpoint `is_valid` mid-sequence, then assert Theorem B:
$h \le 1 + \log_4(5000.5) \Rightarrow h \le 7$ at $n = 10\,000$, $m = 8$, plus the $m = 3$
degenerate stress where the height may only ever grow by one at a time —
root splits are the only elevator.

### Stage 3 — binary tries (`stage_03_trie.rs`)

Implement `insert`, `contains`, `remove`, `count` per Algorithm T over
`u32`, MSB first. `remove` must prune (see §4) — the reinsert-cycle test
notices structure that lingers only indirectly, but the model checker
compares every return value against a `HashSet` across 20 000 mixed ops,
including families of keys sharing a 24-bit prefix and keys differing only
in the last bit tested.

### Stage 4 — Patricia (`stage_04_patricia.rs`)

Implement `insert`, `contains`, `node_count` for the compressed form over
`u64`. The five insertion steps are in the stub's doc comment (blind hunt →
crit bit via `(k ^ l).leading_zeros()` → re-descend → splice). Keep the
strictly-increasing-bits invariant and the node count takes care of itself:
the tests hold you to `node_count()` $\le 2\,\text{keys} - 1$ after *every* insert, on
adversarial shared-prefix families that would cost a plain trie dozens of
nodes per key — and they verify that three wildly different insertion
orders of the same key set produce identical node counts and answers.

## 8. In the real world

**B-trees run the storage world.**

- **SQLite**: every table and every index *is* a B-tree — one file, pages
  of 4096 bytes by default, tables as B+-trees keyed by rowid (interior
  pages hold only keys; rows live in the leaves). The file format
  documentation reads like §6.2.4 with serialization details.
- **PostgreSQL**: the default index type (`CREATE INDEX` = `USING btree`)
  is a Lehman–Yao **B-link tree** on 8 KB pages — a B-tree variant whose
  extra "right-sibling" pointers let readers proceed without locks during
  splits. MySQL/InnoDB stores entire tables as clustered B+-trees.
- **LMDB** (and its descendants): a memory-mapped **copy-on-write B+-tree**
  — writers never overwrite pages, so readers need no locks at all.
- **Filesystems**: NTFS directories are B-trees; ext4 uses B+-tree-shaped
  extent trees and HTree directories; XFS indexes free space and directories
  with B+-trees; **btrfs is literally named "B-tree filesystem"**; APFS and
  HFS+ keep catalogs in B-trees. When you `ls`, a B-tree answers.

**Tries route the internet and finish your sentences.**

- **IP forwarding is longest-prefix match**: given a destination address,
  find the most specific route entry whose prefix matches — precisely a
  walk down an MSB-first binary trie where route entries mark nodes.
  Hardware routers implement compressed multibit tries in TCAM/ASICs.
- **Linux `fib_trie`**: the kernel's IPv4 routing table is a
  **level-compressed Patricia variant (LPC-trie)** — Patricia's
  skip-the-boring-bits idea plus dynamic multi-bit fan-out; the older
  `fib_hash` was replaced because trie lookup scales better with full BGP
  tables (~a million routes). The BSD kernels' historic `radix.c` routing
  code is a direct Patricia implementation with Knuth-style back pointers.
- **Autocomplete and dictionaries**: type-ahead search, spell checkers, and
  predictive keyboards store word sets in tries (often as DAWGs/compact
  tries) — "all completions of this prefix" is exactly "this subtree".
- Elsewhere: crit-bit trees (djb's name for exactly stage 4's structure),
  Judy arrays and ART (adaptive radix trees) inside modern in-memory
  databases, and Merkle-Patricia tries authenticating Ethereum's state.

## 9. Check your understanding

Answer before moving on (hints inverted at the end of each item):

1. Splitting an overfull node of $m$ keys leaves halves of $\lfloor m/2 \rfloor$ and
   $\lceil m/2 \rceil - 1$ keys. Verify both are $\ge \lceil m/2 \rceil - 1$ for odd *and* even $m$ —
   and find the $m$ where a *full* (not overfull) node's split would fail.
   *(Hint: try $m = 3$ with 2 keys: 0 and 1. That is why insertion overflows
   first, splits second.)*
2. Why can a B-tree's height change **only** at the root, and which
   invariant does that single fact preserve for free?
   *(Hint: a split adds a sibling on the same level; B5 moves every leaf
   down together.)*
3. In Theorem B's proof, where exactly do the "+1" and the "/2" in
   $\log_t((n+1)/2)$ come from?
   *(Hint: the root's minimum is 1 key = 2 subtrees, not $t$; sum the
   geometric series again.)*
4. A Patricia search never verifies the bits it skips. Why is one full
   comparison at the leaf enough for an exact membership answer — what
   would go wrong if the tree were *not* built from the actual key set?
   *(Hint: every key in the tree shares the skipped bits with the leaf you
   reached; a probe key need not.)*
5. For $n$ uniformly random 32-bit keys, about how many trie levels does a
   search really traverse, and why does the answer stop being true when an
   adversary picks the keys?
   *(Hint: expected number of keys sharing a $k$-bit prefix is $(n-1)/2^k$.)*

## 10. Exercises from the text

Ratings are Knuth's (00 immediate · 10 a minute · 20 fifteen minutes to an
hour · 30 hours · 40 term project; M = mathematical; ▶ = especially
instructive). Statements are paraphrased; check the numbering against your
printing. Log your work in `course/module-11-btree-trie/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 6.2.4-1 | 20 | Insert a given key sequence into an order-7 B-tree by hand, drawing every split. (Do it with your stage-1 code as the answer key.) |
| ▶6.2.4-2 | 22 | Design **deletion**: remove a key from a leaf (swapping with a neighbor key if it sits in an internal node), then repair underflow by *borrowing* from a rich sibling or *merging* with a poor one — the mirror image of splitting. Our lab omits this; implementing it against `is_valid` is the natural extension project. |
| 6.2.4-3 | M24 | How well can Theorem B's bound be attained? Construct trees achieving the extremes and derive the corresponding *lower* bound on height. |
| ▶6.2.4-B* | 25 | **B*-trees**: before splitting an overfull node, try shifting a key to a sibling; split two nodes into three only when both are full. Prove the $\ge 2/3$ occupancy invariant. |
| 6.3-1 | 20 | Trace Algorithm T on a small key set and draw the resulting trie. |
| 6.3-9 | 23 | Trie deletion with pruning (you built the binary case in stage 3); characterize exactly which nodes may be removed. |
| ▶6.3-P | 24 | Reconstruct Knuth's *exact* Patricia: keys stored in branch nodes, reached by tagged **back pointers**, $n$ nodes for $n$ keys. Adapt stage 4. |
| 6.3-34 | M28 | The average-depth analysis of random tries: derive the $\lg n + O(1)$ behavior of §4's sketch exactly. (Hard; Knuth deploys Mellin-transform machinery.) |

## 11. Proof techniques you practiced

- **Extremal counting** — Theorem B: build the *stingiest* legal tree of
  height $h$, count its $2t^{h-1} - 1$ keys, and every tree is bounded.
  The direct descendant of Lamé's Fibonacci argument from Module 01.
- **Invariant preservation under local surgery** — the five B-tree
  properties survive every split (checked the arithmetic in §2; `is_valid`
  in stage 2 is the executable form), and Patricia's strictly-increasing
  bit indices survive every splice. The invariant + local-step pattern is
  the same one that proved Euclid correct in Module 01.
- **First-moment (expected-value) sketch** — the $\sim\lg n$ trie depth: bound
  the expected number of prefix-sharing keys and locate where it crosses 1.
  Your first probabilistic analysis of a *structure's shape* rather than an
  algorithm's steps; it returns with hashing and randomized structures.
- **Canonical-form / order-independence argument** — Patricia's shape
  depends only on the key *set* (the splice point is determined by crit
  bits, not history); stage 4 turns that theorem into a differential test.
- **Model checking as empirical proof** — every stage cross-examines your
  structure against `BTreeSet`/`HashSet` oracles over thousands of
  operations: not a proof, but the fastest falsifier of a wrong one.

## 12. Where this leads

- **Deletion and B\*/B+-variants** — the exercises above; from there, the
  Lehman–Yao concurrent B-link tree is a short (and eye-opening) read.
- **External sorting** (§5.4) is the other half of the "disks change
  everything" story — replacement selection and polyphase merge are the
  sorting counterparts of this module's searching.
- **Module 13 (bitwise tricks & BDDs)** doubles down on treating data as
  bits: BDDs are, in a precise sense, tries over Boolean function tables
  with sharing — Patricia's compression idea taken to its logical extreme.
- The **trie ↔ radix sort** duality (§5 here, Module 06) recurs whenever a
  digital structure appears: suffix trees/arrays, kd-tries, and the
  adaptive radix trees of modern main-memory databases.
