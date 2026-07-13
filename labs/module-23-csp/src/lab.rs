//! Module 23 — Constraint Satisfaction.
//! Source: TAOCP Vol. 4 Fascicle 7, §7.2.2.3 (toward Vol. 4C).
//!
//! Lesson: `course/module-23-csp/README.md`. Advanced-tier scaffolding: each
//! item states its contract — the algorithm, its invariant, and the exact
//! panic wording the grader checks — and trusts you with the translation.
//! Stuck? `./grade 23 -s <stage> --hint`.
//!
//! Conventions (the tests rely on all three):
//! - domains are sorted and duplicate-free (constructors enforce it);
//! - search assigns values in ascending order, variables in index order
//!   (except MRV), so solutions come out in lexicographic order;
//! - a search **node** is counted every time a value is tentatively placed
//!   on a variable, before any consistency test.

// ---------------------------------------------------------------------------
// Stage 1 — the model, and basic backtracking (Algorithm B applied to CSP)
// ---------------------------------------------------------------------------

/// One binary constraint: `(x, y)` may take exactly the pairs in `allowed`
/// (sorted, deduplicated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    pub x: usize,
    pub y: usize,
    pub allowed: Vec<(u32, u32)>,
}

/// A binary CSP over variables `0..domains.len()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Csp {
    pub domains: Vec<Vec<u32>>,
    pub constraints: Vec<Constraint>,
}

impl Csp {
    /// A CSP with the given domains and no constraints. Sort and dedup each
    /// domain; an empty domain is legal (the instance just has no solutions).
    pub fn new(domains: Vec<Vec<u32>>) -> Csp {
        let _ = domains;
        todo!("store the domains, each sorted and deduplicated — see §7.2.2.3's model")
    }

    /// Add the constraint "`(x, y)` must take one of `allowed`" (sort and
    /// dedup the pair list).
    ///
    /// Contract: panic if `x == y` — message must contain `"distinct"` — and
    /// panic if either variable is `>= domains.len()` — message must contain
    /// `"range"`. The grader checks those exact substrings.
    pub fn add_allowed(&mut self, x: usize, y: usize, allowed: Vec<(u32, u32)>) {
        let _ = (x, y, allowed);
        todo!("validate the variable pair, then push the normalized constraint")
    }

    /// Is this complete assignment a solution? Every value must lie in its
    /// variable's domain and every constraint must allow the induced pair.
    ///
    /// Contract: panic when `assignment.len() != domains.len()` — message
    /// must contain `"length"`.
    pub fn check(&self, assignment: &[u32]) -> bool {
        let _ = assignment;
        todo!("domain membership for every variable, then every constraint's pair")
    }

    /// All solutions by *basic* backtracking — Algorithm 7.2.2B specialized
    /// to CSP: variables in index order, values ascending, reject a placement
    /// as soon as a constraint with both endpoints assigned is violated (no
    /// lookahead). Return the solutions (lexicographic by construction) and
    /// the node count — every tentative placement counts, accepted or not.
    pub fn solve_basic(&self) -> (Vec<Vec<u32>>, u64) {
        todo!("recursive descent over variables; count each placement before testing it")
    }
}

/// The n-queens problem as a CSP: variable `i` = the queen in column `i`,
/// value = its row in `0..n`; for each column pair `i < j` allow exactly the
/// row pairs on different rows and different diagonals (`|a - b| != j - i`).
pub fn queens_csp(n: usize) -> Csp {
    let _ = n;
    todo!("one domain 0..n per column; one allowed-pair constraint per column pair")
}

/// Proper k-coloring of a graph as a CSP: one variable per vertex, domain
/// `0..k`, and for every edge allow exactly the unequal color pairs.
/// (A self-loop or an out-of-range endpoint is rejected by `add_allowed`'s
/// panics.)
pub fn coloring_csp(vertices: usize, edges: &[(usize, usize)], k: u32) -> Csp {
    let _ = (vertices, edges, k);
    todo!("domains 0..k; per edge, the pairs (a, b) with a != b")
}

// ---------------------------------------------------------------------------
// Stage 2 — lookahead: forward checking, and the MRV ordering heuristic
// ---------------------------------------------------------------------------

impl Csp {
    /// All solutions by backtracking with **forward checking**: placing
    /// `x = a` filters every unassigned neighbor's *live* domain through the
    /// constraints incident to the pair, and the branch is abandoned the
    /// moment a live domain empties; undo the filtering on backtrack (the
    /// save-and-restore that §7.2.2.3's dancing cells make O(1)). Variables
    /// in index order, values ascending over the LIVE domain, solutions
    /// lexicographic; same node definition as `solve_basic` — only live
    /// values are ever placed, which is where the savings come from.
    pub fn solve_fc(&self) -> (Vec<Vec<u32>>, u64) {
        todo!("live domains + a removal log per placement; restore on backtrack")
    }

    /// Forward checking plus **minimum remaining values**: the next variable
    /// is the unassigned one with the smallest live domain, ties broken by
    /// the smallest index. Same solution set as `solve_fc`; sort the
    /// solutions lexicographically before returning (the search order no
    /// longer guarantees it). Its node count is its own.
    pub fn solve_fc_mrv(&self) -> (Vec<Vec<u32>>, u64) {
        todo!("as solve_fc, but pick the tightest variable first; sort solutions at the end")
    }
}

// ---------------------------------------------------------------------------
// Stage 3 — arc consistency (AC-3)
// ---------------------------------------------------------------------------

impl Csp {
    /// Prune every domain to the **arc-consistent** fixpoint: repeatedly
    /// delete any value without *support* — for the arc `(x, y)` of a
    /// constraint, `a ∈ D_x` needs some `b ∈ D_y` with `(a, b)` allowed, and
    /// symmetrically for the reverse arc. Re-examine the arcs aimed at a
    /// pruned variable until nothing changes. Return `false` iff some domain
    /// empties. The fixpoint is the unique maximal arc-consistent family of
    /// subdomains, so your worklist order cannot change the answer — and
    /// deleting only unsupported values means the solution set is preserved
    /// exactly.
    pub fn ac3(&mut self) -> bool {
        todo!("worklist of directed arcs; revise(tail, head) deletes unsupported tail values")
    }
}

// ---------------------------------------------------------------------------
// Stage 4 — translating CSP to SAT (the direct encoding)
// ---------------------------------------------------------------------------

/// The direct encoding of a CSP into CNF (DIMACS literal convention, as in
/// module 10). SAT variable `var[v][j]` (1-based) asserts "CSP variable `v`
/// takes `domains[v][j]`".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectEncoding {
    pub num_vars: usize,
    pub clauses: Vec<Vec<i32>>,
    pub var: Vec<Vec<i32>>,
}

/// Build the direct encoding: number the SAT variables domain-by-domain in
/// index order, then emit — in this order — one **at-least-one** clause per
/// CSP variable, the **at-most-one** pair clauses per CSP variable, and one
/// **conflict** clause `(!var[x][i] ∨ !var[y][j])` for every in-domain pair a
/// constraint forbids. Sizes the tests pin: `num_vars = Σ |D_v|`; clauses
/// `= n + Σ_v C(|D_v|, 2) + Σ_c #forbidden-in-domain-pairs`.
pub fn encode_direct(csp: &Csp) -> DirectEncoding {
    let _ = csp;
    todo!("number vars per domain slot; ALO, AMO, then conflict clauses")
}

/// Count the encoding's models by brute-force truth table (the oracle that
/// certifies model preservation on small instances).
///
/// Contract: panic when `num_vars > 24` — message must contain
/// `"truth table"`.
pub fn count_models(enc: &DirectEncoding) -> u64 {
    let _ = enc;
    todo!("iterate the 2^num_vars assignments; a model satisfies every clause")
}
