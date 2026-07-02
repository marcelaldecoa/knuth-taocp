//! Module 20 — Optimum Sorting and Sorting Networks.
//! Source: TAOCP Vol. 3, 2nd ed., §5.3 (§5.3.1 minimum-comparison sorting,
//! §5.3.4 networks for sorting).
//!
//! Four themes:
//!   * §5.3.1 — the comparison-tree model and the information-theoretic
//!     lower bound `ceil(lg n!)` on the number of comparisons.
//!   * Algorithm 5.3.1M — merge insertion (Ford & Johnson), which meets the
//!     minimum number of comparisons `S(n)` for all small `n`.
//!   * Algorithm 5.3.4M — Batcher's odd-even merge, an *oblivious* sorting
//!     network whose comparators are fixed in advance.
//!   * Theorem Z (the zero-one principle) — a network sorts every input iff
//!     it sorts every 0-1 input; the certificate we use to check networks.

// ===========================================================================
// Stage 1 — Comparison lower bounds by decision trees (§5.3.1).
// ===========================================================================

/// The information-theoretic lower bound on the worst-case number of
/// comparisons any comparison-based sort of `n` elements must make:
/// `ceil(lg n!)`.
///
/// A comparison sort is a binary decision tree: each internal node is one
/// comparison, each leaf a permutation. To be correct the tree needs at least
/// `n!` leaves, and a binary tree of height `h` has at most `2^h` leaves, so
/// `2^h >= n!`, i.e. `h >= lg n!`; since `h` is an integer, `h >= ceil(lg n!)`.
///
/// Note the *gap*: this is only a lower bound. The true minimum `S(n)` equals
/// `ceil(lg n!)` for `1 <= n <= 11`, but `S(12) = 30` while
/// `ceil(lg 12!) = 29` — the first place where the counting bound is not
/// tight. `S(n)` is unknown in general.
pub fn min_comparisons_lower_bound(n: usize) -> u32 {
    // Build n! (fits in u128 for n <= 34) then take ceil(log2 .).
    let mut fact: u128 = 1;
    for k in 2..=n as u128 {
        fact = fact.checked_mul(k).expect("n! overflows u128 (n too large)");
    }
    // Smallest c with 2^c >= fact  ==  ceil(lg fact).
    let mut c = 0u32;
    let mut p: u128 = 1;
    while p < fact {
        p <<= 1;
        c += 1;
    }
    c
}

/// A straight comparison sort (binary insertion sort) that *counts* the key
/// comparisons it performs. Correct for any input; used so students can watch
/// a real algorithm's comparison count sit above `min_comparisons_lower_bound`.
pub fn sort_and_count(a: &mut [i64]) -> u64 {
    let mut comps = 0u64;
    for i in 1..a.len() {
        let key = a[i];
        // Binary search for the insertion point in the sorted prefix a[0..i].
        let (mut lo, mut hi) = (0usize, i);
        while lo < hi {
            let mid = (lo + hi) / 2;
            comps += 1;
            if key < a[mid] {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        // Shift a[lo..i] up by one and drop the key in.
        let mut j = i;
        while j > lo {
            a[j] = a[j - 1];
            j -= 1;
        }
        a[lo] = key;
    }
    comps
}

/// Is the slice sorted into nondecreasing order?
pub fn is_sorted(a: &[i64]) -> bool {
    a.windows(2).all(|w| w[0] <= w[1])
}

// ===========================================================================
// Stage 2 — Merge insertion / Ford-Johnson (Algorithm 5.3.1M).
// ===========================================================================

/// Sort `a` in place by merge insertion (Ford & Johnson, 1959).
pub fn ford_johnson_sort(a: &mut [i64]) {
    let _ = ford_johnson_comparisons(a);
}

/// Sort `a` in place by merge insertion and return the number of key
/// comparisons performed. The worst case over all inputs of size `n` equals
///
/// ```text
///   F(n) = sum_{k=1..n} ceil(lg (3k/4)),
/// ```
///
/// which is `0,1,3,5,7,10,13,16,19,22,26,30` for `n = 1..12`. These meet the
/// minimum `S(n)` for every `n <= 11` and, remarkably, still hit `S(12)=30`.
pub fn ford_johnson_comparisons(a: &mut [i64]) -> u64 {
    let idx: Vec<usize> = (0..a.len()).collect();
    let mut comps = 0u64;
    let sorted = mi_sort(&idx, a, &mut comps);
    let out: Vec<i64> = sorted.iter().map(|&i| a[i]).collect();
    a.copy_from_slice(&out);
    comps
}

/// Sort the indices `idx` into increasing order of `key[idx]`, Ford-Johnson
/// style, counting key comparisons in `comps`. Returns the sorted indices.
///
/// M1. [Pair.]    Compare the elements two at a time; `hi`/`lo` per pair.
/// M2. [Recurse.] Sort the `hi` elements — that is the *main chain*.
/// M3. [Insert.]  Fold each `lo` partner (and any odd straggler) into the main
///                chain by binary insertion, in Jacobsthal-number order so that
///                every insertion stays inside its `ceil(lg .)` budget.
fn mi_sort(idx: &[usize], key: &[i64], comps: &mut u64) -> Vec<usize> {
    let len = idx.len();
    if len <= 1 {
        return idx.to_vec();
    }
    let odd = len % 2 == 1;
    let stray = if odd { idx[len - 1] } else { usize::MAX };
    let pair_len = len - if odd { 1 } else { 0 };

    // M1. [Pair.] partner[hi] = lo for each of this level's pairs.
    let mut partner = vec![usize::MAX; key.len()];
    let mut hi_list = Vec::with_capacity(pair_len / 2);
    let mut j = 0;
    while j + 1 < pair_len + 1 && j < pair_len {
        let (x, y) = (idx[j], idx[j + 1]);
        *comps += 1;
        let (h, l) = if key[x] >= key[y] { (x, y) } else { (y, x) };
        partner[h] = l;
        hi_list.push(h);
        j += 2;
    }

    // M2. [Recurse.] Sort the larger elements: the main chain's backbone.
    let main = mi_sort(&hi_list, key, comps);
    let s = main.len();

    // Seed the chain: b_1 (partner of the smallest a) is < a_1, so it leads.
    let mut chain: Vec<usize> = Vec::with_capacity(len);
    chain.push(partner[main[0]]);
    chain.extend_from_slice(&main);

    // M3. [Insert.] Fold in b_2..b_s (and the straggler) in Jacobsthal order.
    let max_index = s + if odd { 1 } else { 0 }; // 1-based
    for i in jacobsthal_order(max_index) {
        if i == 1 {
            continue; // b_1 already placed
        }
        if i <= s {
            let a_idx = main[i - 1];
            let upper = chain.iter().position(|&x| x == a_idx).unwrap();
            bin_insert(&mut chain, partner[a_idx], upper, key, comps);
        } else {
            // The odd straggler: no bounding a, so search the whole chain.
            let upper = chain.len();
            bin_insert(&mut chain, stray, upper, key, comps);
        }
    }
    chain
}

/// Binary-insert `item` into `chain[0..upper)` by its key, counting comparisons.
fn bin_insert(chain: &mut Vec<usize>, item: usize, upper: usize, key: &[i64], comps: &mut u64) {
    let v = key[item];
    let (mut lo, mut hi) = (0usize, upper);
    while lo < hi {
        let mid = (lo + hi) / 2;
        *comps += 1;
        if v < key[chain[mid]] {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    chain.insert(lo, item);
}

/// The order in which merge insertion folds in the smaller elements: group the
/// 1-based indices `2..=maxi` by Jacobsthal numbers J = 1,1,3,5,11,21,43,...
/// and, within each group `(J_k, J_{k+1}]`, insert from the top index down.
/// This keeps the prefix a binary insertion searches within `2^k - 1` elements.
fn jacobsthal_order(maxi: usize) -> Vec<usize> {
    if maxi < 2 {
        return Vec::new();
    }
    let mut order = Vec::new();
    let mut jprev = 1usize; // J_2
    let mut jcur = 3usize; //  J_3
    let mut lower = 2usize; // J_2 + 1
    loop {
        let upper = jcur.min(maxi);
        let mut i = upper;
        while i >= lower {
            order.push(i);
            i -= 1;
        }
        if jcur >= maxi {
            break;
        }
        lower = jcur + 1;
        let jnext = jcur + 2 * jprev;
        jprev = jcur;
        jcur = jnext;
    }
    order
}

// ===========================================================================
// Stage 3 — Sorting networks: Batcher's odd-even merge (Algorithm 5.3.4M).
// ===========================================================================

/// Batcher's odd-even merge sorting network on `n` wires (`n` a power of two).
/// A network is a fixed list of comparators `(i, j)`; `apply_network` sends the
/// smaller value to the wire listed first. The comparators do not depend on the
/// data — the network is *oblivious*.
pub fn odd_even_merge_network(n: usize) -> Vec<(usize, usize)> {
    assert!(
        n > 0 && n & (n - 1) == 0,
        "odd-even merge sort needs n a power of two"
    );
    let mut net = Vec::new();
    oe_sort(0, n, &mut net);
    net
}

fn oe_sort(lo: usize, n: usize, net: &mut Vec<(usize, usize)>) {
    if n > 1 {
        let m = n / 2;
        oe_sort(lo, m, net);
        oe_sort(lo + m, m, net);
        oe_merge(lo, n, 1, net);
    }
}

fn oe_merge(lo: usize, n: usize, r: usize, net: &mut Vec<(usize, usize)>) {
    let m = r * 2;
    if m < n {
        oe_merge(lo, n, m, net); // even subsequence
        oe_merge(lo + r, n, m, net); // odd subsequence
        let mut i = lo + r;
        while i + r < lo + n {
            net.push((i, i + r));
            i += m;
        }
    } else {
        net.push((lo, lo + r));
    }
}

/// Apply a network to `a`: each comparator `(i, j)` compare-exchanges so the
/// smaller value ends up on wire `i`.
pub fn apply_network(net: &[(usize, usize)], a: &mut [i64]) {
    for &(i, j) in net {
        if a[i] > a[j] {
            a.swap(i, j);
        }
    }
}

/// The *depth* (parallel delay) of a network on `n` wires: the fewest layers
/// into which the comparators can be packed so that comparators in one layer
/// touch disjoint wires. Equivalently the longest chain of data-dependent
/// comparators. For Batcher's network the depth is `t(t+1)/2` with `t = lg n`.
pub fn network_depth(net: &[(usize, usize)], n: usize) -> usize {
    let mut ready = vec![0usize; n]; // first free layer for each wire
    let mut depth = 0;
    for &(i, j) in net {
        let layer = ready[i].max(ready[j]);
        let next = layer + 1;
        ready[i] = next;
        ready[j] = next;
        depth = depth.max(next);
    }
    depth
}

// ===========================================================================
// Stage 4 — The zero-one principle and bitonic sorting (Theorem 5.3.4Z).
// ===========================================================================

/// Theorem Z (the zero-one principle): a comparator network sorts *all* inputs
/// iff it sorts all `2^n` sequences of 0s and 1s. This function checks the
/// right-hand side — a `2^n` certificate, far cheaper than the `n!` of all
/// permutations — and hence certifies the network sorts everything.
pub fn sorts_all_zero_one(net: &[(usize, usize)], n: usize) -> bool {
    assert!(n < 32, "sorts_all_zero_one enumerates 2^n inputs; keep n < 32");
    for mask in 0u32..(1u32 << n) {
        let mut a: Vec<i64> = (0..n).map(|k| ((mask >> k) & 1) as i64).collect();
        apply_network(net, &mut a);
        if !is_sorted(&a) {
            return false;
        }
    }
    true
}

/// Batcher's bitonic sorting network on `n` wires (`n` a power of two). Sorts a
/// bitonic sequence by repeatedly splitting it; the recursion builds an
/// ascending run and a descending run then merges them. Descending comparators
/// are recorded as `(j, i)` with `j > i`, so `apply_network` still sends the
/// smaller value to the wire listed first.
pub fn bitonic_sort_network(n: usize) -> Vec<(usize, usize)> {
    assert!(
        n > 0 && n & (n - 1) == 0,
        "bitonic sort needs n a power of two"
    );
    let mut net = Vec::new();
    bitonic_sort(0, n, true, &mut net);
    net
}

fn bitonic_sort(lo: usize, n: usize, asc: bool, net: &mut Vec<(usize, usize)>) {
    if n > 1 {
        let m = n / 2;
        bitonic_sort(lo, m, true, net);
        bitonic_sort(lo + m, m, false, net);
        bitonic_merge(lo, n, asc, net);
    }
}

fn bitonic_merge(lo: usize, n: usize, asc: bool, net: &mut Vec<(usize, usize)>) {
    if n > 1 {
        let m = n / 2;
        for i in lo..lo + m {
            if asc {
                net.push((i, i + m));
            } else {
                net.push((i + m, i));
            }
        }
        bitonic_merge(lo, m, asc, net);
        bitonic_merge(lo + m, m, asc, net);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Stage 1 -----------------------------------------------------------

    #[test]
    fn ceil_lg_factorial_pinned() {
        // §5.3.1: ceil(lg n!) for n = 1..12.
        let want = [0, 1, 3, 5, 7, 10, 13, 16, 19, 22, 26, 29];
        for n in 1..=12usize {
            assert_eq!(min_comparisons_lower_bound(n), want[n - 1], "n={n}");
        }
    }

    #[test]
    fn info_bound_below_true_minimum_at_12() {
        // The famous gap: S(12) = 30 > 29 = ceil(lg 12!).
        assert_eq!(min_comparisons_lower_bound(12), 29);
    }

    #[test]
    fn sort_and_count_is_a_correct_sort() {
        let mut x = 0x1234_5678u64;
        for n in 0..40usize {
            let mut a: Vec<i64> = (0..n)
                .map(|_| {
                    x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    (x >> 33) as i64 % 100
                })
                .collect();
            let mut b = a.clone();
            b.sort();
            let c = sort_and_count(&mut a);
            assert_eq!(a, b);
            if n >= 2 {
                assert!(c > 0);
            }
        }
    }

    // ---- Stage 2 -----------------------------------------------------------

    const FJ_WORST: [u64; 13] = [0, 0, 1, 3, 5, 7, 10, 13, 16, 19, 22, 26, 30];

    #[test]
    fn merge_insertion_sorts_and_permutes() {
        let mut x = 0x9e37_79b9u64;
        for n in 0..=50usize {
            let mut a: Vec<i64> = (0..n)
                .map(|_| {
                    x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    (x >> 30) as i64 % 1000
                })
                .collect();
            let mut b = a.clone();
            b.sort();
            let mut c = a.clone();
            let cnt = ford_johnson_comparisons(&mut c);
            ford_johnson_sort(&mut a);
            assert_eq!(a, b, "n={n} sorted");
            assert_eq!(c, b, "n={n} count-variant sorted");
            if n <= 12 {
                assert!(cnt <= FJ_WORST[n], "n={n} used {cnt} > F(n)={}", FJ_WORST[n]);
            }
        }
    }

    #[test]
    fn merge_insertion_meets_worst_case_bound() {
        // Exhaustively: the maximum over all permutations of 1..=n equals F(n).
        for n in 1..=8usize {
            let mut perm: Vec<i64> = (1..=n as i64).collect();
            let mut worst = 0u64;
            loop {
                let mut a = perm.clone();
                let c = ford_johnson_comparisons(&mut a);
                assert!(is_sorted(&a));
                worst = worst.max(c);
                if !next_permutation(&mut perm) {
                    break;
                }
            }
            assert_eq!(worst, FJ_WORST[n], "worst case for n={n}");
        }
    }

    // ---- Stage 3 -----------------------------------------------------------

    #[test]
    fn odd_even_counts_and_depths() {
        // Batcher's comparator counts and depths (§5.3.4).
        assert_eq!(odd_even_merge_network(2).len(), 1);
        assert_eq!(odd_even_merge_network(4).len(), 5);
        assert_eq!(odd_even_merge_network(8).len(), 19);
        assert_eq!(odd_even_merge_network(16).len(), 63);
        for &(n, t) in &[(2usize, 1usize), (4, 2), (8, 3), (16, 4)] {
            let net = odd_even_merge_network(n);
            assert_eq!(network_depth(&net, n), t * (t + 1) / 2, "depth n={n}");
        }
    }

    #[test]
    fn odd_even_sorts_all_permutations_small() {
        for n in [2usize, 4, 8] {
            let net = odd_even_merge_network(n);
            let mut perm: Vec<i64> = (1..=n as i64).collect();
            loop {
                let mut a = perm.clone();
                apply_network(&net, &mut a);
                assert!(is_sorted(&a), "n={n} perm={perm:?}");
                if !next_permutation(&mut perm) {
                    break;
                }
            }
        }
    }

    // ---- Stage 4 -----------------------------------------------------------

    #[test]
    fn zero_one_principle_certifies() {
        for n in [2usize, 4, 8, 16] {
            assert!(sorts_all_zero_one(&odd_even_merge_network(n), n));
            assert!(sorts_all_zero_one(&bitonic_sort_network(n), n));
        }
    }

    #[test]
    fn dropping_a_comparator_breaks_it() {
        let net = odd_even_merge_network(8);
        let mut any_broke = false;
        for k in 0..net.len() {
            let mut b = net.clone();
            b.remove(k);
            if !sorts_all_zero_one(&b, 8) {
                any_broke = true;
            }
        }
        assert!(any_broke);
    }

    fn next_permutation(a: &mut [i64]) -> bool {
        if a.len() < 2 {
            return false;
        }
        let mut i = a.len() - 1;
        while i > 0 && a[i - 1] >= a[i] {
            i -= 1;
        }
        if i == 0 {
            return false;
        }
        let mut j = a.len() - 1;
        while a[j] <= a[i - 1] {
            j -= 1;
        }
        a.swap(i - 1, j);
        a[i..].reverse();
        true
    }
}
