//! Stage 4 — Traversing binary trees, Algorithm 2.3.1T (§2.3.1).
//!
//! Implement `BinaryTree`'s traversals and `from_traversals` in src/lab.rs.
//! Lesson: course/module-03-structures/README.md, §5.
//!
//! `inorder` must be the **explicit-stack** formulation of Algorithm T (not
//! recursion): the stack invariant is the lesson. `preorder`/`postorder` are
//! the sibling orders. `from_traversals` rebuilds the unique tree from its
//! preorder + inorder — a round-trip test of all three.

use lab_03_structures::*;

/// The expression tree for a*(b - c) + d/e, the running example of §2.3.1.
///
/// ```text
///           +
///         /   \
///        *     /
///       / \   / \
///      a   - d   e
///         / \
///        b   c
/// ```
fn expression_tree() -> BinaryTree<char> {
    let mut t = BinaryTree::new();
    let a = t.add_node('a', LAMBDA, LAMBDA);
    let b = t.add_node('b', LAMBDA, LAMBDA);
    let c = t.add_node('c', LAMBDA, LAMBDA);
    let minus = t.add_node('-', b, c);
    let times = t.add_node('*', a, minus);
    let d = t.add_node('d', LAMBDA, LAMBDA);
    let e = t.add_node('e', LAMBDA, LAMBDA);
    let div = t.add_node('/', d, e);
    let plus = t.add_node('+', times, div);
    t.set_root(plus);
    t
}

fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

// --------------------------------------------- worked example ----

#[test]
fn traversals_of_the_expression_tree() {
    // §2.3.1: preorder/inorder/postorder of a*(b-c)+d/e are the prefix,
    // infix, and postfix (Polish) forms of the expression.
    let t = expression_tree();
    assert_eq!(t.preorder(), chars("+*a-bc/de"));
    assert_eq!(t.inorder(), chars("a*b-c+d/e"));
    assert_eq!(t.postorder(), chars("abc-*de/+"));
}

#[test]
fn empty_tree_traverses_to_nothing() {
    let t: BinaryTree<i32> = BinaryTree::new();
    assert!(t.is_empty());
    assert_eq!(t.inorder(), Vec::<i32>::new());
    assert_eq!(t.preorder(), Vec::<i32>::new());
    assert_eq!(t.postorder(), Vec::<i32>::new());
}

#[test]
fn single_node_tree() {
    let mut t = BinaryTree::new();
    let r = t.add_node(42, LAMBDA, LAMBDA);
    t.set_root(r);
    assert_eq!(t.inorder(), vec![42]);
    assert_eq!(t.preorder(), vec![42]);
    assert_eq!(t.postorder(), vec![42]);
    assert_eq!(t.root(), r);
    assert_eq!(t.info(r), &42);
    assert_eq!(t.llink(r), LAMBDA);
    assert_eq!(t.rlink(r), LAMBDA);
}

#[test]
fn a_left_leaning_chain_is_reversed_by_inorder() {
    // Root has only left children: 3 -> 2 -> 1. Inorder visits leftmost
    // first, so 1, 2, 3; preorder is 3, 2, 1; postorder is 1, 2, 3.
    let mut t = BinaryTree::new();
    let n1 = t.add_node(1, LAMBDA, LAMBDA);
    let n2 = t.add_node(2, n1, LAMBDA);
    let n3 = t.add_node(3, n2, LAMBDA);
    t.set_root(n3);
    assert_eq!(t.inorder(), vec![1, 2, 3]);
    assert_eq!(t.preorder(), vec![3, 2, 1]);
    assert_eq!(t.postorder(), vec![1, 2, 3]);
}

// --------------------------------------------- BST inorder is sorted ----

/// Insert `keys` into a binary *search* tree, returning the arena. Inorder of
/// a BST is the defining theorem: it lists the keys in ascending order.
fn build_bst(keys: &[i64]) -> BinaryTree<i64> {
    let mut t = BinaryTree::new();
    for &k in keys {
        if t.is_empty() {
            let r = t.add_node(k, LAMBDA, LAMBDA);
            t.set_root(r);
            continue;
        }
        // Walk down to the correct empty slot, then splice in a new node.
        let mut p = t.root();
        loop {
            if k < *t.info(p) {
                let l = t.llink(p);
                if l == LAMBDA {
                    let n = t.add_node(k, LAMBDA, LAMBDA);
                    t.set_llink(p, n);
                    break;
                }
                p = l;
            } else {
                let r = t.rlink(p);
                if r == LAMBDA {
                    let n = t.add_node(k, LAMBDA, LAMBDA);
                    t.set_rlink(p, n);
                    break;
                }
                p = r;
            }
        }
    }
    t
}

#[test]
fn bst_inorder_is_sorted() {
    let mut rng: u64 = 0xC0FFEE;
    let mut next = || {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        rng >> 33
    };
    for _ in 0..200 {
        let n = next() as usize % 40; // 0..=39 distinct keys
        let mut keys: Vec<i64> = Vec::new();
        for _ in 0..n {
            let k = (next() % 1000) as i64;
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        let t = build_bst(&keys);
        let mut expected = keys.clone();
        expected.sort();
        assert_eq!(t.inorder(), expected, "inorder of a BST must be sorted");
    }
}

// --------------------------------------------- from_traversals round-trip --

#[test]
fn rebuild_from_preorder_and_inorder() {
    // The pair (preorder, inorder) determines a tree with distinct keys
    // uniquely; rebuild it and all three traversals must agree with the
    // original.
    let t = expression_tree();
    let rebuilt = from_traversals(&t.preorder(), &t.inorder());
    assert_eq!(rebuilt.preorder(), t.preorder());
    assert_eq!(rebuilt.inorder(), t.inorder());
    assert_eq!(rebuilt.postorder(), t.postorder());
}

#[test]
fn from_traversals_round_trips_random_bsts() {
    let mut rng: u64 = 424242;
    let mut next = || {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        rng >> 33
    };
    for _ in 0..200 {
        let n = 1 + next() as usize % 30;
        let mut keys: Vec<i64> = Vec::new();
        for _ in 0..n {
            let k = (next() % 2000) as i64;
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        let t = build_bst(&keys);
        let pre = t.preorder();
        let ino = t.inorder();
        let rebuilt = from_traversals(&pre, &ino);
        // Reconstruction is exact: every traversal matches.
        assert_eq!(rebuilt.preorder(), pre);
        assert_eq!(rebuilt.inorder(), ino);
        assert_eq!(rebuilt.postorder(), t.postorder());
    }
}
