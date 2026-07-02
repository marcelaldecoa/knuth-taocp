//! Module 21 — Boolean Functions and Optimal Evaluation.
//! Source: TAOCP Vol. 4A, §7.1.1 (Boolean basics) and §7.1.2 (Boolean chains).
//!
//! YOUR WORKSPACE. Replace each `todo!()` with an implementation, then run
//! `./grade 21`. Work the stages in order; the lesson
//! (`course/module-21-boolean/README.md`) walks you through the theory.
//!
//! # The truth-table-as-integer convention (used everywhere below)
//!
//! An `n`-variable Boolean function `f(x_1, ..., x_n)` is stored as a single
//! `u64` truth table: bit `i` of `table` equals `f(x)` where the integer `i`
//! encodes the argument, `x_j = (i >> (j-1)) & 1`. With `n <= 6` the whole
//! function fits in one word, and function algebra becomes bitwise machine
//! instructions — the trick Knuth leans on throughout Volume 4A.

// ===========================================================================
// Stage 1 — Truth tables and normal forms (§7.1.1)
// ===========================================================================

/// An `n`-variable Boolean function stored as its truth table (bit `i` is
/// `f(i)`). Equal iff they agree on every input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoolFunc {
    /// Number of variables, `0 <= n <= 6`.
    pub n: u32,
    /// Truth table: bit `i` is `f(i)`. Only the low `2^n` bits are meaningful.
    pub table: u64,
}

impl BoolFunc {
    /// Build a function by tabulating a closure over all `2^n` inputs: set bit
    /// `x` of the table iff `f(x)` is true.
    pub fn from_closure(n: u32, f: impl Fn(u32) -> bool) -> BoolFunc {
        let _ = (n, f);
        todo!("tabulate f over 0..2^n into a u64")
    }

    /// `f(x)` — read bit `x` of the truth table.
    pub fn eval(&self, x: u32) -> bool {
        let _ = x;
        todo!("read bit x of self.table")
    }

    /// Number of minterms — inputs where `f` is true (= `popcount(table)`).
    pub fn num_minterms(&self) -> u32 {
        todo!("popcount of the meaningful table bits")
    }

    /// The complement `¬f`: flip every meaningful table bit.
    pub fn complement(&self) -> BoolFunc {
        todo!("bitwise-not the table, masked to 2^n bits")
    }

    /// DNF as a list of **minterms** — one full assignment per row where
    /// `f = 1`, written as signed literals (`+j` if `x_j` true, `-j` if
    /// false). `to_dnf().len()` equals `num_minterms()`.
    pub fn to_dnf(&self) -> Vec<Vec<i32>> {
        todo!("emit a minterm for each 1-row of the table")
    }

    /// CNF as a list of **maxterms** — one clause per row where `f = 0`, false
    /// exactly at that row (so `x_j` appears as `-j` when true, `+j` when
    /// false). `f` is the AND of its maxterms.
    pub fn to_cnf(&self) -> Vec<Vec<i32>> {
        todo!("emit a maxterm for each 0-row of the table")
    }

    /// Reconstruct a function from a DNF (list of product terms, each an AND
    /// of literals). `f(x)` is true iff some term is satisfied at `x`.
    pub fn from_dnf(n: u32, terms: &[Vec<i32>]) -> BoolFunc {
        let _ = (n, terms);
        todo!("OR the product terms over all inputs")
    }

    /// Reconstruct a function from a CNF (list of clauses, each an OR of
    /// literals). `f(x)` is true iff every clause is satisfied at `x`.
    pub fn from_cnf(n: u32, clauses: &[Vec<i32>]) -> BoolFunc {
        let _ = (n, clauses);
        todo!("AND the clauses over all inputs")
    }
}

// ===========================================================================
// Stage 2 — Boolean chains and combinational cost (§7.1.2)
// ===========================================================================
//
// Gate encoding: every 2-input gate is one of the 16 binary operations, a
// 4-bit truth table `op`. For inputs (a, b) the output is bit `2*a + b` of
// `op`, so AND = 0b1000, XOR = 0b0110, and so on.

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

/// Apply the 2-input gate `op` to Boolean inputs `a`, `b` (output = bit
/// `2*a + b` of `op`).
pub fn apply_gate(op: u8, a: bool, b: bool) -> bool {
    let _ = (op, a, b);
    todo!("index bit 2*a+b of op")
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
/// `x_{i+1}`); value `n + k` is produced by `steps[k]`. Each step references
/// strictly earlier values. The chain's result is the value at `output`.
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
    /// A chain over `n` inputs with no gates (output defaults to `x_1`).
    pub fn new(n: u32) -> Chain {
        let _ = n;
        todo!("Chain with n inputs, empty steps, output = 0")
    }

    /// Append a gate `op(value[left], value[right])`; return its value index
    /// (`n + steps.len()`) and make it the current output.
    pub fn gate(&mut self, op: u8, left: usize, right: usize) -> usize {
        let _ = (op, left, right);
        todo!("push a Step, set output, return the new value index")
    }

    /// Choose which value is the chain's output.
    pub fn set_output(&mut self, idx: usize) {
        let _ = idx;
        todo!("set self.output")
    }
}

/// Evaluate a chain on input `x` (bit `i` of `x` is `x_{i+1}`).
pub fn eval_chain(chain: &Chain, x: u32) -> bool {
    let _ = (chain, x);
    todo!("fill inputs, run each gate, return values[output]")
}

/// Number of gates in the chain — its combinational cost.
pub fn chain_cost(chain: &Chain) -> usize {
    let _ = chain;
    todo!("count the steps")
}

/// Does the chain compute `f`? Check agreement on all `2^n` inputs.
pub fn chain_computes(chain: &Chain, f: &BoolFunc) -> bool {
    let _ = (chain, f);
    todo!("compare eval_chain vs f.eval over every input")
}

// ===========================================================================
// Stage 3 — Median, threshold, and symmetric functions (§7.1.1)
// ===========================================================================

/// The majority (median) of a bit vector: true iff strictly more than half
/// the bits are true.
pub fn majority(bits: &[bool]) -> bool {
    let _ = bits;
    todo!("2 * popcount > len")
}

/// The threshold function: true iff at least `k` of the bits are true.
pub fn threshold_at_least(bits: &[bool], k: usize) -> bool {
    let _ = (bits, k);
    todo!("popcount >= k")
}

/// A symmetric function of `n` variables: `weights[j]` is the value when
/// exactly `j` inputs are true (`weights.len() == n + 1`).
pub fn symmetric_function(n: u32, weights: &[bool]) -> BoolFunc {
    let _ = (n, weights);
    todo!("value at x = weights[popcount(x)]")
}

/// Is `f` monotone (nondecreasing)? Raising any input from 0 to 1 never drops
/// the output from 1 to 0.
pub fn is_monotone(f: &BoolFunc) -> bool {
    let _ = f;
    todo!("check f(x) <= f(x | bit b) for every x and clear bit b")
}

/// Is `f` self-dual? `f(¬x) = ¬f(x)` for every input (complementing all `n`
/// argument bits).
pub fn is_self_dual(f: &BoolFunc) -> bool {
    let _ = f;
    todo!("check f(x) != f(complement of x) for every input")
}

// ===========================================================================
// Stage 4 — Optimum chains for small functions (§7.1.2)
// ===========================================================================

/// The full 2-input basis: all 16 binary operations.
pub fn full_basis() -> Vec<u8> {
    todo!("the numbers 0..16")
}

/// A "standard" basis {AND, OR, NOT} (NOT as the 2-input gate `NOTL`).
pub fn standard_basis() -> Vec<u8> {
    todo!("[AND, OR, NOTL]")
}

/// The combinational complexity `C(f)`: the minimum number of gates in a
/// Boolean chain over `basis` that computes `f`.
///
/// BFS over *states*, where a state is the set of functions a real chain has
/// computed so far (start = constants + projections). One move appends a gate
/// `state ∪ { g(a, b) }` at cost `+1`. The first state containing `f` gives
/// `C(f)`. (A per-function frontier undercounts — see the lesson.) Keep
/// `n <= 3`.
pub fn optimal_cost(f: &BoolFunc, basis: &[u8]) -> usize {
    let _ = (f, basis);
    todo!("BFS over sets of reachable functions")
}
