# Hints — Module 12: The Spectral Test

## Stage 1: The lattice structure of linear congruential sequences

1. The whole test rests on one observation: the tuple that starts at `x_n` is `x_n·(1, a, a^2, ..., a^{t-1})` reduced coordinate-by-coordinate mod `m`, so the overlapping t-tuples form a lattice. A vector `u` is *dual* when `u·x ≡ 0 (mod m)` for every such point, which — because `x_n` runs over all residues — is equivalent to a congruence on the powers of `a` alone.
2. For `tuples`, don't recompute each window from scratch: generate `count + t - 1` sequence terms once, then slide a width-`t` window. For `is_dual_vector`, evaluate `u1 + a·u2 + a^2·u3 + ... ≡ 0 (mod m)` by accumulating powers of `a` mod `m` as you sweep the coordinates, and reject the all-zero `u` explicitly (it satisfies the congruence trivially).
3. Do the multiply `a*x` in `i128` before `% m`. Keep a running `pow` (a^i mod m, start 1) and `s` (partial sum mod m); each step `s = (s + (ui % m) * pow).rem_euclid(m); pow = pow * a % m;`. Coordinates of `u` can be negative, so use `rem_euclid`, and return `s == 0 && u != 0`.

## Stage 2: The two-dimensional spectral test, exactly

1. `nu_2^2` is the squared length of the shortest nonzero vector of the dual lattice `{(u1,u2) : u1 + a·u2 ≡ 0 (mod m)}`. Reach for Gauss–Lagrange reduction — the 2-D ancestor of LLL, essentially Euclid's algorithm on vectors — which the lesson proves both terminates and returns a true shortest vector.
2. Start from the basis `v1 = (m, 0)`, `v2 = (-a, 1)`. Repeatedly: make `v1` the longer vector, subtract the nearest-integer multiple of `v2` from it, and stop when reducing no longer shortens. Everything stays exact in `i128` (squared norms reach ~`m^2 < 2^62`).
3. Loop: compute norms `n1, n2`; if `n1 < n2` swap so `v1` is longer; `q = round((v1·v2)/(v2·v2))`; `v1 -= q·v2`; if `|v1|^2 >= |v2|^2` return `|v2|^2`. For the rounded quotient with positive divisor `n`, use `(2*d + n).div_euclid(2*n)`. Hand-trace `a=137, m=256` to `274` to check yourself.

## Stage 3: Short dual vectors in three dimensions

1. In three dimensions `(u2, u3)` can be chosen freely and `u1` is then pinned modulo `m`; the shortest representative is the *centered* residue with `|u1| <= m/2`. So the minimization is a search over `(u2, u3)`, and the trick is proving a finite search box is exhaustive — Hermite's bound guarantees `nu_3^2` is small.
2. Scan `(u2, u3)` over a growing box `[-B, B]^2`, always taking the centered `u1`, and keep the best squared norm. Seed `best = m^2` (the class `(0,0)` gives `(m,0,0)`, and `u = 0` is excluded). The certification: once `B^2 >= best`, any vector with `|u2| > B` or `|u3| > B` already has norm `> B^2 >= best`, so nothing outside can win — double `B` until that holds.
3. For each `u3`, maintain `r = (a·u2 + a^2·u3) mod m` incrementally as `u2` runs `-B..=B` (add `a` each step, subtract `m` on overflow). Centered residue: `u1 = if 2*r > m { m - r } else { -r }`. Skip `(u2,u3)=(0,0)`. After the sweep, `if b*b >= best { return best } else { b *= 2 }`.

## Stage 4: Figures of merit: judging real generators

1. `nu_t` alone is not comparable across moduli; the figure of merit `mu_t` normalizes it by the density of the lattice — it is the volume of the `t`-ball of radius `nu_t` divided by the determinant `m`. This is the number Knuth's Table 1 uses to rate generators (mu >= 0.1 passes, mu >= 1 is excellent).
2. `mu_2 = (area of disk of radius nu_2)/m = π·nu_2^2/m`, built directly on your stage-2 `nu2_squared`. `mu_3 = (volume of ball of radius nu_3)/m = (4/3)π·nu_3^3/m`; `nu_3 = sqrt(nu_3^2)` is the *only* place floating point enters the module.
3. `mu2(a,m) = std::f64::consts::PI * nu2_squared(a,m) as f64 / m as f64`. `mu3(a,m)`: take `nu3 = (nu3_squared_certified(a,m) as f64).sqrt()`, then `4.0/3.0 * PI * nu3.powi(3) / m as f64`. Sanity: RANDU `(65539, 2^31)` gives `mu3 ≈ 2.5e-6` (catastrophic); Park–Miller `48271` gives `mu3 ≈ 3.35` (excellent).
