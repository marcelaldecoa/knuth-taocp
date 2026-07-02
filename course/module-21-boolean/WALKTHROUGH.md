# Walkthrough — Module 21 (design commentary)

Read this *after* a stage is green. It explains why the reference
(`reference/src/m21_boolean.rs`) is shaped the way it is — the invariants, the
idioms worth stealing, and how each piece beats a naive version. You do not need
any of it to pass; it is the "compare with Knuth's answer" step made explicit.

## Stage 1: Truth tables and normal forms

The entire representation is `struct BoolFunc { n, table: u64 }`. The invariant
is that only the low `2ⁿ` bits of `table` carry meaning; every method that could
expose the high bits (`complement`, `num_minterms`) masks with `table_mask(n)`.
That single discipline is what makes `n = 6` (all 64 bits) work without a
special case — `table_mask` returns `u64::MAX` there instead of overflowing
`1 << 64`.

`from_closure` is the one place a "rule" becomes a "table": it evaluates the
closure once per row and ORs the bit in. Everything downstream reads the table,
never the closure, so cost is paid once.

The normal-form code factors out three tiny helpers — `assignment_literals`,
`literal_holds`, `term_satisfied`, `clause_satisfied` — so DNF and CNF are exact
duals in the source, not just on paper: `to_cnf` is `to_dnf` on the 0-rows with
every literal negated, and `from_cnf` is `from_dnf` with `any`/`all` swapped.
Writing them as duals makes the round-trip tests almost obviously correct. The
naive alternative — bespoke loops for each of the four conversions — invites the
off-by-one sign error that this symmetry rules out. Idiom worth stealing:
`terms.iter().any(|t| term_satisfied(t, x))` reads like the definition of DNF,
because it is.

## Stage 2: Boolean chains and combinational cost

The nibble gate encoding (`output = bit 2a+b of op`) is the design decision that
pays off twice. Here it makes `apply_gate` a one-liner and lets a chain hold
gates as plain `u8`s; in stage 4 the *same* encoding lets us apply a gate to two
64-bit truth tables with four bitwise terms. Choosing a representation that
serves both the scalar and the vectorized path is the move to notice.

`Chain` stores values implicitly: indices `0..n` are inputs, `n + k` is step
`k`. `eval_chain` rebuilds the value vector each call — `O(gates)` space, no
caching. That is deliberate: chains here are tiny, and a stateless evaluator is
easier to reason about than one that memoizes. `gate()` returns the new value
index and sets it as the output, so building a chain reads like writing the
formula top to bottom; `set_output` exists only for the rare case where you want
an *internal* value (tested explicitly). The correctness of `chain_computes` is
by exhaustion — it checks all `2ⁿ` rows — which is the only sound certificate for
a straight-line program and is cheap at this scale.

## Stage 3: Median, threshold, and symmetric functions

Every function here is a two-line reduction to `count_ones`, which is the point:
median, threshold, and symmetric functions are exactly the ones whose value
depends only on the popcount, so the compact `weights` array (`n+1` bits) is
their natural home. `symmetric_function` asserts `weights.len() == n + 1` with a
message the stage's `should_panic` test keys on — a reminder that an input
contract is part of the function.

`is_monotone` checks only single-bit raises rather than all `x ⊆ y` pairs. That
is the invariant worth internalizing: monotonicity is a *local* condition
(closure under covering the Boolean lattice one bit at a time implies the global
order), so the check is `O(2ⁿ · n)` instead of `O(4ⁿ)`. The Dedekind test then
leans on this: at `n = 4` it runs `is_monotone` on all 65536 functions in a
blink, reproducing `2, 3, 6, 20, 168`. `is_self_dual` masks the *argument* with
`2ⁿ − 1` (not the table mask) because it complements the input index, a distinct
mask from the table's — a subtlety easy to get wrong.

## Stage 4: Optimum chains for small functions

This is the stage where the obvious algorithm is wrong, and the reference is
built around that fact. The doc comment on `optimal_cost` spells out the trap:
the "frontier of reachable functions" closure `R_c = R_{c−1} ∪ {g(a,b)}`
undercounts because it assumes both operands are jointly free, and it reports
`C(majority₃) = 3` when the truth is 4.

The fix is to make the search state a *set of functions* — precisely the values
a real chain has on hand — so that appending a gate `state ∪ {g(a,b)}` models
sharing correctly. The invariant is exact: **a state is reachable at BFS depth
`c` iff some chain of `c` gates computes exactly that set of functions.** Because
BFS explores depth in order, the first state containing the target is found at
depth `C(f)`. States are canonicalized as sorted `Vec<u64>` and deduplicated in a
`HashSet`, which is what keeps the search finite despite many gate orderings
reaching the same set.

`combine_tables` is the vectorized gate from stage 2's encoding — it evaluates a
gate on all `2ⁿ` rows of two functions at once, so each BFS edge is a few machine
instructions. The `n ≤ 3` ceiling is honest engineering: 256 functions keep the
state space small, whereas `n = 4` explodes it — the same wall that sends
*industrial* exact synthesis to SAT solvers. Pairing the search with an explicit
optimal chain in the tests (the 4-gate majority, the 2-gate parity) is the
upper-meets-lower-bound proof that the pinned costs are exactly right, not merely
plausible.
