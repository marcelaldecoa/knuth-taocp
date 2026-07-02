//! Module 14 — Conflict-Driven Clause Learning.
//! Source: TAOCP Vol. 4B, §7.2.2.2, Algorithm C (simplified; see the lesson's
//! scope note for exactly which of Knuth's refinements we keep and drop).
//!
//! Literal convention (DIMACS style, identical to module 10; Knuth's internal
//! `2k + sign` coding is a trivial re-coding of this): a literal is a nonzero
//! `i32`; `+v` means "variable v is true", `-v` means "variable v is false",
//! for `1 <= v <= num_vars`. A clause is a `Vec<i32>`; a formula is the
//! conjunction of its clauses. This module is self-contained — it shares
//! module 10's conventions but imports nothing from it.
//!
//! The solver is split exactly along the lab's stage boundaries:
//! - `ClauseDb`  — clauses + two watched literals + current values (stage 1);
//! - `Trail`     — the assignment stack with decision levels and reasons
//!   (stage 2);
//! - `analyze`   — first-UIP conflict analysis by backward resolution
//!   (stage 3);
//! - `solve`     — the CDCL driver loop (stage 4).

// ---------------------------------------------------------------------------
// CNF formulas (shared by stages 3 and 4)
// ---------------------------------------------------------------------------

/// A formula in conjunctive normal form. `num_vars` declares the universe
/// `1..=num_vars`; every literal in `clauses` must satisfy
/// `1 <= lit.abs() <= num_vars`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cnf {
    pub num_vars: usize,
    pub clauses: Vec<Vec<i32>>,
}

impl Cnf {
    /// Evaluate the formula under a *complete* assignment:
    /// `assignment[v - 1]` is the value of variable `v`. A clause is true iff
    /// some literal is true; the formula is true iff every clause is (so the
    /// empty clause is false everywhere and the empty formula is true).
    ///
    /// Panics if `assignment.len() < num_vars`.
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
// Stage 1 — the clause database with two watched literals
// ---------------------------------------------------------------------------

/// Clause database with lazy unit propagation via **two watched literals**
/// (§7.2.2.2 — the data structure behind Knuth's Algorithm C, and before it
/// Algorithm D's "watched literal" refinement).
///
/// Invariant maintained by `propagate` (checkable with
/// [`check_watch_invariant`](ClauseDb::check_watch_invariant)): every clause
/// of length ≥ 2 watches exactly its slots 0 and 1, and whenever a watched
/// literal is false, either the *other* watched literal is true or the clause
/// has at most one non-false literal (it is unit or conflicting, and
/// `propagate` is about to deal with it).
///
/// The crucial property: the invariant is *stable under unassignment* (in
/// LIFO order), so backtracking touches no watch list at all — see the lesson
/// for the proof.
#[derive(Debug, Clone)]
pub struct ClauseDb {
    num_vars: usize,
    /// Clause `c`'s watched literals are `clauses[c][0]` and `clauses[c][1]`.
    clauses: Vec<Vec<i32>>,
    /// `watches[code(l)]` = indices of clauses currently watching literal `l`.
    watches: Vec<Vec<usize>>,
    /// `values[v - 1]` = current value of variable `v` (`None` = unassigned).
    values: Vec<Option<bool>>,
    /// Literals assigned true whose watchers have not been examined yet.
    queue: std::collections::VecDeque<i32>,
    /// Forced `(literal, reason clause)` pairs not yet collected by
    /// `take_implications` — the bridge to the stage-2 trail.
    implications: Vec<(i32, usize)>,
    /// A clause that was already falsified when it was added; reported by the
    /// next call to `propagate`.
    pending_conflict: Option<usize>,
}

impl ClauseDb {
    /// An empty database over variables `1..=num_vars`.
    pub fn new(num_vars: usize) -> ClauseDb {
        ClauseDb {
            num_vars,
            clauses: Vec::new(),
            watches: vec![Vec::new(); 2 * num_vars],
            values: vec![None; num_vars],
            queue: std::collections::VecDeque::new(),
            implications: Vec::new(),
            pending_conflict: None,
        }
    }

    pub fn num_vars(&self) -> usize {
        self.num_vars
    }

    pub fn num_clauses(&self) -> usize {
        self.clauses.len()
    }

    /// The literals of clause `c` (slots 0 and 1 are its watches).
    pub fn clause(&self, c: usize) -> &[i32] {
        &self.clauses[c]
    }

    /// Watch-list index of a literal: `+v -> 2(v-1)`, `-v -> 2(v-1) + 1`.
    /// (This is exactly Knuth's `2k + sign` literal coding.)
    fn code(lit: i32) -> usize {
        let v = lit.unsigned_abs() as usize;
        2 * (v - 1) + usize::from(lit < 0)
    }

    fn set(&mut self, lit: i32) {
        self.values[(lit.unsigned_abs() - 1) as usize] = Some(lit > 0);
    }

    /// The value of literal `lit` under the current partial assignment
    /// (`None` = its variable is unassigned).
    pub fn value(&self, lit: i32) -> Option<bool> {
        assert!(
            lit != 0 && lit.unsigned_abs() as usize <= self.num_vars,
            "literal {lit} out of range 1..={}",
            self.num_vars
        );
        self.values[(lit.unsigned_abs() - 1) as usize].map(|v| if lit > 0 { v } else { !v })
    }

    /// Add a clause and return its index. The first two non-false literals
    /// are moved to slots 0 and 1 and watched.
    ///
    /// Effects depend on the clause's status under the current assignment:
    /// - **≥ 2 non-false literals** — watched, nothing else happens;
    /// - **exactly 1 non-false literal, unassigned** — the clause is unit:
    ///   the literal is forced immediately (recorded as an implication with
    ///   this clause as reason). This is how learned clauses assert their
    ///   literal after a backjump, and how unit input clauses fire at level 0;
    /// - **all literals false** — the clause is conflicting on arrival; the
    ///   next `propagate` reports it.
    ///
    /// Mid-search callers must order the clause so slot 1 holds the most
    /// recently falsified literal (the CDCL driver puts the backjump-level
    /// literal there); clauses added before any assignment need no care.
    ///
    /// Panics on out-of-range literals or a repeated variable (definiteness:
    /// dedup or drop tautologies *before* adding — `solve` does).
    pub fn add_clause(&mut self, lits: Vec<i32>) -> usize {
        for (i, &lit) in lits.iter().enumerate() {
            assert!(
                lit != 0 && lit.unsigned_abs() as usize <= self.num_vars,
                "literal {lit} out of range 1..={}",
                self.num_vars
            );
            assert!(
                lits[..i].iter().all(|&l| l.unsigned_abs() != lit.unsigned_abs()),
                "variable {} occurs twice in {lits:?}",
                lit.unsigned_abs()
            );
        }
        let c = self.clauses.len();
        let mut lits = lits;

        // Move up to two non-false literals into the watch slots 0 and 1.
        let mut found = 0usize;
        for i in 0..lits.len() {
            if self.value(lits[i]) != Some(false) {
                lits.swap(found, i);
                found += 1;
                if found == 2 {
                    break;
                }
            }
        }

        if lits.len() >= 2 {
            self.watches[Self::code(lits[0])].push(c);
            self.watches[Self::code(lits[1])].push(c);
        }
        match found {
            0 => self.pending_conflict = Some(c), // dead on arrival
            1 if self.value(lits[0]).is_none() => {
                // Unit under the current assignment: force it now.
                let lit = lits[0];
                self.set(lit);
                self.implications.push((lit, c));
                self.queue.push_back(lit);
            }
            _ => {} // satisfied or ≥ 2 non-false: nothing to do
        }
        self.clauses.push(lits);
        c
    }

    /// Make `lit` true (a *decision*, or a test's scripted assignment) and
    /// schedule its watcher lists for examination by the next `propagate`.
    ///
    /// Panics if the variable is already assigned — deciding an assigned
    /// variable is always a driver bug.
    pub fn assign(&mut self, lit: i32) {
        assert!(
            self.value(lit).is_none(),
            "variable {} is already assigned",
            lit.unsigned_abs()
        );
        self.set(lit);
        self.queue.push_back(lit);
    }

    /// Undo the assignment of `var`. Callers must unassign in LIFO order
    /// (most recently assigned first), i.e. pop a *suffix* of the trail —
    /// that is what makes the watch invariant survive with **no fixup**.
    /// Only call between propagations (the queue must be empty).
    pub fn unassign(&mut self, var: usize) {
        assert!(
            1 <= var && var <= self.num_vars,
            "variable {var} out of range 1..={}",
            self.num_vars
        );
        assert!(
            self.queue.is_empty(),
            "unassign is only legal between propagations"
        );
        self.values[var - 1] = None;
    }

    /// Drain the implications recorded since the last call: every literal
    /// that was *forced* (by `propagate` or by adding a unit clause), paired
    /// with the index of the clause that forced it, in assignment order.
    /// The CDCL driver feeds these straight into `Trail::enqueue`.
    pub fn take_implications(&mut self) -> Vec<(i32, usize)> {
        std::mem::take(&mut self.implications)
    }

    /// Run unit propagation to a fixed point over the watched literals.
    /// Returns `Some(c)` the moment clause `c` is found with all literals
    /// false (a **conflict** — remaining queue entries are discarded, since
    /// the driver is about to backjump anyway), else `None`.
    ///
    /// For each newly true literal `p`, only the clauses watching `¬p` are
    /// examined (that is the whole point — clauses watching two other
    /// literals cannot have become unit or false). Each such clause is
    /// resolved by the first case that applies:
    /// 1. the other watch is true → clause satisfied, keep watches;
    /// 2. some other literal is non-false → move the watch there;
    /// 3. the other watch is unassigned → the clause is unit: force it;
    /// 4. the other watch is false → conflict.
    pub fn propagate(&mut self) -> Option<usize> {
        if let Some(c) = self.pending_conflict.take() {
            self.queue.clear();
            return Some(c);
        }
        while let Some(p) = self.queue.pop_front() {
            let np = -p; // this literal just became FALSE
            let ws = std::mem::take(&mut self.watches[Self::code(np)]);
            let mut kept: Vec<usize> = Vec::with_capacity(ws.len());
            let mut i = 0;
            while i < ws.len() {
                let c = ws[i];
                i += 1;
                // Normalize: the false watch sits in slot 1.
                if self.clauses[c][0] == np {
                    self.clauses[c].swap(0, 1);
                }
                debug_assert_eq!(self.clauses[c][1], np);
                let w0 = self.clauses[c][0];
                // Case 1: satisfied by the other watch.
                if self.value(w0) == Some(true) {
                    kept.push(c);
                    continue;
                }
                // Case 2: find a replacement watch among the tail literals.
                let mut moved = false;
                for k in 2..self.clauses[c].len() {
                    let lk = self.clauses[c][k];
                    if self.value(lk) != Some(false) {
                        self.clauses[c].swap(1, k);
                        self.watches[Self::code(lk)].push(c);
                        moved = true;
                        break;
                    }
                }
                if moved {
                    continue;
                }
                // No replacement: the clause keeps watching np, and w0 is its
                // last hope.
                kept.push(c);
                match self.value(w0) {
                    // Case 4: everything false — conflict.
                    Some(false) => {
                        kept.extend_from_slice(&ws[i..]);
                        self.watches[Self::code(np)] = kept;
                        self.queue.clear();
                        return Some(c);
                    }
                    // Case 3: unit — force w0 with reason c.
                    None => {
                        self.set(w0);
                        self.implications.push((w0, c));
                        self.queue.push_back(w0);
                    }
                    Some(true) => unreachable!("handled as case 1"),
                }
            }
            self.watches[Self::code(np)] = kept;
        }
        None
    }

    /// Check the watched-literal invariant (call it after `propagate`
    /// returns `None`; mid-propagation states may legitimately violate it).
    ///
    /// Structural part: every clause of length ≥ 2 appears in exactly the two
    /// watch lists of its slot-0 and slot-1 literals; shorter clauses are
    /// never watched. Semantic part: if a watched literal is false, then the
    /// other watch is true, or the clause has at most one non-false literal
    /// (unit or conflicting).
    pub fn check_watch_invariant(&self) -> bool {
        let mut occurrences = vec![0usize; self.clauses.len()];
        for list in &self.watches {
            for &c in list {
                occurrences[c] += 1;
            }
        }
        for (c, clause) in self.clauses.iter().enumerate() {
            if clause.len() < 2 {
                if occurrences[c] != 0 {
                    return false;
                }
                continue;
            }
            if occurrences[c] != 2
                || !self.watches[Self::code(clause[0])].contains(&c)
                || !self.watches[Self::code(clause[1])].contains(&c)
            {
                return false;
            }
            let non_false = clause
                .iter()
                .filter(|&&l| self.value(l) != Some(false))
                .count();
            let v0 = self.value(clause[0]);
            let v1 = self.value(clause[1]);
            for (w, other) in [(v0, v1), (v1, v0)] {
                if w == Some(false) && other != Some(true) && non_false >= 2 {
                    return false;
                }
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// Stage 2 — the trail: decisions, levels, reasons
// ---------------------------------------------------------------------------

/// The **trail** (§7.2.2.2: Knuth's array `L` with the level boundaries `i_d`
/// and reason array): the sequence of all currently true literals in the
/// order they were assigned, partitioned into *decision levels*. Level 0
/// holds consequences of the formula alone; level `d ≥ 1` starts at the
/// `d`-th decision. Each propagated literal remembers the clause that forced
/// it (its **reason**); decisions have none. Trail + reasons *are* the
/// implication graph of the lesson, stored as one flat array.
#[derive(Debug, Clone)]
pub struct Trail {
    /// The literals, oldest first.
    lits: Vec<i32>,
    /// `level_start[d]` = index in `lits` where level `d` begins;
    /// `level_start[0] == 0` always, and the current decision level is
    /// `level_start.len() - 1`.
    level_start: Vec<usize>,
    /// Per variable: `Some((level, reason))` while on the trail.
    info: Vec<Option<(usize, Option<usize>)>>,
}

impl Trail {
    /// An empty trail over variables `1..=num_vars`, at decision level 0.
    pub fn new(num_vars: usize) -> Trail {
        Trail {
            lits: Vec::new(),
            level_start: vec![0],
            info: vec![None; num_vars],
        }
    }

    /// The current decision level (0 before any decision).
    pub fn decision_level(&self) -> usize {
        self.level_start.len() - 1
    }

    /// Number of literals currently on the trail.
    pub fn len(&self) -> usize {
        self.lits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lits.is_empty()
    }

    /// All trail literals in assignment order (oldest first).
    pub fn literals(&self) -> &[i32] {
        &self.lits
    }

    fn push(&mut self, lit: i32, reason: Option<usize>) {
        let v = lit.unsigned_abs() as usize;
        assert!(
            v >= 1 && v <= self.info.len(),
            "literal {lit} out of range 1..={}",
            self.info.len()
        );
        assert!(
            self.info[v - 1].is_none(),
            "variable {v} is already on the trail"
        );
        self.info[v - 1] = Some((self.decision_level(), reason));
        self.lits.push(lit);
    }

    /// Open a new decision level and record `lit` as its decision.
    /// Decisions never carry a reason.
    pub fn decide(&mut self, lit: i32) {
        self.level_start.push(self.lits.len());
        self.push(lit, None);
    }

    /// Record `lit` at the *current* level. Propagated literals pass
    /// `Some(reason clause)`; `None` is for scripted root-level assumptions.
    pub fn enqueue(&mut self, lit: i32, reason: Option<usize>) {
        self.push(lit, reason);
    }

    /// The decision level at which `var` was assigned (`None` if it is not
    /// on the trail).
    pub fn level_of(&self, var: usize) -> Option<usize> {
        self.info[var - 1].map(|(level, _)| level)
    }

    /// The clause that forced `var`, or `None` for decisions and unassigned
    /// variables.
    pub fn reason_of(&self, var: usize) -> Option<usize> {
        self.info[var - 1].and_then(|(_, reason)| reason)
    }

    /// The literals assigned at decision level `level`, in trail order.
    pub fn assignments_at_level(&self, level: usize) -> &[i32] {
        assert!(
            level <= self.decision_level(),
            "level {level} does not exist (current level is {})",
            self.decision_level()
        );
        let start = self.level_start[level];
        let end = self
            .level_start
            .get(level + 1)
            .copied()
            .unwrap_or(self.lits.len());
        &self.lits[start..end]
    }

    /// Pop every literal above `level` (i.e. levels `level + 1 ..`), erasing
    /// their level/reason records, and return them **newest first** — exactly
    /// the LIFO order in which the caller must `ClauseDb::unassign` them.
    /// Levels `0..=level` are untouched.
    pub fn backjump(&mut self, level: usize) -> Vec<i32> {
        assert!(
            level <= self.decision_level(),
            "cannot backjump to level {level}: current level is {}",
            self.decision_level()
        );
        if level == self.decision_level() {
            return Vec::new();
        }
        let cut = self.level_start[level + 1];
        let mut popped = self.lits.split_off(cut);
        popped.reverse();
        for &lit in &popped {
            self.info[(lit.unsigned_abs() - 1) as usize] = None;
        }
        self.level_start.truncate(level + 1);
        popped
    }
}

// ---------------------------------------------------------------------------
// Stage 3 — first-UIP conflict analysis (the heart of Algorithm C)
// ---------------------------------------------------------------------------

/// First-UIP conflict analysis (Algorithm 7.2.2.2C's steps C7/C8 — Knuth's
/// "blit"/"stamp" computation): given clause `conflict` whose literals are
/// all false on `trail`, derive by backward resolution the **learned clause**
/// and the level to backjump to.
///
/// Method: maintain a resolvent, initially the conflict clause. Walk the
/// trail backwards; while the resolvent contains **more than one** literal of
/// the current decision level, resolve it with the reason clause of the most
/// recently assigned such literal (that literal drops out; the reason's other
/// literals — all false — enter). Literals from level 0 are discarded (false
/// forever). The loop must terminate at the latest at the current decision,
/// which has no reason but by then is the *only* current-level literal left —
/// the **first UIP** is whichever single current-level literal survives.
///
/// Returns `(learned, backjump_level)` where
/// - `learned[0]` is the **asserting literal** (the negation of the first
///   UIP), and the remaining literals are false at levels `1..=backjump`;
/// - `learned[1]` (when present) is a literal from `backjump_level` itself —
///   the watch-ordering `ClauseDb::add_clause` wants;
/// - `backjump_level` is the second-highest level in the clause, `0` if the
///   learned clause is unit.
///
/// Panics if called at decision level 0 (a root conflict means UNSAT; there
/// is nothing to learn) or if some conflict-clause literal is not false.
pub fn analyze(conflict: usize, trail: &Trail, db: &ClauseDb) -> (Vec<i32>, usize) {
    let cur = trail.decision_level();
    assert!(cur >= 1, "a conflict at level 0 is final: the formula is UNSAT");
    for &lit in db.clause(conflict) {
        assert_eq!(
            db.value(lit),
            Some(false),
            "conflict clause {conflict} has a non-false literal {lit}"
        );
    }

    let mut seen = vec![false; db.num_vars() + 1];
    let mut learned: Vec<i32> = vec![0]; // slot 0 reserved for the asserting literal
    let mut open = 0usize; // current-level literals in the resolvent
    let mut reason: &[i32] = db.clause(conflict);
    let mut pivot = 0usize; // variable resolved on (0 = none, first round)
    let mut i = trail.len();

    let uip = loop {
        // Merge the reason's literals into the resolvent.
        for &q in reason {
            let v = q.unsigned_abs() as usize;
            if v == pivot || seen[v] {
                continue; // the pivot drops out; duplicates merge
            }
            seen[v] = true;
            let level = trail
                .level_of(v)
                .expect("every literal of a reason clause is assigned");
            if level == cur {
                open += 1;
            } else if level > 0 {
                learned.push(q); // a false literal from a lower level
            } // level-0 literals are false forever: drop them
        }
        // Walk back to the most recent trail literal in the resolvent. It is
        // at the current level (lower-level members lie deeper in the trail
        // than every current-level one, and open > 0 whenever we get here).
        let p = loop {
            i -= 1;
            let p = trail.literals()[i];
            if seen[p.unsigned_abs() as usize] {
                break p;
            }
        };
        open -= 1;
        if open == 0 {
            break p; // p is the first UIP: the lone current-level literal
        }
        // Resolve on p: replace it by its reason's other literals.
        pivot = p.unsigned_abs() as usize;
        let r = trail
            .reason_of(pivot)
            .expect("only the decision lacks a reason, and it is always the last current-level literal standing");
        reason = db.clause(r);
    };

    learned[0] = -uip;
    // Backjump level = highest level below `cur` in the clause; also move a
    // literal of that level into slot 1 for the watch discipline.
    let mut back = 0;
    let mut back_slot = 0;
    for (k, &lit) in learned.iter().enumerate().skip(1) {
        let level = trail.level_of(lit.unsigned_abs() as usize).unwrap();
        if level > back {
            back = level;
            back_slot = k;
        }
    }
    if back_slot > 1 {
        learned.swap(1, back_slot);
    }
    (learned, back)
}

/// Does `cnf` logically imply the clause `lits`? Brute force: `cnf ⊨ C` iff
/// no assignment satisfies `cnf` while falsifying every literal of `C`.
/// The honest cross-check that learned clauses are sound. Panics above 22
/// variables.
pub fn brute_force_implies(cnf: &Cnf, lits: &[i32]) -> bool {
    assert!(cnf.num_vars <= 22, "brute force is limited to 22 variables");
    for &lit in lits {
        assert!(
            lit != 0 && lit.unsigned_abs() as usize <= cnf.num_vars,
            "literal {lit} out of range 1..={}",
            cnf.num_vars
        );
    }
    for bits in 0u64..(1u64 << cnf.num_vars) {
        let assignment: Vec<bool> = (0..cnf.num_vars).map(|v| bits >> v & 1 == 1).collect();
        let clause_false = lits.iter().all(|&lit| {
            let value = assignment[(lit.unsigned_abs() - 1) as usize];
            if lit > 0 {
                !value
            } else {
                value
            }
        });
        if clause_false && cnf.evaluate(&assignment) {
            return false; // counterexample: cnf true, clause false
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Stage 4 — the CDCL driver (Algorithm 7.2.2.2C, simplified)
// ---------------------------------------------------------------------------

/// Algorithm 7.2.2.2C (conflict-driven clause learning), simplified: no
/// restarts, no clause purging, and an integer variant of the VSIDS activity
/// heuristic in place of Knuth's damped `ACT` scores. Returns a model
/// (`model[v - 1]` = value of variable `v`) or `None` if unsatisfiable.
///
/// **Decision heuristic (documented, as the tests require):** each variable
/// has an integer *activity*, bumped by 1 whenever the variable appears in a
/// learned clause and halved for all variables every 128 conflicts (a cheap,
/// deterministic VSIDS: recent conflicts dominate). Decisions pick the
/// unassigned variable of maximal activity (lowest index breaks ties) and
/// assign its *saved phase* — the polarity it last had, initially `false`
/// (phase saving keeps the solver working on the same part of the search
/// space after a backjump).
///
/// Input clauses are cleaned first: duplicate literals are merged and
/// tautologies (containing `v` and `¬v`) dropped — both are legal CNF but
/// break the watch discipline's "distinct variables" precondition.
pub fn solve(cnf: &Cnf) -> Option<Vec<bool>> {
    let n = cnf.num_vars;
    // C1. [Initialize.] Load the clause database; trail at level 0.
    let mut db = ClauseDb::new(n);
    let mut trail = Trail::new(n);
    for clause in &cnf.clauses {
        let mut c = clause.clone();
        c.sort_unstable();
        c.dedup();
        if c.iter().any(|&l| c.binary_search(&-l).is_ok()) {
            continue; // tautology: true under every assignment
        }
        db.add_clause(c);
    }
    let mut activity: Vec<u64> = vec![0; n + 1];
    let mut phase: Vec<bool> = vec![false; n + 1];
    let mut conflicts: u64 = 0;

    loop {
        // C2. [Propagate.] Unit propagation to a fixed point; move every
        // forced literal onto the trail with its reason.
        let conflict = db.propagate();
        for (lit, reason) in db.take_implications() {
            trail.enqueue(lit, Some(reason));
        }
        if let Some(c) = conflict {
            // C3. [Root conflict?] A conflict with no decisions active means
            // the empty clause is derivable: UNSAT.
            if trail.decision_level() == 0 {
                return None;
            }
            // C4. [Analyze.] Learn the first-UIP clause.
            let (learned, back) = analyze(c, &trail, &db);
            conflicts += 1;
            for &lit in &learned {
                activity[lit.unsigned_abs() as usize] += 1;
            }
            if conflicts % 128 == 0 {
                for a in &mut activity {
                    *a >>= 1; // decay: recent conflicts dominate
                }
            }
            // C5. [Backjump.] Pop the trail to the backjump level, saving
            // phases; the watch lists need no repair (stage-1 lemma).
            for lit in trail.backjump(back) {
                let v = lit.unsigned_abs() as usize;
                phase[v] = lit > 0;
                db.unassign(v);
            }
            // C6. [Learn.] Install the clause; being asserting, it
            // immediately forces its slot-0 literal at the new level, and
            // the next C2 propagates from there.
            db.add_clause(learned);
        } else {
            // C7. [Complete?] No conflict and no unassigned variable: every
            // clause is satisfied (watched propagation misses no falsified
            // clause) — a model.
            let decision = (1..=n)
                .filter(|&v| db.value(v as i32).is_none())
                .max_by_key(|&v| (activity[v], std::cmp::Reverse(v)));
            let Some(v) = decision else {
                return Some((1..=n).map(|v| db.value(v as i32).unwrap()).collect());
            };
            // C8. [Decide.] New level; saved phase.
            let lit = if phase[v] { v as i32 } else { -(v as i32) };
            trail.decide(lit);
            db.assign(lit);
        }
    }
}

/// Brute-force model search over all `2^num_vars` assignments (variable 1 is
/// the least significant bit) — the honest cross-check for `solve` on small
/// instances. Panics above 25 variables.
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

/// The pigeonhole formula PHP(m, n): "m pigeons in n holes, each pigeon in
/// some hole, no hole with two pigeons"; unsatisfiable iff m > n. Haken
/// (1985): every *resolution* refutation of PHP(n+1, n) is exponential in n —
/// and since CDCL's learned clauses are resolution consequences, clause
/// learning provably cannot make pigeonhole easy. §7.2.2.2's canonical hard
/// family, and this module's scale test.
///
/// Variable `x[p][h] = p * holes + h + 1` ("pigeon p sits in hole h") for
/// `p in 0..pigeons`, `h in 0..holes`.
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

/// The van der Waerden formula waerden(j, k; n) — Knuth's running example in
/// §7.2.2.2. Variable `i` means "integer i is red" (false = blue); clauses
/// forbid j-term all-red and k-term all-blue arithmetic progressions inside
/// `1..=n`. Satisfiable iff `n < W(j, k)`; W(3, 3) = 9.
pub fn waerden_cnf(j: usize, k: usize, n: usize) -> Cnf {
    let mut clauses: Vec<Vec<i32>> = Vec::new();
    for d in 1..=n {
        for a in 1..=n {
            if a + (j - 1) * d > n {
                break;
            }
            clauses.push((0..j).map(|t| -((a + t * d) as i32)).collect());
        }
    }
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
// Unit tests: worked examples (the lesson's traces, machine-checked)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// The stage-1 mechanics: chains propagate lazily, conflicts name the
    /// clause, and the invariant holds at every fixed point.
    #[test]
    fn watched_propagation_chain_and_conflict() {
        let mut db = ClauseDb::new(4);
        db.add_clause(vec![-1, 2]); // c0
        db.add_clause(vec![-2, 3]); // c1
        db.add_clause(vec![-3, -1, 4]); // c2 — its watch must *move* before it fires
        assert!(db.check_watch_invariant());
        db.assign(1);
        assert_eq!(db.propagate(), None);
        assert_eq!(db.value(2), Some(true));
        assert_eq!(db.value(3), Some(true));
        assert_eq!(db.value(4), Some(true));
        assert_eq!(db.take_implications(), vec![(2, 0), (3, 1), (4, 2)]);
        assert!(db.check_watch_invariant());

        // (x1)(¬x1 ∨ x2)(¬x2): the conflict is clause 2, uniquely.
        let mut db = ClauseDb::new(2);
        db.add_clause(vec![1]);
        db.add_clause(vec![-1, 2]);
        let c = db.add_clause(vec![-2]);
        assert_eq!(db.propagate(), Some(c));
    }

    /// The lesson's §4 worked example, machine-checked: 8 variables, three
    /// decision levels, first UIP x4, learned clause (¬x4 ∨ x8), backjump to
    /// level 2. The trail is scripted by hand so the trace matches the text
    /// line by line.
    #[test]
    fn first_uip_worked_example() {
        let clauses: Vec<Vec<i32>> = vec![
            vec![-1, 2],     // C0
            vec![-1, 3, 7],  // C1
            vec![-2, -3, 4], // C2
            vec![-4, 5, 8],  // C3
            vec![-4, 6],     // C4
            vec![-5, -6],    // C5 — the conflict
        ];
        let mut db = ClauseDb::new(8);
        for c in &clauses {
            db.add_clause(c.clone());
        }
        let mut trail = Trail::new(8);
        for (lit, reason) in [
            (-7, None),    // level 1 decision
            (-8, None),    // level 2 decision
            (1, None),     // level 3 decision
            (2, Some(0)),  // C0
            (3, Some(1)),  // C1
            (4, Some(2)),  // C2
            (5, Some(3)),  // C3
            (6, Some(4)),  // C4
        ] {
            if reason.is_none() {
                trail.decide(lit);
            } else {
                trail.enqueue(lit, reason);
            }
            db.assign(lit);
        }
        let (learned, back) = analyze(5, &trail, &db);
        let mut sorted = learned.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![-4, 8]);
        assert_eq!(learned[0], -4, "slot 0 holds the asserting literal");
        assert_eq!(back, 2);
        // Sound: the learned clause is implied by the original formula.
        let cnf = Cnf { num_vars: 8, clauses };
        assert!(brute_force_implies(&cnf, &learned));
        assert!(!brute_force_implies(&cnf, &[8])); // but not by magic
    }

    /// Backjumping pops exactly the popped levels, newest first, and the
    /// watch invariant survives with no fixup.
    #[test]
    fn trail_backjump_and_free_backtracking() {
        let mut db = ClauseDb::new(5);
        db.add_clause(vec![-1, 2]);
        db.add_clause(vec![-3, 4]);
        let mut trail = Trail::new(5);
        trail.decide(1);
        db.assign(1);
        assert_eq!(db.propagate(), None);
        for (l, r) in db.take_implications() {
            trail.enqueue(l, Some(r));
        }
        trail.decide(3);
        db.assign(3);
        assert_eq!(db.propagate(), None);
        for (l, r) in db.take_implications() {
            trail.enqueue(l, Some(r));
        }
        assert_eq!(trail.literals(), &[1, 2, 3, 4]);
        assert_eq!(trail.assignments_at_level(2), &[3, 4]);
        let popped = trail.backjump(1);
        assert_eq!(popped, vec![4, 3], "newest first");
        for lit in popped {
            db.unassign(lit.unsigned_abs() as usize);
        }
        assert!(db.check_watch_invariant());
        assert_eq!(trail.level_of(2), Some(1));
        assert_eq!(trail.level_of(4), None);
        // Watches still work after the un-repaired backtrack.
        trail.decide(3);
        db.assign(3);
        assert_eq!(db.propagate(), None);
        assert_eq!(db.value(4), Some(true));
    }

    /// The full solver against brute force on deterministic random 3-SAT.
    #[test]
    fn solver_agrees_with_brute_force() {
        let mut x: u64 = 20260702;
        let mut rng = move || {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            x >> 33
        };
        for trial in 0..80u64 {
            let n = 4 + (trial % 7) as usize; // 4..=10 variables
            let m = n * (2 + (trial % 4) as usize); // ratios 2..=5
            let mut clauses = Vec::new();
            for _ in 0..m {
                let mut vars = Vec::new();
                while vars.len() < 3 {
                    let v = 1 + (rng() as usize) % n;
                    if !vars.contains(&v) {
                        vars.push(v);
                    }
                }
                clauses.push(
                    vars.iter()
                        .map(|&v| if rng() % 2 == 0 { v as i32 } else { -(v as i32) })
                        .collect(),
                );
            }
            let cnf = Cnf { num_vars: n, clauses };
            match (solve(&cnf), solve_brute(&cnf)) {
                (Some(model), Some(_)) => assert!(cnf.evaluate(&model)),
                (None, None) => {}
                (a, b) => panic!(
                    "trial {trial}: solve says {:?}, brute force says {:?}",
                    a.is_some(),
                    b.is_some()
                ),
            }
        }
    }

    /// §7.2.2.2's running example W(3,3) = 9, and pigeonhole both ways.
    #[test]
    fn waerden_and_pigeonhole() {
        let sat = waerden_cnf(3, 3, 8);
        let model = solve(&sat).expect("waerden(3,3;8) is satisfiable");
        assert!(sat.evaluate(&model));
        assert_eq!(solve(&waerden_cnf(3, 3, 9)), None);

        let php44 = pigeonhole_cnf(4, 4);
        let model = solve(&php44).expect("4 pigeons fit in 4 holes");
        assert!(php44.evaluate(&model));
        assert_eq!(solve(&pigeonhole_cnf(5, 4)), None);
        assert_eq!(solve(&pigeonhole_cnf(6, 5)), None);
    }
}
