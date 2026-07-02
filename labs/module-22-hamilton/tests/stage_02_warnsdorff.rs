//! Stage 2 — Warnsdorff's rule: the knight's tour (§7.2.2.4's heuristic).
//!
//! Implement `knight_moves`, `warnsdorff_tour`, and `is_valid_tour` in
//! src/lab.rs. Lesson: course/module-22-hamilton/README.md.

use lab_22_hamilton::{is_valid_tour, knight_moves, warnsdorff_tour};

// ---- knight_moves ----------------------------------------------------------

#[test]
fn corner_has_two_moves_center_has_eight() {
    // On 8x8, the corner (0,0) reaches only (2,1) and (1,2).
    let mut c = knight_moves(8, 0);
    c.sort();
    // (1,2) = square 10 and (2,1) = square 17, in sorted order.
    assert_eq!(c, vec![1 * 8 + 2, 2 * 8 + 1]);
    // A deeply central square has all eight moves.
    assert_eq!(knight_moves(8, 3 * 8 + 3).len(), 8);
}

#[test]
fn knight_moves_stay_on_the_board() {
    for board in 3..=8usize {
        for sq in 0..board * board {
            for m in knight_moves(board, sq) {
                assert!(m < board * board, "{board}: move {m} off board from {sq}");
                // A knight move changes (row, col) by (±1,±2) or (±2,±1).
                let (r0, c0) = (sq / board, sq % board);
                let (r1, c1) = (m / board, m % board);
                let dr = (r0 as i64 - r1 as i64).abs();
                let dc = (c0 as i64 - c1 as i64).abs();
                assert!((dr, dc) == (1, 2) || (dr, dc) == (2, 1), "bad move {sq}->{m}");
            }
        }
    }
}

// ---- warnsdorff_tour succeeds ---------------------------------------------

#[test]
fn tour_from_corner_on_several_boards() {
    // From the top-left corner (square 0), Warnsdorff completes a full tour.
    for &board in &[5usize, 6, 7, 8] {
        let tour = warnsdorff_tour(board, 0).unwrap_or_else(|| panic!("{board}x{board} corner"));
        assert_eq!(tour.len(), board * board, "{board}x{board} visits every square");
        assert!(is_valid_tour(board, &tour), "{board}x{board} tour is valid");
    }
}

#[test]
fn tour_visits_every_square_exactly_once() {
    let board = 8;
    let tour = warnsdorff_tour(board, 0).unwrap();
    let mut seen = vec![false; board * board];
    for &sq in &tour {
        assert!(!seen[sq], "square {sq} visited twice");
        seen[sq] = true;
    }
    assert!(seen.into_iter().all(|b| b), "every square must be visited");
}

#[test]
fn tour_from_various_starts_on_6x6_and_8x8() {
    // 6x6 admits a Warnsdorff tour from every start; 8x8 from most.
    for start in 0..36 {
        let t = warnsdorff_tour(6, start).unwrap_or_else(|| panic!("6x6 from {start}"));
        assert!(is_valid_tour(6, &t));
    }
    for &start in &[0usize, 1, 9, 18, 63] {
        let t = warnsdorff_tour(8, start).unwrap_or_else(|| panic!("8x8 from {start}"));
        assert!(is_valid_tour(8, &t));
    }
}

// ---- the honest heuristic story: it can fail ------------------------------

#[test]
fn warnsdorff_can_fail_even_though_a_tour_exists() {
    // From square 2 of a 5x5 board the greedy walk paints itself into a
    // corner, though a full tour from square 2 exists (backtracking finds
    // one). Likewise on 8x8 from square 24. Heuristics are not guarantees.
    assert!(warnsdorff_tour(5, 2).is_none(), "5x5 from square 2 should get stuck");
    assert!(warnsdorff_tour(8, 24).is_none(), "8x8 from square 24 should get stuck");
    // Yet with this tie-break the 5x5 center start does succeed.
    assert!(warnsdorff_tour(5, 2 * 5 + 2).is_some(), "5x5 from center succeeds");
}

// ---- is_valid_tour rejects bad tours --------------------------------------

#[test]
fn validator_rejects_corrupted_tours() {
    let board = 6;
    let good = warnsdorff_tour(board, 0).unwrap();
    assert!(is_valid_tour(board, &good));

    // Break the knight-move chain by swapping two interior squares.
    let mut broken = good.clone();
    broken.swap(4, 25);
    assert!(!is_valid_tour(board, &broken), "non-knight step must be rejected");

    // Wrong length is invalid.
    let mut short = good.clone();
    short.pop();
    assert!(!is_valid_tour(board, &short), "incomplete tour must be rejected");

    // A repeated square (not a permutation) is invalid.
    let mut dup = good.clone();
    let last = dup.len() - 1;
    dup[last] = dup[0];
    assert!(!is_valid_tour(board, &dup), "repeated square must be rejected");
}
