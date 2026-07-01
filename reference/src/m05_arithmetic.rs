//! Module 05 — Arithmetic.
//! Source: TAOCP Vol. 2, 3rd ed., Ch. 4: §4.3.1 (classical algorithms),
//! §4.3.3 (faster multiplication), §4.5.2 (binary gcd), §4.5.4 (primality).
//!
//! Big numbers are nonnegative integers in radix b = 2^32, stored as
//! little-endian `Vec<u32>` limbs ("digits" in Knuth's sense) with **no
//! trailing zero limbs** — the canonical form. The empty vector is zero.

use std::cmp::Ordering;

// ---------------------------------------------------------------------------
// Stage 1 — Algorithms 4.3.1A and 4.3.1S: addition, subtraction, comparison.
// ---------------------------------------------------------------------------

/// Compare two canonical big numbers (little-endian base-2^32 limbs).
///
/// Because both operands are canonical (no trailing zero limbs), a longer
/// limb vector is strictly larger; equal lengths are compared limbwise from
/// the most significant limb down.
pub fn big_cmp(u: &[u32], v: &[u32]) -> Ordering {
    if u.len() != v.len() {
        return u.len().cmp(&v.len());
    }
    for j in (0..u.len()).rev() {
        match u[j].cmp(&v[j]) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    Ordering::Equal
}

/// Algorithm 4.3.1A (Addition of nonnegative integers), extended to
/// operands of unequal length by treating missing digits as 0.
///
/// The carry k satisfies 0 <= k <= 1 at every step: the largest possible
/// digit sum is (b-1) + (b-1) + 1 = 2b - 1 < 2b, so the carry out of one
/// digit position is at most 1.
pub fn big_add(u: &[u32], v: &[u32]) -> Vec<u32> {
    let n = u.len().max(v.len());
    let mut w = Vec::with_capacity(n + 1);
    // A1. [Initialize.] Set j <- 0, k <- 0. (j runs over digit positions;
    //     k is the carry.)
    let mut k: u64 = 0;
    for j in 0..n {
        // A2. [Add digits.] Set w_j <- (u_j + v_j + k) mod b and
        //     k <- floor((u_j + v_j + k) / b). (Missing digits are 0.)
        let uj = *u.get(j).unwrap_or(&0) as u64;
        let vj = *v.get(j).unwrap_or(&0) as u64;
        let t = uj + vj + k;
        w.push(t as u32); // t mod b
        k = t >> 32; //       floor(t / b), always 0 or 1
        // A3. [Loop on j.] Increase j by one; if j < n go back to A2.
    }
    // Final carry: w_n <- k. Only pushed when nonzero, keeping the result
    // canonical (Knuth stores it unconditionally; canonical form trims it).
    if k != 0 {
        w.push(k as u32);
    }
    w
}

/// Algorithm 4.3.1S (Subtraction of nonnegative integers), extended to
/// operands of unequal length. Computes u - v.
///
/// Panics if u < v: the algorithm is defined only when the result is
/// **nonnegative** (Knuth requires u >= v; when u < v the final borrow
/// k = -1 would signal a negative result).
pub fn big_sub(u: &[u32], v: &[u32]) -> Vec<u32> {
    assert!(
        big_cmp(u, v) != Ordering::Less,
        "big_sub: u >= v required; result must be nonnegative"
    );
    let mut w = Vec::with_capacity(u.len());
    // S1. [Initialize.] Set j <- 0, k <- 0. (k is the borrow, 0 or -1.)
    let mut k: i64 = 0;
    for j in 0..u.len() {
        // S2. [Subtract digits.] Set w_j <- (u_j - v_j + k) mod b and
        //     k <- floor((u_j - v_j + k) / b). (k is -1 or 0.)
        let vj = *v.get(j).unwrap_or(&0) as i64;
        let t = u[j] as i64 - vj + k;
        if t < 0 {
            w.push((t + (1i64 << 32)) as u32);
            k = -1;
        } else {
            w.push(t as u32);
            k = 0;
        }
        // S3. [Loop on j.] Increase j by one; if j < n go back to S2.
    }
    // Since u >= v the final borrow is 0; trim to canonical form.
    debug_assert_eq!(k, 0);
    while w.last() == Some(&0) {
        w.pop();
    }
    w
}

/// Convert a `u128` into canonical little-endian base-2^32 limbs.
pub fn big_from_u128(mut x: u128) -> Vec<u32> {
    let mut w = Vec::new();
    while x != 0 {
        w.push(x as u32);
        x >>= 32;
    }
    w
}

/// Convert canonical limbs back to `u128`, or `None` if the value needs
/// more than 128 bits.
pub fn big_to_u128(u: &[u32]) -> Option<u128> {
    if u.len() > 4 {
        return None;
    }
    let mut x: u128 = 0;
    for &limb in u.iter().rev() {
        x = (x << 32) | limb as u128;
    }
    Some(x)
}

// ---------------------------------------------------------------------------
// Stage 2 — Algorithm 4.3.1M: classical multiplication.
// ---------------------------------------------------------------------------

/// Algorithm 4.3.1M (Multiplication of nonnegative integers).
///
/// Classical digit-by-digit multiplication, O(n*m) elementary steps.
/// The key invariant of step M4: with t = u_i * v_j + w_{i+j} + k,
///     t <= (b-1)^2 + (b-1) + (b-1) = b^2 - 1 < b^2,
/// so t always fits in a double-length word (`u64` here) and the carry
/// k = floor(t / b) fits in a single digit.
pub fn big_mul(u: &[u32], v: &[u32]) -> Vec<u32> {
    if u.is_empty() || v.is_empty() {
        return Vec::new(); // 0 * anything = 0, canonical form
    }
    let (m, n) = (u.len(), v.len());
    // M1. [Initialize.] Set w_{m-1}, ..., w_0 all to zero; set j <- 0.
    //     (We zero the whole product area w_{m+n-1..0} up front.)
    let mut w = vec![0u32; m + n];
    for j in 0..n {
        // M2. [Zero multiplier?] If v_j = 0, set w_{j+m} <- 0 and go to M6.
        if v[j] == 0 {
            continue; // w[j + m] is already 0
        }
        // M3. [Initialize i.] Set i <- 0, k <- 0.
        let mut k: u64 = 0;
        for i in 0..m {
            // M4. [Multiply and add.] Set t <- u_i * v_j + w_{i+j} + k,
            //     then w_{i+j} <- t mod b, k <- floor(t / b).
            //     (t < b^2, so k < b: the carry fits one digit.)
            let t = u[i] as u64 * v[j] as u64 + w[i + j] as u64 + k;
            w[i + j] = t as u32;
            k = t >> 32;
            // M5. [Loop on i.] Increase i by one; if i < m go back to M4.
        }
        w[j + m] = k as u32;
        // M6. [Loop on j.] Increase j by one; if j < n go back to M2.
    }
    while w.last() == Some(&0) {
        w.pop();
    }
    w
}

/// Radix conversion (§4.4): render a big number as a decimal string.
///
/// Repeatedly divides by 10^9 (the largest power of ten below 2^32) using
/// one pass of short division per output chunk — §4.4's "divide by the new
/// radix" method with a super-digit radix.
pub fn big_to_decimal(u: &[u32]) -> String {
    if u.is_empty() {
        return "0".to_string();
    }
    const CHUNK: u64 = 1_000_000_000; // 10^9 < 2^32
    let mut limbs = u.to_vec();
    let mut chunks: Vec<u32> = Vec::new();
    while !limbs.is_empty() {
        // Short division of the limb vector by 10^9, most significant first.
        let mut rem: u64 = 0;
        for d in limbs.iter_mut().rev() {
            let cur = (rem << 32) | *d as u64;
            *d = (cur / CHUNK) as u32;
            rem = cur % CHUNK;
        }
        while limbs.last() == Some(&0) {
            limbs.pop();
        }
        chunks.push(rem as u32);
    }
    // Most significant chunk unpadded, the rest zero-padded to 9 digits.
    let mut s = chunks.last().unwrap().to_string();
    for c in chunks.iter().rev().skip(1) {
        s.push_str(&format!("{c:09}"));
    }
    s
}

// ---------------------------------------------------------------------------
// Stage 3 — §4.3.3: Karatsuba multiplication.
// ---------------------------------------------------------------------------

/// Below this many limbs (in the *smaller* operand) classical Algorithm M
/// wins: the recursion's bookkeeping (three recursive calls, additions,
/// allocations) costs more than the saved digit products.
const KARATSUBA_CUTOFF: usize = 32;

/// Karatsuba multiplication (§4.3.3, after Karatsuba–Ofman 1962).
///
/// Split each operand at limb position p: u = u1*b^p + u0, v = v1*b^p + v0.
/// The identity
///     u*v = z2*b^{2p} + z1*b^p + z0,  where
///     z2 = u1*v1,  z0 = u0*v0,  z1 = (u0+u1)(v0+v1) - z2 - z0
/// uses only THREE half-size multiplications instead of four, giving the
/// recurrence T(n) = 3 T(n/2) + O(n) and hence T(n) = O(n^{lg 3}).
pub fn big_mul_karatsuba(u: &[u32], v: &[u32]) -> Vec<u32> {
    // Small or very unbalanced problems: classical multiplication.
    if u.len().min(v.len()) < KARATSUBA_CUTOFF {
        return big_mul(u, v);
    }
    // K1. [Split.] p = half the larger length; u = u1*b^p + u0, v likewise.
    //     (A high part may be empty; low parts are trimmed to canonical form.)
    let p = u.len().max(v.len()) / 2;
    let (u0, u1) = split_at_limb(u, p);
    let (v0, v1) = split_at_limb(v, p);
    // K2. [Three recursive products.]
    let z0 = big_mul_karatsuba(&u0, &v0);
    let z2 = big_mul_karatsuba(&u1, &v1);
    let mid = big_mul_karatsuba(&big_add(&u0, &u1), &big_add(&v0, &v1));
    // K3. [Middle term.] z1 = (u0+u1)(v0+v1) - z2 - z0 = u1*v0 + u0*v1 >= 0.
    let z1 = big_sub(&big_sub(&mid, &z2), &z0);
    // K4. [Recombine.] u*v = z2*b^{2p} + z1*b^p + z0.
    let w = big_add(&shift_limbs(&z2, 2 * p), &shift_limbs(&z1, p));
    big_add(&w, &z0)
}

/// Split x at limb position p: returns (low p limbs canonicalized, rest).
fn split_at_limb(x: &[u32], p: usize) -> (Vec<u32>, Vec<u32>) {
    if x.len() <= p {
        return (x.to_vec(), Vec::new());
    }
    let mut lo = x[..p].to_vec();
    while lo.last() == Some(&0) {
        lo.pop();
    }
    (lo, x[p..].to_vec()) // high part is canonical because x is
}

/// Multiply by b^p (prepend p zero limbs); zero stays the empty vector.
fn shift_limbs(x: &[u32], p: usize) -> Vec<u32> {
    if x.is_empty() {
        return Vec::new();
    }
    let mut w = vec![0u32; p];
    w.extend_from_slice(x);
    w
}

// ---------------------------------------------------------------------------
// Stage 4 — Algorithm 4.5.2B: the binary gcd algorithm.
// ---------------------------------------------------------------------------

/// Algorithm 4.5.2B (Greatest common divisor, binary method).
///
/// Uses only subtraction, halving, and parity tests — no division — which
/// is why it beats Euclid on hardware where division is slow.
///
/// Knuth states Algorithm B for positive u, v; we extend it with
/// gcd(0, n) = gcd(n, 0) = n (and gcd(0, 0) = 0), consistent with §4.5.2's
/// convention.
pub fn binary_gcd(mut u: u64, mut v: u64) -> u64 {
    if u == 0 {
        return v;
    }
    if v == 0 {
        return u;
    }
    // B1. [Find power of 2.] Set k <- 0, and then repeatedly set
    //     u <- u/2, v <- v/2, k <- k+1, zero or more times until u and v
    //     are not both even.
    let mut k: u32 = 0;
    while u & 1 == 0 && v & 1 == 0 {
        u >>= 1;
        v >>= 1;
        k += 1;
    }
    // B2. [Initialize.] (Now the original u and v have been divided by 2^k,
    //     and at most one of the current u, v is even.) If u is odd, set
    //     t <- -v and go to B4. Otherwise set t <- u.
    //     (t is signed; u64 magnitudes fit comfortably in i128.)
    let mut t: i128 = if u & 1 == 1 { -(v as i128) } else { u as i128 };
    let mut entering_at_b4 = u & 1 == 1;
    loop {
        if !entering_at_b4 {
            // B3. [Halve t.] (t is even and nonzero here.) Set t <- t/2.
            t /= 2;
        }
        entering_at_b4 = false;
        // B4. [Is t even?] If t is even, go back to B3.
        if t & 1 == 0 {
            continue;
        }
        // B5. [Reset max(u, v).] If t > 0, set u <- t; otherwise set
        //     v <- -t. (The larger of u and v is replaced by |t|; both
        //     u and v are odd now.)
        if t > 0 {
            u = t as u64;
        } else {
            v = (-t) as u64;
        }
        // B6. [Subtract.] Set t <- u - v. If t != 0, go back to B3.
        //     Otherwise the algorithm terminates with output u * 2^k.
        t = u as i128 - v as i128;
        if t == 0 {
            return u << k;
        }
    }
}

// ---------------------------------------------------------------------------
// Stage 5 — §4.5.4: probabilistic primality testing (Miller–Rabin lineage).
// ---------------------------------------------------------------------------

/// (a * b) mod m without overflow, via 128-bit intermediate. Panics if m = 0.
pub fn mul_mod(a: u64, b: u64, m: u64) -> u64 {
    assert!(m > 0, "mul_mod: modulus must be nonzero");
    ((a as u128 * b as u128) % m as u128) as u64
}

/// a^e mod m by binary (left-to-right on the bits of e, here right-to-left)
/// exponentiation — §4.6.3's "S-and-X" method adapted to modular arithmetic.
/// Panics if m = 0; pow_mod(a, 0, m) = 1 mod m.
pub fn pow_mod(mut a: u64, mut e: u64, m: u64) -> u64 {
    assert!(m > 0, "pow_mod: modulus must be nonzero");
    let mut y: u64 = 1 % m;
    a %= m;
    while e > 0 {
        if e & 1 == 1 {
            y = mul_mod(y, a, m);
        }
        a = mul_mod(a, a, m);
        e >>= 1;
    }
    y
}

/// The strong pseudoprime test to base a (§4.5.4; the witness test at the
/// heart of Algorithm P and of the Miller–Rabin procedure).
///
/// Requires odd n >= 3 (panics otherwise, message contains "odd").
/// Write n - 1 = 2^s * d with d odd. Returns true ("n is a strong probable
/// prime to base a") iff
///     a^d = 1 (mod n),  or  a^(2^r * d) = -1 (mod n) for some 0 <= r < s.
/// Every odd prime passes for every base; a composite n passes for at most
/// (n-1)/4 of the bases 1 <= a < n. Bases with a = 0 (mod n) pass trivially.
pub fn is_strong_probable_prime(n: u64, a: u64) -> bool {
    assert!(
        n >= 3 && n & 1 == 1,
        "strong pseudoprime test requires odd n >= 3"
    );
    let a = a % n;
    if a == 0 {
        return true; // the test is vacuous for such bases
    }
    // P1. [Decompose.] n - 1 = 2^s * d with d odd, s >= 1.
    let s = (n - 1).trailing_zeros();
    let d = (n - 1) >> s;
    // P2. [First power.] x <- a^d mod n; pass if x = 1 or x = n - 1.
    let mut x = pow_mod(a, d, n);
    if x == 1 || x == n - 1 {
        return true;
    }
    // P3. [Square repeatedly.] If some square is -1 (mod n), pass. If we
    //     reach 1 without passing through -1 we have found a square root
    //     of 1 other than +-1, so n is definitely composite; likewise if
    //     a^(n-1) != 1 (Fermat) — both surface as "never saw -1".
    for _ in 1..s {
        x = mul_mod(x, x, n);
        if x == n - 1 {
            return true;
        }
    }
    false
}

/// The twelve smallest primes: a deterministic witness set for all of u64.
/// (Jaeschke 1993 pinned down minimal witness sets far beyond 2^32, and
/// later work in the same program — Sinclair; Sorenson–Webster 2015 —
/// verified that no composite below 3.3 * 10^24 > 2^64 passes the strong
/// test to all twelve of these bases: a post-Knuth refinement of §4.5.4.)
const U64_WITNESSES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

/// Deterministic primality test for every u64 value, via the strong
/// pseudoprime test with the fixed witness set {2, 3, 5, ..., 37}.
pub fn is_prime_u64(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    for &p in &U64_WITNESSES {
        if n == p {
            return true;
        }
        if n % p == 0 {
            return false;
        }
    }
    // n is odd, > 37, and coprime to all twelve witnesses.
    U64_WITNESSES.iter().all(|&a| is_strong_probable_prime(n, a))
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition_worked_examples() {
        // Base-2^32 analogue of §4.3.1's schoolbook carries:
        // (2^32 - 1) + 1 = 2^32 = [0, 1].
        assert_eq!(big_add(&[u32::MAX], &[1]), vec![0, 1]);
        // (2^64 - 1) + 1 = 2^64: the carry ripples through both limbs.
        assert_eq!(big_add(&[u32::MAX, u32::MAX], &[1]), vec![0, 0, 1]);
        // Unequal lengths and the identity element (zero = empty vec).
        assert_eq!(big_add(&[], &[7]), vec![7]);
        assert_eq!(big_add(&[7], &[]), vec![7]);
        assert_eq!(big_add(&[], &[]), Vec::<u32>::new());
    }

    #[test]
    fn subtraction_worked_examples() {
        // 2^64 - 1 = [MAX, MAX]: the borrow ripples through the low limbs.
        assert_eq!(big_sub(&[0, 0, 1], &[1]), vec![u32::MAX, u32::MAX]);
        assert_eq!(big_sub(&[5, 9], &[5, 9]), Vec::<u32>::new());
        assert_eq!(big_cmp(&[0, 1], &[u32::MAX]), Ordering::Greater);
    }

    #[test]
    #[should_panic(expected = "nonnegative")]
    fn subtraction_rejects_negative_results() {
        big_sub(&[1], &[2]);
    }

    #[test]
    fn u128_roundtrip() {
        for x in [0u128, 1, u32::MAX as u128, 1 << 32, u128::MAX] {
            assert_eq!(big_to_u128(&big_from_u128(x)), Some(x));
        }
        assert_eq!(big_to_u128(&[0, 0, 0, 0, 1]), None);
    }

    #[test]
    fn multiplication_worked_examples() {
        // (2^32 - 1)^2 = 2^64 - 2^33 + 1 = [1, 0xFFFF_FFFE].
        assert_eq!(big_mul(&[u32::MAX], &[u32::MAX]), vec![1, 0xFFFF_FFFE]);
        assert_eq!(big_mul(&[], &[1, 2, 3]), Vec::<u32>::new());
        // 10! = 3628800, built limb by limb.
        let mut f = vec![1u32];
        for i in 2..=10 {
            f = big_mul(&f, &[i]);
        }
        assert_eq!(big_to_u128(&f), Some(3_628_800));
        assert_eq!(big_to_decimal(&f), "3628800");
        assert_eq!(big_to_decimal(&[]), "0");
    }

    #[test]
    fn karatsuba_matches_classical() {
        let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut lcg = || {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (x >> 32) as u32
        };
        for (la, lb) in [(0, 5), (31, 33), (64, 64), (7, 200), (150, 90)] {
            let a = canon((0..la).map(|_| lcg()).collect());
            let b = canon((0..lb).map(|_| lcg()).collect());
            assert_eq!(big_mul_karatsuba(&a, &b), big_mul(&a, &b), "{la}x{lb}");
        }
    }

    fn canon(mut v: Vec<u32>) -> Vec<u32> {
        while v.last() == Some(&0) {
            v.pop();
        }
        v
    }

    #[test]
    fn binary_gcd_knuth_example() {
        // §4.5.2 traces Algorithm B on u = 40902, v = 24140: gcd = 34.
        assert_eq!(binary_gcd(40902, 24140), 34);
        assert_eq!(binary_gcd(0, 17), 17);
        assert_eq!(binary_gcd(17, 0), 17);
        assert_eq!(binary_gcd(0, 0), 0);
        assert_eq!(binary_gcd(1 << 20, 1 << 13), 1 << 13);
        // Agreement with Euclid (module 01) on a small grid.
        for m in 1..=50u64 {
            for n in 1..=50u64 {
                assert_eq!(binary_gcd(m, n), crate::m01_algorithms::euclid_e(m, n));
            }
        }
    }

    #[test]
    fn primality_worked_examples() {
        // 2047 = 23 * 89 is the smallest strong pseudoprime to base 2.
        assert!(is_strong_probable_prime(2047, 2));
        assert!(!is_strong_probable_prime(2047, 3));
        assert!(!is_prime_u64(2047));
        // The Carmichael number 561 = 3*11*17 fools the Fermat test to
        // every coprime base, but not the strong test to base 2.
        assert_eq!(pow_mod(2, 560, 561), 1); // Fermat is fooled...
        assert!(!is_strong_probable_prime(561, 2)); // ...the strong test isn't
        assert!(!is_prime_u64(561));
        // Mersenne prime 2^61 - 1; and 2^61 + 1 = 3 * 768614336404564651.
        assert!(is_prime_u64((1u64 << 61) - 1));
        assert!(!is_prime_u64((1u64 << 61) + 1));
        assert_eq!(mul_mod(u64::MAX - 1, u64::MAX - 2, u64::MAX), 2);
    }
}
