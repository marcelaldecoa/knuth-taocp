# Hints — Module 09: Backtracking and Dancing Links

## Stage 1: Basic backtrack: n queens

1. Reach for Algorithm 7.2.2B: the whole search is a tree where level `l` fixes the queen in row `l`, and the only property you test at step B3 is "does this queen attack an earlier one?" Cost is measured in tree nodes, so a good pruning test at B3 abandons whole subtrees near the root. Remember the convention: `n = 0` has exactly one (empty) solution.

2. Recurse row by row with a mutable array `a` where `a[r]` is the column chosen in row `r`. At each row try every column `0..n`, keep only the ones passing the attack test against rows already placed, descend, and let the loop's overwrite of `a[row]` serve as the implicit B5 backtrack — no explicit undo needed. Thread a `visit` closure so the same recursion serves both counting and returning the first solution.

3. The property test is the load-bearing line: two queens attack iff same column or same diagonal, i.e. `a[r] == col || (row - r) == (col.max(c) - col.min(c))` where `c = a[r]`. Structure it as `fn queens_ok(a, row, col) -> bool` looping `r in 0..row`, and in the recursion do `if row == n { count += 1; visit(a); return; }` before the column loop.

## Stage 2: Backtracking with bitwise state

1. This is Walker's method (§7.2.2): the same tree as stage 1, but the three attack constraints — columns used, "down" diagonals (constant r+c), "up" diagonals (constant r−c) — are each a set of forbidden columns, and a set fits in a machine word. The free columns become a one-instruction computation instead of a loop.

2. Carry `cols`, `diag_down`, `diag_up` as `u32` masks down the recursion (n ≤ 31). The legal columns in this row are `free = all & !(cols | diag_down | diag_up)`. When you place a queen and move to the next row the diagonals slide: down-diagonals shift left by one, up-diagonals shift right by one. Terminate when `cols == all` (every column filled) by returning 1.

3. Iterate the set bits of `free` with the ruler-function trick `bit = free & free.wrapping_neg()` then `free ^= bit`, and recurse with `go(all, cols | bit, (diag_down | bit) << 1, (diag_up | bit) >> 1)`. The `<< 1` on down and `>> 1` on up is the crux — get them backwards and the counts are wrong; check against stage 1 for n ≤ 12.

## Stage 3: Exact cover via dancing links

1. This is Algorithm 7.2.2.1X. The heart of it is the couplet: removing node `x` from a doubly linked list is two assignments that leave `x`'s own links untouched, so `x` still remembers where it belongs and restoring it is two more assignments needing no extra memory. `cover`/`uncover` must be exact inverses, which forces uncover to walk everything in the reverse order that cover walked it.

2. Use parallel `Vec<usize>` arrays `left, right, up, down, col`, plus `size` per column header and `node_option` per node. Node 0 is the root `h`; nodes `1..=num_items` are column headers linked in a circular horizontal list. `add_option` pushes one new node per item, splices each into the bottom of its column (just above the header) and into a circular horizontal row. Branch on the column of minimum `size` (MRV) so the tree stays small.

3. `cover(c)` first unlinks header `c` horizontally (`right[left[c]] = right[c]; left[right[c]] = left[c]`), then for each node `i` down column `c` and each `j` to its right, splices `j` out vertically and decrements `size[col[j]]`. `uncover(c)` reverses it: walk `i` up from `up[c]`, `j` left from `left[i]`, re-increment size and relink `down[up[j]] = j; up[down[j]] = j`, then relink the header last. In `search`: if `right[root] == root` record `partial` (mapped through `node_option`, sorted); else cover the chosen column, iterate its options covering `col[j]` for each `j` rightward, recurse, then uncover leftward.

## Stage 4: Sudoku as an exact-cover problem

1. A filled Sudoku is an exact cover of 324 items: 81 cell, 81 row-digit, 81 column-digit, 81 box-digit constraints, each covered exactly once. Every placement (r, c, d) is an option touching exactly four items. The whole point is reuse — feed it to the identical `ExactCover` engine from stage 3 and decode the answer back.

2. Lay out the item indices in four contiguous blocks so each placement maps to four fixed formulas, and handle givens by offering only the single clued digit for a filled cell (all nine digits for an empty one). Keep a side table mapping each option index back to its `(r, c, d)` so you can decode the solution. Take the first solution only via `solve_first`, and return `None` when there is none.

3. Use `cell(r,c) = 9*r + c`, `row(r,d) = 81 + 9*r + (d-1)`, `col(c,d) = 162 + 9*c + (d-1)`, `box(b,d) = 243 + 9*b + (d-1)` with `b = (r/3)*3 + c/3`. For each cell add options `ec.add_option(&[9*r+c, 81+9*r+(d-1), 162+9*c+(d-1), 243+9*b+(d-1)])` (pushing `(r,c,d)` to your side table), then `let sol = ec.solve_first()?;` and write `out[r][c] = d` for each decoded option.
