//! Stage 2 — forward checking and the MRV ordering heuristic.
//! TAOCP Vol. 4 Fascicle 7, §7.2.2.3 (lookahead, dancing cells, ordering).

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

#[test]
fn forward_checking_finds_exactly_the_basic_solutions() {
    // Same solution set on the classics — lookahead prunes work, never
    // solutions.
    for n in 4..=6usize {
        let csp = queens_csp(n);
        let (basic, _) = csp.solve_basic();
        let (fc, _) = csp.solve_fc();
        assert_eq!(basic, fc, "queens n = {n}");
    }
    let c5 = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)];
    let col = coloring_csp(5, &c5, 3);
    assert_eq!(col.solve_basic().0, col.solve_fc().0);
}

#[test]
fn mrv_finds_the_same_set_sorted() {
    for n in 4..=6usize {
        let csp = queens_csp(n);
        let (basic, _) = csp.solve_basic();
        let (mrv, _) = csp.solve_fc_mrv();
        // solve_basic emits lexicographic order; MRV must sort before
        // returning, so the two are directly comparable.
        assert_eq!(basic, mrv, "queens n = {n}");
    }
}

#[test]
fn lookahead_never_places_more_than_basic() {
    // With the same static order, forward checking visits a subtree of the
    // basic tree: every placement FC makes, basic makes too.
    for seed in 1..=20u64 {
        let csp = random_csp(seed);
        let (sb, nb) = csp.solve_basic();
        let (sf, nf) = csp.solve_fc();
        assert_eq!(sb, sf, "seed {seed}");
        assert!(nf <= nb, "seed {seed}: fc made {nf} placements, basic {nb}");
        for s in &sf {
            assert!(csp.check(s), "seed {seed}: bogus solution {s:?}");
        }
    }
}

#[test]
fn node_counts_are_pinned_for_the_worked_examples() {
    // The contract fixes the order (index/ascending; MRV ties to the lowest
    // index) and the node definition (count every tentative placement of a
    // live value), so these trees are fully determined. The lesson walks
    // queens-6: basic 894 → forward checking 130 → MRV 118.
    let q5 = queens_csp(5);
    assert_eq!(q5.solve_basic().1, 220);
    assert_eq!(q5.solve_fc().1, 53);
    assert_eq!(q5.solve_fc_mrv().1, 53);

    let q6 = queens_csp(6);
    assert_eq!(q6.solve_basic().1, 894);
    assert_eq!(q6.solve_fc().1, 130);
    assert_eq!(q6.solve_fc_mrv().1, 118);

    let c5 = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)];
    let col = coloring_csp(5, &c5, 3);
    assert_eq!(col.solve_basic().1, 138);
    assert_eq!(col.solve_fc().1, 75);
}

#[test]
fn mrv_breaks_ties_toward_the_lowest_index() {
    // All three variables start with live domains of size 2, so the very
    // first MRV pick is a three-way tie — the contract says take the LOWEST
    // index (variable 0). Only constraint: (0, 1) may be (0, 1) alone.
    // Picking 0 first lets forward checking collapse variable 1 immediately:
    // exactly 5 placements. A tie-break toward a higher index (e.g. picking
    // the unconstrained variable 2 first) wanders: 8 placements.
    let mut csp = Csp::new(vec![vec![0, 1]; 3]);
    csp.add_allowed(0, 1, vec![(0, 1)]);
    let (sols, nodes) = csp.solve_fc_mrv();
    assert_eq!(sols, vec![vec![0, 1, 0], vec![0, 1, 1]]);
    assert_eq!(nodes, 5, "MRV must break ties toward the lowest variable index");
}

#[test]
fn a_wipeout_is_detected_before_descending() {
    // X < Y < Z < W over {1, 2, 3}: insoluble. Forward checking placing
    // X = 1 already empties W's live domain two constraints later — the
    // node count stays tiny (at most the first column of placements plus
    // the doomed descents basic would have made).
    let less: Vec<(u32, u32)> = (1..=3u32)
        .flat_map(|a| (1..=3u32).map(move |b| (a, b)))
        .filter(|&(a, b)| a < b)
        .collect();
    let mut csp = Csp::new(vec![vec![1, 2, 3]; 4]);
    csp.add_allowed(0, 1, less.clone());
    csp.add_allowed(1, 2, less.clone());
    csp.add_allowed(2, 3, less);
    let (sols, n_fc) = csp.solve_fc();
    assert!(sols.is_empty());
    let (sols_b, n_b) = csp.solve_basic();
    assert!(sols_b.is_empty());
    assert!(n_fc < n_b, "lookahead must beat basic here ({n_fc} vs {n_b})");
}
