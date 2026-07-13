//! Stage 4 — translating CSP to SAT: the direct encoding.
//! TAOCP Vol. 4 Fascicle 7, §7.2.2.3 (CSP → SAT), meeting §7.2.2.2.

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
    let domains: Vec<Vec<u32>> = (0..3)
        .map(|_| (0..2 + r.below(2) as u32).collect())
        .collect();
    let mut csp = Csp::new(domains);
    for x in 0..3usize {
        for y in (x + 1)..3 {
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
fn variables_are_numbered_domain_by_domain() {
    // Two variables with |D| = 2 and 3: SAT vars 1..=5, var[0] = [1, 2],
    // var[1] = [3, 4, 5]; the first clauses are the at-least-one rows.
    let csp = Csp::new(vec![vec![10, 20], vec![7, 8, 9]]);
    let enc = encode_direct(&csp);
    assert_eq!(enc.num_vars, 5);
    assert_eq!(enc.var, vec![vec![1, 2], vec![3, 4, 5]]);
    assert_eq!(enc.clauses[0], vec![1, 2]);
    assert_eq!(enc.clauses[1], vec![3, 4, 5]);
}

#[test]
fn c5_coloring_encodes_to_the_pinned_sizes() {
    // C5 with 3 colors: 15 vars; 5 ALO + 5·C(3,2)=15 AMO + 5·3=15 conflict
    // clauses (each edge forbids the 3 equal-color pairs) = 35 clauses;
    // model count = the 30 proper colorings.
    let c5 = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)];
    let csp = coloring_csp(5, &c5, 3);
    let enc = encode_direct(&csp);
    assert_eq!(enc.num_vars, 15);
    assert_eq!(enc.clauses.len(), 35);
    assert_eq!(count_models(&enc), 30);
}

#[test]
fn queens_4_encodes_to_the_pinned_sizes() {
    // 16 vars; 4 ALO + 4·C(4,2)=24 AMO + 52 conflicts = 80 clauses; per
    // column pair at distance d the forbidden pairs are 4 equal rows plus
    // 2·(4−d) diagonal pairs: (10+8+6) + (10+8) + 10 = 52. Two models — the
    // two 4-queens solutions.
    let csp = queens_csp(4);
    let enc = encode_direct(&csp);
    assert_eq!(enc.num_vars, 16);
    assert_eq!(enc.clauses.len(), 80);
    assert_eq!(count_models(&enc), 2);
}

#[test]
fn models_correspond_to_solutions_exactly() {
    // ALO + AMO force every model to pick exactly one value per variable,
    // and the conflict clauses forbid exactly the disallowed pairs — a
    // bijection between models and solutions. Certify it on random
    // instances against the search from stage 1.
    for seed in 1..=25u64 {
        let csp = random_csp(seed);
        let enc = encode_direct(&csp);
        let (sols, _) = csp.solve_basic();
        assert_eq!(
            count_models(&enc),
            sols.len() as u64,
            "seed {seed}: encoding gained or lost models"
        );
    }
}

#[test]
fn clause_shapes_are_sound() {
    let csp = queens_csp(4);
    let enc = encode_direct(&csp);
    let n = enc.num_vars as i32;
    for cl in &enc.clauses {
        assert!(!cl.is_empty());
        for &lit in cl {
            assert!(lit != 0 && lit.abs() <= n, "literal {lit} out of range");
        }
    }
    // Everything after the ALO block is a binary NEGATIVE clause (AMO and
    // conflicts both have that shape in the direct encoding).
    for cl in enc.clauses.iter().skip(csp.domains.len()) {
        assert_eq!(cl.len(), 2, "AMO/conflict clauses are binary: {cl:?}");
        assert!(cl.iter().all(|&l| l < 0), "AMO/conflict literals are negative: {cl:?}");
    }
}

#[test]
#[should_panic(expected = "truth table")]
fn count_models_refuses_large_encodings() {
    // Documented contract: more than 24 SAT variables panics with a message
    // containing "truth table" (queens-6 has 36).
    let enc = encode_direct(&queens_csp(6));
    count_models(&enc);
}
