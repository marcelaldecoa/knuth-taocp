//! Module 10 — Satisfiability (TAOCP Vol. 4B, §7.2.2.2).
//!
//! **Scaffolding tier — Module 05 and up:** the stub states the algorithm and
//! the contract and trusts you to translate it to Rust; the guided-tour aids of
//! Modules 01–04 are gone by design. The nets remain for every stage — the
//! lesson, three graduated hints (`--hint`), the reference, and the walkthrough.
//! (The taper is described in docs/for-newcomers.md §5.)
//!
//! YOUR WORKSPACE. Replace each `todo!()` with an implementation, then run
//! `./grade 10` from the repository root. Work the stages in order — each test
//! file `tests/stage_NN_*.rs` corresponds to one stage, and the lesson in
//! `course/module-10-sat/README.md` walks you through the theory each stage
//! needs.
//!
//! Literal convention (DIMACS style): a literal is a nonzero `i32`; `+v` means
//! "variable v is true", `-v` means "variable v is false", for
//! `1 <= v <= num_vars`. A clause is a `Vec<i32>` (the disjunction of its
//! literals); a CNF formula is the conjunction of its clauses. The *empty
//! clause* is unsatisfiable; the *empty formula* is satisfied by anything.
//!
//! Complete assignments are `&[bool]` with `assignment[v - 1]` giving the
//! value of variable `v`. Partial assignments are `Vec<Option<bool>>`,
//! indexed the same way, `None` meaning "not yet assigned".

// ---------------------------------------------------------------------------
// Stage 1 — CNF representation and DIMACS
// ---------------------------------------------------------------------------

/// A formula in conjunctive normal form.
///
/// `num_vars` declares the universe of variables `1..=num_vars`; every literal
/// in `clauses` must satisfy `1 <= lit.abs() <= num_vars`. `parse_dimacs`
/// enforces this; if you build a `Cnf` by hand, keep the invariant yourself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cnf {
    pub num_vars: usize,
    pub clauses: Vec<Vec<i32>>,
}

impl Cnf {
    /// Stage 1 — Parse the DIMACS `cnf` format (the lingua franca of SAT
    /// solving, used for all of §7.2.2.2's benchmark experiments).
    ///
    /// Grammar accepted:
    /// - comment lines starting with `c` (skipped, anywhere);
    /// - exactly one header line `p cnf <num_vars> <num_clauses>`, which must
    ///   precede all clause data;
    /// - then whitespace-separated integer literals, each clause terminated
    ///   by `0`. Clauses may span lines and share lines; any whitespace
    ///   (spaces, tabs, newlines) separates tokens.
    ///
    /// Rejected with `Err(reason)`: a missing/duplicated/malformed header,
    /// clause data before the header, non-integer tokens, literals whose
    /// variable exceeds `num_vars`, an unterminated final clause, and a clause
    /// count differing from the header's declaration. A bare `0` yields an
    /// empty clause — legal DIMACS, and unsatisfiable.
    pub fn parse_dimacs(input: &str) -> Result<Cnf, String> {
        let _ = input;
        todo!("parse the DIMACS cnf format")
    }

    /// Stage 1 — Serialize back to DIMACS: header line, then one clause per
    /// line, each terminated by `0`. Lossless round trip:
    /// `Cnf::parse_dimacs(&cnf.to_dimacs()) == Ok(cnf)` for well-formed `cnf`.
    pub fn to_dimacs(&self) -> String {
        todo!("serialize back to DIMACS")
    }

    /// Stage 1 — Evaluate the formula under a *complete* assignment:
    /// `assignment[v - 1]` is the value of variable `v`.
    ///
    /// A clause is true iff at least one of its literals is true; the formula
    /// is true iff every clause is. Consequently an empty clause makes the
    /// formula false under every assignment, and the empty formula is true.
    ///
    /// Panics if `assignment.len() < num_vars` (an incomplete assignment has
    /// no truth value — that is what stage 2's partial assignments are for).
    pub fn evaluate(&self, assignment: &[bool]) -> bool {
        let _ = assignment;
        todo!("evaluate under a complete assignment")
    }
}

// ---------------------------------------------------------------------------
// Stage 2 — Unit propagation
// ---------------------------------------------------------------------------

/// The outcome of running unit propagation to a fixed point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Propagation {
    /// No clause became empty. Carries the literals that were *forced*, in the
    /// order they were assigned (`+v` = v forced true, `-v` = v forced false).
    /// Empty vector = nothing was unit; in particular, running
    /// `unit_propagate` again immediately yields `Implied(vec![])`
    /// (idempotence).
    Implied(Vec<i32>),
    /// Some clause has all its literals false under the (extended) assignment.
    /// Literals forced *before* the conflict was discovered remain assigned in
    /// `assignment` — callers that need to undo them snapshot first (exactly
    /// what the DPLL driver in stage 3 does).
    Conflict,
}

/// Stage 2 — Unit propagation, the inference engine inside every SAT solver
/// (§7.2.2.2; inside Knuth's Algorithm D this is what the watched-literal
/// machinery of steps D3/D6 accomplishes lazily — here we use straightforward
/// repeated scanning, perfectly adequate at lab scale).
///
/// Repeatedly:
/// - a clause whose literals are all false under `assignment` is a
///   **conflict** → return `Propagation::Conflict` immediately;
/// - a clause with no true literal and exactly one unassigned literal is a
///   **unit clause** → that literal is forced true (no other way to satisfy
///   the clause), so assign it and record it.
///
/// Stops at a fixed point (a full scan forces nothing) and returns
/// `Propagation::Implied(forced)`. Satisfied clauses are skipped; pre-assigned
/// variables are respected and never overwritten.
///
/// Panics if `assignment.len() != cnf.num_vars`.
pub fn unit_propagate(cnf: &Cnf, assignment: &mut Vec<Option<bool>>) -> Propagation {
    let _ = (cnf, assignment);
    todo!("run unit propagation to a fixed point")
}

// ---------------------------------------------------------------------------
// Stage 3 — DPLL (Algorithm 7.2.2.2D with simplified data structures)
// ---------------------------------------------------------------------------

/// Stage 3 — Algorithm 7.2.2.2D (Davis–Putnam–Logemann–Loveland). Returns a
/// model — a complete assignment with `model[v - 1]` = value of variable `v` —
/// or `None` if the formula is unsatisfiable.
///
/// ```text
/// D1. [Initialize.]   Empty partial assignment; nothing forced yet.
/// D3/D6. [Propagate.] Run unit_propagate; a conflict kills this branch.
/// D2. [Success?]      Every clause satisfied → return the model. Otherwise
///                     pick a branching literal from an unsatisfied clause.
/// D4. [Two-way branch.] Try the literal true; recurse.
/// D7. [Backtrack.]    Undo, try the complementary value; recurse.
/// D8. [Failure.]      Both branches refuted → UNSAT here.
/// ```
///
/// Variables still unassigned at success may be completed with `false`; any
/// completion works, because every clause already contains a true literal.
pub fn solve(cnf: &Cnf) -> Option<Vec<bool>> {
    let _ = cnf;
    todo!("implement Algorithm 7.2.2.2D (DPLL)")
}

/// Stage 3 — Brute-force model search: try all `2^num_vars` complete
/// assignments (in the order induced by counting, variable 1 = least
/// significant bit) and return the first model, or `None`. The honest
/// cross-check for `solve` on small instances. Panics above 25 variables.
pub fn solve_brute(cnf: &Cnf) -> Option<Vec<bool>> {
    let _ = cnf;
    todo!("try all 2^num_vars complete assignments")
}

/// Stage 3 — The pigeonhole formula PHP(m, n): "m pigeons sit in n holes, each
/// pigeon in some hole, no hole holding two pigeons." Unsatisfiable iff m > n —
/// obvious to us, yet by Haken's theorem (1985) every resolution refutation of
/// PHP(n+1, n) has size exponential in n.
///
/// Variable `x[p][h] = p * holes + h + 1` for pigeon `p in 0..pigeons` and
/// hole `h in 0..holes` means "pigeon p sits in hole h". Clauses:
/// - for each pigeon p: (x[p][0] ∨ ... ∨ x[p][holes-1]) — sits somewhere;
/// - for each hole h and pair p < q: (¬x[p][h] ∨ ¬x[q][h]) — no sharing.
pub fn pigeonhole_cnf(pigeons: usize, holes: usize) -> Cnf {
    let _ = (pigeons, holes);
    todo!("build the pigeonhole formula PHP(pigeons, holes)")
}

/// Stage 3 — The van der Waerden formula waerden(j, k; n) — Knuth's running
/// example throughout §7.2.2.2. Variable `i` (for `1 <= i <= n`) means
/// "integer i is coloured red" (false = blue). Clauses forbid monochromatic
/// arithmetic progressions:
/// - for every AP `a, a+d, ..., a+(j-1)d` within `1..=n` (d >= 1):
///   (¬x_a ∨ ¬x_{a+d} ∨ ...) — no j-term all-red AP;
/// - for every AP of length k likewise: (x_a ∨ x_{a+d} ∨ ...) — no k-term
///   all-blue AP.
///
/// Satisfiable iff n < W(j, k), the van der Waerden number. W(3, 3) = 9:
/// waerden(3, 3; 8) is satisfiable and waerden(3, 3; 9) is not.
pub fn waerden_cnf(j: usize, k: usize, n: usize) -> Cnf {
    let _ = (j, k, n);
    todo!("build the van der Waerden formula waerden(j, k; n)")
}

// ---------------------------------------------------------------------------
// Stage 4 — Encoding problems into SAT (§7.2.2.2 encodings)
// ---------------------------------------------------------------------------

/// Stage 4 — Clauses asserting that *exactly one* of the given literals is
/// true, in the pairwise ("binomial") encoding:
/// - at-least-one: the clause `vars` itself;
/// - at-most-one: (¬a ∨ ¬b) for every pair — O(n²) clauses, zero new
///   variables.
///
/// `vars` may be any literals, not only positive ones. For the empty slice
/// this returns the single empty clause: "exactly one of nothing" is
/// unsatisfiable, as it should be.
pub fn exactly_one(vars: &[i32]) -> Vec<Vec<i32>> {
    let _ = vars;
    todo!("build the pairwise exactly-one encoding")
}

/// Stage 4 — The n-queens problem as CNF. Variable `x[r][c] = r * n + c + 1`
/// means "a queen sits on row r, column c" (0-based). Clauses:
/// - exactly one queen in each row;
/// - at most one queen in each column;
/// - at most one queen on each diagonal, both directions.
///
/// Satisfiable for n = 1 and n >= 4; unsatisfiable for n = 2, 3.
pub fn queens_cnf(n: usize) -> Cnf {
    let _ = n;
    todo!("encode the n-queens problem as CNF")
}

/// Stage 4 — Decode a model of `queens_cnf(n)`: returns `cols` with `cols[r]`
/// = the column (0-based) of the queen in row r — the first true variable of
/// the row (a genuine model has exactly one). Panics if some row has no queen.
pub fn decode_queens(model: &[bool], n: usize) -> Vec<usize> {
    let _ = (model, n);
    todo!("decode a queens model into column positions")
}

/// Stage 4 — Proper k-colouring of a graph as CNF. Vertices are
/// `0..n_vertices`; variable `x[v][c] = v * k + c + 1` means "vertex v gets
/// colour c" (colours 0-based). Clauses: exactly one colour per vertex, and
/// for every edge (u, w) and colour c: (¬x[u][c] ∨ ¬x[w][c]) — endpoints
/// differ.
pub fn coloring_cnf(n_vertices: usize, edges: &[(usize, usize)], k: usize) -> Cnf {
    let _ = (n_vertices, edges, k);
    todo!("encode graph k-colouring as CNF")
}

/// Stage 4 — Decode a model of `coloring_cnf(n_vertices, _, k)`: `colours[v]`
/// = the colour (0-based) assigned to vertex v. Panics if some vertex has none.
pub fn decode_coloring(model: &[bool], n_vertices: usize, k: usize) -> Vec<usize> {
    let _ = (model, n_vertices, k);
    todo!("decode a colouring model into per-vertex colours")
}
