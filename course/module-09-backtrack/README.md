# Module 09 — Backtracking and Dancing Links

> **Source:** *The Art of Computer Programming*, Vol. 4B (2022), §7.2.2
> (backtrack programming) and §7.2.2.1 (dancing links).
> **Lab:** `labs/module-09-backtrack` · **Grade it:** `./grade 9`
>
> Self-contained: complete it without the book.

Backtracking is how a computer explores a combinatorial space it is far too
large to list: build a candidate one choice at a time, and the instant a
partial candidate can't possibly be completed, abandon it and *back up*. This
module builds the paradigm three times — a textbook backtracker, a bit-parallel
one, and then Knuth's *dancing links*, one of the most quietly beautiful data
structures in all of computing.

---

## 1. The backtrack paradigm (§7.2.2)

We want all sequences x₁x₂…x_n where each x_l comes from a domain D_l and the
whole sequence satisfies some property. Backtracking organizes the search as a
tree: at *level* l we have committed to x₁…x_{l−1}, and we try each value of x_l
in turn. Knuth's generic **Algorithm B** (backtrack):

```text
B1. [Initialize.]  Set l <- 1.
B2. [Enter level l.] If l > n, VISIT x_1 x_2 ... x_n and go to B5.
                     Otherwise set x_l to its first candidate value.
B3. [Try x_l.]     If x_1 ... x_l is a legal partial solution, go to B4.
                   Else go to B5 (this value fails; try the next).
B4. [Advance.]     l <- l + 1; go to B2.
B5. [Backtrack.]   Set x_l to its next candidate; if none remain, decrease l.
                   If l = 0, terminate; else repeat B3.
```

The single most important idea: **cost is measured in tree nodes, not leaves.**
A good property test prunes whole subtrees near the root, so the tree explored
can be astronomically smaller than the D₁×…×D_n cartesian product. The art of
backtracking is choosing the variable order and the pruning test so the tree
stays small.

### Running example: n queens

Place n queens on an n×n board, none attacking another. One queen per row, so
x_l = the column of the queen in row l; the domain is {0,…,n−1}. The property
test: queens in rows r < l must differ from x_l in column *and* in diagonal —

    x_r ≠ x_l   and   |x_l − x_r| ≠ |l − r|.

Here is the search tree for **n = 4** (columns 0–3), pruned by the test. Only
the branches that survive to depth 2 are drawn in full:

```text
row0: q@0 ── row1: q@2 ── row2: (0,1,3 all attacked) ✗   backtrack
             q@3 ── row2: q@1 ── row3: (all attacked) ✗
row0: q@1 ── row1: q@3 ── row2: q@0 ── row3: q@2  ✓  SOLUTION (1,3,0,2)
row0: q@2 ── (mirror of q@1) ───────────────────  ✓  SOLUTION (2,0,3,1)
row0: q@3 ── (mirror of q@0) ✗
```

Two solutions — matching the known count. The full sequence of counts is
1, 1, 0, 0, 2, 10, 4, 40, 92, 352, 724, … (OEIS A000170); n = 2 and n = 3 have
none. You'll reproduce this exactly in stage 1.

### How big is the tree? (Knuth's estimator — a gem)

You often want to *predict* the running time of a backtrack search without
running it. Knuth's 1975 idea (§7.2.2): take a single **random** walk from the
root — at each node, count its children c, pick one uniformly, and multiply the
running product of the c's. The expected value of that product, over random
walks, is *exactly* the number of nodes in the tree. A handful of random probes
estimates the cost of a search that might take years to run in full. It is the
Monte Carlo method applied to a deterministic tree, and it is astonishingly
effective in practice.

---

## 2. Bitwise backtracking — Walker's method (stage 2)

For n queens the three constraints — column, "down" diagonal (constant r+c),
"up" diagonal (constant r−c) — are each a *set* of forbidden values. Represent
each set as a bitmask in a machine word. At a given row let

    cols       = columns used by earlier queens
    diag_down  = down-diagonals hitting this row
    diag_up    = up-diagonals hitting this row
    free       = all & !(cols | diag_down | diag_up)

Then `free` is exactly the set of legal columns, computed in one instruction.
Iterate its bits with the *ruler-function* trick `bit = free & (-free)` (isolate
the lowest set bit — you met this in §7.1.3 territory). When you place a queen on
`bit` and descend one row, the diagonals shift by one:

```text
go(cols, diag_down, diag_up):
    if cols == all: return 1
    free = all & !(cols | diag_down | diag_up)
    total = 0
    while free != 0:
        bit = free & (-free);  free ^= bit
        total += go(cols|bit, (diag_down|bit) << 1, (diag_up|bit) >> 1)
    return total
```

Why the shifts? A down-diagonal has constant r+c; moving to row r+1 raises every
threatened column by 1, i.e. a left shift. An up-diagonal has constant r−c; it
lowers by 1, a right shift. Same tree as Algorithm B, but each node costs a few
bitwise ops instead of a loop — enough to push n = 13, 14 into reach (73712 and
365596 solutions). This is Knuth's beloved style: the algorithm and the machine
word fit each other like a glove.

---

## 3. Dancing links — Algorithm 7.2.2.1X (stage 3)

### The exact cover problem

Given a set of *items* and a collection of *options* (each option is a subset of
items), find a subcollection of options that covers **every item exactly once**.
Sudoku, tilings, the fifteen puzzle's cousins, and countless other puzzles are
exact-cover problems in disguise. Picture it as a 0/1 matrix: columns = items,
rows = options, and we want a set of rows summing to the all-ones vector.

### Algorithm X

The recursive nondeterministic algorithm is trivial to state:

```text
X. If the matrix has no columns, the current partial solution is complete.
   Otherwise choose a column c (an item still to be covered).
   For each row r that has a 1 in column c:
       include r in the partial solution;
       for each column j with r_j = 1:  delete column j and every row
           touching it (they now conflict with r);
       recurse on the reduced matrix;
       undo the deletions (backtrack).
```

The whole game is doing those deletions and — crucially — those *undeletions*
fast. That is what dancing links delivers.

### The dance

Store the 1s of the matrix as nodes in a sparse, **circular, doubly linked
mesh**: every node has left/right neighbours (its option's other items) and
up/down neighbours (its column's other options). Each column has a *header*
node, and the headers hang off a *root* header h:

```text
        h ── C0 ── C1 ── C2 ── C3 ──┐   (header list, circular)
        │    │     │     │     │     │
        └────┴─────┴─────┴─────┴─────┘
             │     │     │
            node  node  node        (column C1 has three options)
             │     │     │
            ...   ...   ...
```

To **remove** a node x from a doubly linked list:

```text
    right[left[x]] <- right[x]
    left[right[x]] <- left[x]
```

Now here is the trick that names the method. We did *not* change `x`'s own
links. So x still remembers exactly where it belongs, and to **put it back**:

```text
    right[left[x]] <- x
    left[right[x]] <- x
```

Two assignments to remove, two to restore — and the restore needs *no extra
memory*, because the removed node held the information all along. Knuth quotes
this couplet as the heart of the technique; the links seem to *dance* as options
are covered and uncovered. `cover(c)` removes a column header and every row that
uses it; `uncover(c)` reverses it, walking the rows in the **opposite order** —
which matters, because undeletion must mirror deletion precisely (last removed,
first restored) for the pointers to line up.

### The MRV heuristic

Which column to branch on? Choosing the column with the **fewest** remaining
options (minimum remaining values, Knuth's step "choose c of minimal size")
keeps the branching factor small and can shrink the search tree by orders of
magnitude. It costs a single pass over the header list, and it is the difference
between a Sudoku solving in microseconds and one taking seconds.

### Trace: Knuth's example

Items a…g (0…6), options {c,e}, {a,d,g}, {b,c,f}, {a,d,f}, {b,g}, {d,e,g}. The
MRV rule and the dance grind out the *unique* cover {a,d,f}+{b,g}+{c,e} (option
indices 0, 3, 4). Stage 3 checks that, plus that solving twice gives the same
answer — proof that uncover restored the structure bit for bit.

---

## 4. Sudoku as exact cover (stage 4)

A filled Sudoku is exactly a set of 81 triples (row, col, digit) satisfying four
families of constraints, each of which must hold **exactly once**:

| Constraint family | Count | "covered exactly once" means |
|---|---|---|
| cell (r,c) is filled | 81 | each cell holds exactly one digit |
| row r has digit d | 81 | each digit appears once per row |
| column c has digit d | 81 | once per column |
| box b has digit d | 81 | once per 3×3 box |

That's **324 items**. Each candidate placement (r, c, d) is an **option**
touching exactly four items: cell(r,c), row(r,d), col(c,d), box(b,d). A full
grid ⇔ an exact cover. Givens are handled by only offering the one forced option
for a clued cell. Feed it to your `ExactCover`, take the first solution, decode
each chosen option back to (r, c, d), and you have solved Sudoku — with the
identical engine that solved the toy example. That reuse is the whole point of
reducing problems to exact cover.

---

## 5. Stage-by-stage lab guide

Open `labs/module-09-backtrack/src/lab.rs`.

- **Stage 1 — `count_queens_solutions`, `first_queens_solution`.** Write the
  recursive backtracker with the column/diagonal property test. Keep the B1–B5
  labels as comments. Convention: n = 0 has one (empty) solution.
- **Stage 2 — `count_queens_bitwise`.** The three-mask recursion above. Use
  `u32` masks (n ≤ 31) and `free & free.wrapping_neg()` for the low bit.
- **Stage 3 — `ExactCover`.** The big one. Recommended representation: parallel
  `Vec<usize>` arrays `left, right, up, down, col` plus `size` (per column) and
  `node_option` (which option each node belongs to). Node 0 is the root; nodes
  1..=num_items are column headers. `add_option` splices a new row into the
  bottom of each of its columns and into a circular horizontal list. Implement
  `cover`/`uncover` exactly as the two-assignment couplet dictates, then the
  recursive `search` with MRV column choice. `solve_first` is `search` with an
  early stop after one solution — you'll need it for stage 4.
- **Stage 4 — `solve_sudoku`.** Build the 324-item cover, add options (all nine
  digits for empty cells, the single given digit for clues), `solve_first`,
  decode. Return None when there's no cover.

Run `./grade 9`; stages unlock in order.

---

## 6. Check your understanding

1. Why does backtracking's cost depend on the *order* in which variables are
   assigned, even though the set of solutions doesn't?
2. In the bitwise queens code, why is it `<< 1` for the down-diagonal and
   `>> 1` for the up-diagonal, rather than the reverse?
3. Uncover walks rows bottom-to-top and columns right-to-left — the reverse of
   cover. Construct a two-column example where doing it in the *same* order as
   cover corrupts the links.
4. Sudoku's exact-cover matrix has 729 options and 324 items. Where does 729
   come from, and why is every option's row weight exactly 4?
5. Why does the MRV heuristic never *increase* the number of solutions found,
   only the speed of finding them?

## 7. Exercises from the text

Ratings: 00 immediate · 20 an hour · 30 hours · 40 term project · 50 open.
▶ = especially instructive. Log attempts in `exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 7.2.2–1 | 10 | How many nodes does the n-queens tree have for n = 4? Draw it. |
| ▶7.2.2–5 | 22 | Add the "all solutions" vs "one solution" distinction to Algorithm B; where do they differ? |
| 7.2.2.1–8 | 26 | Show that cover/uncover are exact inverses; prove the structure is restored. |
| ▶7.2.2.1–? | 30 | Extend `ExactCover` with *secondary* items (columns that may be covered at most once, not exactly once) — the "XC" variant. |
| 7.2.2.1–? | 35 | Implement Knuth's random-probe tree-size estimator and compare its prediction to the true n-queens tree for n ≤ 12. |

## 8. Where this leads

- **Secondary items and colors (XCC).** Knuth's §7.2.2.1 continues into exact
  cover with *color* constraints — the framework behind word puzzles, packing,
  and much more. Your `ExactCover` is one field away from XC.
- **Constraint propagation.** The MRV heuristic is the doorway to the
  constraint-satisfaction material heading toward Vol. 4C.
- **SAT (Module 10).** Backtracking + inference is exactly DPLL; dancing links
  and unit propagation are two faces of the same "propagate then branch" idea.
  The next module builds a SAT solver on that foundation.
