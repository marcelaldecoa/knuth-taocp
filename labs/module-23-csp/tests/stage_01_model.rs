//! Stage 1 — the CSP model and basic backtracking.
//! TAOCP Vol. 4 Fascicle 7, §7.2.2.3; Algorithm 7.2.2B for the search.

use lab_23_csp::*;

/// Deterministic LCG (Knuth's MMIX constants) — the course's standard way to
/// get reproducible "random" instances with zero external crates.
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
        (self.next() >> 33) % n.max(1) // discard the weak low bits, then reduce
    }
}

/// An independent oracle: enumerate the full Cartesian product of the
/// domains and keep the assignments every constraint allows. Deliberately
/// shares no code path with the lab's backtracker (linear `contains`, no
/// pruning).
fn oracle_solutions(csp: &Csp) -> Vec<Vec<u32>> {
    fn rec(csp: &Csp, cur: &mut Vec<u32>, out: &mut Vec<Vec<u32>>) {
        if cur.len() == csp.domains.len() {
            let all_ok = csp
                .constraints
                .iter()
                .all(|c| c.allowed.contains(&(cur[c.x], cur[c.y])));
            if all_ok {
                out.push(cur.clone());
            }
            return;
        }
        for &a in &csp.domains[cur.len()] {
            cur.push(a);
            rec(csp, cur, out);
            cur.pop();
        }
    }
    let mut out = Vec::new();
    rec(csp, &mut Vec::new(), &mut out);
    out
}

/// A small random CSP: 4 variables, domains of size 2–3, a random allowed
/// set for each of a few random variable pairs.
fn random_csp(seed: u64) -> Csp {
    let mut r = Lcg(seed);
    let domains: Vec<Vec<u32>> = (0..4)
        .map(|_| (0..2 + r.below(2) as u32).collect())
        .collect();
    let mut csp = Csp::new(domains);
    for x in 0..4usize {
        for y in (x + 1)..4 {
            if r.below(3) == 0 {
                continue; // leave some pairs unconstrained
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

#[test]
fn queens_counts_match_the_text() {
    // The classic table (§7.2.2): 2, 10, 4, 40, 92 solutions for n = 4..8.
    for (n, want) in [(4usize, 2usize), (5, 10), (6, 4), (7, 40), (8, 92)] {
        let (sols, _) = queens_csp(n).solve_basic();
        assert_eq!(sols.len(), want, "queens n = {n}");
    }
}

#[test]
fn queens_solutions_are_lexicographic_and_verified() {
    let csp = queens_csp(6);
    let (sols, _) = csp.solve_basic();
    // First solution in lexicographic order is the well-known (1,3,5,0,2,4).
    assert_eq!(sols[0], vec![1, 3, 5, 0, 2, 4]);
    let mut sorted = sols.clone();
    sorted.sort();
    assert_eq!(sols, sorted, "solve_basic must emit solutions in lexicographic order");
    for s in &sols {
        assert!(csp.check(s), "reported solution fails check(): {s:?}");
    }
    // And check() rejects a non-solution (two queens sharing row 0).
    assert!(!csp.check(&[0, 0, 1, 3, 5, 2]));
}

#[test]
fn coloring_counts_are_the_chromatic_polynomial() {
    // P(C5, 3) = (3-1)^5 + (-1)^5 (3-1) = 30; P(K3, 3) = 3! = 6; K4 needs 4.
    let c5 = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)];
    assert_eq!(coloring_csp(5, &c5, 3).solve_basic().0.len(), 30);
    let k3 = [(0, 1), (0, 2), (1, 2)];
    assert_eq!(coloring_csp(3, &k3, 3).solve_basic().0.len(), 6);
    let k4 = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
    assert_eq!(coloring_csp(4, &k4, 3).solve_basic().0.len(), 0);
    // A path on 3 vertices with 2 colors: P = k(k-1)^2 = 2.
    assert_eq!(coloring_csp(3, &[(0, 1), (1, 2)], 2).solve_basic().0.len(), 2);
}

#[test]
fn agrees_with_the_product_enumeration_oracle() {
    for seed in 1..=20u64 {
        let csp = random_csp(seed);
        let (sols, nodes) = csp.solve_basic();
        let want = oracle_solutions(&csp);
        assert_eq!(sols, want, "seed {seed}: solution sets differ");
        // Basic backtracking never places more values than the full product
        // tree contains placements.
        let product_places: u64 = {
            let mut layers = 0u64;
            let mut width = 1u64;
            for d in &csp.domains {
                width = width.saturating_mul(d.len() as u64);
                layers = layers.saturating_add(width);
                let _ = &d;
            }
            layers
        };
        assert!(nodes <= product_places, "seed {seed}: {nodes} nodes > product tree");
    }
}

#[test]
fn node_count_is_pinned_for_queens_5() {
    // With the contract's exact order (variables by index, values ascending,
    // count every tentative placement) the tree size is fully determined:
    // 220 nodes for queens-5.
    let (_, nodes) = queens_csp(5).solve_basic();
    assert_eq!(nodes, 220);
}

#[test]
fn domains_are_normalized() {
    let csp = Csp::new(vec![vec![3, 1, 2, 1], vec![5, 5]]);
    assert_eq!(csp.domains, vec![vec![1, 2, 3], vec![5]]);
}

#[test]
#[should_panic(expected = "distinct")]
fn self_constraint_is_rejected() {
    // Documented contract: constraining a variable against itself panics
    // with a message containing "distinct".
    let mut csp = Csp::new(vec![vec![0, 1]; 2]);
    csp.add_allowed(1, 1, vec![(0, 0)]);
}

#[test]
#[should_panic(expected = "range")]
fn out_of_range_variable_is_rejected() {
    // Documented contract: an endpoint >= n panics with "range".
    let mut csp = Csp::new(vec![vec![0, 1]; 2]);
    csp.add_allowed(0, 2, vec![(0, 0)]);
}

#[test]
#[should_panic(expected = "length")]
fn wrong_length_assignment_is_rejected() {
    // Documented contract: check() panics with "length" on a size mismatch.
    let csp = Csp::new(vec![vec![0, 1]; 2]);
    csp.check(&[0]);
}
