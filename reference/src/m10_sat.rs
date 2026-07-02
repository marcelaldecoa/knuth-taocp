//! Module 10 — Satisfiability.
//! Source: TAOCP Vol. 4B, §7.2.2.2.
//!
//! Literal convention (DIMACS style; Knuth's internal `2k + sign` coding is a
//! trivial re-coding of this): a literal is a nonzero `i32`; `+v` means
//! "variable v is true", `-v` means "variable v is false", for
//! `1 <= v <= num_vars`. A clause is a `Vec<i32>` (the disjunction of its
//! literals); a CNF formula is the conjunction of its clauses. The *empty
//! clause* is unsatisfiable (a disjunction of nothing is false); the *empty
//! formula* is satisfied by anything (a conjunction of nothing is true).
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
    /// Parse the DIMACS `cnf` format (the lingua franca of SAT solving, used
    /// for all of §7.2.2.2's benchmark experiments).
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
    /// clause data before the header, non-integer tokens (trailing garbage),
    /// literals whose variable exceeds `num_vars`, an unterminated final
    /// clause, and a clause count differing from the header's declaration.
    /// A bare `0` yields an empty clause — legal DIMACS, and unsatisfiable.
    pub fn parse_dimacs(input: &str) -> Result<Cnf, String> {
        let mut header: Option<(usize, usize)> = None;
        let mut clauses: Vec<Vec<i32>> = Vec::new();
        let mut current: Vec<i32> = Vec::new();

        for line in input.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('c') {
                continue; // comment or blank line
            }
            if line.starts_with('p') {
                if header.is_some() {
                    return Err("duplicate 'p cnf' header".to_string());
                }
                let toks: Vec<&str> = line.split_whitespace().collect();
                if toks.len() != 4 || toks[0] != "p" || toks[1] != "cnf" {
                    return Err(format!("malformed header line: {line:?}"));
                }
                let v: usize = toks[2]
                    .parse()
                    .map_err(|_| format!("bad variable count {:?} in header", toks[2]))?;
                let c: usize = toks[3]
                    .parse()
                    .map_err(|_| format!("bad clause count {:?} in header", toks[3]))?;
                header = Some((v, c));
                continue;
            }
            let (num_vars, _) = header.ok_or("clause data before 'p cnf' header")?;
            for tok in line.split_whitespace() {
                let lit: i32 = tok
                    .parse()
                    .map_err(|_| format!("unexpected token {tok:?} (not an integer)"))?;
                if lit == 0 {
                    clauses.push(std::mem::take(&mut current));
                } else {
                    if lit.unsigned_abs() as usize > num_vars {
                        return Err(format!(
                            "literal {lit} out of range: header declares {num_vars} variables"
                        ));
                    }
                    current.push(lit);
                }
            }
        }

        let (num_vars, num_clauses) = header.ok_or("missing 'p cnf' header")?;
        if !current.is_empty() {
            return Err("unterminated final clause (missing trailing 0)".to_string());
        }
        if clauses.len() != num_clauses {
            return Err(format!(
                "header declares {num_clauses} clauses but {} were given",
                clauses.len()
            ));
        }
        Ok(Cnf { num_vars, clauses })
    }

    /// Serialize back to DIMACS: header line, then one clause per line, each
    /// terminated by `0`. Lossless round trip:
    /// `Cnf::parse_dimacs(&cnf.to_dimacs()) == Ok(cnf)` for well-formed `cnf`.
    pub fn to_dimacs(&self) -> String {
        let mut out = format!("p cnf {} {}\n", self.num_vars, self.clauses.len());
        for clause in &self.clauses {
            for lit in clause {
                out.push_str(&lit.to_string());
                out.push(' ');
            }
            out.push_str("0\n");
        }
        out
    }

    /// Evaluate the formula under a *complete* assignment:
    /// `assignment[v - 1]` is the value of variable `v`.
    ///
    /// A clause is true iff at least one of its literals is true; the formula
    /// is true iff every clause is. Consequently an empty clause makes the
    /// formula false under every assignment, and the empty formula is true.
    ///
    /// Panics if `assignment.len() < num_vars` (an incomplete assignment has
    /// no truth value — that is what stage 2's partial assignments are for).
    pub fn evaluate(&self, assignment: &[bool]) -> bool {
        assert!(
            assignment.len() >= self.num_vars,
            "assignment covers {} variables but the formula has {}",
            assignment.len(),
            self.num_vars
        );
        self.clauses.iter().all(|clause| {
            clause.iter().any(|&lit| {
                let value = assignment[(lit.unsigned_abs() - 1) as usize];
                if lit > 0 {
                    value
                } else {
                    !value
                }
            })
        })
    }
}

// ---------------------------------------------------------------------------
// Stage 2 — Unit propagation
// ---------------------------------------------------------------------------

/// The outcome of running unit propagation to a fixed point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Propagation {
    /// No clause became empty. Carries the literals that were *forced*, in
    /// the order they were assigned (`+v` = v forced true, `-v` = v forced
    /// false). Empty vector = nothing was unit; in particular, running
    /// `unit_propagate` again immediately yields `Implied(vec![])`
    /// (idempotence).
    Implied(Vec<i32>),
    /// Some clause has all its literals false under the (extended)
    /// assignment. Literals forced *before* the conflict was discovered
    /// remain assigned in `assignment` — callers that need to undo them
    /// snapshot first (exactly what the DPLL driver in stage 3 does).
    Conflict,
}

/// The value of literal `lit` under a partial assignment, or `None` if its
/// variable is unassigned.
fn literal_value(lit: i32, assignment: &[Option<bool>]) -> Option<bool> {
    assignment[(lit.unsigned_abs() - 1) as usize].map(|v| if lit > 0 { v } else { !v })
}

/// Unit propagation, the inference engine inside every SAT solver
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
/// `Propagation::Implied(forced)`. Satisfied clauses are skipped;
/// pre-assigned variables are respected and never overwritten.
///
/// Panics if `assignment.len() != cnf.num_vars`.
pub fn unit_propagate(cnf: &Cnf, assignment: &mut Vec<Option<bool>>) -> Propagation {
    assert_eq!(
        assignment.len(),
        cnf.num_vars,
        "partial assignment must have one slot per variable"
    );
    let mut forced: Vec<i32> = Vec::new();
    loop {
        let mut changed = false;
        for clause in &cnf.clauses {
            let mut satisfied = false;
            let mut unit: Option<i32> = None;
            let mut unassigned = 0usize;
            for &lit in clause {
                match literal_value(lit, assignment) {
                    Some(true) => {
                        satisfied = true;
                        break;
                    }
                    Some(false) => {}
                    None => {
                        unassigned += 1;
                        unit = Some(lit);
                    }
                }
            }
            if satisfied {
                continue;
            }
            match unassigned {
                // All literals false (or the clause is empty): unsatisfiable.
                0 => return Propagation::Conflict,
                // Exactly one unassigned literal l, none true: l must hold.
                1 => {
                    let lit = unit.expect("counted one unassigned literal");
                    assignment[(lit.unsigned_abs() - 1) as usize] = Some(lit > 0);
                    forced.push(lit);
                    changed = true;
                }
                _ => {}
            }
        }
        if !changed {
            return Propagation::Implied(forced);
        }
    }
}

// ---------------------------------------------------------------------------
// Stage 3 — DPLL (Algorithm 7.2.2.2D with simplified data structures)
// ---------------------------------------------------------------------------

/// Algorithm 7.2.2.2D (Davis–Putnam–Logemann–Loveland). Returns a model — a
/// complete assignment with `model[v - 1]` = value of variable `v` — or
/// `None` if the formula is unsatisfiable.
///
/// Knuth's Algorithm D is *cyclic* DPLL with watched literals and an active
/// ring; this implementation keeps the same skeleton (initialize, propagate
/// units, branch two ways, backtrack) with plain scanning instead of lazy
/// data structures — see the D1–D8 step comments in the kernel for the
/// correspondence. Completeness: unit propagation only assigns values forced
/// by the clauses (it preserves the set of models extending the current
/// partial assignment), and each branch node tries *both* values of the
/// chosen variable; hence `None` is returned only if no complete assignment
/// satisfies the formula.
///
/// Variables still unassigned at success are completed with `false`; any
/// completion works, because every clause already contains a true literal.
pub fn solve(cnf: &Cnf) -> Option<Vec<bool>> {
    // D1. [Initialize.] Empty partial assignment; nothing forced yet.
    let mut assignment: Vec<Option<bool>> = vec![None; cnf.num_vars];
    if dpll(cnf, &mut assignment) {
        Some(assignment.iter().map(|v| v.unwrap_or(false)).collect())
    } else {
        None
    }
}

/// Recursive DPLL kernel: extends `assignment` to satisfy `cnf` and returns
/// `true`, or returns `false` (restoring `assignment` to its state on entry)
/// if no extension exists.
fn dpll(cnf: &Cnf, assignment: &mut Vec<Option<bool>>) -> bool {
    let snapshot = assignment.clone();

    // D3. [Look for unit clauses.] / D6. [Update.] Propagate every forced
    // literal to a fixed point; a conflict means this branch is dead.
    if unit_propagate(cnf, assignment) == Propagation::Conflict {
        *assignment = snapshot; // D7. [Backtrack.]
        return false;
    }

    // D2. [Success?] If every clause has a true literal, rejoice. Otherwise
    // take a branching literal from the first unsatisfied clause (which has
    // >= 2 unassigned literals, since propagation reached a fixed point).
    let mut branch: Option<i32> = None;
    'clauses: for clause in &cnf.clauses {
        let mut candidate: Option<i32> = None;
        for &lit in clause {
            match literal_value(lit, assignment) {
                Some(true) => continue 'clauses, // clause satisfied
                None => {
                    if candidate.is_none() {
                        candidate = Some(lit);
                    }
                }
                Some(false) => {}
            }
        }
        branch = candidate;
        break;
    }
    let Some(lit) = branch else {
        return true; // D2: all clauses satisfied — a model.
    };
    let var = lit.unsigned_abs() as usize;

    // D4. [Two-way branch.] Try the literal as it appears first — a cheap
    // heuristic: this choice immediately satisfies the clause we found.
    assignment[var - 1] = Some(lit > 0);
    if dpll(cnf, assignment) {
        return true; // D5. [Move on.]
    }
    // D7. [Backtrack.] Undo, then try the complementary value.
    *assignment = snapshot.clone();
    assignment[var - 1] = Some(lit < 0);
    if dpll(cnf, assignment) {
        return true;
    }
    // D8. [Failure.] Both branches refuted: restore and report UNSAT here.
    *assignment = snapshot;
    false
}

/// Brute-force model search: try all `2^num_vars` complete assignments (in
/// the order induced by counting, variable 1 = least significant bit) and
/// return the first model, or `None`. The honest cross-check for `solve` on
/// small instances. Panics above 25 variables — 2^25 evaluations is where
/// "small" stops.
pub fn solve_brute(cnf: &Cnf) -> Option<Vec<bool>> {
    assert!(cnf.num_vars <= 25, "brute force is limited to 25 variables");
    for bits in 0u64..(1u64 << cnf.num_vars) {
        let assignment: Vec<bool> = (0..cnf.num_vars).map(|v| bits >> v & 1 == 1).collect();
        if cnf.evaluate(&assignment) {
            return Some(assignment);
        }
    }
    None
}

/// The pigeonhole formula PHP(m, n): "m pigeons sit in n holes, each pigeon
/// in some hole, no hole holding two pigeons." Unsatisfiable iff m > n —
/// obvious to us, yet by Haken's theorem (1985) every resolution refutation
/// of PHP(n+1, n) has size exponential in n, so DPLL-style solvers provably
/// need exponential time on it (§7.2.2.2's canonical hard family).
///
/// Variable `x[p][h] = p * holes + h + 1` for pigeon `p in 0..pigeons` and
/// hole `h in 0..holes` means "pigeon p sits in hole h". Clauses:
/// - for each pigeon p: (x[p][0] ∨ ... ∨ x[p][holes-1]) — sits somewhere;
/// - for each hole h and pair p < q: (¬x[p][h] ∨ ¬x[q][h]) — no sharing.
pub fn pigeonhole_cnf(pigeons: usize, holes: usize) -> Cnf {
    let var = |p: usize, h: usize| (p * holes + h + 1) as i32;
    let mut clauses: Vec<Vec<i32>> = Vec::new();
    for p in 0..pigeons {
        clauses.push((0..holes).map(|h| var(p, h)).collect());
    }
    for h in 0..holes {
        for p in 0..pigeons {
            for q in p + 1..pigeons {
                clauses.push(vec![-var(p, h), -var(q, h)]);
            }
        }
    }
    Cnf {
        num_vars: pigeons * holes,
        clauses,
    }
}

/// The van der Waerden formula waerden(j, k; n) — Knuth's running example
/// throughout §7.2.2.2. Variable `i` (for `1 <= i <= n`) means "integer i is
/// coloured red" (false = blue). Clauses forbid monochromatic arithmetic
/// progressions:
/// - for every AP `a, a+d, ..., a+(j-1)d` within `1..=n` (d >= 1):
///   (¬x_a ∨ ¬x_{a+d} ∨ ...) — no j-term all-red AP;
/// - for every AP of length k likewise: (x_a ∨ x_{a+d} ∨ ...) — no k-term
///   all-blue AP.
///
/// Satisfiable iff n < W(j, k), the van der Waerden number. W(3, 3) = 9:
/// waerden(3, 3; 8) is satisfiable and waerden(3, 3; 9) is not.
pub fn waerden_cnf(j: usize, k: usize, n: usize) -> Cnf {
    let mut clauses: Vec<Vec<i32>> = Vec::new();
    // No j-term arithmetic progression entirely red (all true).
    for d in 1..=n {
        for a in 1..=n {
            if a + (j - 1) * d > n {
                break;
            }
            clauses.push((0..j).map(|t| -((a + t * d) as i32)).collect());
        }
    }
    // No k-term arithmetic progression entirely blue (all false).
    for d in 1..=n {
        for a in 1..=n {
            if a + (k - 1) * d > n {
                break;
            }
            clauses.push((0..k).map(|t| (a + t * d) as i32).collect());
        }
    }
    Cnf {
        num_vars: n,
        clauses,
    }
}

// ---------------------------------------------------------------------------
// Stage 4 — Encoding problems into SAT (§7.2.2.2 encodings)
// ---------------------------------------------------------------------------

/// Clauses asserting that *exactly one* of the given literals is true, in
/// the pairwise ("binomial") encoding:
/// - at-least-one: the clause `vars` itself;
/// - at-most-one: (¬a ∨ ¬b) for every pair — O(n²) clauses, zero new
///   variables. (The sequential and commander encodings of §7.2.2.2 trade
///   auxiliary variables for O(n) clauses; at lab sizes pairwise wins.)
///
/// `vars` may be any literals, not only positive ones. For the empty slice
/// this returns the single empty clause: "exactly one of nothing" is
/// unsatisfiable, as it should be.
pub fn exactly_one(vars: &[i32]) -> Vec<Vec<i32>> {
    let mut clauses: Vec<Vec<i32>> = vec![vars.to_vec()]; // at least one
    for i in 0..vars.len() {
        for j in i + 1..vars.len() {
            clauses.push(vec![-vars[i], -vars[j]]); // at most one
        }
    }
    clauses
}

/// The n-queens problem as CNF. Variable `x[r][c] = r * n + c + 1` means "a
/// queen sits on row r, column c" (0-based). Clauses:
/// - exactly one queen in each row;
/// - at most one queen in each column (n rows each holding exactly one queen
///   force the n columns to hold exactly one each, so at-most-one suffices);
/// - at most one queen on each diagonal, both directions.
///
/// Satisfiable for n = 1 and n >= 4; unsatisfiable for n = 2, 3.
pub fn queens_cnf(n: usize) -> Cnf {
    let var = |r: usize, c: usize| (r * n + c + 1) as i32;
    let mut clauses: Vec<Vec<i32>> = Vec::new();
    // Rows: exactly one queen each.
    for r in 0..n {
        let row: Vec<i32> = (0..n).map(|c| var(r, c)).collect();
        clauses.extend(exactly_one(&row));
    }
    // Columns: at most one queen each.
    for c in 0..n {
        for r1 in 0..n {
            for r2 in r1 + 1..n {
                clauses.push(vec![-var(r1, c), -var(r2, c)]);
            }
        }
    }
    // Diagonals: at most one queen each. Squares (r1,c1), (r2,c2) with
    // r1 < r2 attack diagonally iff |c1 - c2| = r2 - r1.
    for r1 in 0..n {
        for c1 in 0..n {
            for r2 in r1 + 1..n {
                let d = r2 - r1;
                if c1 + d < n {
                    clauses.push(vec![-var(r1, c1), -var(r2, c1 + d)]);
                }
                if c1 >= d {
                    clauses.push(vec![-var(r1, c1), -var(r2, c1 - d)]);
                }
            }
        }
    }
    Cnf {
        num_vars: n * n,
        clauses,
    }
}

/// Decode a model of `queens_cnf(n)`: returns `cols` with `cols[r]` = the
/// column (0-based) of the queen in row r — the first true variable of the
/// row (a genuine model has exactly one). Panics if some row has no queen.
pub fn decode_queens(model: &[bool], n: usize) -> Vec<usize> {
    (0..n)
        .map(|r| {
            (0..n)
                .find(|&c| model[r * n + c])
                .expect("model must place a queen in every row")
        })
        .collect()
}

/// Proper k-colouring of a graph as CNF. Vertices are `0..n_vertices`;
/// variable `x[v][c] = v * k + c + 1` means "vertex v gets colour c"
/// (colours 0-based). Clauses: exactly one colour per vertex, and for every
/// edge (u, w) and colour c: (¬x[u][c] ∨ ¬x[w][c]) — endpoints differ.
pub fn coloring_cnf(n_vertices: usize, edges: &[(usize, usize)], k: usize) -> Cnf {
    let var = |v: usize, c: usize| (v * k + c + 1) as i32;
    let mut clauses: Vec<Vec<i32>> = Vec::new();
    for v in 0..n_vertices {
        let colours: Vec<i32> = (0..k).map(|c| var(v, c)).collect();
        clauses.extend(exactly_one(&colours));
    }
    for &(u, w) in edges {
        for c in 0..k {
            clauses.push(vec![-var(u, c), -var(w, c)]);
        }
    }
    Cnf {
        num_vars: n_vertices * k,
        clauses,
    }
}

/// Decode a model of `coloring_cnf(n_vertices, _, k)`: `colours[v]` = the
/// colour (0-based) assigned to vertex v. Panics if some vertex has none.
pub fn decode_coloring(model: &[bool], n_vertices: usize, k: usize) -> Vec<usize> {
    (0..n_vertices)
        .map(|v| {
            (0..k)
                .find(|&c| model[v * k + c])
                .expect("model must colour every vertex")
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Unit tests: worked examples from §7.2.2.2
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimacs_round_trip() {
        let text = "c a comment\np cnf 3 2\n1 -2 0\n2 3 0\n";
        let cnf = Cnf::parse_dimacs(text).unwrap();
        assert_eq!(cnf.num_vars, 3);
        assert_eq!(cnf.clauses, vec![vec![1, -2], vec![2, 3]]);
        assert_eq!(Cnf::parse_dimacs(&cnf.to_dimacs()), Ok(cnf));
    }

    #[test]
    fn evaluate_hand_example() {
        // (x1 ∨ ¬x2) ∧ (x2 ∨ x3)
        let cnf = Cnf {
            num_vars: 3,
            clauses: vec![vec![1, -2], vec![2, 3]],
        };
        assert!(cnf.evaluate(&[true, true, false]));
        assert!(cnf.evaluate(&[false, false, true]));
        assert!(!cnf.evaluate(&[false, true, false]));
    }

    #[test]
    fn propagation_chain_and_conflict() {
        // (x1) ∧ (¬x1 ∨ x2) ∧ (¬x2 ∨ x3) forces x1, x2, x3 in turn.
        let cnf = Cnf {
            num_vars: 3,
            clauses: vec![vec![1], vec![-1, 2], vec![-2, 3]],
        };
        let mut a = vec![None; 3];
        assert_eq!(
            unit_propagate(&cnf, &mut a),
            Propagation::Implied(vec![1, 2, 3])
        );
        assert_eq!(a, vec![Some(true); 3]);
        // Idempotence.
        assert_eq!(unit_propagate(&cnf, &mut a), Propagation::Implied(vec![]));
        // (x1) ∧ (¬x1): conflict.
        let bad = Cnf {
            num_vars: 1,
            clauses: vec![vec![1], vec![-1]],
        };
        assert_eq!(unit_propagate(&bad, &mut vec![None]), Propagation::Conflict);
    }

    #[test]
    fn waerden_running_example() {
        // §7.2.2.2's running example: W(3, 3) = 9. Eight integers can be
        // 2-coloured with no monochromatic 3-term AP; nine cannot.
        let sat = waerden_cnf(3, 3, 8);
        let model = solve(&sat).expect("waerden(3,3;8) is satisfiable");
        assert!(sat.evaluate(&model));
        assert_eq!(solve(&waerden_cnf(3, 3, 9)), None);
        assert_eq!(solve_brute(&waerden_cnf(3, 3, 9)), None);
    }

    #[test]
    fn pigeonhole_and_queens() {
        assert_eq!(solve(&pigeonhole_cnf(4, 3)), None);
        let php33 = pigeonhole_cnf(3, 3);
        let model = solve(&php33).expect("3 pigeons fit in 3 holes");
        assert!(php33.evaluate(&model));

        assert_eq!(solve(&queens_cnf(3)), None);
        let q6 = queens_cnf(6);
        let model = solve(&q6).expect("6-queens is solvable");
        assert!(q6.evaluate(&model));
        let cols = decode_queens(&model, 6);
        for r1 in 0..6 {
            for r2 in r1 + 1..6 {
                assert_ne!(cols[r1], cols[r2]);
                assert_ne!(cols[r1].abs_diff(cols[r2]), r2 - r1);
            }
        }
    }

    #[test]
    fn petersen_is_3_colourable_k4_is_not_2_colourable() {
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for i in 0..5 {
            edges.push((i, (i + 1) % 5)); // outer 5-cycle
            edges.push((5 + i, 5 + (i + 2) % 5)); // inner pentagram
            edges.push((i, 5 + i)); // spokes
        }
        let cnf = coloring_cnf(10, &edges, 3);
        let model = solve(&cnf).expect("Petersen graph is 3-colourable");
        let colours = decode_coloring(&model, 10, 3);
        for &(u, w) in &edges {
            assert_ne!(colours[u], colours[w]);
        }
        let k4: Vec<(usize, usize)> = vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
        assert_eq!(solve(&coloring_cnf(4, &k4, 2)), None);
    }
}
