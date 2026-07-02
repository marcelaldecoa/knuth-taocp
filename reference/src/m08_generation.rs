//! Module 08 — Combinatorial Generation.
//! Source: TAOCP Vol. 4A, §7.2.1 (Generating Basic Combinatorial Patterns):
//! Algorithm 7.2.1.1G (Gray binary), 7.2.1.2L (lexicographic permutations),
//! 7.2.1.2P (plain changes), 7.2.1.3T (combinations), 7.2.1.4P (partitions).

// ---------------------------------------------------------------------------
// Stage 1 — Gray binary code (Algorithm 7.2.1.1G)
// ---------------------------------------------------------------------------

/// Algorithm 7.2.1.1G (Gray binary generation), step-faithful.
///
/// Returns all `2^n` binary n-tuples as `u64` words (bit `j` of the word is
/// `a_j`), in Gray order: successive words differ in exactly one bit. The
/// sequence begins at `0...0` and equals the reflected binary code
/// `g(k) = k XOR floor(k/2)` for `k = 0, 1, ..., 2^n - 1`.
///
/// The algorithm keeps a parity bit `a_inf` — the parity of the number of
/// 1-bits of the current word — so that the bit to flip can be chosen without
/// any counter: flip bit 0 on odd-numbered steps, otherwise flip the bit just
/// left of the rightmost 1.
///
/// Panics if `n > 30` (the whole table of `2^n` words is materialized).
pub fn gray_code(n: u32) -> Vec<u64> {
    assert!(n <= 30, "gray_code: n must be at most 30");
    let mut out = Vec::with_capacity(1usize << n);

    // G1. [Initialize.] Set a_j <- 0 for 0 <= j < n; also set a_inf <- 0.
    let mut a: u64 = 0; // bits a_{n-1} ... a_1 a_0
    let mut a_inf: u64 = 0; // parity of the visit count

    loop {
        // G2. [Visit.] Visit the n-tuple (a_{n-1}, ..., a_1, a_0).
        out.push(a);

        // G3. [Change parity.] Set a_inf <- 1 - a_inf.
        a_inf = 1 - a_inf;

        // G4. [Choose j.] If a_inf = 1, set j <- 0. Otherwise set j to the
        //     smallest index such that a_{j-1} = 1 (one more than the
        //     position of the rightmost 1-bit).
        let j = if a_inf == 1 { 0 } else { a.trailing_zeros() + 1 };

        // G5. [Complement a_j.] Terminate if j = n; otherwise set
        //     a_j <- 1 - a_j and return to G2.
        if j == n {
            return out;
        }
        a ^= 1u64 << j;
    }
}

/// The rank of a Gray codeword: the `k` with `g(k) = k XOR (k >> 1) = word`.
///
/// Inverting `g` means solving `k_j = g_j XOR k_{j+1}` from the top bit down,
/// i.e. bit `j` of `k` is the parity of the bits of `word` at positions
/// `>= j`:
///
/// ```text
/// k = word XOR (word >> 1) XOR (word >> 2) XOR ...
/// ```
pub fn gray_rank(word: u64) -> u64 {
    let mut k = 0u64;
    let mut g = word;
    while g != 0 {
        k ^= g;
        g >>= 1;
    }
    k
}

// ---------------------------------------------------------------------------
// Stage 2 — Lexicographic permutations (Algorithm 7.2.1.2L)
// ---------------------------------------------------------------------------

/// One step of Algorithm 7.2.1.2L: transform `a` into the lexicographically
/// next permutation of its elements. Returns `false` — leaving `a`
/// **unchanged** (it is then sorted in non-increasing order) — when `a` is
/// already the last permutation; this mirrors Knuth's "terminate if j = 0".
///
/// ```text
/// L2. [Find j.]     j <- n-1; while a_j >= a_{j+1}, j <- j-1.
///                   Terminate if j = 0.
/// L3. [Increase.]   l <- n; while a_j >= a_l, l <- l-1. Swap a_j <-> a_l.
/// L4. [Reverse.]    Reverse a_{j+1} ... a_n.
/// ```
///
/// Because L2 and L3 compare with `>=` (not `>`), the algorithm is correct
/// for **multisets**: repeated elements produce each distinct arrangement
/// exactly once.
pub fn next_permutation(a: &mut [u32]) -> bool {
    let n = a.len();
    if n <= 1 {
        return false;
    }
    // L2. [Find j.] Find the largest j with a_j < a_{j+1} (1-based); the
    //     suffix a_{j+1} >= ... >= a_n is the longest non-increasing tail.
    let mut j = n - 1; // 1-based index j; compares a[j-1] with a[j]
    while a[j - 1] >= a[j] {
        j -= 1;
        if j == 0 {
            return false; // whole array non-increasing: last permutation
        }
    }
    // L3. [Increase a_j.] Find the rightmost l with a_j < a_l and swap.
    let mut l = n;
    while a[j - 1] >= a[l - 1] {
        l -= 1;
    }
    a.swap(j - 1, l - 1);
    // L4. [Reverse a_{j+1} ... a_n.] The tail is non-increasing; reversing
    //     it makes it non-decreasing — the smallest possible suffix, so the
    //     result is the immediate lexicographic successor.
    a[j..].reverse();
    true
}

/// All `n!` permutations of `1, 2, ..., n` in lexicographic order, generated
/// by Algorithm 7.2.1.2L (visit, then step until L2 terminates).
pub fn all_permutations(n: u32) -> Vec<Vec<u32>> {
    // L1. [Visit.] The first permutation is the sorted one.
    let mut a: Vec<u32> = (1..=n).collect();
    let mut out = vec![a.clone()];
    while next_permutation(&mut a) {
        out.push(a.clone());
    }
    out
}

// ---------------------------------------------------------------------------
// Stage 3 — Plain changes (Algorithm 7.2.1.2P)
// ---------------------------------------------------------------------------

/// Algorithm 7.2.1.2P (plain changes; Steinhaus–Johnson–Trotter): all `n!`
/// permutations of `1, 2, ..., n`, each obtained from its predecessor by one
/// **adjacent** transposition.
///
/// Control tables: `c_j` records, odometer-fashion, how far element `j` has
/// travelled in its current sweep (`0 <= c_j < j`), and `o_j = ±1` is its
/// current direction. The offset `s` accounts for elements larger than `j`
/// that are currently parked at the left end.
///
/// ```text
/// P1. [Initialize.]       c_j <- 0, o_j <- 1 for 1 <= j <= n.
/// P2. [Visit.]            Visit a_1 ... a_n.
/// P3. [Prepare.]          j <- n, s <- 0.
/// P4. [Ready to change?]  q <- c_j + o_j. If q < 0 go to P7;
///                         if q = j go to P6.
/// P5. [Change.]           Interchange a_{j-c_j+s} <-> a_{j-q+s};
///                         c_j <- q; return to P2.
/// P6. [Increase s.]       Terminate if j = 1; otherwise s <- s + 1.
/// P7. [Switch direction.] o_j <- -o_j, j <- j - 1, go back to P4.
/// ```
///
/// For n = 3 the sequence is 123, 132, 312, 321, 231, 213; the final
/// permutation is always `2 1 3 4 ... n` (for n >= 2), one adjacent swap
/// away from the identity — plain changes is a cyclic (Hamiltonian-cycle)
/// order on the permutations.
pub fn plain_changes(n: u32) -> Vec<Vec<u32>> {
    let n = n as usize;
    if n == 0 {
        return vec![vec![]]; // the single (empty) permutation
    }
    // P1. [Initialize.] a_j <- j, c_j <- 0, o_j <- 1 for 1 <= j <= n.
    let mut a: Vec<u32> = (1..=n as u32).collect();
    let mut c = vec![0i64; n + 1]; // c[1..=n]; c[0] unused
    let mut o = vec![1i64; n + 1]; // o[1..=n]; o[0] unused
    let mut out = Vec::new();
    loop {
        // P2. [Visit.]
        out.push(a.clone());
        // P3. [Prepare for change.]
        let mut j = n;
        let mut s: i64 = 0;
        loop {
            // P4. [Ready to change?]
            let q = c[j] + o[j];
            if q >= 0 && q != j as i64 {
                // P5. [Change.] Interchange a_{j-c_j+s} <-> a_{j-q+s}.
                let u = (j as i64 - c[j] + s) as usize;
                let v = (j as i64 - q + s) as usize;
                a.swap(u - 1, v - 1); // 1-based -> 0-based
                c[j] = q;
                break; // return to P2
            }
            if q == j as i64 {
                // P6. [Increase s.]
                if j == 1 {
                    return out;
                }
                s += 1;
            }
            // P7. [Switch direction.]
            o[j] = -o[j];
            j -= 1;
            // back to P4
        }
    }
}

// ---------------------------------------------------------------------------
// Stage 4 — Combinations (Algorithm 7.2.1.3T)
// ---------------------------------------------------------------------------

/// Algorithm 7.2.1.3T (lexicographic combinations): all `C(n, k)`
/// combinations of `k` elements of `{0, 1, ..., n-1}`.
///
/// The algorithm maintains `c_1 < c_2 < ... < c_k` and visits the strings
/// `c_k ... c_2 c_1` in **lexicographic order of that reversed reading** —
/// equivalently, the k-sets appear in *colexicographic* order. Each visit is
/// returned here in ascending order `[c_1, ..., c_k]`; e.g. `(n, k) = (4, 2)`
/// yields {0,1}, {0,2}, {1,2}, {0,3}, {1,3}, {2,3}.
///
/// ```text
/// T1. [Initialize.]   c_j <- j-1 for 1 <= j <= k; c_{k+1} <- n;
///                     c_{k+2} <- 0; j <- k.
/// T2. [Visit.]        Visit c_k ... c_2 c_1.
///                     Then, if j > 0, set x <- j and go to T6.
/// T3. [Easy case?]    If c_1 + 1 < c_2, set c_1 <- c_1 + 1, return to T2.
///                     Otherwise set j <- 2.
/// T4. [Find j.]       c_{j-1} <- j-2; x <- c_j + 1.
///                     If x = c_{j+1}, j <- j+1 and repeat T4.
/// T5. [Done?]         Terminate if j > k.
/// T6. [Increase c_j.] c_j <- x, j <- j-1, return to T2.
/// ```
///
/// Algorithm T as stated requires `0 < k < n`; the (single-visit) edge cases
/// `k = 0` and `k = n` are handled directly. Panics if `k > n`.
pub fn combinations(n: u32, k: u32) -> Vec<Vec<u32>> {
    assert!(k <= n, "combinations: need k <= n");
    let (n, t) = (n as usize, k as usize);
    if t == 0 {
        return vec![vec![]]; // the empty combination
    }
    if t == n {
        return vec![(0..n as u32).collect()]; // Algorithm T assumes t < n
    }
    // c[1..=t] is the combination; c[t+1] = n and c[t+2] = 0 are sentinels.
    let mut c = vec![0i64; t + 3];
    // T1. [Initialize.]
    for j in 1..=t {
        c[j] = j as i64 - 1;
    }
    c[t + 1] = n as i64;
    c[t + 2] = 0;
    let mut j = t;
    let mut out = Vec::new();
    loop {
        // T2. [Visit.] (Returned in ascending order c_1 < ... < c_t.)
        out.push((1..=t).map(|i| c[i] as u32).collect());
        let x;
        if j > 0 {
            x = j as i64;
        } else {
            // T3. [Easy case?]
            if c[1] + 1 < c[2] {
                c[1] += 1;
                continue; // return to T2
            }
            j = 2;
            // T4. [Find j.]
            loop {
                c[j - 1] = j as i64 - 2;
                let cand = c[j] + 1;
                if cand == c[j + 1] {
                    j += 1;
                } else {
                    x = cand;
                    break;
                }
            }
            // T5. [Done?]
            if j > t {
                return out;
            }
        }
        // T6. [Increase c_j.]
        c[j] = x;
        j -= 1;
        // return to T2
    }
}

// ---------------------------------------------------------------------------
// Stage 5 — Partitions of an integer (Algorithm 7.2.1.4P)
// ---------------------------------------------------------------------------

/// Algorithm 7.2.1.4P: all partitions of `n` (each written with parts in
/// non-increasing order) in **reverse lexicographic** order, from `[n]` down
/// to `[1, 1, ..., 1]`. `partitions(0)` returns the single empty partition.
///
/// ```text
/// P1. [Initialize.]           a_0 <- 0, m <- 1.
/// P2. [Store the final part.] a_m <- n, q <- m - [n = 1].
/// P3. [Visit.]                Visit a_1 ... a_m. Go to P5 if a_q != 2.
/// P4. [Change 2 to 1+1.]      a_q <- 1, q <- q-1, m <- m+1, a_m <- 1;
///                             return to P3.
/// P5. [Decrease a_q.]         Terminate if q = 0. Otherwise x <- a_q - 1,
///                             a_q <- x, n <- m - q + 1, m <- q + 1.
/// P6. [Copy x if necessary.]  If n <= x go to P2; otherwise a_m <- x,
///                             m <- m+1, n <- n-x, repeat P6.
/// ```
///
/// Here `q` always points at the rightmost part exceeding 1, and the `n` of
/// steps P2/P5/P6 is the amount that still has to be redistributed.
pub fn partitions(n: u32) -> Vec<Vec<u32>> {
    if n == 0 {
        return vec![vec![]];
    }
    let mut out = Vec::new();
    // P1. [Initialize.]
    let mut a = vec![0i64; n as usize + 1]; // a_0 .. a_n; a_0 = 0 sentinel
    let mut m: usize = 1;
    let mut rem = n as i64; // the "n" that P2/P5/P6 keep re-using
    loop {
        // P2. [Store the final part.]
        a[m] = rem;
        let mut q = m - usize::from(rem == 1);
        loop {
            // P3. [Visit.]
            out.push(a[1..=m].iter().map(|&x| x as u32).collect());
            if a[q] == 2 {
                // P4. [Change 2 to 1+1.]
                a[q] = 1;
                q -= 1;
                m += 1;
                a[m] = 1;
                continue; // return to P3
            }
            // P5. [Decrease a_q.]
            if q == 0 {
                return out;
            }
            let x = a[q] - 1;
            a[q] = x;
            rem = (m - q + 1) as i64;
            m = q + 1;
            // P6. [Copy x if necessary.]
            while rem > x {
                a[m] = x;
                m += 1;
                rem -= x;
            }
            break; // go to P2
        }
    }
}

/// The number of partitions `p(n)`, computed by the recurrence from Euler's
/// pentagonal number theorem (TAOCP Vol. 4A, §7.2.1.4):
///
/// ```text
/// p(n) = sum_{k >= 1} (-1)^{k+1} ( p(n - k(3k-1)/2) + p(n - k(3k+1)/2) )
/// ```
///
/// with `p(0) = 1` and `p(m) = 0` for `m < 0`. The generalized pentagonal
/// numbers `k(3k∓1)/2 = 1, 2, 5, 7, 12, 15, ...` make this about `O(n^1.5)`
/// arithmetic operations — vastly cheaper than generating the partitions.
/// `p(100) = 190569292`.
pub fn partition_count(n: u32) -> u64 {
    let n = n as usize;
    let mut p = vec![0i128; n + 1];
    p[0] = 1;
    for m in 1..=n {
        let mut total: i128 = 0;
        let mut k: usize = 1;
        loop {
            let g1 = k * (3 * k - 1) / 2; // pentagonal number
            if g1 > m {
                break;
            }
            let sign: i128 = if k % 2 == 1 { 1 } else { -1 };
            total += sign * p[m - g1];
            let g2 = k * (3 * k + 1) / 2; // second-kind pentagonal number
            if g2 <= m {
                total += sign * p[m - g2];
            }
            k += 1;
        }
        p[m] = total;
    }
    p[n] as u64
}

/// The conjugate (transpose) of a partition: flip its Ferrers diagram about
/// the main diagonal. Part `j` of the conjugate counts the parts of `p` that
/// are `>= j`. Conjugation is an involution — `conjugate(conjugate(p)) = p`
/// — and it exchanges "number of parts" with "largest part".
///
/// Panics if `p` is not a valid partition (positive, non-increasing parts).
pub fn conjugate(p: &[u32]) -> Vec<u32> {
    assert!(
        p.windows(2).all(|w| w[0] >= w[1]),
        "conjugate: parts must be non-increasing"
    );
    assert!(p.iter().all(|&x| x > 0), "conjugate: parts must be positive");
    if p.is_empty() {
        return Vec::new();
    }
    (1..=p[0])
        .map(|j| p.iter().take_while(|&&x| x >= j).count() as u32)
        .collect()
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gray_matches_table_and_closed_form() {
        // §7.2.1.1: the reflected Gray binary code for n = 3 runs
        // 000, 001, 011, 010, 110, 111, 101, 100.
        assert_eq!(
            gray_code(3),
            vec![0b000, 0b001, 0b011, 0b010, 0b110, 0b111, 0b101, 0b100]
        );
        for n in 0..=12u32 {
            let g = gray_code(n);
            assert_eq!(g.len(), 1 << n);
            for (k, &w) in g.iter().enumerate() {
                let k = k as u64;
                assert_eq!(w, k ^ (k >> 1), "g({k})");
                assert_eq!(gray_rank(w), k);
            }
        }
    }

    #[test]
    fn gray_changed_bit_is_the_ruler_function() {
        // From g(k-1) to g(k) the flipped bit sits at position rho(k) =
        // number of trailing 0s of k = number of trailing 1s of k - 1.
        let g = gray_code(10);
        for k in 1..g.len() {
            let diff = g[k - 1] ^ g[k];
            assert_eq!(diff.count_ones(), 1);
            assert_eq!(diff.trailing_zeros(), (k as u64).trailing_zeros());
            assert_eq!(diff.trailing_zeros(), (k as u64 - 1).trailing_ones());
        }
    }

    #[test]
    fn lex_permutations_of_multisets() {
        // §7.2.1.2 opens with the multiset {1, 2, 2, 3}: Algorithm L visits
        // 1223, 1232, 1322, 2123, 2132, 2213, 2231, 2312, 2321, 3122, 3212,
        // 3221 — twelve arrangements in increasing lexicographic order.
        let mut a = vec![1, 2, 2, 3];
        let mut seen = vec![a.clone()];
        while next_permutation(&mut a) {
            seen.push(a.clone());
        }
        assert_eq!(seen.len(), 12);
        assert_eq!(seen[0], vec![1, 2, 2, 3]);
        assert_eq!(seen[1], vec![1, 2, 3, 2]);
        assert_eq!(seen[11], vec![3, 2, 2, 1]);
        assert!(seen.windows(2).all(|w| w[0] < w[1]), "lex increasing");
        // And plain 1..=n: n! permutations, first sorted, last reversed.
        let perms = all_permutations(4);
        assert_eq!(perms.len(), 24);
        assert_eq!(perms[0], vec![1, 2, 3, 4]);
        assert_eq!(perms[23], vec![4, 3, 2, 1]);
    }

    #[test]
    fn plain_changes_matches_the_text() {
        // §7.2.1.2: for n = 3 plain changes runs 123, 132, 312, 321, 231, 213.
        assert_eq!(
            plain_changes(3),
            vec![
                vec![1, 2, 3],
                vec![1, 3, 2],
                vec![3, 1, 2],
                vec![3, 2, 1],
                vec![2, 3, 1],
                vec![2, 1, 3],
            ]
        );
        for n in 1..=6u32 {
            let seq = plain_changes(n);
            let fact: usize = (1..=n as usize).product();
            assert_eq!(seq.len(), fact);
            for w in seq.windows(2) {
                let diffs: Vec<usize> =
                    (0..n as usize).filter(|&i| w[0][i] != w[1][i]).collect();
                assert_eq!(diffs.len(), 2, "one transposition");
                assert_eq!(diffs[1], diffs[0] + 1, "adjacent");
            }
            if n >= 2 {
                let mut expect: Vec<u32> = (1..=n).collect();
                expect.swap(0, 1);
                assert_eq!(*seq.last().unwrap(), expect, "ends at 2 1 3 4 ...");
            }
        }
    }

    #[test]
    fn combinations_in_algorithm_t_order() {
        // Hand trace of Algorithm T for (n, k) = (4, 2): the strings
        // c_2 c_1 run 10, 20, 21, 30, 31, 32.
        assert_eq!(
            combinations(4, 2),
            vec![
                vec![0, 1],
                vec![0, 2],
                vec![1, 2],
                vec![0, 3],
                vec![1, 3],
                vec![2, 3],
            ]
        );
        // Pascal check on a grid.
        let mut binom = [[0u64; 11]; 11];
        for n in 0..=10usize {
            binom[n][0] = 1;
            for k in 1..=n {
                binom[n][k] = binom[n - 1][k - 1] + if k < n { binom[n - 1][k] } else { 0 };
            }
            for k in 0..=n {
                assert_eq!(
                    combinations(n as u32, k as u32).len() as u64,
                    binom[n][k],
                    "C({n},{k})"
                );
            }
        }
    }

    #[test]
    fn partitions_of_five_and_the_counts() {
        // §7.2.1.4: the p(5) = 7 partitions in reverse lexicographic order.
        assert_eq!(
            partitions(5),
            vec![
                vec![5],
                vec![4, 1],
                vec![3, 2],
                vec![3, 1, 1],
                vec![2, 2, 1],
                vec![2, 1, 1, 1],
                vec![1, 1, 1, 1, 1],
            ]
        );
        for n in 0..=25u32 {
            assert_eq!(partitions(n).len() as u64, partition_count(n), "p({n})");
        }
        assert_eq!(partition_count(10), 42);
        assert_eq!(partition_count(50), 204_226);
        assert_eq!(partition_count(100), 190_569_292);
    }

    #[test]
    fn conjugation_is_an_involution() {
        // Ferrers-diagram transpose: the conjugate of 4+1 is 2+1+1+1.
        assert_eq!(conjugate(&[4, 1]), vec![2, 1, 1, 1]);
        assert_eq!(conjugate(&[3, 2, 1]), vec![3, 2, 1]); // self-conjugate
        assert_eq!(conjugate(&[]), Vec::<u32>::new());
        for p in partitions(8) {
            let c = conjugate(&p);
            assert_eq!(c.len() as u32, p[0]);
            assert_eq!(c[0] as usize, p.len());
            assert_eq!(conjugate(&c), p);
        }
    }
}
