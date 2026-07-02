# Hints — Module 21

Graduated hints, surfaced by `./grade 21 --stage K --hint J`. Try the stage
first; each hint is more specific than the last.

## Stage 1: Truth tables and normal forms

1. The whole function is one integer: bit `i` of `table` is `f(i)`, where the
   integer `i` encodes the assignment `x_j = (i >> (j-1)) & 1`. Every method is
   a loop over `0 .. 2^n` doing bit operations. Remember to mask to `2^n` bits
   so `n = 6` (all 64 bits) does not overflow.
2. For `to_dnf`, emit one term per 1-row; for `to_cnf`, one clause per 0-row.
   A term/clause is a `Vec<i32>` of signed literals: for row `x`, variable `j`
   contributes `+j` if `(x >> (j-1)) & 1 == 1` else `-j`. A maxterm negates
   those literals so the clause is false exactly at that row. `from_dnf` ORs the
   product terms; `from_cnf` ANDs the clauses.
3. A literal `l` holds at `x` when `(l > 0) == ((x >> (l.abs()-1)) & 1 == 1)`;
   a product term is satisfied iff *all* its literals hold, a clause iff *some*
   literal holds. `from_closure(n, |x| terms.iter().any(|t| all_hold(t, x)))`.

## Stage 2: Boolean chains and combinational cost

1. A gate is a 4-bit nibble; the output for `(a, b)` is bit `2*a + b` of the op.
   So `apply_gate(op, a, b) = (op >> (2*(a as u8) + b as u8)) & 1 == 1`. Chain
   values are indexed `0..n` for inputs, then one per step.
2. `eval_chain`: build a `Vec<bool>` seeded with the `n` input bits of `x`, then
   push `apply_gate(step.op, values[step.left], values[step.right])` for each
   step, and return `values[output]`. `chain_computes` compares this to
   `f.eval(x)` for every `x in 0..2^n`.
3. `gate(op, l, r)` pushes a `Step`, sets `output` to the new index
   `n + steps.len() - 1`, and returns it. XOR from AND/OR/NOT:
   `t2 = OR(x1,x2); t3 = AND(x1,x2); t4 = NOTL(t3,t3); out = AND(t2,t4)`.

## Stage 3: Median, threshold, and symmetric functions

1. `majority` is a strict majority: `2 * (count of true) > len`. `threshold_at_least`
   is `(count of true) >= k`. A symmetric function depends only on
   `x.count_ones()`.
2. `symmetric_function(n, weights)` = `from_closure(n, |x| weights[popcount(x)])`
   (assert `weights.len() == n + 1`). `is_self_dual`: for every `x`, compare
   `f(x)` with `f(¬x)` where `¬x = (!x) & (2^n - 1)`; self-dual means they always
   differ.
3. `is_monotone`: for every `x` and every bit `b` that is 0 in `x`, set
   `y = x | (1 << b)` and fail if `f(x) && !f(y)`. For the Dedekind test, loop
   `table in 0 .. 2^(2^n)`, count how many `BoolFunc { n, table }` are monotone,
   and compare with `[2, 3, 6, 20, 168]`.

## Stage 4: Optimum chains for small functions

1. `C(f)` is the fewest gates over `basis`. Do NOT grow a set of "reachable
   functions" and combine pairs — that undercounts, because it treats a gate's
   two operands as jointly free (it reports 3 for majority-of-3, which truly
   needs 4). You must model *sharing*.
2. BFS over **states**, where a state is the *set of functions computed so far*
   (a `Vec<u64>`, kept sorted so equal sets compare equal). Start with the
   constants `0` and `2^n - 1` and the `n` projection tables (all cost 0). Apply
   gate `g` to two truth tables at once with bit ops:
   `r = (g&1?!a&!b:0) | (g&2?!a&b:0) | (g&4?a&!b:0) | (g&8?a&b:0)`, masked.
3. Dequeue `(state, cost)`; for each ordered pair `a, b` in the state and each
   `g` in the basis, form `t = combine(g, a, b)`. If `t` equals the target,
   return `cost + 1`. Otherwise, if `t` is new to the state, push the sorted
   `state ∪ {t}` (deduped by a `HashSet`) with `cost + 1`. Keep `n <= 3`.
