//! Stage 2 — The family algebra: union, intersection, difference, join
//! (§7.1.4). Implement `union`, `intersect`, `diff`, `join` in src/lab.rs,
//! all memoized. Canonicity turns algebraic laws into `==` on `Ref`s, and
//! a brute-force mirror over `HashSet<BTreeSet<u32>>` keeps the semantics
//! honest. The lesson: course/module-17-zdd-xcc/README.md.

use lab_17_zdd_xcc::{Ref, Zdd};
use std::collections::{BTreeSet, HashSet};

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

/// Build the family whose members are exactly `sets`, via unit/single,
/// join (one set = join of its singletons) and union (family = union of
/// its one-set families).
fn build(z: &mut Zdd, sets: &[Vec<u32>]) -> Ref {
    let mut f = z.empty();
    for s in sets {
        let mut one = z.unit();
        for &v in s {
            let sv = z.single(v);
            one = z.join(one, sv);
        }
        f = z.union(f, one);
    }
    f
}

/// A random family over variables 0..n_vars: each member set is a random
/// bitmask (duplicates collapse, exactly as they should).
fn random_family(state: &mut u64, n_vars: u32, n_sets: usize) -> Vec<Vec<u32>> {
    (0..n_sets)
        .map(|_| {
            let mask = (lcg(state) >> 32) & ((1u64 << n_vars) - 1);
            (0..n_vars).filter(|&v| mask >> v & 1 == 1).collect()
        })
        .collect()
}

type Model = HashSet<BTreeSet<u32>>;

fn model_of(sets: &[Vec<u32>]) -> Model {
    sets.iter().map(|s| s.iter().copied().collect()).collect()
}

fn model_join(a: &Model, b: &Model) -> Model {
    let mut out = Model::new();
    for x in a {
        for y in b {
            out.insert(x.union(y).copied().collect());
        }
    }
    out
}

/// Read a family back out of the ZDD as a mathematical set of sets.
fn extract(z: &Zdd, f: Ref) -> Model {
    z.sets(f).into_iter().map(|s| s.into_iter().collect()).collect()
}

#[test]
fn join_hand_examples() {
    // {{a},{b}} ⊔ {{c}} = {{a,c},{b,c}} — Knuth's motivating example.
    let mut z = Zdd::new();
    let ab = build(&mut z, &[vec![0], vec![1]]);
    let c = build(&mut z, &[vec![2]]);
    let j = z.join(ab, c);
    assert_eq!(z.sets(j), vec![vec![0, 2], vec![1, 2]]);
    assert_eq!(z.count_sets(j), 2);

    // {∅} is join's identity; ∅ annihilates.
    let (u, e) = (z.unit(), z.empty());
    assert_eq!(z.join(ab, u), ab);
    assert_eq!(z.join(u, ab), ab);
    assert_eq!(z.join(ab, e), e);
    assert_eq!(z.join(e, ab), e);
}

#[test]
fn join_counts_multiply_on_disjoint_supports() {
    // f over variables 0..5, g over 5..10: every pair (a, b) gives a
    // distinct union, so |f ⊔ g| = |f|·|g|.
    let mut state = 0x5eed_1701u64;
    for _ in 0..10 {
        let mut z = Zdd::new();
        let fs = random_family(&mut state, 5, 6);
        let gs: Vec<Vec<u32>> = random_family(&mut state, 5, 7)
            .into_iter()
            .map(|s| s.into_iter().map(|v| v + 5).collect())
            .collect();
        let f = build(&mut z, &fs);
        let g = build(&mut z, &gs);
        let j = z.join(f, g);
        assert_eq!(z.count_sets(j), z.count_sets(f) * z.count_sets(g));
    }
}

#[test]
fn algebraic_laws_as_ref_equality() {
    // Canonicity means the laws of the algebra are literal `==` on Refs.
    let mut state = 0xdecade_2026u64;
    for round in 0..20 {
        let mut z = Zdd::new();
        let f = {
            let s = random_family(&mut state, 7, 8);
            build(&mut z, &s)
        };
        let g = {
            let s = random_family(&mut state, 7, 8);
            build(&mut z, &s)
        };
        let h = {
            let s = random_family(&mut state, 7, 8);
            build(&mut z, &s)
        };

        // Commutativity.
        assert_eq!(z.union(f, g), z.union(g, f), "∪ comm, round {round}");
        assert_eq!(z.intersect(f, g), z.intersect(g, f), "∩ comm");
        assert_eq!(z.join(f, g), z.join(g, f), "⊔ comm");

        // Associativity.
        let ab = z.union(f, g);
        let bc = z.union(g, h);
        assert_eq!(z.union(ab, h), z.union(f, bc), "∪ assoc");
        let iab = z.intersect(f, g);
        let ibc = z.intersect(g, h);
        assert_eq!(z.intersect(iab, h), z.intersect(f, ibc), "∩ assoc");
        let jab = z.join(f, g);
        let jbc = z.join(g, h);
        assert_eq!(z.join(jab, h), z.join(f, jbc), "⊔ assoc");

        // Distributivity: ∩ over ∪, and ⊔ over ∪.
        let gh = z.union(g, h);
        let lhs = z.intersect(f, gh);
        let fg = z.intersect(f, g);
        let fh = z.intersect(f, h);
        assert_eq!(lhs, z.union(fg, fh), "∩ over ∪");
        let jlhs = z.join(f, gh);
        let jfg = z.join(f, g);
        let jfh = z.join(f, h);
        assert_eq!(jlhs, z.union(jfg, jfh), "⊔ over ∪");

        // Absorption and idempotence.
        let fg = z.intersect(f, g);
        assert_eq!(z.union(f, fg), f, "absorption");
        let fg = z.union(f, g);
        assert_eq!(z.intersect(f, fg), f, "absorption dual");
        assert_eq!(z.union(f, f), f);
        assert_eq!(z.intersect(f, f), f);

        // De Morgan analogues with difference:
        // f \ (g ∪ h) = (f \ g) ∩ (f \ h);  f \ (g ∩ h) = (f \ g) ∪ (f \ h).
        let gh = z.union(g, h);
        let l = z.diff(f, gh);
        let dg = z.diff(f, g);
        let dh = z.diff(f, h);
        assert_eq!(l, z.intersect(dg, dh), "diff over ∪");
        let gh = z.intersect(g, h);
        let l = z.diff(f, gh);
        assert_eq!(l, z.union(dg, dh), "diff over ∩");

        // Difference basics.
        assert_eq!(z.diff(f, f), z.empty());
        assert_eq!(z.diff(f, z.empty()), f);
        let d = z.diff(f, g);
        let i = z.intersect(f, g);
        assert_eq!(z.union(d, i), f, "f = (f \\ g) ∪ (f ∩ g)");
        assert_eq!(z.intersect(d, g), z.empty(), "(f \\ g) ∩ g = ∅");
    }
}

#[test]
fn brute_force_agreement_on_random_families() {
    // For n <= 8 variables, mirror every operation in plain set-of-sets
    // arithmetic and compare through `sets()`.
    let mut state = 0x0bad_cafe_u64;
    for round in 0..25 {
        let n_vars = 3 + (lcg(&mut state) >> 32) % 6; // 3..=8
        let fs = random_family(&mut state, n_vars as u32, 6);
        let gs = random_family(&mut state, n_vars as u32, 6);
        let (mf, mg) = (model_of(&fs), model_of(&gs));
        let mut z = Zdd::new();
        let f = build(&mut z, &fs);
        let g = build(&mut z, &gs);

        // The builder itself must round-trip.
        assert_eq!(extract(&z, f), mf, "build/sets round trip, round {round}");

        let u = z.union(f, g);
        assert_eq!(extract(&z, u), mf.union(&mg).cloned().collect::<Model>(), "∪");
        let i = z.intersect(f, g);
        assert_eq!(extract(&z, i), mf.intersection(&mg).cloned().collect::<Model>(), "∩");
        let d = z.diff(f, g);
        assert_eq!(extract(&z, d), mf.difference(&mg).cloned().collect::<Model>(), "\\");
        let j = z.join(f, g);
        assert_eq!(extract(&z, j), model_join(&mf, &mg), "⊔");

        // Counts and membership agree with the mirror too.
        assert_eq!(z.count_sets(j), model_join(&mf, &mg).len() as u128);
        for s in &fs {
            assert!(z.contains_set(u, s));
            assert_eq!(z.contains_set(g, s), mg.contains(&s.iter().copied().collect()));
        }
    }
}

#[test]
fn results_are_canonical_too() {
    // Operation results obey the zero-suppression audit as well: no node
    // anywhere in the arena has HI = ⊥.
    let mut state = 0x17_2026u64;
    let mut z = Zdd::new();
    for _ in 0..10 {
        let fs = random_family(&mut state, 8, 8);
        let gs = random_family(&mut state, 8, 8);
        let f = build(&mut z, &fs);
        let g = build(&mut z, &gs);
        let _ = z.union(f, g);
        let _ = z.intersect(f, g);
        let _ = z.diff(f, g);
        let _ = z.join(f, g);
    }
    for i in 2..z.len() {
        assert_ne!(z.hi(Ref(i as u32)), z.empty(), "node {i} violates zero-suppression");
    }
}
