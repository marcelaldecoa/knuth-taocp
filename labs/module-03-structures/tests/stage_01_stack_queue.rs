//! Stage 1 — Stacks and queues in sequential storage (§2.2.1–2.2.2).
//!
//! Implement `ArrayStack`, `ArrayQueue` and `stack_permutable` in
//! src/lab.rs. Lesson: course/module-03-structures/README.md, §1 and §3.
//!
//! Contract under test: `push`/`enqueue` return `Err(Overflow)` when full,
//! `pop`/`dequeue` return `Err(Underflow)` when empty — reported, never
//! panicked.

use lab_03_structures::*;

// ---------------------------------------------------------------- stack --

#[test]
fn stack_is_lifo() {
    let mut s = ArrayStack::new(10);
    for x in [10, 20, 30] {
        s.push(x).unwrap();
    }
    assert_eq!(s.len(), 3);
    assert_eq!(s.peek(), Some(&30));
    assert_eq!(s.pop(), Ok(30));
    assert_eq!(s.pop(), Ok(20));
    s.push(99).unwrap();
    assert_eq!(s.pop(), Ok(99));
    assert_eq!(s.pop(), Ok(10));
    assert!(s.is_empty());
}

#[test]
fn stack_overflow_is_reported_not_panicked() {
    let mut s = ArrayStack::new(2);
    assert_eq!(s.push('x'), Ok(()));
    assert_eq!(s.push('y'), Ok(()));
    assert!(s.is_full());
    // A full stack must refuse politely — and stay intact.
    assert_eq!(s.push('z'), Err(Overflow));
    assert_eq!(s.len(), 2);
    assert_eq!(s.pop(), Ok('y'));
    assert_eq!(s.push('z'), Ok(())); // room again after a pop
    assert_eq!(s.capacity(), 2);
}

#[test]
fn stack_underflow_is_reported_not_panicked() {
    let mut s: ArrayStack<i64> = ArrayStack::new(4);
    assert_eq!(s.pop(), Err(Underflow));
    assert_eq!(s.peek(), None);
    s.push(7).unwrap();
    assert_eq!(s.pop(), Ok(7));
    assert_eq!(s.pop(), Err(Underflow)); // and again after emptying
    assert!(s.is_empty());
}

#[test]
fn zero_capacity_stack_is_always_full() {
    let mut s: ArrayStack<u8> = ArrayStack::new(0);
    assert!(s.is_full() && s.is_empty());
    assert_eq!(s.push(1), Err(Overflow));
    assert_eq!(s.pop(), Err(Underflow));
}

// ---------------------------------------------------------------- queue --

#[test]
fn queue_is_fifo() {
    let mut q = ArrayQueue::new(5);
    for x in 1..=4 {
        q.enqueue(x).unwrap();
    }
    assert_eq!(q.front(), Some(&1));
    assert_eq!(q.dequeue(), Ok(1));
    assert_eq!(q.dequeue(), Ok(2));
    q.enqueue(5).unwrap();
    assert_eq!(q.dequeue(), Ok(3));
    assert_eq!(q.dequeue(), Ok(4));
    assert_eq!(q.dequeue(), Ok(5));
    assert!(q.is_empty());
}

#[test]
fn queue_overflow_and_underflow_are_reported() {
    let mut q = ArrayQueue::new(3);
    assert_eq!(q.dequeue(), Err(Underflow));
    q.enqueue(1).unwrap();
    q.enqueue(2).unwrap();
    q.enqueue(3).unwrap();
    assert!(q.is_full());
    assert_eq!(q.enqueue(4), Err(Overflow));
    assert_eq!(q.len(), 3);
    assert_eq!(q.dequeue(), Ok(1)); // intact after the refused insert
    assert_eq!(q.enqueue(4), Ok(()));
    assert_eq!(q.capacity(), 3);
}

#[test]
fn all_capacity_cells_are_usable() {
    // With the R/F pointers alone, only M-1 of M cells are usable
    // (§2.2.2 exercise 1); the contract here demands the standard cure
    // (keep a count), so a capacity-M queue holds M items.
    let mut q = ArrayQueue::new(4);
    for x in 0..4 {
        assert_eq!(q.enqueue(x), Ok(()), "cell {x} must be usable");
    }
    for x in 0..4 {
        assert_eq!(q.dequeue(), Ok(x));
    }
}

#[test]
fn queue_wraps_around_the_buffer_end() {
    // Drive F and R around a small circle many times, checking against a
    // simple model. If the pointers do not wrap mod M this dies quickly.
    let mut q = ArrayQueue::new(3);
    let mut model: Vec<u64> = Vec::new();
    let mut rng: u64 = 2026;
    let mut step = || {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        rng >> 33
    };
    for i in 0..3000u64 {
        if step() % 2 == 0 {
            let r = q.enqueue(i);
            if model.len() < 3 {
                assert_eq!(r, Ok(()), "enqueue #{i} with room");
                model.push(i);
            } else {
                assert_eq!(r, Err(Overflow), "enqueue #{i} while full");
            }
        } else {
            let r = q.dequeue();
            if model.is_empty() {
                assert_eq!(r, Err(Underflow));
            } else {
                assert_eq!(r, Ok(model.remove(0)));
            }
        }
        assert_eq!(q.len(), model.len());
    }
}

// --------------------------------------------------- railway shunting ----

/// Does `perm` contain the pattern 3-1-2, i.e. positions i < j < k with
/// perm[j] < perm[k] < perm[i]? §2.2.1 (exercises 2–5): a permutation is
/// stack-realizable iff it avoids this pattern.
fn contains_312(perm: &[usize]) -> bool {
    let n = perm.len();
    for i in 0..n {
        for j in i + 1..n {
            for k in j + 1..n {
                if perm[j] < perm[k] && perm[k] < perm[i] {
                    return true;
                }
            }
        }
    }
    false
}

/// All permutations of 1..=n via Heap's algorithm (deterministic order).
fn permutations(n: usize) -> Vec<Vec<usize>> {
    fn heap(k: usize, a: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if k <= 1 {
            out.push(a.clone());
            return;
        }
        for i in 0..k {
            heap(k - 1, a, out);
            if k % 2 == 0 {
                a.swap(i, k - 1);
            } else {
                a.swap(0, k - 1);
            }
        }
    }
    let mut a: Vec<usize> = (1..=n).collect();
    let mut out = Vec::new();
    heap(n, &mut a, &mut out);
    out
}

#[test]
fn small_shunting_cases() {
    assert!(stack_permutable(&[])); // zero cars: vacuously fine
    assert!(stack_permutable(&[1]));
    assert!(stack_permutable(&[1, 2, 3, 4])); // pop each car immediately
    assert!(stack_permutable(&[4, 3, 2, 1])); // push all, then pop all
    assert!(stack_permutable(&[2, 1, 3]));
    assert!(!stack_permutable(&[3, 1, 2])); // THE forbidden pattern
    assert!(!stack_permutable(&[4, 1, 3, 2])); // contains 4..1..3 ~ 312
}

#[test]
fn non_permutations_are_rejected() {
    assert!(!stack_permutable(&[1, 1]));
    assert!(!stack_permutable(&[0, 1]));
    assert!(!stack_permutable(&[2, 3])); // of {1,2}? no: 3 out of range
    assert!(!stack_permutable(&[5, 4, 3, 2])); // 5 > n = 4
}

#[test]
fn permutations_of_four_split_exactly_on_pattern_312() {
    // The classic §2.2.1 fact: of the 24 orders of cars 1,2,3,4, exactly
    // the ones containing pattern 312 are unrealizable; C(4) = 14 remain.
    let mut realizable = 0;
    for p in permutations(4) {
        assert_eq!(
            stack_permutable(&p),
            !contains_312(&p),
            "wrong verdict on {p:?}"
        );
        if stack_permutable(&p) {
            realizable += 1;
        }
    }
    assert_eq!(realizable, 14); // the Catalan number C(4)
}

#[test]
fn realizable_counts_are_catalan_numbers() {
    // |{stack-realizable permutations of n}| = C(n) = binom(2n, n)/(n+1).
    let catalan = [1usize, 1, 2, 5, 14, 42, 132];
    for n in 0..=6 {
        let count = permutations(n)
            .iter()
            .filter(|p| stack_permutable(p))
            .count();
        assert_eq!(count, catalan[n], "C({n})");
    }
}
