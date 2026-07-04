# Exercises — Module 11 (B-trees & Digital Searching)

Self-contained problems on this module's material — B-tree insertion and its
height bound, B-tree deletion, B\*-trees, binary tries, trie pruning, and
Patricia. You can work every one **without the books**: each states the problem
in full, gives a **hint** to peek at when stuck, and a worked **answer sketch**
to check against after you try. Computational answers here are reproduced by the
code you write in the lab (or a few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise, its number is noted for
readers who own Volume 3.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Insert into an order-7 B-tree by hand | 20 | ⬜ |
| 2 | ▶ B-tree deletion: borrow and merge | 22 | ⬜ |
| 3 | Attainability of Theorem B; the matching lower bound | M24 | ⬜ |
| 4 | ▶ B\*-trees: the 2-for-3 split and $\ge 2/3$ occupancy | 25 | ⬜ |
| 5 | Trace Algorithm T; draw the trie | 20 | ⬜ |
| 6 | Trie pruning: which nodes may be removed | 23 | ⬜ |
| 7 | ▶ Knuth's exact Patricia: back pointers, $n$ nodes | 24 | ⬜ |
| 8 | Exact average trie depth (Mellin machinery) | M28 | ⬜ |

## Problems

### 1. Insert into an order-7 B-tree by hand (rating 20 · cf. 6.2.4–1)

**Problem.** Insert the keys $1, 2, 3, \ldots, 13$ in that order into an
initially empty B-tree of order $m = 7$. A node holds at most $m - 1 = 6$ keys;
by Algorithm B it overflows when it reaches $7$ keys and then splits, the median
rising into the parent. Draw the tree after each split, give the final tree, and
state its height. Confirm the height obeys Theorem B.

**Hint.** With $m = 7$ a node overflows at $7$ keys; the split uses
$\text{mid} = \lfloor 7/2 \rfloor = 3$, so (in $1$-indexed key order) $K_1 K_2 K_3$
stay, $K_4$ rises, and $K_5 K_6 K_7$ move to a new right sibling — three keys on
each side, both $\ge t - 1 = \lceil 7/2\rceil - 1 = 3$.

**Answer sketch.** Keys $1$–$6$ fill one leaf: `[1 2 3 4 5 6]`. Inserting $7$
overflows it; the median $4$ rises, giving root `[4]` over leaves `[1 2 3]` and
`[5 6 7]`. Keys $8, 9, 10$ fill the right leaf to `[5 6 7 8 9 10]`; inserting $11$
overflows it, and its median $8$ rises to the root. Keys $12, 13$ then extend the
last leaf. The final tree is

```text
        [4  8]
       /  |   \
 [1 2 3] [5 6 7] [9 10 11 12 13]
```

height $2$. Theorem B with $t = \lceil 7/2\rceil = 4$, $n = 13$ gives
$h \le 1 + \log_4\!\big((13+1)/2\big) = 1 + \log_4 7 \approx 2.40$, so $h = 2$ is
within the bound. Note that ascending input — the pattern that degrades a plain
BST into a linked list — leaves the B-tree perfectly balanced; it only ever grew
at the root (the two splits after $7$ and after $11$).

### 2. ▶ B-tree deletion: borrow and merge (rating 22 · cf. 6.2.4–2)

**Problem.** The lab implements only insertion (overflow $\to$ split). Design
**deletion** as its mirror image. To remove a key $K$: if $K$ sits in an internal
node, first swap it with its in-order neighbour (predecessor or successor) so the
actual removal happens in a leaf; delete it there. A non-root node that drops
below the minimum $t - 1 = \lceil m/2\rceil - 1$ keys **underflows** and must be
repaired by either *borrowing* from a sibling with a key to spare or *merging*
with a minimal sibling. Describe both repairs, verify their key-counts stay legal,
and explain why all five B-tree properties survive and the tree shrinks only at
the root.

**Hint.** Underflow is the dual of overflow. If an adjacent sibling has $\ge t$
keys, rotate one through the parent (borrow); if every adjacent sibling has exactly
$t - 1$ keys, fuse the deficient node, the parent separator, and one sibling into a
single node (merge). Check that a merge never exceeds $m - 1$ keys.

**Answer sketch.** *Borrow.* The deficient node has $t - 2$ keys; a sibling has
$\ge t$. Move the parent's separator key down into the deficient node and promote
the sibling's adjacent (nearest) key up into the separator slot, moving the
corresponding child link too. The deficient node returns to $t - 1$ keys, the
sibling stays $\ge t - 1$, the parent is unchanged — done, no propagation.

*Merge.* All siblings are minimal ($t - 1$ keys). Combine the deficient node
($t - 2$ keys), the parent separator ($1$ key), and a minimal sibling ($t - 1$
keys) into one node holding $(t-2) + 1 + (t-1) = 2t - 2$ keys. This is always
$\le m - 1$: for even $m$, $t = m/2$ gives $2t - 2 = m - 2$; for odd $m$,
$t = (m+1)/2$ gives $2t - 2 = m - 1$ — legal either way. The parent loses one key
and one child; if it now underflows, recurse upward. If the *root* loses its last
key, delete it and make the merged node the new root — the **only** step that
lowers the tree, exactly mirroring how B5 is the only step that raises it. Leaf
levels stay uniform because borrowing and merging move keys sideways, never
changing any leaf's depth, so property 4 is preserved. Test the implementation
against `is_valid` from stage 2, which already checks every property.

### 3. Attainability of Theorem B; the matching lower bound (rating M24 · cf. 6.2.4–3)

**Problem.** Theorem B bounds the height above:
$h \le 1 + \log_t\!\big((n+1)/2\big)$ with $t = \lceil m/2\rceil$. Prove the
matching **lower** bound $h \ge \log_m(n+1)$ by counting the *maximum* number of
keys a height-$h$ B-tree can hold, and identify which trees attain each bound.
Evaluate both bounds for $m = 8$, $n = 10\,000$.

**Hint.** The upper-bound proof in the lesson counts the *stingiest* legal tree
(every node minimal) and gets $n \ge 2t^{h-1} - 1$. For the lower bound, count the
*fullest* tree: every node holds $m - 1$ keys and has $m$ children.

**Answer sketch.** In a completely full tree of height $h$, level $j$ has
$m^{j-1}$ nodes ($1, m, m^2, \ldots$), each carrying $m - 1$ keys, so

$$
n \le (m-1)\big(1 + m + \cdots + m^{h-1}\big) = (m-1)\cdot\frac{m^h - 1}{m - 1} = m^h - 1.
$$

Hence $n + 1 \le m^h$, i.e. $h \ge \log_m(n+1)$. Together with Theorem B,

$$
\log_m(n+1) \ \le\ h \ \le\ 1 + \log_t\!\Big(\tfrac{n+1}{2}\Big).
$$

The upper bound is attained by the minimal tree Knuth's proof exhibits (root with
$1$ key and $2$ children, every deeper node with exactly $t$ children); the lower
bound is attained by the completely full tree just counted. These are the two
extremal instances that bracket every B-tree — the same "pin down the extreme,
bound everything" move as Lamé's theorem in Module 01. For $m = 8$, $n = 10\,000$:
$\log_8(10001) \approx 4.43$ and $1 + \log_4(5000.5) \approx 7.14$, so the height
must be $5$, $6$, or $7$ — the search costs at most $7$ page reads whatever the
insertion order.

### 4. ▶ B\*-trees: the 2-for-3 split and $\ge 2/3$ occupancy (rating 25 · cf. 6.2.4–B\*)

**Problem.** A **B\*-tree** raises the minimum occupancy from $\approx 1/2$ to
$\approx 2/3$ by splitting less eagerly. When a node overflows and an adjacent
sibling is **full**, instead of a $1 \to 2$ split you perform a $2 \to 3$ split:
pool the two full siblings, the overflow key, and the parent separator, and
redistribute into *three* nodes with two new separators promoted. (When a sibling
has room, you shift a key to it and do not split at all.) Show that each of the
three resulting nodes ends up at least about $2/3$ full, versus the ordinary
B-tree's $1/2$.

**Hint.** Count the pot. Two full nodes hold $2(m-1)$ keys; add the overflow key
and the parent separator, then subtract the two keys that become new separators.
Divide what remains among three nodes.

**Answer sketch.** The two full siblings hold $2(m-1)$ keys; with the incoming
overflow key and the parent separator that is $2(m-1) + 2 = 2m$ keys governing the
span. Two of them are promoted upward as the pair of new separators, leaving
$2m - 2$ keys to distribute over three new nodes — at least $\lfloor (2m-2)/3
\rfloor$ per node. Dividing by the capacity $m - 1$,

$$
\frac{(2m-2)/3}{m-1} = \frac{2}{3},
$$

so occupancy approaches $2/3$ as $m$ grows (finite $m$ gives values like
$169/255 \approx 0.66$ at $m = 256$), against the ordinary split's guarantee of
only $\lceil m/2\rceil - 1 \approx (m-1)/2$ keys, i.e. $1/2$. The price is more
elaborate splitting logic and touching four pages instead of three; the reward is
denser pages, hence a smaller tree and fewer I/Os. The invariant "every non-root
node is $\ge 2/3$ full" is preserved by construction because a $2 \to 3$ split is
only triggered when both participants are full, and it leaves all three products
comfortably above the floor.

### 5. Trace Algorithm T; draw the trie (rating 20 · cf. 6.3–1)

**Problem.** Using Algorithm T over $4$-bit keys, most-significant-bit first,
insert the set $\{0000, 0001, 0110, 0111\}$ (that is $0, 1, 6, 7$). At depth $i$
the trie branches on bit $i$. Draw the resulting binary trie, count its nodes
(root and internal nodes included), and identify every **one-way** node (a node
with a single child). What do those one-way nodes reveal?

**Hint.** All four keys begin with bit $0 = 0$, so the root's HI side is empty and
the entire tree hangs beneath one LO edge. Keys $0000, 0001$ share the prefix
$000$; keys $0110, 0111$ share $011$.

**Answer sketch.** The trie has **$10$ nodes**. From the root, all keys take
LO (leading bit $0$) to a node testing bit $1$: its LO side leads to the pair
$\{0000, 0001\}$, its HI side to $\{0110, 0111\}$. On the LO side, bit $2 = 0$ is a
one-way corridor to a node that finally splits on bit $3$ into leaves $0000$ and
$0001$; symmetrically on the HI side, bit $2 = 1$ is a corridor to the bit-$3$
split of $0110$ and $0111$.

```text
                 (root, bit0)
                 0|
              (bit1)
             0/     \1
         (bit2=0)  (bit2=1)      <- one-way corridors
            |          |
         (bit3)     (bit3)
         0/  \1      0/  \1
      0000  0001   0110  0111
```

The **one-way** nodes — the root and the two bit-$2$ nodes — test bits on which
every key below them agrees, so they store no branching information at all: pure
overhead. That waste (up to $32$ such nodes for a shared $32$-bit prefix) is
exactly what Patricia removes by recording the *next distinguishing bit* and
jumping to it. This $4$-bit toy mirrors the $32$-bit structure the lab builds.

### 6. Trie pruning: which nodes may be removed (rating 23 · cf. 6.3–9)

**Problem.** Deletion in a binary trie runs Algorithm T backwards: unmark the
key's terminal node, then **prune**. Characterize *exactly* which nodes a single
deletion may remove, and argue why pruning is necessary — the lesson warns that
without it, insert/remove cycles "leak a $32$-node chain per key."

**Hint.** A node is removable iff it is **unmarked** (no key ends there) **and
childless**. Walk back up the just-searched path, detaching such nodes, and stop
at the first node that is either marked or still has a child.

**Answer sketch.** Pruning removes precisely the maximal suffix of the deleted
key's root-to-leaf path made of nodes that are both unmarked and, once the node
below is detached, childless. Concretely: unmark the terminal node; if it is now
childless, detach it; its parent may in turn become unmarked-and-childless, so
detach that too; continue upward and **stop at the first node that is marked**
(another key terminates there) **or still has a child** (another key branches off
there). Nodes off the path, marked nodes, and genuine branch points are never
touched. Pruning is necessary because inserting a key can create a chain of up to
$32$ fresh nodes; if deletion only unmarked the terminal node, that chain would
persist. Repeated insert/remove of the same key would then accumulate dead
structure that no longer mirrors the set's contents — and stage 3's reinsert-cycle
test is built to notice exactly this residue.

### 7. ▶ Knuth's exact Patricia: back pointers, $n$ nodes (rating 24 · cf. 6.3–P)

**Problem.** The lab builds the *compressed radix-trie* form of Patricia:
separate leaves plus $n - 1$ two-way branch nodes, $2n - 1$ nodes for $n$ keys.
Knuth's own Algorithm P is one twist cleverer — it has **no leaf nodes at all**,
storing each key inside a branch node reached by a *tagged back pointer*, so $n$
keys use exactly $n$ nodes. Describe the conversion: what each branch node stores,
how tagged links replace leaves, and why membership still needs exactly one full
key comparison.

**Hint.** $n$ leaves $+\, (n-1)$ branches $= 2n - 1$. If each branch node also
holds one key, and a child link that would have pointed *down* to a leaf is instead
*tagged* to point *back up* to the branch that stores that key, the branches alone
carry all the keys.

**Answer sketch.** In the compressed form, each of the $n - 1$ branch nodes tests
one bit index (strictly increasing down every root-to-leaf path) and has two child
links, each going either to a deeper branch or to one of the $n$ leaves that hold
the keys. Knuth's form deletes the leaves: every branch node additionally stores
one key, and whenever a child link would have descended to a leaf it is **tagged**,
meaning "follow this link back up to the key-bearing node it names" rather than
"descend." A search performs bit tests, following untagged links downward until it
crosses a tagged link, which delivers the single key to test. Because the branch
nodes only ever tested the bits that *distinguish* keys and skipped the rest
(Patricia's first idea), the descent never verified the skipped bits, so it must
still finish with **one full comparison** of the probe key against the reached key
— the same single end-of-search comparison the lesson describes for the leaf form.
Counting: $n - 1$ branch nodes plus a header hold all $n$ keys, i.e. $n$ nodes
total, independent of key length. This is a reconstruction/design exercise; the
back-pointer bookkeeping (which link is tagged, and re-splicing it on insertion)
is the delicate part Knuth works out in §6.3.

### 8. Exact average trie depth (Mellin machinery) (rating M28 · cf. 6.3–34)

**Problem.** The lesson's first-moment sketch argues the expected depth of a
random binary trie on $n$ keys is $\approx \lg n$. Turn the sketch into the exact
statement: show the expected number of the other $n - 1$ keys sharing a fixed
key's first $k$ bits is $(n-1)/2^k$, locate where that count crosses $1$, and state
the exact average depth the lesson quotes — $\lg n + 1.33\ldots$ plus a tiny
oscillating term. Explain why extracting that constant needs Mellin-transform
machinery, which is why this is a term project.

**Hint.** For uniform random keys, another key agrees with a fixed one on the
first $k$ bits with probability $2^{-k}$; the expected number that do is
$(n-1)/2^k$, which drops below $1$ once $k > \lg(n-1)$. The exact average is a
harmonic-style sum over a divide-and-conquer recurrence.

**Answer sketch.** Fix a key $x$. Each other key independently matches $x$'s first
$k$ bits with probability $2^{-k}$, so by linearity the expected number sharing an
$k$-bit prefix with $x$ is $(n-1)/2^k$ — below $1$ as soon as $k > \lg(n-1)$. A
search path is "crowded" only while other keys still share its prefix, so typical
branching ceases near depth $\lg n$: a random trie is about as shallow as a
perfectly balanced BST, with no balancing code at all. The **exact** average depth
of a random binary trie has the form

$$
\lg n + \frac{\gamma}{\ln 2} + \tfrac12 + P(\lg n),
$$

whose constant is $\approx 1.33$ and whose $P$ is a periodic fluctuation of
minuscule amplitude ($\sim 10^{-6}$) — matching the lesson's "$\lg n + 1.33\ldots$
plus a tiny oscillating term." Getting the constant and the oscillation exactly
means writing the average as a harmonic sum and evaluating it by the Mellin
transform: the poles of the transformed kernel contribute the $\lg n$ term, the
constant, and the periodic term in one stroke. That analytic machinery — Knuth's
tour de force in §6.3 — is well beyond a hand computation, which is what earns the
problem its M28 term-project rating. Treat the above as a guided road map, not a
finished derivation.

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
