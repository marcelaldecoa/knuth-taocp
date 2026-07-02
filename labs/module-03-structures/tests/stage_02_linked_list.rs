//! Stage 2 — Linked allocation with an AVAIL list (§2.2.3).
//!
//! Implement `LinkedArena` in src/lab.rs. Lesson:
//! course/module-03-structures/README.md, §2.
//!
//! The arena is Knuth's linked memory: parallel INFO/LINK fields addressed by
//! `usize`, the null link Λ = `LAMBDA`, and an AVAIL stack of free cells.
//! `allocate` is `P <= AVAIL`, `free` is `AVAIL <= P`. The key contract:
//! freed cells are reused *before* new memory is drawn from the pool, so
//! `cells_in_memory()` never exceeds the peak number of simultaneously live
//! nodes.

use lab_03_structures::*;

// ------------------------------------------------------------- basics ----

#[test]
fn push_front_builds_a_list_in_order() {
    let mut arena: LinkedArena<u32> = LinkedArena::new();
    let mut first = LAMBDA;
    // Pushing 3, then 2, then 1 at the front yields the list 1 -> 2 -> 3.
    for x in [3, 2, 1] {
        first = arena.push_front(first, x);
    }
    assert_eq!(arena.to_vec(first), vec![1, 2, 3]);
}

#[test]
fn empty_list_is_lambda() {
    let arena: LinkedArena<u8> = LinkedArena::new();
    // The empty list is headed by Λ; walking it yields nothing.
    assert_eq!(arena.to_vec(LAMBDA), Vec::<u8>::new());
    assert_eq!(arena.cells_in_memory(), 0);
}

#[test]
fn info_and_link_expose_the_fields() {
    let mut arena: LinkedArena<char> = LinkedArena::new();
    let a = arena.allocate('a');
    let b = arena.allocate('b');
    // A freshly allocated cell has LINK = Λ.
    assert_eq!(arena.link(a), LAMBDA);
    assert_eq!(arena.info(a), &'a');
    // set_link / link round-trip.
    arena.set_link(a, b);
    assert_eq!(arena.link(a), b);
    assert_eq!(arena.to_vec(a), vec!['a', 'b']);
}

// --------------------------------------------------- delete_after ----

#[test]
fn delete_after_unlinks_the_successor() {
    // §2.2.3's worked example (from the reference unit tests): build 1->2->3,
    // delete the node after the first (removing 2), then the list reads 1->3.
    let mut arena: LinkedArena<u32> = LinkedArena::new();
    let mut first = LAMBDA;
    for x in [3, 2, 1] {
        first = arena.push_front(first, x);
    }
    assert_eq!(arena.to_vec(first), vec![1, 2, 3]);
    assert_eq!(arena.delete_after(first), Some(2));
    assert_eq!(arena.to_vec(first), vec![1, 3]);
}

#[test]
fn delete_after_the_last_node_is_none() {
    let mut arena: LinkedArena<u32> = LinkedArena::new();
    let only = arena.push_front(LAMBDA, 42);
    // The single node has no successor.
    assert_eq!(arena.delete_after(only), None);
    assert_eq!(arena.to_vec(only), vec![42]);
}

// ------------------------------------------------- AVAIL reuse ----

#[test]
fn avail_list_reuses_freed_cells() {
    // The distinguishing property of a real free list: after a delete, the
    // next allocate must draw the freed cell, not grow the pool.
    let mut arena: LinkedArena<u32> = LinkedArena::new();
    let mut first = LAMBDA;
    for x in [3, 2, 1] {
        first = arena.push_front(first, x);
    }
    assert_eq!(arena.delete_after(first), Some(2)); // frees one cell
    assert_eq!(arena.to_vec(first), vec![1, 3]);
    let before = arena.cells_in_memory();
    arena.push_front(first, 9); // must reuse the just-freed cell
    assert_eq!(arena.cells_in_memory(), before, "freed cell must be reused");
}

#[test]
fn cells_in_memory_tracks_the_peak_not_the_total() {
    // Allocate/free in a churn: LIFO reuse means the pool is bounded by the
    // largest number of cells alive at any one instant, however many total
    // allocations happen.
    let mut arena: LinkedArena<u64> = LinkedArena::new();
    // Phase 1: build a 5-node list (peak = 5).
    let mut first = LAMBDA;
    for x in 0..5u64 {
        first = arena.push_front(first, x);
    }
    assert_eq!(arena.cells_in_memory(), 5);
    // Phase 2: churn — repeatedly delete the second node and push a new head.
    // Live count never exceeds 5, so the pool must not grow.
    for _ in 0..1000 {
        arena.delete_after(first);
        first = arena.push_front(first, 7);
        assert_eq!(arena.cells_in_memory(), 5, "pool must stay at the peak");
    }
    assert_eq!(arena.to_vec(first).len(), 5);
}

#[test]
fn avail_is_lifo_last_freed_is_first_reused() {
    let mut arena: LinkedArena<u32> = LinkedArena::new();
    let a = arena.allocate(1);
    let b = arena.allocate(2);
    let c = arena.allocate(3);
    assert_eq!(arena.cells_in_memory(), 3);
    // Free in the order a, b, c; AVAIL is a stack, so c is on top.
    arena.free(a);
    arena.free(b);
    arena.free(c);
    // Next allocate pops c's cell, then b's, then a's — LIFO — and never grows.
    let x = arena.allocate(10);
    let y = arena.allocate(20);
    let z = arena.allocate(30);
    assert_eq!([x, y, z], [c, b, a]);
    assert_eq!(arena.cells_in_memory(), 3);
}

// ------------------------------------------------- property test ----

/// Drive the arena against a `Vec<u64>` model with a hand-rolled LCG,
/// interleaving push_front and delete_after. The list contents must always
/// match, and the pool must never exceed the model's peak length.
#[test]
fn arena_matches_a_vec_model_under_random_ops() {
    let mut arena: LinkedArena<u64> = LinkedArena::new();
    let mut first = LAMBDA;
    // Model: front element of `model` is the head of the list.
    let mut model: std::collections::VecDeque<u64> = std::collections::VecDeque::new();
    let mut peak = 0usize;

    let mut rng: u64 = 12345;
    let mut next = || {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        rng >> 33
    };

    for i in 0..4000u64 {
        if next() % 2 == 0 {
            // push_front
            first = arena.push_front(first, i);
            model.push_front(i);
        } else if !model.is_empty() {
            // delete the node after the head (removing the 2nd element),
            // matching delete_after(first).
            if model.len() >= 2 {
                let removed = arena.delete_after(first);
                let expected = model.remove(1);
                assert_eq!(removed, expected);
            } else {
                // single node: delete_after is a no-op returning None.
                assert_eq!(arena.delete_after(first), None);
            }
        }
        peak = peak.max(model.len());
        let contents: Vec<u64> = model.iter().copied().collect();
        assert_eq!(arena.to_vec(first), contents, "list mismatch at step {i}");
        assert!(
            arena.cells_in_memory() <= peak,
            "pool {} exceeded peak {peak} at step {i}",
            arena.cells_in_memory()
        );
    }
}
