//! Module 13 — Bitwise Tricks and Binary Decision Diagrams.
//! Source: TAOCP Vol. 4A, §7.1.3 (bitwise tricks) and §7.1.4 (BDDs).
//!
//! Part 1 (§7.1.3): word-level bit manipulation implemented *without* the
//! corresponding hardware intrinsics — `trailing_zeros`, `count_ones` and
//! friends are exactly what we are re-deriving, so using them would be
//! circular. (Tests are allowed to use them as oracles.)
//!
//! Part 2 (§7.1.4): reduced ordered binary decision diagrams with
//! hash-consing, so that *equality of boolean functions is equality of
//! `Ref`s* — the canonicity theorem made executable.

use std::collections::HashMap;

// ============================================================================
// §7.1.3 — Bitwise tricks
// ============================================================================

/// The de Bruijn constant used by `ruler`. Its 6-bit windows, read by
/// left-shifting (zeros entering from the right), take all 64 values —
/// see §7.1.3, where Knuth explains the trick (Eq. (46) and nearby).
const DEBRUIJN: u64 = 0x03f7_9d71_b4cb_0a89;

/// `RULER_TABLE[(DEBRUIJN << a) >> 58] == a`: inverts the de Bruijn window
/// map. Built at compile time; the reference tests verify all 64 windows are
/// distinct (that is what makes DEBRUIJN a de Bruijn sequence).
const RULER_TABLE: [u32; 64] = {
    let mut t = [0u32; 64];
    let mut a = 0;
    while a < 64 {
        t[((DEBRUIJN << a) >> 58) as usize] = a as u32;
        a += 1;
    }
    t
};

/// The ruler function ρ(x): the number of trailing zero bits of `x`, i.e.
/// the position of the rightmost 1-bit (§7.1.3).
///
/// Method: isolate the rightmost 1 as `2^a = x & (-x)` (two's-complement
/// identity, proved in the lesson), then multiply by a de Bruijn constant.
/// Multiplying by `2^a` is a left shift by `a`, so the top six bits of the
/// product are the `a`-th window of the de Bruijn cycle — all 64 windows are
/// distinct, so a 64-entry table recovers `a`.
///
/// Panics if `x == 0`: the ruler function is undefined there (no rightmost
/// 1-bit exists).
pub fn ruler(x: u64) -> u32 {
    assert!(x != 0, "ruler(0) is undefined: no rightmost 1-bit");
    // Isolate the rightmost 1-bit: x & (-x) == 2^rho(x).
    let b = x & x.wrapping_neg();
    // Left-shift the de Bruijn cycle by rho(x) and read the top window.
    RULER_TABLE[(b.wrapping_mul(DEBRUIJN) >> 58) as usize]
}

/// Sideways addition ν(x): the number of 1-bits of `x`, by the SWAR
/// divide-and-conquer scheme of §7.1.3 (Knuth's Eq. (62)).
///
/// ```text
/// S1. [Pairs.]    x <- x - ((x >> 1) & 0x5555...);   2-bit fields hold ν of each pair
/// S2. [Nibbles.]  x <- (x & 0x3333...) + ((x >> 2) & 0x3333...);
/// S3. [Bytes.]    x <- (x + (x >> 4)) & 0x0f0f...;   each byte holds its own ν (≤ 8)
/// S4. [Total.]    return (x * 0x0101...) >> 56;      high byte = sum of all bytes
/// ```
///
/// Step S1 uses the identity ν(v) = v − ⌊v/2⌋ for a 2-bit field v; step S4
/// works because the byte counts total at most 64 < 256, so the multiply
/// accumulates them without carries between byte lanes.
pub fn sideways_addition(x: u64) -> u32 {
    // S1. [Pairs.] Each 2-bit field v becomes v - floor(v/2) = nu(v).
    let x = x - ((x >> 1) & 0x5555_5555_5555_5555);
    // S2. [Nibbles.] Add adjacent 2-bit counts into 4-bit fields.
    let x = (x & 0x3333_3333_3333_3333) + ((x >> 2) & 0x3333_3333_3333_3333);
    // S3. [Bytes.] Add adjacent 4-bit counts; one mask suffices (max 8).
    let x = (x + (x >> 4)) & 0x0f0f_0f0f_0f0f_0f0f;
    // S4. [Total.] Multiply by 0x0101...: the high byte accumulates all bytes.
    (x.wrapping_mul(0x0101_0101_0101_0101) >> 56) as u32
}

/// Extract the rightmost 1-bit of `x`: `x & (-x)` in two's complement
/// (§7.1.3). Returns 0 when `x == 0` (there is nothing to extract, and the
/// identity gracefully yields 0 & 0 = 0).
pub fn extract_rightmost_one(x: u64) -> u64 {
    x & x.wrapping_neg()
}

/// Smear the rightmost 1-bit to the right: `x | (x − 1)` (§7.1.3).
///
/// Writing `x = y 1 0^a` in binary, `x − 1 = y 0 1^a`, so the OR is
/// `y 1 1^a`: everything at or below the rightmost 1 becomes 1 and the rest
/// of `x` is untouched. For `x == 0` the wrapping subtraction gives
/// `0 | u64::MAX = u64::MAX` — all ones — the conventional value.
pub fn smear_right(x: u64) -> u64 {
    x | x.wrapping_sub(1)
}

/// Gosper's hack (§7.1.3, exercise 20; HAKMEM item 175): the next larger
/// integer with the same number of 1-bits as `x` ("snoob").
///
/// ```text
/// G1. [Isolate.]      u <- x & (-x).            (lowest 1-bit of x)
/// G2. [Carry.]        v <- x + u.               (the rightmost run of 1s
///                                                carries: its top bit moves
///                                                up one, the rest vanish)
/// G3. [Redistribute.] return v | (((x ^ v) / u) >> 2).
///                     (x ^ v = the vacated run plus the new bit — a block
///                      of ones; dividing by u right-justifies it; >> 2
///                      drops the two bits already accounted for.)
/// ```
///
/// Domain: `x` must be nonzero (panics otherwise), and a successor of the
/// same weight must exist in 64 bits — i.e. `x` must not be the *largest*
/// 64-bit integer of its weight (all 1-bits flush against bit 63). Outside
/// that domain step G2 overflows and the result is meaningless.
pub fn next_same_weight(x: u64) -> u64 {
    assert!(
        x != 0,
        "next_same_weight(0) has no meaning: 0 is the only integer of weight 0"
    );
    // G1. [Isolate.]
    let u = x & x.wrapping_neg();
    // G2. [Carry.]
    let v = x.wrapping_add(u);
    // G3. [Redistribute.]
    v | (((x ^ v) / u) >> 2)
}

// ============================================================================
// §7.1.4 — Reduced ordered binary decision diagrams
// ============================================================================

/// A handle to a BDD node: an index into the `Bdd` arena. `Ref(0)` and
/// `Ref(1)` are the ⊥ and ⊤ sinks; every `Ref(i)` with `i < bdd.len()` is
/// valid. Because the arena is hash-consed, **two `Ref`s from the same
/// `Bdd` are equal iff they denote the same boolean function** — that is
/// the whole point (§7.1.4, canonicity).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Ref(pub u32);

/// Sentinel variable index for the two sink nodes (below every real level).
const TERMINAL_VAR: u32 = u32::MAX;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    var: u32,
    lo: Ref,
    hi: Ref,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Op {
    And,
    Or,
    Xor,
}

/// A shared arena of ROBDD nodes over variables x0 < x1 < x2 < … (smaller
/// index tested nearer the root), maintained *reduced* at all times by the
/// two invariants of §7.1.4:
///
/// 1. no node has `lo == hi` (such a test would be redundant), and
/// 2. no two nodes share a `(var, lo, hi)` triple (the *unique table*
///    enforces this by hash-consing).
///
/// Boolean operations use memoized Shannon expansion ("apply"):
/// f ⋄ g = (¬x_v ∧ (f₀ ⋄ g₀)) ∨ (x_v ∧ (f₁ ⋄ g₁)), v the top variable.
pub struct Bdd {
    nodes: Vec<Node>,
    unique: HashMap<(u32, Ref, Ref), Ref>,
    memo: HashMap<(Op, Ref, Ref), Ref>,
}

impl Default for Bdd {
    fn default() -> Self {
        Self::new()
    }
}

impl Bdd {
    /// A fresh arena containing only the sinks ⊥ = `Ref(0)`, ⊤ = `Ref(1)`.
    pub fn new() -> Self {
        let sink = |r| Node { var: TERMINAL_VAR, lo: r, hi: r };
        Bdd {
            nodes: vec![sink(Ref(0)), sink(Ref(1))],
            unique: HashMap::new(),
            memo: HashMap::new(),
        }
    }

    /// The constant function `b`: one of the two sinks.
    pub fn constant(&self, b: bool) -> Ref {
        Ref(b as u32)
    }

    /// Total number of nodes ever created in the arena (including sinks and
    /// nodes not reachable from any particular root). Every `Ref(i)` with
    /// `i < len()` is valid.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// True iff `f` is one of the two sink nodes.
    pub fn is_terminal(&self, f: Ref) -> bool {
        f.0 <= 1
    }

    /// The variable tested at node `f`. Panics on a sink (sinks test nothing).
    pub fn var(&self, f: Ref) -> u32 {
        assert!(!self.is_terminal(f), "sinks have no branch variable");
        self.nodes[f.0 as usize].var
    }

    /// The `x_v = 0` child of node `f`. Panics on a sink.
    pub fn low(&self, f: Ref) -> Ref {
        assert!(!self.is_terminal(f), "sinks have no LO child");
        self.nodes[f.0 as usize].lo
    }

    /// The `x_v = 1` child of node `f`. Panics on a sink.
    pub fn high(&self, f: Ref) -> Ref {
        assert!(!self.is_terminal(f), "sinks have no HI child");
        self.nodes[f.0 as usize].hi
    }

    /// Level of `f` for top-variable comparisons: sinks sit below every
    /// variable (TERMINAL_VAR = u32::MAX).
    fn level(&self, f: Ref) -> u32 {
        self.nodes[f.0 as usize].var
    }

    /// The reduction discipline in one place ("MK" in the literature):
    /// return the unique node `(v, lo, hi)`, creating it only if it does not
    /// exist, and eliding it entirely if `lo == hi`.
    fn mk(&mut self, v: u32, lo: Ref, hi: Ref) -> Ref {
        if lo == hi {
            return lo; // reduction rule 1: never build a redundant test
        }
        if let Some(&r) = self.unique.get(&(v, lo, hi)) {
            return r; // reduction rule 2: hash-consing via the unique table
        }
        let r = Ref(self.nodes.len() as u32);
        self.nodes.push(Node { var: v, lo, hi });
        self.unique.insert((v, lo, hi), r);
        r
    }

    /// The projection function x_i.
    pub fn variable(&mut self, i: u32) -> Ref {
        assert!(i != TERMINAL_VAR, "variable index reserved for sinks");
        self.mk(i, Ref(0), Ref(1))
    }

    /// Memoized apply (Shannon expansion), the engine behind and/or/xor.
    ///
    /// ```text
    /// A1. [Trivial?]  If op(f, g) is decided by sink identities
    ///                 (0 ∧ g = 0, 1 ∧ g = g, f ⊕ f = 0, ...), return it.
    /// A2. [Memo.]     If (op, f, g) is cached, return the cached Ref.
    /// A3. [Expand.]   v <- min(var(f), var(g));
    ///                 r <- mk(v, apply(op, f0, g0), apply(op, f1, g1)),
    ///                 where (f0, f1) = (low(f), high(f)) if var(f) = v,
    ///                 else (f, f) — f does not depend on x_v.
    /// A4. [Cache.]    memo[(op, f, g)] <- r; return r.
    /// ```
    fn apply(&mut self, op: Op, f: Ref, g: Ref) -> Ref {
        let (zero, one) = (Ref(0), Ref(1));
        // A1. [Trivial?]
        match op {
            Op::And => {
                if f == zero || g == zero {
                    return zero;
                }
                if f == one || f == g {
                    return g;
                }
                if g == one {
                    return f;
                }
            }
            Op::Or => {
                if f == one || g == one {
                    return one;
                }
                if f == zero || f == g {
                    return g;
                }
                if g == zero {
                    return f;
                }
            }
            Op::Xor => {
                if f == g {
                    return zero;
                }
                if f == zero {
                    return g;
                }
                if g == zero {
                    return f;
                }
            }
        }
        // A2. [Memo.] And/Or/Xor are commutative: normalize the key.
        let key = if f.0 <= g.0 { (op, f, g) } else { (op, g, f) };
        if let Some(&r) = self.memo.get(&key) {
            return r;
        }
        // A3. [Expand on the top variable.]
        let v = self.level(f).min(self.level(g));
        let (f0, f1) = if self.level(f) == v {
            (self.low(f), self.high(f))
        } else {
            (f, f)
        };
        let (g0, g1) = if self.level(g) == v {
            (self.low(g), self.high(g))
        } else {
            (g, g)
        };
        let lo = self.apply(op, f0, g0);
        let hi = self.apply(op, f1, g1);
        let r = self.mk(v, lo, hi);
        // A4. [Cache.]
        self.memo.insert(key, r);
        r
    }

    /// Conjunction f ∧ g.
    pub fn and(&mut self, f: Ref, g: Ref) -> Ref {
        self.apply(Op::And, f, g)
    }

    /// Disjunction f ∨ g.
    pub fn or(&mut self, f: Ref, g: Ref) -> Ref {
        self.apply(Op::Or, f, g)
    }

    /// Exclusive or f ⊕ g.
    pub fn xor(&mut self, f: Ref, g: Ref) -> Ref {
        self.apply(Op::Xor, f, g)
    }

    /// Complement ¬f, as f ⊕ 1 (one memoized traversal, O(B(f))).
    pub fn not(&mut self, f: Ref) -> Ref {
        let one = self.constant(true);
        self.apply(Op::Xor, f, one)
    }

    /// Evaluate the function rooted at `f` on the given assignment
    /// (`assignment[i]` is the value of x_i). Just follow the branches —
    /// a BDD *is* its own evaluation algorithm, O(n) per query.
    pub fn eval(&self, f: Ref, assignment: &[bool]) -> bool {
        let mut r = f;
        while !self.is_terminal(r) {
            let n = &self.nodes[r.0 as usize];
            r = if assignment[n.var as usize] { n.hi } else { n.lo };
        }
        r == Ref(1)
    }

    /// Number of distinct nodes reachable from `f`, sinks included: the
    /// size B(f) whose behavior under variable reordering stage 3
    /// investigates. `node_count(constant) == 1`.
    pub fn node_count(&self, f: Ref) -> usize {
        let mut set = std::collections::HashSet::new();
        let mut stack = vec![f];
        while let Some(r) = stack.pop() {
            if set.insert(r) && !self.is_terminal(r) {
                stack.push(self.nodes[r.0 as usize].lo);
                stack.push(self.nodes[r.0 as usize].hi);
            }
        }
        set.len()
    }

    /// Algorithm 7.1.4C (Count solutions): the number of assignments of
    /// (x_0, …, x_{n_vars−1}) satisfying the function rooted at `f`.
    ///
    /// ```text
    /// C1. [Sinks.]  c(⊥) = 0, c(⊤) = 1, taking a sink's level to be n_vars.
    /// C2. [Nodes.]  For a node at level v with children l, h:
    ///               c = 2^(level(l)−v−1)·c(l) + 2^(level(h)−v−1)·c(h).
    /// C3. [Root.]   Answer: 2^level(f) · c(f).
    /// ```
    ///
    /// The powers of two account for *skipped* variables: an edge that jumps
    /// k levels leaves k−1 variables unconstrained, each free to take either
    /// value (proof in the lesson). Memoized: O(B(f)) arithmetic operations.
    pub fn count_models(&self, f: Ref, n_vars: usize) -> u128 {
        let mut memo: HashMap<Ref, u128> = HashMap::new();
        let c = self.count_rec(f, n_vars, &mut memo);
        // C3. [Root.] Variables above the root's level are unconstrained.
        c << self.level_at(f, n_vars)
    }

    /// A node's level, with sinks pinned at `n_vars`.
    fn level_at(&self, f: Ref, n_vars: usize) -> usize {
        if self.is_terminal(f) {
            n_vars
        } else {
            let v = self.nodes[f.0 as usize].var as usize;
            assert!(v < n_vars, "node variable x{v} out of range for n_vars = {n_vars}");
            v
        }
    }

    fn count_rec(&self, f: Ref, n_vars: usize, memo: &mut HashMap<Ref, u128>) -> u128 {
        // C1. [Sinks.]
        if f == Ref(0) {
            return 0;
        }
        if f == Ref(1) {
            return 1;
        }
        if let Some(&c) = memo.get(&f) {
            return c;
        }
        // C2. [Nodes.]
        let v = self.level_at(f, n_vars);
        let (lo, hi) = (self.low(f), self.high(f));
        let c_lo = self.count_rec(lo, n_vars, memo);
        let c_hi = self.count_rec(hi, n_vars, memo);
        let c = (c_lo << (self.level_at(lo, n_vars) - v - 1))
            + (c_hi << (self.level_at(hi, n_vars) - v - 1));
        memo.insert(f, c);
        c
    }
}

// ============================================================================
// §7.1.4 applications
// ============================================================================

/// The number of independent sets of the graph with vertices `0..n` and the
/// given edges (the empty set counts). §7.1.4 opens with exactly this
/// family of examples.
///
/// Build f = ⋀_{(u,v) ∈ E} ¬(x_u ∧ x_v) — "no edge has both endpoints
/// chosen" — and count its models over n variables.
pub fn independent_set_count(n: usize, edges: &[(usize, usize)]) -> u128 {
    let mut bdd = Bdd::new();
    let mut f = bdd.constant(true);
    for &(u, v) in edges {
        assert!(u < n && v < n && u != v, "edge must join two distinct vertices < n");
        let xu = bdd.variable(u as u32);
        let xv = bdd.variable(v as u32);
        let both = bdd.and(xu, xv);
        let ok = bdd.not(both);
        f = bdd.and(f, ok);
    }
    bdd.count_models(f, n)
}

/// The number of ways to place n nonattacking queens on an n×n board, by
/// building one BDD over the n² cell variables x_{r·n+c} ("a queen sits on
/// cell (r, c)") and counting its models — §7.1.4 uses the queens problem
/// to showcase BDD model counting.
///
/// Constraints: at least one queen in every row, and for every attacking
/// pair of cells (same row, column, or diagonal) not both occupied.
/// Together these force exactly one queen per row, so the models are
/// exactly the classical solutions.
///
/// The BDD grows exponentially with n under this variable order — keep
/// n ≤ 6 (the tests stop there; the answers are 2, 10, 4 for n = 4, 5, 6).
pub fn queens_bdd_count(n: usize) -> u128 {
    assert!(n >= 1, "the board must have at least one cell");
    let mut bdd = Bdd::new();
    let idx = |r: usize, c: usize| (r * n + c) as u32;
    let mut f = bdd.constant(true);
    // No two queens on attacking cells: for every pair of distinct cells in
    // the same row, column, or diagonal, forbid both being occupied.
    for r1 in 0..n {
        for c1 in 0..n {
            for r2 in r1..n {
                for c2 in 0..n {
                    if (r2, c2) <= (r1, c1) {
                        continue;
                    }
                    let attacks =
                        r1 == r2 || c1 == c2 || r1.abs_diff(r2) == c1.abs_diff(c2);
                    if attacks {
                        let a = bdd.variable(idx(r1, c1));
                        let b = bdd.variable(idx(r2, c2));
                        let both = bdd.and(a, b);
                        let ok = bdd.not(both);
                        f = bdd.and(f, ok);
                    }
                }
            }
        }
    }
    // At least one queen in every row (with the pair constraints: exactly one).
    for r in 0..n {
        let mut any = bdd.constant(false);
        for c in 0..n {
            let x = bdd.variable(idx(r, c));
            any = bdd.or(any, x);
        }
        f = bdd.and(f, any);
    }
    bdd.count_models(f, n * n)
}

// ============================================================================
// Tests: worked examples from §7.1.3–7.1.4
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debruijn_windows_are_distinct() {
        // The property that makes the ruler trick work at all.
        let mut seen = [false; 64];
        for a in 0..64 {
            let w = ((DEBRUIJN << a) >> 58) as usize;
            assert!(!seen[w], "window {w} repeats at shift {a}");
            seen[w] = true;
        }
    }

    #[test]
    fn ruler_matches_the_definition() {
        // rho(x) for x = 1, 2, 3, ... begins 0, 1, 0, 2, 0, 1, 0, 3 — the
        // "ruler" pattern that names the function (§7.1.3).
        let expect = [0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0, 4];
        for (i, &e) in expect.iter().enumerate() {
            assert_eq!(ruler(i as u64 + 1), e, "rho({})", i + 1);
        }
        for a in 0..64 {
            assert_eq!(ruler(1u64 << a), a);
            assert_eq!(ruler(u64::MAX << a), a);
        }
    }

    #[test]
    fn sideways_addition_worked_examples() {
        // nu(0) = 0, nu(2^k) = 1, nu(2^k - 1) = k, nu(u64::MAX) = 64.
        assert_eq!(sideways_addition(0), 0);
        for k in 0..64 {
            assert_eq!(sideways_addition(1u64 << k), 1);
            assert_eq!(sideways_addition((1u64 << k) - 1), k as u32);
        }
        assert_eq!(sideways_addition(u64::MAX), 64);
        assert_eq!(sideways_addition(0xdead_beef), 0xdead_beefu64.count_ones());
    }

    #[test]
    fn rightmost_one_and_smear() {
        // The lesson's running example x = 01011000.
        let x = 0b0101_1000u64;
        assert_eq!(extract_rightmost_one(x), 0b0000_1000);
        assert_eq!(smear_right(x), 0b0101_1111);
        assert_eq!(extract_rightmost_one(0), 0);
        assert_eq!(smear_right(0), u64::MAX);
        assert_eq!(smear_right(1), 1);
    }

    #[test]
    fn gosper_hand_trace() {
        // The lesson's trace: 0110 -> 1001; and the weight-2 chain from 3.
        assert_eq!(next_same_weight(0b0110), 0b1001);
        let chain = [0b0011u64, 0b0101, 0b0110, 0b1001, 0b1010, 0b1100];
        for w in chain.windows(2) {
            assert_eq!(next_same_weight(w[0]), w[1]);
        }
        assert_eq!(next_same_weight(1), 2);
    }

    #[test]
    fn bdd_canonicity_small() {
        // Median (majority) of three: <xyz> = xy | xz | yz, built two ways.
        let mut b = Bdd::new();
        let (x, y, z) = (b.variable(0), b.variable(1), b.variable(2));
        let xy = b.and(x, y);
        let xz = b.and(x, z);
        let yz = b.and(y, z);
        let m1 = b.or(xy, xz);
        let m1 = b.or(m1, yz);
        // <xyz> = (x AND (y OR z)) OR (y AND z)
        let y_or_z = b.or(y, z);
        let t = b.and(x, y_or_z);
        let m2 = b.or(t, yz);
        assert_eq!(m1, m2, "canonicity: same function, same Ref");
        // The median BDD: one x-node, two y-nodes, one z-node, two sinks.
        assert_eq!(b.node_count(m1), 6);
        // Majority of 3 has 3 models with two 1s plus 1 with three 1s.
        assert_eq!(b.count_models(m1, 3), 4);
    }

    #[test]
    fn count_models_skip_factor() {
        // f = x3 over 10 variables: the root skips x0..x2 (factor 2^3) and
        // the edges to the sinks skip x4..x9 (factor 2^6): 2^9 models.
        let mut b = Bdd::new();
        let f = b.variable(3);
        assert_eq!(b.count_models(f, 10), 1 << 9);
        let t = b.constant(true);
        assert_eq!(b.count_models(t, 10), 1 << 10);
        assert_eq!(b.count_models(t, 0), 1);
    }

    #[test]
    fn ordering_experiment_exact_sizes() {
        // f = OR of 8 conjunctions x_a AND x_b over 16 variables.
        // Good order (pairs adjacent): B(f) = 2k + 2 = 18.
        // Bad order (all firsts, then all seconds): B(f) = 2^(k+1) = 512.
        let k = 8u32;
        let mut good = Bdd::new();
        let mut f = good.constant(false);
        for i in 0..k {
            let a = good.variable(2 * i);
            let b = good.variable(2 * i + 1);
            let ab = good.and(a, b);
            f = good.or(f, ab);
        }
        assert_eq!(good.node_count(f), 2 * k as usize + 2);
        assert_eq!(good.count_models(f, 16), (1u128 << 16) - 3u128.pow(8));

        let mut bad = Bdd::new();
        let mut g = bad.constant(false);
        for i in 0..k {
            let a = bad.variable(i);
            let b = bad.variable(k + i);
            let ab = bad.and(a, b);
            g = bad.or(g, ab);
        }
        assert_eq!(bad.node_count(g), 512);
        assert_eq!(bad.count_models(g, 16), (1u128 << 16) - 3u128.pow(8));
    }

    #[test]
    fn independent_sets_and_queens() {
        // Path P_4: F_6 = 8 independent sets; cycle C_5: L_5 = 11.
        assert_eq!(independent_set_count(4, &[(0, 1), (1, 2), (2, 3)]), 8);
        assert_eq!(
            independent_set_count(5, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)]),
            11
        );
        assert_eq!(independent_set_count(3, &[]), 8);
        assert_eq!(queens_bdd_count(1), 1);
        assert_eq!(queens_bdd_count(4), 2);
        assert_eq!(queens_bdd_count(5), 10);
    }
}
