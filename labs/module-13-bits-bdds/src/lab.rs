//! Module 13 — Bitwise Tricks and Binary Decision Diagrams
//! (TAOCP Vol. 4A, §7.1.3–7.1.4).
//!
//! **Scaffolding tier — Module 05 and up:** the stub states the algorithm and
//! the contract and trusts you to translate it to Rust; the guided-tour aids of
//! Modules 01–04 are gone by design. The nets remain for every stage — the
//! lesson, three graduated hints (`--hint`), the reference, and the walkthrough.
//! (The taper is described in docs/for-newcomers.md §5.)
//!
//! YOUR WORKSPACE. Replace each `todo!()` with an implementation, then run
//! `./grade 13` from the repository root. The lesson in
//! `course/module-13-bits-bdds/README.md` develops all the theory you need.
//!
//! Ground rule for stage 1: the whole point is to *re-derive* the hardware
//! intrinsics, so do not call `count_ones`, `trailing_zeros`,
//! `leading_zeros`, or their cousins here. (The tests use them as oracles —
//! that's their job, not yours.)

// ============================================================================
// Stage 1 — §7.1.3 bitwise tricks
// ============================================================================

/// The ruler function ρ(x): the number of trailing zeros of `x`, i.e. the
/// position of the rightmost 1-bit. Must panic for x = 0 with a message
/// containing the word "undefined" (ρ(0) does not exist; definiteness!).
///
/// Do NOT call `x.trailing_zeros()`. Two honest routes (lesson, §1.4):
///
/// ```text
/// R1. [Isolate.]  b <- x & (-x).  (b = 2^a where a = rho(x); use
///                 x.wrapping_neg() for -x.)
/// R2. [Hash.]     Multiply b by a 64-bit de Bruijn constant, e.g.
///                 0x03f79d71b4cb0a89; the top 6 bits of the product are
///                 the a-th window of the de Bruijn cycle.
/// R3. [Look up.]  A 64-entry table maps that window back to a.
///                 (Build the table once: entry (constant << a) >> 58 is a.)
/// ```
///
/// Alternative: binary search with masks — test x & 0xFFFFFFFF to decide
/// whether the rightmost 1 is in the low 32 bits, then narrow to 16, 8, 4,
/// 2, 1, accumulating the shift. Either way, no intrinsics.
pub fn ruler(x: u64) -> u32 {
    let _ = x;
    todo!("implement the ruler function via de Bruijn multiply or mask binary search")
}

/// Sideways addition ν(x): how many 1-bits does `x` have? SWAR scheme of
/// §7.1.3 — no `count_ones` allowed:
///
/// ```text
/// S1. [Pairs.]    x <- x - ((x >> 1) & 0x5555555555555555);
/// S2. [Nibbles.]  x <- (x & 0x3333333333333333) + ((x >> 2) & 0x3333333333333333);
/// S3. [Bytes.]    x <- (x + (x >> 4)) & 0x0f0f0f0f0f0f0f0f;
/// S4. [Total.]    return (x.wrapping_mul(0x0101010101010101) >> 56) as u32;
/// ```
///
/// After S1 each 2-bit field holds the count of its own two bits (why:
/// ν(v) = v − ⌊v/2⌋ for 0 ≤ v < 4 — check all four cases). After S2/S3 each
/// nibble/byte holds its own count; S4 sums the eight bytes in one multiply
/// (safe because the total ≤ 64 < 256: no inter-byte carries).
pub fn sideways_addition(x: u64) -> u32 {
    let _ = x;
    todo!("implement SWAR sideways addition")
}

/// Extract the rightmost 1-bit: return `x & (-x)`, which is 2^ρ(x) for
/// x ≠ 0 and 0 for x = 0. Use `wrapping_neg`; the lesson proves the
/// identity from the two's-complement representation.
pub fn extract_rightmost_one(x: u64) -> u64 {
    let _ = x;
    todo!("return x AND (-x)")
}

/// Smear the rightmost 1-bit rightward: `x | (x − 1)`. Writing x = y10^a,
/// x − 1 = y01^a, so the result is y11^a: bits at or below the rightmost 1
/// become 1, higher bits are untouched. Use `wrapping_sub` so that x = 0
/// yields u64::MAX (the conventional all-ones value).
pub fn smear_right(x: u64) -> u64 {
    let _ = x;
    todo!("return x OR (x - 1) with wrapping subtraction")
}

/// Gosper's hack ("snoob"): the smallest integer greater than `x` with the
/// same number of 1-bits. Panic for x = 0 with a message containing
/// "weight 0" (there is nothing to do). The result is meaningful only when
/// a successor exists in 64 bits, i.e. when `x` is not the largest 64-bit
/// value of its weight (all 1s flush at the top) — outside that domain the
/// carry in G2 overflows; document it as the reference does.
///
/// ```text
/// G1. [Isolate.]      u <- x & (-x).
/// G2. [Carry.]        v <- x + u.       (the rightmost 1-run collapses;
///                                        one bit pops out just above it)
/// G3. [Redistribute.] return v | (((x ^ v) / u) >> 2).
/// ```
///
/// Trace 0110 → 1001 by hand before coding; the lesson tells the story
/// (smear, increment, redistribute the leftover 1s at the bottom).
pub fn next_same_weight(x: u64) -> u64 {
    let _ = x;
    todo!("implement Gosper's hack")
}

// ============================================================================
// Stages 2–3 — §7.1.4 reduced ordered BDDs
// ============================================================================

/// A handle to a BDD node: an index into your `Bdd` arena. The contract the
/// tests rely on:
///
/// - `Ref(0)` and `Ref(1)` are the ⊥ and ⊤ sinks;
/// - every `Ref(i)` with `i < bdd.len()` is a valid node;
/// - **hash-consing**: two `Ref`s from the same `Bdd` are equal (plain
///   `==`) iff they denote the same boolean function. Canonicity as an
///   executable theorem — the tests check `==` on `Ref`s relentlessly.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Ref(pub u32);

/// A shared arena of reduced ordered BDD nodes over variables
/// x0 < x1 < x2 < … (smaller index tested nearer the root).
///
/// Maintain the two reduction invariants of §7.1.4 *at every moment*:
/// 1. never create a node with `lo == hi` (return the child instead);
/// 2. never create a duplicate `(var, lo, hi)` node (keep a unique table —
///    `HashMap<(u32, Ref, Ref), Ref>` — and look before you push).
///
/// Suggested fields: `nodes: Vec<Node>` with `Node { var: u32, lo: Ref,
/// hi: Ref }` (sinks first, with a sentinel var such as `u32::MAX`), the
/// unique table, and a memo table `HashMap<(Op, Ref, Ref), Ref>` for apply.
/// Standard `HashMap`/`HashSet` are fine — the no-dependency rule bars
/// external crates only.
pub struct Bdd {
    // Your arena, unique table, and apply memo go here.
}

impl Bdd {
    /// A fresh arena containing only the sinks ⊥ = Ref(0) and ⊤ = Ref(1).
    pub fn new() -> Self {
        todo!("create the arena with the two sinks")
    }

    /// The constant function `b`: one of the two sinks.
    pub fn constant(&self, b: bool) -> Ref {
        let _ = b;
        todo!("return the sink for b")
    }

    /// Total number of nodes in the arena, sinks included. Every `Ref(i)`
    /// with `i < len()` must be valid — tests iterate the whole arena to
    /// check the reduction invariants.
    pub fn len(&self) -> usize {
        todo!("return the arena size")
    }

    /// True iff `f` is one of the two sinks.
    pub fn is_terminal(&self, f: Ref) -> bool {
        let _ = f;
        todo!("test for the sinks")
    }

    /// The variable tested at node `f`. Panic on a sink.
    pub fn var(&self, f: Ref) -> u32 {
        let _ = f;
        todo!("return the branch variable")
    }

    /// The x_v = 0 child of node `f`. Panic on a sink.
    pub fn low(&self, f: Ref) -> Ref {
        let _ = f;
        todo!("return the LO child")
    }

    /// The x_v = 1 child of node `f`. Panic on a sink.
    pub fn high(&self, f: Ref) -> Ref {
        let _ = f;
        todo!("return the HI child")
    }

    /// The projection function x_i: the unique node (i, ⊥, ⊤). Route it
    /// through your reduction-enforcing node maker ("mk"), like everything
    /// else.
    pub fn variable(&mut self, i: u32) -> Ref {
        let _ = i;
        todo!("make the node (i, Ref(0), Ref(1)) via mk")
    }

    /// Conjunction f ∧ g by memoized Shannon expansion (the lesson's
    /// Algorithm A):
    ///
    /// ```text
    /// A1. [Trivial?]  0 ∧ g = 0, 1 ∧ g = g, f ∧ f = f, and symmetrically.
    /// A2. [Memo.]     Return the cached result for (And, f, g) if any.
    /// A3. [Expand.]   v <- topmost (smallest) variable of f, g;
    ///                 split each operand: (f0, f1) = (low, high) if it
    ///                 tests v, else (f, f); recurse; combine with mk.
    /// A4. [Cache.]    Memoize and return.
    /// ```
    pub fn and(&mut self, f: Ref, g: Ref) -> Ref {
        let _ = (f, g);
        todo!("apply AND")
    }

    /// Disjunction f ∨ g — same skeleton as `and` with the dual trivial
    /// cases (1 ∨ g = 1, 0 ∨ g = g, f ∨ f = f). Factor out a shared
    /// `apply(op, f, g)` helper; only step A1 differs per operator.
    pub fn or(&mut self, f: Ref, g: Ref) -> Ref {
        let _ = (f, g);
        todo!("apply OR")
    }

    /// Exclusive or f ⊕ g (trivial cases: f ⊕ 0 = f, f ⊕ f = 0).
    pub fn xor(&mut self, f: Ref, g: Ref) -> Ref {
        let _ = (f, g);
        todo!("apply XOR")
    }

    /// Complement ¬f. One line once you have xor: ¬f = f ⊕ 1.
    pub fn not(&mut self, f: Ref) -> Ref {
        let _ = f;
        todo!("complement via xor with the true sink")
    }

    /// Evaluate the function rooted at `f` on `assignment` (`assignment[i]`
    /// is the value of x_i). Follow branches from the root until a sink —
    /// a BDD is its own O(n) evaluator.
    pub fn eval(&self, f: Ref, assignment: &[bool]) -> bool {
        let _ = (f, assignment);
        todo!("walk from f to a sink")
    }

    /// Number of distinct nodes reachable from `f`, sinks included — the
    /// BDD size B(f). `node_count(sink) == 1`. Depth-first search with a
    /// visited set.
    pub fn node_count(&self, f: Ref) -> usize {
        let _ = f;
        todo!("count reachable nodes")
    }

    /// Stage 3 — Algorithm 7.1.4C: the number of assignments of
    /// (x_0, …, x_{n_vars−1}) that satisfy `f`. All variables appearing in
    /// `f` must be < n_vars.
    ///
    /// ```text
    /// C1. [Sinks.]  c(⊥) = 0, c(⊤) = 1; a sink's level counts as n_vars.
    /// C2. [Nodes.]  For a node at level v with children l and h,
    ///               c = 2^(level(l)−v−1)·c(l) + 2^(level(h)−v−1)·c(h).
    /// C3. [Root.]   Answer 2^level(f) · c(f).
    /// ```
    ///
    /// The 2^k factors pay for *skipped* levels — an edge that jumps a
    /// variable leaves it free to be 0 or 1 (the lesson proves this
    /// weighting correct). Memoize per node — a local `HashMap<Ref, u128>`
    /// works fine under `&self` — so the cost is O(B(f)) arithmetic
    /// operations, the bound Knuth states for Algorithm 7.1.4C.
    pub fn count_models(&self, f: Ref, n_vars: usize) -> u128 {
        let _ = (f, n_vars);
        todo!("count satisfying assignments with level-skip weighting")
    }
}

// ============================================================================
// Stage 4 — §7.1.4 applications
// ============================================================================

/// The number of independent sets (the empty set included) of the graph
/// with vertices `0..n` and the given `edges`.
///
/// Plan: f = ⋀_{(u,v) ∈ E} ¬(x_u ∧ x_v), then `count_models(f, n)`.
/// Sanity anchors the tests use: path P_n gives Fibonacci F_{n+2}, cycle
/// C_n gives Lucas L_n, the empty graph gives 2^n, K_n gives n + 1.
pub fn independent_set_count(n: usize, edges: &[(usize, usize)]) -> u128 {
    let _ = (n, edges);
    todo!("AND the edge constraints and count models")
}

/// The number of n-queens solutions, computed by building one BDD over the
/// n² cell variables x_{r·n+c} and counting models:
///
/// - for every pair of distinct cells that attack each other (same row,
///   column, or diagonal): ¬(both occupied);
/// - for every row: at least one queen.
///
/// Together: exactly one queen per row and no attacks — the classical
/// problem. Expect 2, 10, 4 solutions for n = 4, 5, 6; the BDD blows up
/// quickly beyond that, so the tests cap n at 6.
pub fn queens_bdd_count(n: usize) -> u128 {
    let _ = n;
    todo!("build the queens BDD and count models")
}
