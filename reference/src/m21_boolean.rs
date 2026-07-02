//! Module 21 — Boolean Functions and Optimal Evaluation.
//! Source: TAOCP Vol. 4A, §7.1.1 (Boolean basics) and §7.1.2 (Boolean
//! evaluation / optimum chains).
//!
//! # The truth-table-as-integer convention
//!
//! An `n`-variable Boolean function `f(x_1, ..., x_n)` is stored as a single
//! `u64` "truth table": bit `i` of `table` equals `f(x)` where the integer
//! `i` is the binary encoding of the argument, `x_j = (i >> (j-1)) & 1`.
//! Since `n <= 6` we have `2^n <= 64`, so the whole function fits in one word.
//!
//! This is the representation Knuth uses relentlessly in Volume 4A, because it
//! turns *function algebra* into *bitwise machine instructions*: the AND of
//! two functions is the AND of their tables, complement is `!table`, and so
//! on. A single 64-bit register manipulates all 2^n input rows in parallel.

use std::collections::HashSet;
use std::collections::VecDeque;

/// Number of truth-table bits for an `n`-variable function, i.e. `2^n`.
#[inline]
fn rows(n: u32) -> u32 {
    1u32 << n
}

/// Low-`2^n`-bits mask. Handles `n == 6` (`2^n == 64`) without overflow.
#[inline]
fn table_mask(n: u32) -> u64 {
    let bits = rows(n);
    if bits >= 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}

// ===========================================================================
// Stage 1 — Truth tables and normal forms (§7.1.1)
// ===========================================================================

/// An `n`-variable Boolean function stored as its truth table (see module
/// docs). Two `BoolFunc`s are equal iff they agree on every input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoolFunc {
    /// Number of variables, `0 <= n <= 6`.
    pub n: u32,
    /// Truth table: bit `i` is `f(i)`. Only the low `2^n` bits are meaningful.
    pub table: u64,
}

impl BoolFunc {
    /// Build a function by tabulating a closure over all `2^n` inputs.
    ///
    /// This is the bridge from "a rule" to "a table": we evaluate `f` once per
    /// input row and pack the answers into a word.
    pub fn from_closure(n: u32, f: impl Fn(u32) -> bool) -> BoolFunc {
        assert!(n <= 6, "n must be <= 6 (2^n <= 64)");
        let mut table = 0u64;
        for x in 0..rows(n) {
            if f(x) {
                table |= 1u64 << x;
            }
        }
        BoolFunc { n, table }
    }

    /// `f(x)` — read bit `x` of the truth table.
    #[inline]
    pub fn eval(&self, x: u32) -> bool {
        debug_assert!(x < rows(self.n));
        (self.table >> x) & 1 == 1
    }

    /// The number of minterms — inputs where `f` is true. Equals
    /// `popcount(table)`.
    pub fn num_minterms(&self) -> u32 {
        (self.table & table_mask(self.n)).count_ones()
    }

    /// The complement `¬f`, obtained by flipping every meaningful table bit.
    /// Function complement = bitwise complement of the table (masked).
    pub fn complement(&self) -> BoolFunc {
        BoolFunc { n: self.n, table: !self.table & table_mask(self.n) }
    }

    /// Disjunctive normal form as a list of **minterms**. Each minterm is a
    /// full assignment written as signed literals: variable `j` (1-based)
    /// appears as `+j` when it is true in that row, `-j` when false.
    ///
    /// `f` is the OR of its minterms; there is exactly one per input where
    /// `f = 1`, so `to_dnf().len() == num_minterms()`.
    pub fn to_dnf(&self) -> Vec<Vec<i32>> {
        let mut terms = Vec::new();
        for x in 0..rows(self.n) {
            if self.eval(x) {
                terms.push(assignment_literals(self.n, x));
            }
        }
        terms
    }

    /// Conjunctive normal form as a list of **maxterms**. Each maxterm is a
    /// clause (OR of literals) that is false at exactly one input — the rows
    /// where `f = 0`. In the clause for row `x`, variable `j` appears as `-j`
    /// when it is true in `x` and `+j` when false, so the clause vanishes
    /// only at `x`. `f` is the AND of its maxterms.
    pub fn to_cnf(&self) -> Vec<Vec<i32>> {
        let mut clauses = Vec::new();
        for x in 0..rows(self.n) {
            if !self.eval(x) {
                // Negate each literal of the assignment so the clause is 0 at x.
                clauses.push(assignment_literals(self.n, x).iter().map(|l| -l).collect());
            }
        }
        clauses
    }

    /// Reconstruct a function from a DNF: a list of **product terms** (each an
    /// AND of literals). `f` is true at `x` iff *some* term is satisfied at
    /// `x`. Terms need not be full minterms — a shorter term covers a whole
    /// subcube. An empty term list is the constant `0`; an empty term is the
    /// constant `1`.
    pub fn from_dnf(n: u32, terms: &[Vec<i32>]) -> BoolFunc {
        BoolFunc::from_closure(n, |x| terms.iter().any(|t| term_satisfied(t, x)))
    }

    /// Reconstruct a function from a CNF: a list of **clauses** (each an OR of
    /// literals). `f` is true at `x` iff *every* clause is satisfied. An empty
    /// clause list is the constant `1`; an empty clause is the constant `0`.
    pub fn from_cnf(n: u32, clauses: &[Vec<i32>]) -> BoolFunc {
        BoolFunc::from_closure(n, |x| clauses.iter().all(|c| clause_satisfied(c, x)))
    }
}

/// The full assignment of `x` as signed literals `+j`/`-j`, `j = 1..=n`.
fn assignment_literals(n: u32, x: u32) -> Vec<i32> {
    (1..=n)
        .map(|j| {
            let bit = (x >> (j - 1)) & 1;
            if bit == 1 {
                j as i32
            } else {
                -(j as i32)
            }
        })
        .collect()
}

/// Is literal `l` (signed, 1-based) satisfied by assignment `x`?
#[inline]
fn literal_holds(l: i32, x: u32) -> bool {
    let j = l.unsigned_abs();
    let bit = (x >> (j - 1)) & 1 == 1;
    (l > 0) == bit
}

/// A product term (AND of literals) is satisfied iff *all* its literals hold.
fn term_satisfied(term: &[i32], x: u32) -> bool {
    term.iter().all(|&l| literal_holds(l, x))
}

/// A clause (OR of literals) is satisfied iff *some* literal holds.
fn clause_satisfied(clause: &[i32], x: u32) -> bool {
    clause.iter().any(|&l| literal_holds(l, x))
}

// ===========================================================================
// Stage 2 — Boolean chains and combinational cost (§7.1.2)
// ===========================================================================
//
// Gate encoding: every 2-input Boolean gate is one of the 16 binary
// operations, encoded as a 4-bit truth table `op`. For inputs (a, b) the
// output is bit `2*a + b` of `op`:
//
//     index = 2*a + b :   (0,0)->0  (0,1)->1  (1,0)->2  (1,1)->3
//
// So `AND` (true only at (1,1)) is `0b1000 = 8`, `XOR` is `0b0110 = 6`, etc.

/// `op = 0b0000` — constant 0.
pub const FALSE: u8 = 0b0000;
/// `op = 0b0001` — NOR.
pub const NOR: u8 = 0b0001;
/// `op = 0b0011` — NOT of the left input (ignores right).
pub const NOTL: u8 = 0b0011;
/// `op = 0b0101` — NOT of the right input (ignores left).
pub const NOTR: u8 = 0b0101;
/// `op = 0b0110` — XOR.
pub const XOR: u8 = 0b0110;
/// `op = 0b0111` — NAND.
pub const NAND: u8 = 0b0111;
/// `op = 0b1000` — AND.
pub const AND: u8 = 0b1000;
/// `op = 0b1001` — XNOR (equality).
pub const XNOR: u8 = 0b1001;
/// `op = 0b1010` — the right input (projection).
pub const RIGHT: u8 = 0b1010;
/// `op = 0b1100` — the left input (projection).
pub const LEFT: u8 = 0b1100;
/// `op = 0b1110` — OR.
pub const OR: u8 = 0b1110;
/// `op = 0b1111` — constant 1.
pub const TRUE: u8 = 0b1111;

/// Apply the 2-input gate `op` to Boolean inputs `a`, `b`.
#[inline]
pub fn apply_gate(op: u8, a: bool, b: bool) -> bool {
    let idx = 2 * (a as u8) + (b as u8);
    (op >> idx) & 1 == 1
}

/// One gate of a Boolean chain: `op(value[left], value[right])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    /// 4-bit gate truth table (see the gate constants).
    pub op: u8,
    /// Index of the left operand among earlier values.
    pub left: usize,
    /// Index of the right operand among earlier values.
    pub right: usize,
}

/// A Boolean chain (§7.1.2): a straight-line program of 2-input gates.
///
/// Values are indexed `0..n` for the inputs `x_1..x_n` (value `i` is input
/// `x_{i+1}`), and value `n + k` is produced by `steps[k]`. Every step may
/// only reference strictly earlier value indices. The chain's result is the
/// value at index `output`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chain {
    /// Number of input variables.
    pub n: u32,
    /// The gate steps, in evaluation order.
    pub steps: Vec<Step>,
    /// Value index selected as the chain's output.
    pub output: usize,
}

impl Chain {
    /// A chain over `n` inputs with no gates. `output` defaults to input
    /// `x_1` (value 0); set it explicitly once you add gates.
    pub fn new(n: u32) -> Chain {
        Chain { n, steps: Vec::new(), output: 0 }
    }

    /// Append a gate `op(value[left], value[right])`, returning the value
    /// index of the new step. The new step also becomes the current `output`,
    /// so the last gate you add is the result unless you say otherwise.
    pub fn gate(&mut self, op: u8, left: usize, right: usize) -> usize {
        let idx = self.n as usize + self.steps.len();
        assert!(left < idx && right < idx, "gate must reference earlier values");
        self.steps.push(Step { op, left, right });
        self.output = idx;
        idx
    }

    /// Choose which value is the chain's output.
    pub fn set_output(&mut self, idx: usize) {
        self.output = idx;
    }
}

/// Evaluate a chain on input `x` (bit `i` of `x` is `x_{i+1}`).
pub fn eval_chain(chain: &Chain, x: u32) -> bool {
    let n = chain.n as usize;
    let mut values = Vec::with_capacity(n + chain.steps.len());
    // Inputs.
    for i in 0..n {
        values.push((x >> i) & 1 == 1);
    }
    // Gate steps.
    for step in &chain.steps {
        let v = apply_gate(step.op, values[step.left], values[step.right]);
        values.push(v);
    }
    values[chain.output]
}

/// Number of gates in the chain — its **combinational cost**.
pub fn chain_cost(chain: &Chain) -> usize {
    chain.steps.len()
}

/// Does the chain compute `f`? Checks agreement on every one of the `2^n`
/// inputs (the only honest way to certify a straight-line program).
pub fn chain_computes(chain: &Chain, f: &BoolFunc) -> bool {
    if chain.n != f.n {
        return false;
    }
    for x in 0..rows(f.n) {
        if eval_chain(chain, x) != f.eval(x) {
            return false;
        }
    }
    true
}

// ===========================================================================
// Stage 3 — Median, threshold, and symmetric functions (§7.1.1)
// ===========================================================================

/// The **majority** (median) of a bit vector: true iff strictly more than
/// half the bits are true. For an odd-length vector this is the median value.
pub fn majority(bits: &[bool]) -> bool {
    let ones = bits.iter().filter(|&&b| b).count();
    2 * ones > bits.len()
}

/// The threshold function: true iff at least `k` of the bits are true.
pub fn threshold_at_least(bits: &[bool], k: usize) -> bool {
    bits.iter().filter(|&&b| b).count() >= k
}

/// A **symmetric** function of `n` variables, whose value depends only on how
/// many inputs are true. `weights[j]` is the value when exactly `j` inputs are
/// true, so `weights.len()` must be `n + 1`.
///
/// The compact representation is the whole point: any symmetric function of
/// `n` variables is pinned down by just `n + 1` bits, not `2^n`.
pub fn symmetric_function(n: u32, weights: &[bool]) -> BoolFunc {
    assert_eq!(weights.len(), n as usize + 1, "need one weight per popcount 0..=n");
    BoolFunc::from_closure(n, |x| weights[x.count_ones() as usize])
}

/// Is `f` **monotone** (nondecreasing)? I.e. flipping any input from 0 to 1
/// never turns the output from 1 to 0. It suffices to check single-bit
/// supersets: for every `x` and every clear bit `b`, `f(x) <= f(x | bit b)`.
pub fn is_monotone(f: &BoolFunc) -> bool {
    for x in 0..rows(f.n) {
        for b in 0..f.n {
            if (x >> b) & 1 == 0 {
                let y = x | (1 << b);
                // Raising input b must not drop the output from 1 to 0.
                if f.eval(x) && !f.eval(y) {
                    return false;
                }
            }
        }
    }
    true
}

/// Is `f` **self-dual**? I.e. `f(¬x) = ¬f(x)` for every input, where `¬x`
/// complements all `n` argument bits. Self-dual functions are exactly those
/// that commute with complementation of both inputs and output.
pub fn is_self_dual(f: &BoolFunc) -> bool {
    let argmask = rows(f.n) - 1; // low-n-bit mask on the *argument*
    for x in 0..rows(f.n) {
        let xbar = (!x) & argmask;
        if f.eval(x) == f.eval(xbar) {
            return false;
        }
    }
    true
}

// ===========================================================================
// Stage 4 — Optimum chains for small functions (§7.1.2)
// ===========================================================================

/// The full 2-input basis: all 16 binary operations.
pub fn full_basis() -> Vec<u8> {
    (0u8..16).collect()
}

/// A "standard" basis {AND, OR, NOT}. NOT is provided as the 2-input gate
/// `NOTL` (complement the left operand, ignore the right).
pub fn standard_basis() -> Vec<u8> {
    vec![AND, OR, NOTL]
}

/// Apply gate `op` elementwise to two truth tables at once, using the same
/// `index = 2*a + b` convention as [`apply_gate`]. All `2^n` rows are computed
/// in parallel with a handful of bitwise instructions.
fn combine_tables(op: u8, a: u64, b: u64, mask: u64) -> u64 {
    let mut r = 0u64;
    if op & 0b0001 != 0 {
        r |= !a & !b; // (a,b) = (0,0)
    }
    if op & 0b0010 != 0 {
        r |= !a & b; //  (0,1)
    }
    if op & 0b0100 != 0 {
        r |= a & !b; //  (1,0)
    }
    if op & 0b1000 != 0 {
        r |= a & b; //   (1,1)
    }
    r & mask
}

/// Truth table of the projection `x_{j+1}` over `n` variables.
fn projection_table(j: u32, n: u32, mask: u64) -> u64 {
    let mut t = 0u64;
    for x in 0..rows(n) {
        if (x >> j) & 1 == 1 {
            t |= 1u64 << x;
        }
    }
    t & mask
}

/// The **combinational complexity** `C(f)`: the minimum number of gates in a
/// Boolean chain (using gates from `basis`) that computes `f`.
///
/// # Why the naive "frontier of functions" search is wrong
///
/// A tempting shortcut is to grow a set `R_c` of "functions reachable with `c`
/// gates" by `R_c = R_{c-1} ∪ { g(a, b) : a, b ∈ R_{c-1} }`. That *undercounts*:
/// it treats the two operands `a` and `b` as simultaneously available for free,
/// but each may itself need gates, and those subcircuits do not always share.
/// For majority-of-three the shortcut reports 3, yet no 3-gate chain exists —
/// the true cost is 4.
///
/// # The correct search
///
/// We do BFS over **states**, where a state is the *set of functions a real
/// chain has computed so far* (a chain keeps every intermediate value
/// available, so sharing is automatic). The start state is the free set — the
/// two constants and the `n` projections. One move appends a gate:
/// `state ∪ { g(a, b) }` for `a, b` already in the state, at cost `+1`. The
/// first state containing `f`'s table gives `C(f)`. Deduplicating states by
/// their sorted contents keeps the search finite and, for `n <= 3`, fast.
///
/// Keep `n <= 3`: `n = 4` blows the state space up.
pub fn optimal_cost(f: &BoolFunc, basis: &[u8]) -> usize {
    let n = f.n;
    let mask = table_mask(n);
    let target = f.table & mask;

    // The free starting set: both constants and every projection (cost 0).
    let mut base = vec![0u64, mask];
    for j in 0..n {
        base.push(projection_table(j, n, mask));
    }
    base.sort_unstable();
    base.dedup();
    if base.contains(&target) {
        return 0;
    }

    // BFS over reachable *sets* of functions. A set is canonicalized as a
    // sorted Vec so equal sets hash equal.
    let mut seen: HashSet<Vec<u64>> = HashSet::new();
    let mut queue: VecDeque<(Vec<u64>, usize)> = VecDeque::new();
    seen.insert(base.clone());
    queue.push_back((base, 0));

    while let Some((state, cost)) = queue.pop_front() {
        for &a in &state {
            for &b in &state {
                for &g in basis {
                    let t = combine_tables(g, a, b, mask);
                    if state.contains(&t) {
                        continue; // no new function — skip
                    }
                    if t == target {
                        return cost + 1;
                    }
                    let mut next = state.clone();
                    next.push(t);
                    next.sort_unstable();
                    if seen.insert(next.clone()) {
                        queue.push_back((next, cost + 1));
                    }
                }
            }
        }
    }
    unreachable!("every function of n variables is reachable")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Stage 1 ---------------------------------------------------------

    fn and2(x: u32) -> bool {
        (x & 1 == 1) && ((x >> 1) & 1 == 1)
    }
    fn or2(x: u32) -> bool {
        (x & 1 == 1) || ((x >> 1) & 1 == 1)
    }
    fn xor2(x: u32) -> bool {
        (x & 1) ^ ((x >> 1) & 1) == 1
    }

    #[test]
    fn eval_matches_table() {
        let f = BoolFunc::from_closure(2, and2);
        // AND truth table over (x1=bit0, x2=bit1): only x=0b11 is true.
        assert_eq!(f.table, 0b1000);
        assert!(!f.eval(0) && !f.eval(1) && !f.eval(2) && f.eval(3));
        assert_eq!(f.num_minterms(), 1);
    }

    #[test]
    fn dnf_cnf_of_or() {
        let f = BoolFunc::from_closure(2, or2);
        // OR is true on 3 rows: DNF has 3 minterms, CNF has the 1 false row.
        assert_eq!(f.to_dnf().len(), 3);
        assert_eq!(f.to_cnf().len(), 1);
        assert_eq!(BoolFunc::from_dnf(2, &f.to_dnf()), f);
        assert_eq!(BoolFunc::from_cnf(2, &f.to_cnf()), f);
    }

    #[test]
    fn dnf_roundtrip_all_small() {
        for n in 0..=4u32 {
            for table in 0..rows(n) {
                let f = BoolFunc { n, table: table as u64 };
                assert_eq!(BoolFunc::from_dnf(n, &f.to_dnf()), f);
                assert_eq!(BoolFunc::from_cnf(n, &f.to_cnf()), f);
                assert_eq!(f.to_dnf().len() as u32, f.num_minterms());
            }
        }
    }

    #[test]
    fn de_morgan_via_complement() {
        // ¬(a AND b) == NAND; ¬(a OR b) == NOR, checked as whole truth tables.
        let a_and_b = BoolFunc::from_closure(2, and2);
        let a_or_b = BoolFunc::from_closure(2, or2);
        assert_eq!(a_and_b.complement(), BoolFunc { n: 2, table: 0b0111 });
        assert_eq!(a_or_b.complement(), BoolFunc { n: 2, table: 0b0001 });
    }

    #[test]
    fn constant_and_tautology() {
        let zero = BoolFunc { n: 3, table: 0 };
        let one = BoolFunc { n: 3, table: table_mask(3) };
        assert_eq!(zero.to_dnf().len(), 0);
        assert_eq!(one.to_cnf().len(), 0);
        assert_eq!(BoolFunc::from_dnf(3, &zero.to_dnf()), zero);
        assert_eq!(BoolFunc::from_cnf(3, &one.to_cnf()), one);
    }

    #[test]
    fn xor_dnf() {
        let f = BoolFunc::from_closure(2, xor2);
        assert_eq!(f.num_minterms(), 2);
        assert_eq!(BoolFunc::from_dnf(2, &f.to_dnf()), f);
    }

    // --- Stage 2 ---------------------------------------------------------

    #[test]
    fn xor_chain_from_and_or_not() {
        // XOR = (x1 OR x2) AND NOT(x1 AND x2).
        let mut c = Chain::new(2);
        let or = c.gate(OR, 0, 1);
        let and = c.gate(AND, 0, 1);
        let nand = c.gate(NOTL, and, and);
        let out = c.gate(AND, or, nand);
        c.set_output(out);
        let f = BoolFunc::from_closure(2, xor2);
        assert!(chain_computes(&c, &f));
        assert_eq!(chain_cost(&c), 4);
    }

    #[test]
    fn majority3_carry_chain() {
        // Carry / majority of three: (x1 AND x2) OR (x3 AND (x1 OR x2)).
        let mut c = Chain::new(3);
        let ab = c.gate(AND, 0, 1);
        let aorb = c.gate(OR, 0, 1);
        let t = c.gate(AND, 2, aorb);
        c.gate(OR, ab, t);
        let maj = BoolFunc::from_closure(3, |x| x.count_ones() >= 2);
        assert!(chain_computes(&c, &maj));
        assert_eq!(chain_cost(&c), 4);
    }

    #[test]
    fn full_adder_sum_is_xor3() {
        // Sum bit of a full adder = x1 XOR x2 XOR x3.
        let mut c = Chain::new(3);
        let t = c.gate(XOR, 0, 1);
        c.gate(XOR, t, 2);
        let sum = BoolFunc::from_closure(3, |x| x.count_ones() % 2 == 1);
        assert!(chain_computes(&c, &sum));
        assert_eq!(chain_cost(&c), 2);
    }

    // --- Stage 3 ---------------------------------------------------------

    #[test]
    fn majority_and_threshold() {
        assert!(majority(&[true, true, false]));
        assert!(!majority(&[true, false, false]));
        for bits in [
            [false, false, false],
            [true, false, false],
            [false, true, true],
            [true, true, true],
        ] {
            let k = (bits.len() + 1) / 2;
            assert_eq!(majority(&bits), threshold_at_least(&bits, k));
        }
    }

    #[test]
    fn symmetric_reconstructs_majority() {
        // majority-of-5: value = 1 when popcount >= 3.
        let weights: Vec<bool> = (0..=5).map(|j| j >= 3).collect();
        let sym = symmetric_function(5, &weights);
        let maj = BoolFunc::from_closure(5, |x| x.count_ones() >= 3);
        assert_eq!(sym, maj);
    }

    #[test]
    fn monotone_and_self_dual_flags() {
        let and2f = BoolFunc::from_closure(2, and2);
        let or2f = BoolFunc::from_closure(2, or2);
        let xor2f = BoolFunc::from_closure(2, xor2);
        let maj3 = BoolFunc::from_closure(3, |x| x.count_ones() >= 2);
        let dict = BoolFunc::from_closure(3, |x| x & 1 == 1); // x1

        assert!(is_monotone(&and2f) && is_monotone(&or2f) && is_monotone(&maj3));
        assert!(!is_monotone(&xor2f));

        assert!(is_self_dual(&maj3)); // majority of odd count
        assert!(is_self_dual(&dict)); // a dictatorship
        assert!(!is_self_dual(&and2f));
    }

    #[test]
    fn dedekind_numbers() {
        let expected = [2u64, 3, 6, 20, 168];
        for n in 0..=4u32 {
            let total = 1u64 << rows(n); // 2^(2^n) functions
            let mut count = 0u64;
            for table in 0..total {
                if is_monotone(&BoolFunc { n, table }) {
                    count += 1;
                }
            }
            assert_eq!(count, expected[n as usize], "Dedekind number for n={n}");
        }
    }

    // --- Stage 4 ---------------------------------------------------------

    #[test]
    fn trivial_costs() {
        let basis = full_basis();
        let zero = BoolFunc { n: 3, table: 0 };
        let x1 = BoolFunc::from_closure(3, |x| x & 1 == 1);
        assert_eq!(optimal_cost(&zero, &basis), 0);
        assert_eq!(optimal_cost(&x1, &basis), 0);
    }

    #[test]
    fn xor2_costs_one() {
        let basis = full_basis();
        let f = BoolFunc::from_closure(2, xor2);
        assert_eq!(optimal_cost(&f, &basis), 1);
    }

    #[test]
    fn all_two_variable_functions_cheap() {
        let basis = full_basis();
        for table in 0..16u64 {
            let f = BoolFunc { n: 2, table };
            assert!(optimal_cost(&f, &basis) <= 1);
        }
    }

    #[test]
    fn majority3_and_xor3_optimal() {
        let basis = full_basis();
        let maj = BoolFunc::from_closure(3, |x| x.count_ones() >= 2);
        let xor3 = BoolFunc::from_closure(3, |x| x.count_ones() % 2 == 1);
        assert_eq!(optimal_cost(&xor3, &basis), 2);
        assert_eq!(optimal_cost(&maj, &basis), 4);
    }
}
