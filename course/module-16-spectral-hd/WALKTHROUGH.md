# Walkthrough — Module 16: The Spectral Test in Higher Dimensions

Read this AFTER a stage is green — it explains how the reference solution is
built and why.

## Stage 1: `dual_basis`, `primal_basis`, `check_duality`

The two builders are deliberately written as one thought, because their
correctness is a *single* fact: U·Vᵀ = m·I. `dual_basis` walks the rows
skip-first (`iter_mut().enumerate().skip(1)`) so row 0 stays the pure
modulus row (m, 0, …, 0) while each later row carries `pow = pow*a%m` in
column 0 and a 1 on the diagonal — the reduction `% m` at every step is what
keeps entries below m, so the *pinned* values in the tests (−393225, not
−4295360521) come out. `primal_basis` runs the same power ladder but stores
a^j mod m across the first row and drops m onto each subsequent diagonal.
The idiom worth stealing is maintaining `pow` incrementally instead of
calling a `pow_mod` per entry: it is both faster and impossible to get
out of sync between the two bases.

`check_duality` is the module's correctness anchor made executable, and its
design choice is that it *never panics* — it returns `false` on a length or
shape mismatch (`v.len() != t`, any row `.len() != t`) before touching any
element, then verifies `dot(u[i], v[j]) == if i==j { m } else { 0 }` over all
t² pairs. A naive version that indexed first and asserted shapes would panic
on the deliberately-broken pairs the stage-1 test feeds it; returning a bool
is what lets the test assert `!check_duality(...)` for a perturbed entry, a
wrong m, and a dimension mismatch in one breath.

## Stage 2: `reduce_basis`

The function is Algorithm R as a sweep-to-fixpoint, and every line is one of
the three proofs in §3 made mechanical. The `loop { changed … }` structure
is the termination argument: because Σ|v_i|² strictly drops on each
transformation, a sweep that applies nothing means no pair can fire, so the
basis is at its fixpoint and we return. The single most important line is the
guard `if 2*d.abs() <= n { continue; }` — the inequality is *non-strict*, so
a transformation happens only when 2|d| > n; flip it to `<` and the tie
2|d| = n lets q = ±1 shuffle equal-norm bases forever (the idempotence test
would spin). The mirror update `u[j][k] += q*u[i][k]` sits in the *same* inner
loop as `v[i][k] -= q*v[j][k]`, with the indices crossed and the sign
flipped, because that is exactly the F = I + q·e_j e_iᵀ paired with
E = I − q·e_i e_jᵀ that makes F·Eᵀ = I; any other pairing silently breaks
U·Vᵀ = m·I and the very first stage-2 assertion catches it.

The reference reads `n = v_j·v_j` fresh inside the loop and asserts `n > 0`,
which documents the precondition (rows never collapse to zero because the
transformations are unimodular) and guards the division in `div_round`. The
subtle win over a naive "reduce the shortest pair" heuristic is that sweeping
*all* ordered pairs to a fixpoint is what embeds module 12's 2-D
Gauss–Lagrange reduction inside the higher-dimensional reduction — at t = 2
the fixpoint condition is precisely the reduced condition, so the shortest
row *is* ν₂, which the tests pin against a brute-force grid.

## Stage 3: `shortest_vector_squared`

The design keeps three concerns separate and self-contained: `determinant`
(Bareiss), `adjugate_column` (cofactors via the same determinant), and
`scan_box` (the enumeration). Crucially the function recomputes det and the
adjugate from V alone rather than assuming the caller passes U — so it is a
true shortest-vector routine for *any* basis, which is why the tests can
throw permuted, negated, identity, and shear bases at it and demand the
answer be a property of the lattice, not the basis. Bareiss is chosen over
Gaussian elimination because every division `/ prev` is provably exact (the
divisor is the previous pivot, a leading minor), so no rationals ever appear
and the whole computation stays in exact `i128`; a float determinant here
would risk a misrounded cofactor silently shrinking the box and excluding
the true minimum.

`scan_box` carries two idioms worth stealing. First, the partial sum
Σ_{l<k} x_l·v_l is threaded through the recursion (`partial` updated
incrementally as x moves), so a leaf costs O(t) instead of recomputing the
whole dot product. Second, the `any` flag plus `lo = if any { -z[k] } else { 0 }`
enforces "first nonzero coefficient is positive," halving the search by
exploiting that w and −w have equal norm — and it also cleanly excludes the
all-zero vector (the leaf only updates `best` when `any` is true). The
certificate in the doc comment is the reason the box is *sufficient*, not
merely a heuristic bound: nothing shorter than the seed can escape it.

## Stage 4: `nu_t_squared` and `mu_t`

`nu_t_squared` is ten honest lines: assert the supported range, build the
pair, `reduce_basis`, then search — with a `debug_assert!(check_duality(...))`
after reduction that turns the whole stage-1/stage-2 invariant into a runtime
tripwire during development (free in release). That assertion is the payoff
of carrying U through reduction even though only V is searched: it is a cheap
O(t²) certificate that the dozens of row operations didn't corrupt the
lattice. `mu_t` isolates the *only* floating-point in the module to the last
moment, after the integer ν² is frozen — a minimum certified by floats would
certify nothing. The `match t` Gamma table encodes the exact half-integer
values (Γ(5/2) = 3√π/4 as `0.75*PI.sqrt()`, Γ(7/2) = 15√π/8 as
`1.875*PI.sqrt()`), and the `unreachable!` arm documents that
`nu_t_squared` already rejected t outside 2..=6. The one place to be careful
is the exponent: the reference computes ν_t^t as `nusq.powf(half_t)` where
`half_t = t/2`, i.e. (ν²)^{t/2}, so both the π power and the ν power share the
same `half_t` — matching the closed forms the stage-4 tests expand
independently for t = 4, 5, 6.
