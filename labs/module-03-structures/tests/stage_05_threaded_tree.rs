//! Stage 5 — Threaded binary trees, Algorithm 2.3.1S (§2.3.1).
//!
//! Implement `ThreadedTree` in src/lab.rs. Lesson:
//! course/module-03-structures/README.md, §6.
//!
//! Every would-be Λ link becomes a *thread* to the node's inorder
//! predecessor (left) or successor (right). Threads make `successor`
//! stackless (Algorithm S), and iterating it from the head reproduces the
//! ordinary inorder walk in O(n) total link-follows — the point of the stage.

use lab_03_structures::*;

fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

/// Build a*(b - c) + d/e in the threaded representation by inorder
/// insertions, exactly as §2.3.1 does. Returns the tree and the index of the
/// root ('+').
fn expression_threaded() -> ThreadedTree<char> {
    let mut t = ThreadedTree::new();
    let plus = t.insert_left(t.head(), '+'); // whole tree becomes head's left
    let times = t.insert_left(plus, '*');
    let div = t.insert_right(plus, '/');
    let _a = t.insert_left(times, 'a');
    let minus = t.insert_right(times, '-');
    t.insert_left(minus, 'b');
    t.insert_right(minus, 'c');
    t.insert_left(div, 'd');
    t.insert_right(div, 'e');
    t
}

// --------------------------------------------- worked example ----

#[test]
fn empty_threaded_tree() {
    let t: ThreadedTree<i32> = ThreadedTree::new();
    assert!(t.is_empty());
    assert_eq!(t.len(), 0);
    // Head's successor is the head itself in an empty tree.
    assert_eq!(t.successor(t.head()), t.head());
    assert_eq!(t.inorder_via_threads(), Vec::<i32>::new());
}

#[test]
fn threads_reproduce_the_inorder_walk() {
    // §2.3.1: the threaded tree of a*(b-c)+d/e must walk inorder to a*b-c+d/e
    // with no stack.
    let t = expression_threaded();
    assert_eq!(t.len(), 9);
    assert_eq!(t.inorder_via_threads(), chars("a*b-c+d/e"));
}

#[test]
fn link_follow_count_is_linear() {
    // The reference accounts for exactly 15 link-follows on this 9-node tree:
    // 10 successor calls each follow RLINK once, and the 5 real left links
    // (head→+, +→*, *→a, -→b, /→d) are each descended once. And 15 <= 2n+2.
    let t = expression_threaded();
    let (seq, follows) = t.inorder_via_threads_counting();
    assert_eq!(seq, chars("a*b-c+d/e"));
    assert_eq!(follows, 15);
    assert!(follows <= 2 * t.len() + 2, "traversal must be O(n)");
}

// --------------------------------------------- insert semantics ----

#[test]
fn insert_left_at_head_appends_to_the_end_of_inorder() {
    // insert_left(head, x) makes the current tree x's left subtree, so x
    // lands at the *end* of the inorder sequence.
    let mut t = ThreadedTree::new();
    let a = t.insert_left(t.head(), 'a');
    assert_eq!(t.inorder_via_threads(), chars("a"));
    // insert_right(a, b): b becomes a's inorder successor → "ab".
    t.insert_right(a, 'b');
    assert_eq!(t.inorder_via_threads(), chars("ab"));
    // insert_left(head, z): z appended at the end → "abz".
    t.insert_left(t.head(), 'z');
    assert_eq!(t.inorder_via_threads(), chars("abz"));
}

#[test]
fn insert_right_is_the_inorder_successor() {
    // Build a simple chain and check each insert_right lands right after p.
    let mut t = ThreadedTree::new();
    let a = t.insert_left(t.head(), 'a');
    let b = t.insert_right(a, 'b'); // successor of a
    let _c = t.insert_right(b, 'c'); // successor of b
    assert_eq!(t.inorder_via_threads(), chars("abc"));
    // successor() walks the inorder order; the last node's successor is head.
    assert_eq!(t.info(t.successor(a)), &'b');
    assert_eq!(t.info(t.successor(b)), &'c');
    assert_eq!(t.successor(_c), t.head());
    // Head's successor is the first node in inorder.
    assert_eq!(t.info(t.successor(t.head())), &'a');
}

#[test]
#[should_panic(expected = "insert_right")]
fn insert_right_at_head_is_forbidden() {
    // The head has no INFO and RLINK(head) = head; a right insert there is
    // undefined — use insert_left to append instead.
    let mut t: ThreadedTree<i32> = ThreadedTree::new();
    t.insert_right(t.head(), 7);
}

// --------------------------------------------- property test ----

/// Build a threaded BST by repeatedly locating each key's inorder neighbour
/// and threading it in, then confirm the stackless walk equals the sorted
/// order and the traversal stays within the 2n+2 link-follow bound.
#[test]
fn threaded_walk_matches_sorted_order() {
    let mut rng: u64 = 0x5EED;
    let mut next = || {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        rng >> 33
    };

    for _ in 0..150 {
        let n = 1 + next() as usize % 25;
        let mut keys: Vec<i64> = Vec::new();
        for _ in 0..n {
            let k = (next() % 500) as i64;
            if !keys.contains(&k) {
                keys.push(k);
            }
        }

        // Insert keys as a threaded binary search tree. Track each key's node
        // index so we can find where to insert relative to it.
        let mut t: ThreadedTree<i64> = ThreadedTree::new();
        // We keep a plain-Rust mirror (key, node, left-child?, right-child?)
        // only to navigate; the tree itself carries the threads.
        // Simplest faithful construction: insert in inorder-append fashion is
        // wrong for a BST, so build structurally using a shadow adjacency.
        // Instead, build a normal BST index structure to decide left/right,
        // then mirror inserts into the threaded tree.
        #[derive(Clone, Copy)]
        struct Shadow {
            key: i64,
            node: usize, // corresponding index in the ThreadedTree
            left: usize, // shadow index or usize::MAX
            right: usize,
        }
        let mut shadow: Vec<Shadow> = Vec::new();
        for (i, &k) in keys.iter().enumerate() {
            if i == 0 {
                // First key: the whole (empty) tree hangs to its left at head.
                let node = t.insert_left(t.head(), k);
                shadow.push(Shadow { key: k, node, left: usize::MAX, right: usize::MAX });
                continue;
            }
            // Walk the shadow BST to the empty slot.
            let mut cur = 0usize;
            loop {
                if k < shadow[cur].key {
                    if shadow[cur].left == usize::MAX {
                        let node = t.insert_left(shadow[cur].node, k);
                        let idx = shadow.len();
                        shadow.push(Shadow { key: k, node, left: usize::MAX, right: usize::MAX });
                        shadow[cur].left = idx;
                        break;
                    }
                    cur = shadow[cur].left;
                } else {
                    if shadow[cur].right == usize::MAX {
                        let node = t.insert_right(shadow[cur].node, k);
                        let idx = shadow.len();
                        shadow.push(Shadow { key: k, node, left: usize::MAX, right: usize::MAX });
                        shadow[cur].right = idx;
                        break;
                    }
                    cur = shadow[cur].right;
                }
            }
        }

        let mut expected = keys.clone();
        expected.sort();
        let (seq, follows) = t.inorder_via_threads_counting();
        assert_eq!(seq, expected, "threaded walk must equal sorted order");
        assert_eq!(t.inorder_via_threads(), expected);
        assert!(
            follows <= 2 * t.len() + 2,
            "link-follows {follows} exceeded 2n+2 for n={}",
            t.len()
        );
    }
}
