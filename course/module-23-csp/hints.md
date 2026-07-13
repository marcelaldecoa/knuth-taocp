# Hints — Module 23: Constraint Satisfaction

Graduated nudges, gentlest first. Read the lesson section for the stage
before spending a hint. `./grade 23 -s <stage> --hint [n]`.

## Stage 1: The CSP model and basic backtracking

1. The whole model is bookkeeping: normalize domains and allowed-pair lists
   once (`sort_unstable` + `dedup`) at construction, and every later lookup
   can be a `binary_search`. Which two `assert!`s does `add_allowed`'s
   contract demand, and what must their messages contain?
2. `solve_basic` is Algorithm B wearing CSP clothes: recurse on the index of
   the next unassigned variable, try its domain values in ascending order,
   and bump the node counter on every placement *before* you test it. The
   prefix test only needs constraints whose `x` and `y` are both already
   assigned.
3. For `queens_csp`, the allowed pairs for columns `(i, j)` are exactly the
   `(a, b)` with `a != b && a.abs_diff(b) != (j - i)` — build them with two
   nested loops. `coloring_csp` is the same shape with `a != b`.

## Stage 2: Forward checking and MRV ordering

1. Keep a `live: Vec<Vec<u32>>` copy of the domains. Placing `x = a` walks
   the constraints incident to `x` and retains, in each unassigned
   neighbor's live domain, only the values compatible with `a`. What must
   happen the instant a live domain becomes empty?
2. Backtracking must *undo* the filtering. Log every removal as
   `(variable, values_removed)` while you filter; on the way out, merge each
   log entry back and re-sort that domain. That save-and-restore is the
   slow-but-honest version of §7.2.2.3's dancing cells.
3. MRV changes one line of control flow: instead of "next variable = lowest
   unassigned index", scan the unassigned variables for the smallest
   `live[v].len()`, ties to the lowest index. Sort the collected solutions
   before returning — the search order no longer emits them lexicographically.

## Stage 3: Arc consistency (AC-3)

1. Work per *directed arc*: each constraint contributes `(x → y)` and
   `(y → x)`. Revising `(tail → head)` deletes the tail values with no
   supporting head value. `Vec::retain` plus a search over the head's domain
   does it in a few lines.
2. The subtle part is the requeue: when a revision shrinks `D_v`, which arcs
   can newly fail? Exactly those whose *head* is `v` — their tails leaned on
   the deleted values. Push those back on the worklist and loop until it
   drains.
3. Return `false` the moment any domain empties. Termination is guaranteed
   because every requeue follows a strict shrink of some finite domain; the
   fixpoint you reach is unique, so don't worry about worklist order.

## Stage 4: Translating CSP to SAT

1. Number the SAT variables domain-slot by domain-slot: variable `v`'s
   `j`-th value gets the next integer, starting from 1. Keep that table —
   `var[v][j]` — because every clause you emit is spelled with it.
2. Three clause families, in the contract's order: one at-least-one clause
   per variable (the whole `var[v]` row), then the at-most-one pairs
   `(-var[v][i], -var[v][j])` for `i < j`, then for each constraint one
   conflict clause per *in-domain* pair it does **not** allow.
3. `count_models` is a truth table: for each of the `2^num_vars` bit
   patterns, a clause is satisfied when some literal evaluates true
   (`bits >> (|lit| - 1) & 1`, flipped for negative literals). Guard the
   size first — the contract's panic message must contain "truth table".
