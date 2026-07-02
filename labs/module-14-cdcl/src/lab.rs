//! Module 14 — Conflict-Driven Clause Learning (TAOCP Vol. 4B, §7.2.2.2,
//! Algorithm C). The course capstone.
//!
//! **Scaffolding tier — Module 05 and up:** the stub states the algorithm and
//! the contract and trusts you to translate it to Rust; the guided-tour aids of
//! Modules 01–04 are gone by design. The nets remain for every stage — the
//! lesson, three graduated hints (`--hint`), the reference, and the walkthrough.
//! (The taper is described in docs/for-newcomers.md §5.)
//!
//! YOUR WORKSPACE. Replace each `todo!()` with an implementation, then run
//! `./grade 14` from the repository root. Work the stages in order — the
//! lesson in `course/module-14-cdcl/README.md` develops every definition,
//! invariant and proof these stubs refer to.
//!
//! Literal convention (as in module 10): a literal is a nonzero `i32`; `+v`
//! means "variable v is true", `-v` means "variable v is false", for
//! `1 <= v <= num_vars`. This module is self-contained: nothing is imported
//! from module 10.
//!
//! The four stages build one solver:
//! - stage 1: `ClauseDb` — clauses, values, and **two watched literals**;
//! - stage 2: `Trail` — decisions, decision levels, and reasons;
//! - stage 3: `analyze` — first-UIP conflict analysis (backward resolution);
//! - stage 4: `solve` — the CDCL driver loop, plus formula generators.

// ---------------------------------------------------------------------------
// CNF formulas (used by stages 3 and 4)
// ---------------------------------------------------------------------------

/// A formula in conjunctive normal form: the conjunction of `clauses`, each
/// clause the disjunction of its literals, over variables `1..=num_vars`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cnf {
    pub num_vars: usize,
    pub clauses: Vec<Vec<i32>>,
}

impl Cnf {
    /// Evaluate under a *complete* assignment (`assignment[v - 1]` = value
    /// of variable `v`): true iff every clause contains a true literal.
    /// The empty clause is false everywhere; the empty formula is true.
    /// Panics if `assignment.len() < num_vars`.
    pub fn evaluate(&self, assignment: &[bool]) -> bool {
        let _ = assignment;
        todo!("evaluate every clause; you wrote this once in module 10")
    }
}

// ---------------------------------------------------------------------------
// Stage 1 — the clause database with two watched literals
// ---------------------------------------------------------------------------

/// Clause database with lazy unit propagation via two watched literals
/// (§7.2.2.2 — the engine of Algorithm C).
///
/// **The watch invariant** (lesson §5). Each clause of length ≥ 2 *watches*
/// two of its literals (a good convention: keep them in slots 0 and 1, and
/// swap literals within the clause as watches move). At every propagation
/// fixed point: if a watched literal is false, then the other watched
/// literal is true, or the clause has at most one non-false literal (it is
/// unit or conflicting). Consequence: a clause can only have become unit or
/// false when one of its **watched** literals was falsified — so on each
/// assignment only the watchers of the falsified literal are examined.
///
/// **Backtracking is free.** Unassigning variables (in LIFO order!) can only
/// turn false literals into unassigned ones, and the lesson proves the
/// invariant survives — no watch is ever repaired on backtrack.
///
/// Suggested representation (feel free to change the private parts, but the
/// public methods are the graded contract):
#[allow(dead_code)] // the fields are read once you implement the methods
#[derive(Debug, Clone)]
pub struct ClauseDb {
    /// Number of variables; literals range over `±1..=±num_vars`.
    num_vars: usize,
    /// The clauses; `clauses[c][0]` and `clauses[c][1]` are the watches.
    clauses: Vec<Vec<i32>>,
    /// `watches[code(l)]` = indices of the clauses watching literal `l`,
    /// where `code(+v) = 2(v-1)` and `code(-v) = 2(v-1) + 1` (Knuth's own
    /// literal coding).
    watches: Vec<Vec<usize>>,
    /// `values[v - 1]` = current value of variable `v`, `None` = unassigned.
    values: Vec<Option<bool>>,
    /// True literals whose watcher lists have not been examined yet, in
    /// assignment order (FIFO — this is Knuth's "pointer G chases the trail").
    queue: std::collections::VecDeque<i32>,
    /// Forced `(literal, reason clause)` pairs awaiting `take_implications`.
    implications: Vec<(i32, usize)>,
    /// A clause already falsified when added; the next `propagate` reports it.
    pending_conflict: Option<usize>,
}

impl ClauseDb {
    /// An empty database over variables `1..=num_vars`.
    pub fn new(num_vars: usize) -> ClauseDb {
        let _ = num_vars;
        todo!("allocate the empty database")
    }

    pub fn num_vars(&self) -> usize {
        todo!("report the number of variables")
    }

    pub fn num_clauses(&self) -> usize {
        todo!("report the number of clauses added so far")
    }

    /// The literals of clause `c` (in their current in-clause order).
    pub fn clause(&self, c: usize) -> &[i32] {
        let _ = c;
        todo!("return clause c's literals")
    }

    /// The value of literal `lit` under the current partial assignment:
    /// `Some(true)` / `Some(false)` if its variable is assigned (respecting
    /// the literal's sign), `None` otherwise. Panics on out-of-range input.
    pub fn value(&self, lit: i32) -> Option<bool> {
        let _ = lit;
        todo!("look up the variable, flip for negative literals")
    }

    /// Add a clause, return its index (consecutive from 0). Move two
    /// non-false literals into the watch slots 0 and 1 and register them in
    /// the watch lists; then:
    ///
    /// - all literals false (or the clause is empty) → remember it as a
    ///   pending conflict for the next `propagate`;
    /// - exactly one non-false literal, and it is unassigned → the clause is
    ///   **unit**: force that literal now (assign it, record the implication
    ///   with this clause as reason, put it on the propagation queue). This
    ///   is how a learned clause asserts its literal right after a backjump;
    /// - otherwise → nothing more to do.
    ///
    /// Contract for mid-search callers: order the clause so slot 1 holds a
    /// literal from the highest decision level among the false ones (stage
    /// 3's `analyze` hands you exactly that). Panics if a variable repeats
    /// or a literal is out of range — dedup before adding (stage 4 does).
    pub fn add_clause(&mut self, lits: Vec<i32>) -> usize {
        let _ = lits;
        todo!("register watches; detect unit / conflicting arrivals")
    }

    /// Make `lit` true — a *decision* (or a test script's assignment) — and
    /// enqueue it for watcher examination by the next `propagate`.
    /// Panics if the variable is already assigned.
    pub fn assign(&mut self, lit: i32) {
        let _ = lit;
        todo!("set the value, push the literal on the queue")
    }

    /// Undo the assignment of `var`. Callers unassign in LIFO order (a
    /// suffix of the trail), and only between propagations. Thanks to the
    /// invariant's stability, this touches **no** watch list.
    pub fn unassign(&mut self, var: usize) {
        let _ = var;
        todo!("clear the value — and nothing else")
    }

    /// Drain the `(forced literal, reason clause)` pairs recorded since the
    /// last call, in assignment order. The stage-4 driver feeds them to
    /// `Trail::enqueue`.
    pub fn take_implications(&mut self) -> Vec<(i32, usize)> {
        todo!("hand over and clear the implication log")
    }

    /// Unit propagation to a fixed point, lazily. Returns `Some(c)` as soon
    /// as clause `c` has all literals false (discard the rest of the queue —
    /// the driver backjumps anyway), else `None` with the queue empty.
    ///
    /// Process queued literals **in FIFO order** (the tests rely on this
    /// canonical order; it is Knuth's Algorithm C advancing G along the
    /// trail). For each dequeued literal `p`, only the clauses watching
    /// `¬p` can have become unit or false. For each such clause c
    /// (with `¬p` swapped into slot 1, say):
    ///
    /// ```text
    /// P1. [Satisfied?]    If the other watch (slot 0) is true, keep the
    ///                     watch and move on.
    /// P2. [Move watch.]   If some literal in slots 2.. is non-false, swap
    ///                     it into slot 1 and move c to that literal's
    ///                     watch list. Done with c.
    /// P3. [Unit?]         Otherwise every literal but slot 0 is false. If
    ///                     slot 0 is unassigned, c is unit: force it —
    ///                     assign it, log the implication (lit, c), enqueue
    ///                     it — and keep the watch.
    /// P4. [Conflict.]     If slot 0 is false too, c is falsified: keep the
    ///                     watch, clear the queue, return Some(c).
    /// ```
    pub fn propagate(&mut self) -> Option<usize> {
        todo!("the two-watched-literals propagation loop")
    }

    /// Check the invariant (meaningful after `propagate` returns `None`):
    /// structurally, every clause of length ≥ 2 appears in exactly the watch
    /// lists of its slots 0 and 1 (shorter clauses in none); semantically, a
    /// false watched literal implies the other watch is true or the clause
    /// has ≤ 1 non-false literal. Used heavily by the graders — implement it
    /// honestly and it will catch your own propagation bugs.
    pub fn check_watch_invariant(&self) -> bool {
        todo!("structural + semantic watch checks")
    }
}

// ---------------------------------------------------------------------------
// Stage 2 — the trail: decisions, levels, reasons
// ---------------------------------------------------------------------------

/// The trail: every currently-true literal in assignment order, partitioned
/// into decision levels, each propagated literal remembering the clause that
/// forced it. Trail + reasons = the implication graph of lesson §3, stored
/// as one flat array (Knuth's `L` array with level boundaries).
///
/// Level 0 holds consequences of the formula alone (no decision active);
/// level `d ≥ 1` begins at the d-th decision.
#[allow(dead_code)] // the fields are read once you implement the methods
#[derive(Debug, Clone)]
pub struct Trail {
    /// The literals, oldest first.
    lits: Vec<i32>,
    /// `level_start[d]` = index in `lits` where level `d` begins
    /// (`level_start[0] == 0`; current level = `level_start.len() - 1`).
    level_start: Vec<usize>,
    /// Per variable: `Some((level, reason))` while on the trail; decisions
    /// and root enqueues carry reason `None`.
    info: Vec<Option<(usize, Option<usize>)>>,
}

impl Trail {
    /// An empty trail over `1..=num_vars`, at decision level 0.
    pub fn new(num_vars: usize) -> Trail {
        let _ = num_vars;
        todo!("empty trail, level 0 open")
    }

    /// The current decision level (0 before any decision).
    pub fn decision_level(&self) -> usize {
        todo!("how many decisions are active?")
    }

    /// Number of literals on the trail.
    pub fn len(&self) -> usize {
        todo!("trail length")
    }

    pub fn is_empty(&self) -> bool {
        todo!("is the trail empty?")
    }

    /// All trail literals, oldest first.
    pub fn literals(&self) -> &[i32] {
        todo!("the literals in assignment order")
    }

    /// Open a new decision level with `lit` as its decision (no reason).
    /// Panics if the variable is already on the trail.
    pub fn decide(&mut self, lit: i32) {
        let _ = lit;
        todo!("new level boundary, then record the literal")
    }

    /// Record `lit` at the *current* level; propagated literals pass
    /// `Some(reason clause)`. Panics if the variable is already on the trail.
    pub fn enqueue(&mut self, lit: i32, reason: Option<usize>) {
        let _ = (lit, reason);
        todo!("record the literal at the current level")
    }

    /// Decision level at which `var` was assigned; `None` if unassigned.
    pub fn level_of(&self, var: usize) -> Option<usize> {
        let _ = var;
        todo!("look up the level")
    }

    /// The clause that forced `var`; `None` for decisions and unassigned
    /// variables — reasons exist **only** for propagated literals.
    pub fn reason_of(&self, var: usize) -> Option<usize> {
        let _ = var;
        todo!("look up the reason")
    }

    /// The literals assigned at decision level `level`, in trail order.
    /// Panics if the level does not currently exist.
    pub fn assignments_at_level(&self, level: usize) -> &[i32] {
        let _ = level;
        todo!("slice the trail between level boundaries")
    }

    /// Pop every literal of every level above `level`, erase their
    /// level/reason records, and return them **newest first** — the LIFO
    /// order in which the caller must `ClauseDb::unassign` them. Levels
    /// `0..=level` survive untouched. Backjumping to the current level is a
    /// no-op; panics if `level` exceeds it.
    pub fn backjump(&mut self, level: usize) -> Vec<i32> {
        let _ = level;
        todo!("split off the suffix, clear its records, reverse it")
    }
}

// ---------------------------------------------------------------------------
// Stage 3 — first-UIP conflict analysis
// ---------------------------------------------------------------------------

/// First-UIP conflict analysis (the heart of Algorithm 7.2.2.2C): from a
/// clause `conflict` whose literals are all false on `trail`, derive the
/// learned clause and the backjump level, by resolution *backwards along
/// the trail*:
///
/// ```text
/// A1. [Start.]     The resolvent R := the conflict clause. Count its
///                  literals assigned at the current decision level; the
///                  others (all false at lower levels) will survive into the
///                  learned clause — except level-0 literals, which are
///                  false forever and are simply dropped.
/// A2. [UIP?]       If exactly one literal of R is from the current level,
///                  stop: that literal's variable is the first UIP.
/// A3. [Resolve.]   Otherwise let p = the *most recently assigned*
///                  current-level literal of R (walk the trail backwards —
///                  this is why the trail exists). Replace R by the
///                  resolvent of R and p's reason clause on p's variable:
///                  p drops out, the reason's other literals (all false)
///                  come in (skip variables already seen). Go to A2.
/// ```
///
/// A2 must succeed at the latest at the decision (which has no reason but
/// dominates the level trivially). Return `(learned, backjump_level)`:
/// - `learned[0]` = the **asserting literal**, the negation of the UIP
///   literal as it stands on the trail;
/// - the rest of `learned` = the surviving lower-level literals, with a
///   literal from the backjump level itself in slot 1 (so `add_clause` can
///   watch slots 0 and 1 blindly);
/// - `backjump_level` = the highest level among `learned[1..]`, or 0 when
///   the learned clause is unit.
///
/// Panics if called at decision level 0 — a root conflict means UNSAT and
/// there is nothing to learn.
///
/// Hint: a `seen: Vec<bool>` per variable plus a counter of unresolved
/// current-level literals; walk one index backwards over `trail.literals()`.
pub fn analyze(conflict: usize, trail: &Trail, db: &ClauseDb) -> (Vec<i32>, usize) {
    let _ = (conflict, trail, db);
    todo!("backward resolution to the first UIP")
}

/// Does `cnf` logically imply the clause `lits`? Brute force over all
/// `2^num_vars` assignments: implied iff no assignment satisfies `cnf`
/// while falsifying every literal of `lits`. Your soundness oracle for
/// learned clauses. Panic above 22 variables (definiteness: refuse inputs
/// you cannot honestly decide).
pub fn brute_force_implies(cnf: &Cnf, lits: &[i32]) -> bool {
    let _ = (cnf, lits);
    todo!("search for a countermodel")
}

// ---------------------------------------------------------------------------
// Stage 4 — the CDCL driver
// ---------------------------------------------------------------------------

/// Algorithm 7.2.2.2C, simplified (no restarts, no clause purging): returns
/// a model (`model[v - 1]` = value of variable `v`) or `None` if `cnf` is
/// unsatisfiable.
///
/// ```text
/// C1. [Initialize.]   Load the clauses (merge duplicate literals, drop
///                     tautologies). Trail at level 0.
/// C2. [Propagate.]    Run unit propagation; enqueue every implication on
///                     the trail with its reason.
/// C3. [Conflict?]     If propagation reported a conflict at level 0,
///                     terminate: UNSAT. At level d > 0, go to C4;
///                     otherwise to C5.
/// C4. [Learn.]        (learned, back) := analyze(conflict). Backjump the
///                     trail to level `back`, unassigning the popped
///                     variables in the db (newest first). Add the learned
///                     clause — being asserting, it forces its literal at
///                     the new level. Return to C2.
/// C5. [Done?]         If every variable is assigned, no clause is false
///                     (propagation would have said so): return the model.
/// C6. [Decide.]       Pick an unassigned variable by your heuristic, open
///                     a new level, assign it. Return to C2.
/// ```
///
/// **Document your decision heuristic** (C6) in a comment — the reference
/// uses a deterministic integer VSIDS: bump a variable's activity when it
/// appears in a learned clause, halve all activities every 128 conflicts,
/// decide the unassigned variable of maximal activity (smallest index on
/// ties) with *phase saving* (re-use the polarity the variable had last,
/// initially false). Any deterministic complete heuristic passes the tests;
/// dumb-but-honest (smallest unassigned index, always false) also works —
/// measure how much slower the scale tests get.
pub fn solve(cnf: &Cnf) -> Option<Vec<bool>> {
    let _ = cnf;
    todo!("the CDCL loop: propagate, learn or decide")
}

/// Try all `2^num_vars` assignments (variable 1 = least significant bit);
/// first model wins. The honest cross-check. Panics above 25 variables.
pub fn solve_brute(cnf: &Cnf) -> Option<Vec<bool>> {
    let _ = cnf;
    todo!("enumerate assignments, evaluate")
}

/// The pigeonhole formula PHP(pigeons, holes): every pigeon in some hole,
/// no hole with two pigeons; variable `x[p][h] = p * holes + h + 1`.
/// Unsatisfiable iff pigeons > holes — but Haken proved every resolution
/// refutation of PHP(n+1, n) is exponential, so this family is *provably*
/// hard for CDCL (lesson §8). Clauses: one "sits somewhere" clause per
/// pigeon; one binary "not both here" clause per hole and pigeon pair.
pub fn pigeonhole_cnf(pigeons: usize, holes: usize) -> Cnf {
    let _ = (pigeons, holes);
    todo!("at-least-one per pigeon, at-most-one per hole")
}

/// The van der Waerden formula waerden(j, k; n) (§7.2.2.2's running
/// example): variable i = "integer i is red"; forbid every j-term all-red
/// and every k-term all-blue arithmetic progression inside 1..=n.
/// Satisfiable iff n < W(j, k); W(3, 3) = 9.
pub fn waerden_cnf(j: usize, k: usize, n: usize) -> Cnf {
    let _ = (j, k, n);
    todo!("one clause per arithmetic progression per colour")
}
