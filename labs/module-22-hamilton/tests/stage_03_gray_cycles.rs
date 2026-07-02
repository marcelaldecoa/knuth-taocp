//! Stage 3 — Hamiltonian cycles on the hypercube ARE Gray codes.
//!
//! The beautiful bridge back to Module 08: generating all n-bit strings while
//! changing one bit at a time (a Gray code) is exactly walking a Hamiltonian
//! cycle on the hypercube Q_n. Implement `hypercube_neighbors`,
//! `gray_code_cycle`, and `is_hamiltonian_cycle_on_hypercube` in src/lab.rs.

use lab_22_hamilton::{
    gray_code_cycle, hypercube_neighbors, is_hamiltonian_cycle_on_hypercube,
};

// ---- hypercube_neighbors ---------------------------------------------------

#[test]
fn every_vertex_has_d_neighbors_differing_in_one_bit() {
    for d in 1..=8usize {
        for v in 0..(1u32 << d) {
            let nbrs = hypercube_neighbors(d, v);
            assert_eq!(nbrs.len(), d, "Q_{d} vertex {v} has degree d");
            for w in nbrs {
                assert_eq!((v ^ w).count_ones(), 1, "{v} and {w} differ in one bit");
            }
        }
    }
}

// ---- gray_code_cycle is a Hamiltonian cycle on Q_d ------------------------

#[test]
fn gray_cycle_is_hamiltonian_on_the_hypercube() {
    for d in 1..=8usize {
        let cyc = gray_code_cycle(d);
        assert_eq!(cyc.len(), 1usize << d, "Q_{d} has 2^d vertices");
        assert!(is_hamiltonian_cycle_on_hypercube(d, &cyc), "Q_{d} cycle");
    }
}

#[test]
fn vertex_count_is_two_to_the_d() {
    for d in 0..=8usize {
        assert_eq!(gray_code_cycle(d).len(), 1usize << d);
    }
}

// ---- it matches the standard reflected Gray code g(k) = k ^ (k>>1) --------

#[test]
fn matches_reflected_binary_code() {
    for d in 1..=8usize {
        let cyc = gray_code_cycle(d);
        for (k, &word) in cyc.iter().enumerate() {
            // Reimplement Module 08's Gray map g(k) = k XOR floor(k/2).
            let g = (k as u32) ^ (k as u32 >> 1);
            assert_eq!(word, g, "g({k}) in Q_{d}");
        }
    }
}

#[test]
fn successive_xor_is_a_single_power_of_two() {
    // Adjacent codewords differ in exactly one bit, so their XOR is a power
    // of two — the defining property of a Gray code AND of a hypercube edge.
    for d in 1..=8usize {
        let cyc = gray_code_cycle(d);
        let n = cyc.len();
        for i in 0..n {
            let diff = cyc[i] ^ cyc[(i + 1) % n];
            assert_eq!(diff.count_ones(), 1, "Q_{d}: step {i} flips one bit");
            assert!(diff.is_power_of_two());
        }
    }
}

// ---- cross-check with Module 08's Gray-code adjacency ---------------------

#[test]
fn gray_order_adjacent_words_are_hypercube_neighbors() {
    // "Change one bit at a time" (Module 08) = "walk edges of Q_d" (here):
    // each g(k+1) is a hypercube neighbor of g(k).
    for d in 2..=7usize {
        let cyc = gray_code_cycle(d);
        for i in 0..cyc.len() {
            let cur = cyc[i];
            let next = cyc[(i + 1) % cyc.len()];
            assert!(
                hypercube_neighbors(d, cur).contains(&next),
                "Q_{d}: g({i})={cur} and its successor {next} are not adjacent"
            );
        }
    }
}

// ---- the validator rejects non-cycles -------------------------------------

#[test]
fn validator_rejects_bad_cycles() {
    let d = 3;
    let good = gray_code_cycle(d); // 8 vertices
    assert!(is_hamiltonian_cycle_on_hypercube(d, &good));

    // Wrong length.
    assert!(!is_hamiltonian_cycle_on_hypercube(d, &good[..7]));

    // A repeated vertex (not a permutation of all 2^d vertices).
    let mut dup = good.clone();
    dup[7] = dup[0];
    assert!(!is_hamiltonian_cycle_on_hypercube(d, &dup));

    // A permutation whose consecutive vertices are NOT all adjacent:
    // swapping two entries of the Gray cycle breaks single-bit steps.
    let mut scrambled = good.clone();
    scrambled.swap(1, 4);
    assert!(!is_hamiltonian_cycle_on_hypercube(d, &scrambled));
}
