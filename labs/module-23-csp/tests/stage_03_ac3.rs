//! Stage 3 — arc consistency (AC-3).
//! TAOCP Vol. 4 Fascicle 7, §7.2.2.3 (consistency filtering).

use lab_23_csp::*;

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        (self.next() >> 33) % n.max(1)
    }
}

fn random_csp(seed: u64) -> Csp {
    let mut r = Lcg(seed);
    let domains: Vec<Vec<u32>> = (0..4)
        .map(|_| (0..2 + r.below(2) as u32).collect())
        .collect();
    let mut csp = Csp::new(domains);
    for x in 0..4usize {
        for y in (x + 1)..4 {
            if r.below(3) == 0 {
                continue;
            }
            let dx = csp.domains[x].clone();
            let dy = csp.domains[y].clone();
            let mut allowed = Vec::new();
            for &a in &dx {
                for &b in &dy {
                    if r.below(3) < 2 {
                        allowed.push((a, b));
                    }
                }
            }
            csp.add_allowed(x, y, allowed);
        }
    }
    csp
}

fn less_pairs() -> Vec<(u32, u32)> {
    (1..=3u32)
        .flat_map(|a| (1..=3u32).map(move |b| (a, b)))
        .filter(|&(a, b)| a < b)
        .collect()
}

#[test]
fn the_ordering_chain_collapses_to_singletons() {
    // X < Y < Z over {1, 2, 3}: consistency alone pins 1 < 2 < 3 — no
    // search at all. The textbook demonstration that filtering propagates.
    let mut csp = Csp::new(vec![vec![1, 2, 3]; 3]);
    csp.add_allowed(0, 1, less_pairs());
    csp.add_allowed(1, 2, less_pairs());
    assert!(csp.ac3());
    assert_eq!(csp.domains, vec![vec![1], vec![2], vec![3]]);
}

#[test]
fn a_wipeout_returns_false() {
    // X < Y and Y < X cannot both hold.
    let mut csp = Csp::new(vec![vec![1, 2, 3]; 2]);
    csp.add_allowed(0, 1, less_pairs());
    csp.add_allowed(1, 0, less_pairs());
    assert!(!csp.ac3());
}

#[test]
fn queens_are_already_arc_consistent() {
    // Every row value of every column has support in every other column, so
    // AC-3 deletes nothing on queens-6.
    let mut csp = queens_csp(6);
    assert!(csp.ac3());
    assert_eq!(csp.domains, queens_csp(6).domains);
}

#[test]
fn solutions_are_preserved_exactly() {
    // AC-3 deletes only unsupported values, which no solution can use — so
    // the solution set before and after pruning is identical.
    for seed in 1..=25u64 {
        let csp = random_csp(seed);
        let before = csp.solve_basic().0;
        let mut pruned = csp.clone();
        let alive = pruned.ac3();
        if !alive {
            assert!(before.is_empty(), "seed {seed}: wipeout but solutions existed");
            continue;
        }
        let after = pruned.solve_basic().0;
        assert_eq!(before, after, "seed {seed}: pruning changed the solution set");
        // Pruned domains are subsets of the originals.
        for (d_after, d_before) in pruned.domains.iter().zip(&csp.domains) {
            assert!(d_after.iter().all(|v| d_before.contains(v)), "seed {seed}");
        }
    }
}

#[test]
fn a_second_wave_reaches_arcs_aimed_at_the_pruned_variable() {
    // Two-wave propagation, built so the ONLY way to reach the fixpoint is
    // to re-examine an arc whose HEAD is the variable that was just pruned
    // (requeueing any other set of arcs cannot help):
    //   D0 = {0,1}, D1 = {0,1,2}, D2 = {0,1};
    //   c0 = (0,1) allowing {(0,0),(0,1),(1,0),(1,1)}  — kills 2 from D1;
    //   c1 = (2,1) allowing {(0,0),(0,1),(0,2),(1,2)}  — value 1 of var 2 is
    //        supported ONLY by var 1's doomed value 2.
    // First wave: nothing else moves; when c0 finally deletes 2 from D1, the
    // arc (var 2 → var 1) must be revisited to delete var 2's value 1. The
    // unique fixpoint: D0 = {0,1}, D1 = {0,1}, D2 = {0}.
    let build = || {
        let mut csp = Csp::new(vec![vec![0, 1], vec![0, 1, 2], vec![0, 1]]);
        csp.add_allowed(0, 1, vec![(0, 0), (0, 1), (1, 0), (1, 1)]);
        csp.add_allowed(2, 1, vec![(0, 0), (0, 1), (0, 2), (1, 2)]);
        csp
    };
    let mut csp = build();
    assert!(csp.ac3());
    assert_eq!(
        csp.domains,
        vec![vec![0, 1], vec![0, 1], vec![0]],
        "the second propagation wave was not applied"
    );
    // A correct fixpoint is a fixpoint: running AC-3 again changes nothing.
    // (An under-propagating implementation leaves work behind that a second
    // run would find — this catches that whole class, whatever the queue
    // discipline.)
    let snapshot = csp.domains.clone();
    assert!(csp.ac3());
    assert_eq!(csp.domains, snapshot, "AC-3 left the domains short of the fixpoint");
    // And solutions are what they were before pruning.
    assert_eq!(build().solve_basic().0, csp.solve_basic().0);
}

#[test]
fn the_fixpoint_is_order_independent_and_idempotent() {
    // The maximal arc-consistent subdomains are unique, so inserting the
    // same constraints in the opposite order must reach the same fixpoint,
    // and running AC-3 twice must change nothing further.
    for seed in 1..=15u64 {
        let csp = random_csp(seed);
        let mut fwd = csp.clone();
        let mut rev = Csp::new(csp.domains.clone());
        for c in csp.constraints.iter().rev() {
            rev.add_allowed(c.x, c.y, c.allowed.clone());
        }
        let a = fwd.ac3();
        let b = rev.ac3();
        assert_eq!(a, b, "seed {seed}");
        if a {
            assert_eq!(fwd.domains, rev.domains, "seed {seed}: fixpoints differ");
            let snapshot = fwd.domains.clone();
            assert!(fwd.ac3(), "seed {seed}: second run flipped to wipeout");
            assert_eq!(fwd.domains, snapshot, "seed {seed}: AC-3 is not idempotent");
        }
    }
}
