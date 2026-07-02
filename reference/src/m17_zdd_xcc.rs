//! Module 17 — ZDDs and Exact Covering with Colors.
//! Source: TAOCP Vol. 4A, §7.1.4 (zero-suppressed decision diagrams) and
//! Vol. 4B, §7.2.2.1 (Algorithm 7.2.2.1C, exact cover with colors).
//!
//! Part 1 (§7.1.4): a ZDD arena representing *families of sets* over the
//! variables 0..n, kept canonical by the **zero-suppression rule**: a node
//! whose HI child is ⊥ is never created (contrast the BDD rule of Module
//! 13, which elides nodes with LO = HI). Equality of families is equality
//! of `Ref`s.
//!
//! Part 2 (§7.2.2.1): dancing links extended with *colors* on secondary
//! items — Knuth's Algorithm C. Primary items are covered exactly once;
//! secondary items at most once, and two options sharing a secondary item
//! are compatible iff they assign it the same color (purify/unpurify).

use std::collections::HashMap;

// ============================================================================
// §7.1.4 — Zero-suppressed decision diagrams
// ============================================================================

/// A handle to a ZDD node: an index into the `Zdd` arena. `Ref(0)` is the
/// ⊥ sink (the *empty family* ∅ — no sets at all) and `Ref(1)` is the ⊤
/// sink (the family {∅} whose only member is the empty set). Because the
/// arena is hash-consed and zero-suppressed, **two `Ref`s from the same
/// `Zdd` are equal iff they denote the same family of sets** — §7.1.4's
/// canonical-form theorem for ZDDs, made executable.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Ref(pub u32);

/// Sentinel variable index for the two sinks (below every real level).
const TERMINAL_VAR: u32 = u32::MAX;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    var: u32,
    lo: Ref,
    hi: Ref,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Op {
    Union,
    Intersect,
    Diff,
    Join,
}

/// A shared arena of ZDD nodes over variables x0 < x1 < x2 < … (smaller
/// index tested nearer the root). A node `(v, lo, hi)` denotes the family
///
/// ```text
/// F = lo-family  ∪  { s ∪ {v} : s ∈ hi-family },
/// ```
///
/// i.e. LO = "the member sets not containing v", HI = "the member sets
/// containing v, with v removed". The arena is kept *reduced* at all
/// times by the two invariants of §7.1.4 (ZDD form):
///
/// 1. **zero-suppression**: no node has `hi == ⊥` (such a node would say
///    "the sets containing v are: none", and denotes exactly its LO
///    child) — note the contrast with the BDD rule, which forbids
///    `lo == hi`; a ZDD node with `lo == hi` is meaningful and is kept!
/// 2. **uniqueness**: no two nodes share a `(var, lo, hi)` triple (the
///    unique table enforces this by hash-consing).
pub struct Zdd {
    nodes: Vec<Node>,
    unique: HashMap<(u32, Ref, Ref), Ref>,
    memo: HashMap<(Op, Ref, Ref), Ref>,
}

impl Default for Zdd {
    fn default() -> Self {
        Self::new()
    }
}

impl Zdd {
    /// A fresh arena containing only the sinks ⊥ = `Ref(0)`, ⊤ = `Ref(1)`.
    pub fn new() -> Self {
        let sink = |r| Node { var: TERMINAL_VAR, lo: r, hi: r };
        Zdd {
            nodes: vec![sink(Ref(0)), sink(Ref(1))],
            unique: HashMap::new(),
            memo: HashMap::new(),
        }
    }

    /// The empty family ∅ (no sets at all): the ⊥ sink.
    pub fn empty(&self) -> Ref {
        Ref(0)
    }

    /// The family {∅} whose single member is the empty set: the ⊤ sink.
    /// This is the multiplicative identity of `join` — do not confuse it
    /// with `empty()`, the additive identity of `union`.
    pub fn unit(&self) -> Ref {
        Ref(1)
    }

    /// Total number of nodes ever created in the arena, sinks included.
    /// Every `Ref(i)` with `i < len()` is valid — the tests sweep the whole
    /// arena to audit the zero-suppression invariant.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// True iff `f` is one of the two sinks.
    pub fn is_terminal(&self, f: Ref) -> bool {
        f.0 <= 1
    }

    /// The variable tested at node `f`. Panics on a sink.
    pub fn var(&self, f: Ref) -> u32 {
        assert!(!self.is_terminal(f), "sinks have no branch variable");
        self.nodes[f.0 as usize].var
    }

    /// The LO child of `f` (the member sets *not* containing `var(f)`).
    /// Panics on a sink.
    pub fn lo(&self, f: Ref) -> Ref {
        assert!(!self.is_terminal(f), "sinks have no LO child");
        self.nodes[f.0 as usize].lo
    }

    /// The HI child of `f` (the member sets containing `var(f)`, with it
    /// removed). Panics on a sink. Invariant: never ⊥.
    pub fn hi(&self, f: Ref) -> Ref {
        assert!(!self.is_terminal(f), "sinks have no HI child");
        self.nodes[f.0 as usize].hi
    }

    /// Level of `f` for top-variable comparisons: sinks sit below every
    /// variable (TERMINAL_VAR = u32::MAX).
    fn level(&self, f: Ref) -> u32 {
        self.nodes[f.0 as usize].var
    }

    /// The ZDD reduction discipline in one place: return the unique node
    /// `(v, lo, hi)`, creating it only if it does not exist, and eliding
    /// it entirely when `hi == ⊥` — **the zero-suppression rule**.
    /// (`lo == hi` is *not* elided: that is the BDD rule, not ours.)
    fn mk(&mut self, v: u32, lo: Ref, hi: Ref) -> Ref {
        if hi == Ref(0) {
            return lo; // zero-suppress: "contains v" ∧ "no such set" = nothing
        }
        debug_assert!(
            self.level(lo) > v && self.level(hi) > v,
            "variable order violated"
        );
        if let Some(&r) = self.unique.get(&(v, lo, hi)) {
            return r; // hash-consing via the unique table
        }
        let r = Ref(self.nodes.len() as u32);
        self.nodes.push(Node { var: v, lo, hi });
        self.unique.insert((v, lo, hi), r);
        r
    }

    /// The elementary family {{var}}: exactly one member set, containing
    /// exactly one variable.
    pub fn single(&mut self, var: u32) -> Ref {
        assert!(var != TERMINAL_VAR, "variable index reserved for sinks");
        let (bot, top) = (self.empty(), self.unit());
        self.mk(var, bot, top)
    }

    /// The number of member sets of the family `f`. For ZDDs the recursion
    /// is the naked one — c(⊥) = 0, c(⊤) = 1, c(node) = c(LO) + c(HI) —
    /// with **no** 2^skip factors (contrast Algorithm 7.1.4C on BDDs):
    /// a variable skipped along a ZDD path is *absent* from the member
    /// sets below, one possibility, not two. Memoized: O(Z(f)) additions.
    pub fn count_sets(&self, f: Ref) -> u128 {
        fn rec(z: &Zdd, f: Ref, memo: &mut HashMap<Ref, u128>) -> u128 {
            if f == Ref(0) {
                return 0;
            }
            if f == Ref(1) {
                return 1;
            }
            if let Some(&c) = memo.get(&f) {
                return c;
            }
            let n = &z.nodes[f.0 as usize];
            let c = rec(z, n.lo, memo) + rec(z, n.hi, memo);
            memo.insert(f, c);
            c
        }
        rec(self, f, &mut HashMap::new())
    }

    /// Is `set` a member of the family `f`? One walk from the root, O(n):
    /// at a node for variable v, follow HI if v ∈ set, LO otherwise; if the
    /// walk ever *skips past* a wanted variable (node var > smallest
    /// remaining element), no member set below contains it — answer false.
    /// Accepts `set` in any order (sorted and deduplicated internally).
    pub fn contains_set(&self, f: Ref, set: &[u32]) -> bool {
        let mut s: Vec<u32> = set.to_vec();
        s.sort_unstable();
        s.dedup();
        let mut i = 0;
        let mut r = f;
        loop {
            if self.is_terminal(r) {
                // ⊤ accepts exactly the empty remainder; ⊥ accepts nothing.
                return r == Ref(1) && i == s.len();
            }
            let n = &self.nodes[r.0 as usize];
            if i < s.len() && s[i] < n.var {
                return false; // wanted element was zero-suppressed away
            }
            if i < s.len() && s[i] == n.var {
                i += 1;
                r = n.hi;
            } else {
                r = n.lo;
            }
        }
    }

    /// Enumerate every member set, canonically: each set ascending, the
    /// family sorted lexicographically. Intended for *small* families
    /// (tests and debugging) — the output is exponential in general even
    /// when the ZDD itself is tiny.
    pub fn sets(&self, f: Ref) -> Vec<Vec<u32>> {
        fn rec(z: &Zdd, f: Ref, prefix: &mut Vec<u32>, out: &mut Vec<Vec<u32>>) {
            if f == Ref(0) {
                return;
            }
            if f == Ref(1) {
                out.push(prefix.clone());
                return;
            }
            let n = z.nodes[f.0 as usize];
            rec(z, n.lo, prefix, out);
            prefix.push(n.var);
            rec(z, n.hi, prefix, out);
            prefix.pop();
        }
        let mut out = Vec::new();
        rec(self, f, &mut Vec::new(), &mut out);
        out.sort();
        out
    }

    /// Number of distinct nodes reachable from `f`, sinks included: the
    /// ZDD size Z(f) whose *sparsity* behavior stage 1 showcases.
    pub fn node_count(&self, f: Ref) -> usize {
        let mut set = std::collections::HashSet::new();
        let mut stack = vec![f];
        while let Some(r) = stack.pop() {
            if set.insert(r) && !self.is_terminal(r) {
                stack.push(self.nodes[r.0 as usize].lo);
                stack.push(self.nodes[r.0 as usize].hi);
            }
        }
        set.len()
    }

    // ------------------------------------------------------------------
    // The family algebra (§7.1.4, ZDD synthesis). All four operations
    // are memoized top-variable recursions in the style of Module 13's
    // `apply`; the base cases differ because the sinks now mean ∅ and
    // {∅}, not FALSE and TRUE.
    // ------------------------------------------------------------------

    /// f ∪ g: every set belonging to f or to g.
    ///
    /// ```text
    /// U1. [Trivial?]  ∅ ∪ g = g;  f ∪ ∅ = f;  f ∪ f = f.
    /// U2. [Memo.]     Commutative: normalize the key, look it up.
    /// U3. [Expand.]   v <- the smaller top variable. If only f tests v:
    ///                 mk(v, f_lo ∪ g, f_hi)   (g has no set containing v);
    ///                 symmetric if only g tests v; if both:
    ///                 mk(v, f_lo ∪ g_lo, f_hi ∪ g_hi).
    /// U4. [Cache.]
    /// ```
    pub fn union(&mut self, f: Ref, g: Ref) -> Ref {
        // U1. [Trivial?]
        if f == Ref(0) || f == g {
            return g;
        }
        if g == Ref(0) {
            return f;
        }
        // U2. [Memo.]
        let key = if f.0 <= g.0 { (Op::Union, f, g) } else { (Op::Union, g, f) };
        if let Some(&r) = self.memo.get(&key) {
            return r;
        }
        // U3. [Expand on the smaller top variable.]
        let (vf, vg) = (self.level(f), self.level(g));
        let r = if vf < vg {
            let (lo, hi) = (self.lo(f), self.hi(f));
            let l = self.union(lo, g);
            self.mk(vf, l, hi)
        } else if vg < vf {
            let (lo, hi) = (self.lo(g), self.hi(g));
            let l = self.union(f, lo);
            self.mk(vg, l, hi)
        } else {
            let (fl, fh) = (self.lo(f), self.hi(f));
            let (gl, gh) = (self.lo(g), self.hi(g));
            let l = self.union(fl, gl);
            let h = self.union(fh, gh);
            self.mk(vf, l, h)
        };
        // U4. [Cache.]
        self.memo.insert(key, r);
        r
    }

    /// f ∩ g: every set belonging to both f and g.
    ///
    /// ```text
    /// I1. [Trivial?]  ∅ ∩ g = f ∩ ∅ = ∅;  f ∩ f = f.
    /// I2. [Memo.]     Commutative key.
    /// I3. [Expand.]   If only f tests its top v: no set of g contains v,
    ///                 so answer f_lo ∩ g (drop f's HI wholesale);
    ///                 symmetric; if both test v:
    ///                 mk(v, f_lo ∩ g_lo, f_hi ∩ g_hi).
    /// I4. [Cache.]
    /// ```
    pub fn intersect(&mut self, f: Ref, g: Ref) -> Ref {
        // I1. [Trivial?]
        if f == Ref(0) || g == Ref(0) {
            return Ref(0);
        }
        if f == g {
            return f;
        }
        // I2. [Memo.]
        let key = if f.0 <= g.0 {
            (Op::Intersect, f, g)
        } else {
            (Op::Intersect, g, f)
        };
        if let Some(&r) = self.memo.get(&key) {
            return r;
        }
        // I3. [Expand.]
        let (vf, vg) = (self.level(f), self.level(g));
        let r = if vf < vg {
            let lo = self.lo(f);
            self.intersect(lo, g)
        } else if vg < vf {
            let lo = self.lo(g);
            self.intersect(f, lo)
        } else {
            let (fl, fh) = (self.lo(f), self.hi(f));
            let (gl, gh) = (self.lo(g), self.hi(g));
            let l = self.intersect(fl, gl);
            let h = self.intersect(fh, gh);
            self.mk(vf, l, h)
        };
        // I4. [Cache.]
        self.memo.insert(key, r);
        r
    }

    /// f \ g: every set belonging to f but not to g.
    ///
    /// ```text
    /// D1. [Trivial?]  ∅ \ g = ∅;  f \ ∅ = f;  f \ f = ∅.
    /// D2. [Memo.]     NOT commutative: the key keeps (f, g) in order.
    /// D3. [Expand.]   If only f tests its top v: g cannot cancel any set
    ///                 containing v, so mk(v, f_lo \ g, f_hi);
    ///                 if only g tests its top v: f \ g_lo (g's HI is idle);
    ///                 if both: mk(v, f_lo \ g_lo, f_hi \ g_hi).
    /// D4. [Cache.]
    /// ```
    pub fn diff(&mut self, f: Ref, g: Ref) -> Ref {
        // D1. [Trivial?]
        if f == Ref(0) || f == g {
            return Ref(0);
        }
        if g == Ref(0) {
            return f;
        }
        // D2. [Memo.]
        let key = (Op::Diff, f, g);
        if let Some(&r) = self.memo.get(&key) {
            return r;
        }
        // D3. [Expand.]
        let (vf, vg) = (self.level(f), self.level(g));
        let r = if vf < vg {
            let (lo, hi) = (self.lo(f), self.hi(f));
            let l = self.diff(lo, g);
            self.mk(vf, l, hi)
        } else if vg < vf {
            let lo = self.lo(g);
            self.diff(f, lo)
        } else {
            let (fl, fh) = (self.lo(f), self.hi(f));
            let (gl, gh) = (self.lo(g), self.hi(g));
            let l = self.diff(fl, gl);
            let h = self.diff(fh, gh);
            self.mk(vf, l, h)
        };
        // D4. [Cache.]
        self.memo.insert(key, r);
        r
    }

    /// Knuth's join f ⊔ g: { a ∪ b : a ∈ f, b ∈ g } — the "multiplication"
    /// of the family algebra (§7.1.4; Minato's product). {∅} is its
    /// identity and ∅ annihilates.
    ///
    /// ```text
    /// J1. [Trivial?]  ∅ ⊔ g = f ⊔ ∅ = ∅;  {∅} ⊔ g = g;  f ⊔ {∅} = f.
    /// J2. [Memo.]     Commutative key.
    /// J3. [Expand.]   If only f tests its top v (v below g's support):
    ///                 mk(v, f_lo ⊔ g, f_hi ⊔ g) — v lands in a ∪ b
    ///                 exactly when a came from f's HI. If both test v:
    ///                 HI = (f_hi ⊔ g_hi) ∪ (f_hi ⊔ g_lo) ∪ (f_lo ⊔ g_hi)
    ///                 (v enters the union if either side supplies it),
    ///                 LO = f_lo ⊔ g_lo; answer mk(v, LO, HI).
    /// J4. [Cache.]
    /// ```
    pub fn join(&mut self, f: Ref, g: Ref) -> Ref {
        // J1. [Trivial?]
        if f == Ref(0) || g == Ref(0) {
            return Ref(0);
        }
        if f == Ref(1) {
            return g;
        }
        if g == Ref(1) {
            return f;
        }
        // J2. [Memo.]
        let key = if f.0 <= g.0 { (Op::Join, f, g) } else { (Op::Join, g, f) };
        if let Some(&r) = self.memo.get(&key) {
            return r;
        }
        // J3. [Expand.]
        let (vf, vg) = (self.level(f), self.level(g));
        let r = if vf < vg {
            let (lo, hi) = (self.lo(f), self.hi(f));
            let l = self.join(lo, g);
            let h = self.join(hi, g);
            self.mk(vf, l, h)
        } else if vg < vf {
            let (lo, hi) = (self.lo(g), self.hi(g));
            let l = self.join(f, lo);
            let h = self.join(f, hi);
            self.mk(vg, l, h)
        } else {
            let (fl, fh) = (self.lo(f), self.hi(f));
            let (gl, gh) = (self.lo(g), self.hi(g));
            let l = self.join(fl, gl);
            let hh = self.join(fh, gh);
            let hl = self.join(fh, gl);
            let lh = self.join(fl, gh);
            let h1 = self.union(hh, hl);
            let h = self.union(h1, lh);
            self.mk(vf, l, h)
        };
        // J4. [Cache.]
        self.memo.insert(key, r);
        r
    }
}

// ============================================================================
// §7.1.4 applications — counting structures in graphs
// ============================================================================

/// Shared engine for both graph counters: the number of subsets of
/// {0, …, n_vars−1} containing no *conflicting pair*, computed entirely in
/// the family algebra:
///
/// ```text
/// G1. [Power set.]  P <- ⨆_v ({∅} ∪ {{v}})   — the family of all 2^n
///                   subsets, an n-node ZDD (each join factor reads
///                   "v absent or v present").
/// G2. [Filter.]     F <- P; then for each conflicting pair (u, v):
///                   F <- F \ (P ⊔ {{u,v}}).
///                   (P ⊔ {{u,v}} is every subset ⊇ {u, v}, so the
///                   difference deletes exactly the members of F that
///                   contain both u and v.)
/// G3. [Count.]      Answer count_sets(F).
/// ```
fn conflict_free_count(n_vars: usize, conflicts: &[(usize, usize)]) -> u128 {
    let mut z = Zdd::new();
    // G1. [Power set.]
    let mut p = z.unit();
    for v in 0..n_vars {
        let s = z.single(v as u32);
        let u = z.unit();
        let factor = z.union(u, s); // {∅, {v}}
        p = z.join(p, factor);
    }
    // G2. [Filter.]
    let mut fam = p;
    for &(u, v) in conflicts {
        let su = z.single(u as u32);
        let sv = z.single(v as u32);
        let pair = z.join(su, sv); // {{u, v}}
        let bad = z.join(p, pair); // every subset containing both u and v
        fam = z.diff(fam, bad);
    }
    // G3. [Count.]
    z.count_sets(fam)
}

/// The number of *matchings* of the graph with vertices `0..n_vertices`
/// and the given edges: edge-subsets in which no two chosen edges share
/// an endpoint (the empty matching counts). Construction: ZDD variables
/// are the *edge indices*; two edges conflict when they share a vertex;
/// `conflict_free_count` (documented there) does the rest — a pure
/// family-algebra build: power set of edges, then one `diff` per
/// conflicting edge pair.
pub fn matchings_zdd(n_vertices: usize, edges: &[(usize, usize)]) -> u128 {
    let mut conflicts = Vec::new();
    for (i, &(a, b)) in edges.iter().enumerate() {
        assert!(
            a < n_vertices && b < n_vertices && a != b,
            "edge must join two distinct vertices < n_vertices"
        );
        for (j, &(c, d)) in edges.iter().enumerate().skip(i + 1) {
            if a == c || a == d || b == c || b == d {
                conflicts.push((i, j));
            }
        }
    }
    conflict_free_count(edges.len(), &conflicts)
}

/// The number of *independent sets* of the graph with vertices
/// `0..n_vertices` and the given edges (the empty set counts). ZDD
/// variables are the vertices; each edge is one conflicting pair. The
/// counts must — and do — agree with Module 13's BDD model counter:
/// same family, different diagram.
pub fn independent_sets_zdd(n_vertices: usize, edges: &[(usize, usize)]) -> u128 {
    for &(u, v) in edges {
        assert!(
            u < n_vertices && v < n_vertices && u != v,
            "edge must join two distinct vertices < n_vertices"
        );
    }
    conflict_free_count(n_vertices, edges)
}

// ============================================================================
// §7.2.2.1 — Algorithm 7.2.2.1C: exact cover with colors (XCC)
// ============================================================================

/// An exact-cover-with-colors problem, solved by dancing links extended
/// with color controls (Knuth's Algorithm 7.2.2.1C, recast from his
/// sequential TOP/ULINK/DLINK arrays onto the four-way ring representation
/// of Module 09).
///
/// *Items*: `0..n_primary` are **primary** (each must be covered exactly
/// once); `0..n_secondary` are **secondary** (each covered *at most* once,
/// and every appearance carries a color). Two options that share a
/// secondary item are compatible iff they give it the same color.
///
/// Internal color encoding: `color[x] = 0` for primary nodes,
/// `c + 1 > 0` for a secondary node with user color `c`, and `-1` for a
/// node *purified* by an equal-color commitment (logically "already
/// satisfied; skip me"). `hide`/`unhide` skip purified nodes, which is
/// exactly what makes every operation perfectly reversible.
pub struct Xcc {
    left: Vec<usize>,
    right: Vec<usize>,
    up: Vec<usize>,
    down: Vec<usize>,
    col: Vec<usize>,         // node -> its item header
    size: Vec<usize>,        // header -> number of active nodes in its list
    color: Vec<i64>,         // color control per node (see struct docs)
    node_option: Vec<usize>, // node -> the option it belongs to
    n_primary: usize,
    n_secondary: usize,
    num_options: usize,
    root: usize, // the special header, ringed through PRIMARY headers only
}

impl Xcc {
    /// Create a problem over `n_primary` primary and `n_secondary`
    /// secondary items, with no options yet.
    ///
    /// Layout: node 0 is the root; nodes `1..=n_primary` are primary
    /// headers, circularly linked with the root (the choice loop never
    /// sees anything else); nodes `n_primary+1..=n_primary+n_secondary`
    /// are secondary headers, horizontally self-linked — they are reached
    /// only through their vertical lists, exactly why Knuth parks
    /// secondary items past the N₁ boundary of his item array.
    pub fn new(n_primary: usize, n_secondary: usize) -> Self {
        let n = n_primary + n_secondary + 1;
        let mut x = Xcc {
            left: vec![0; n],
            right: vec![0; n],
            up: vec![0; n],
            down: vec![0; n],
            col: vec![0; n],
            size: vec![0; n],
            color: vec![0; n],
            node_option: vec![usize::MAX; n],
            n_primary,
            n_secondary,
            num_options: 0,
            root: 0,
        };
        // Root ring over primary headers only: root <-> 1 <-> ... <-> n_primary.
        for i in 0..=n_primary {
            x.left[i] = if i == 0 { n_primary } else { i - 1 };
            x.right[i] = if i == n_primary { 0 } else { i + 1 };
        }
        // Secondary headers stand alone horizontally.
        for i in (n_primary + 1)..n {
            x.left[i] = i;
            x.right[i] = i;
        }
        // Every item list starts empty: up/down point back at the header.
        for i in 0..n {
            x.up[i] = i;
            x.down[i] = i;
            x.col[i] = i;
        }
        x
    }

    /// Add one option: a set of primary items plus a set of
    /// `(secondary item, color)` pairs. Returns the option's index.
    /// Items must be distinct within an option (Knuth's precondition).
    /// An option with no primary item is legal but inert — the search
    /// branches only on primary items, so it can never be chosen.
    pub fn add_option(&mut self, primary_items: &[usize], secondary: &[(usize, u32)]) -> usize {
        for (i, &p) in primary_items.iter().enumerate() {
            assert!(p < self.n_primary, "primary item {p} out of range");
            assert!(
                !primary_items[..i].contains(&p),
                "duplicate primary item {p} in option"
            );
        }
        for (i, &(s, _)) in secondary.iter().enumerate() {
            assert!(s < self.n_secondary, "secondary item {s} out of range");
            assert!(
                !secondary[..i].iter().any(|&(t, _)| t == s),
                "duplicate secondary item {s} in option"
            );
        }
        let opt = self.num_options;
        self.num_options += 1;
        let cells: Vec<(usize, i64)> = primary_items
            .iter()
            .map(|&p| (p + 1, 0i64))
            .chain(
                secondary
                    .iter()
                    .map(|&(s, c)| (self.n_primary + 1 + s, c as i64 + 1)),
            )
            .collect();
        let mut first: Option<usize> = None;
        for (c, color) in cells {
            let node = self.left.len();
            // Grow every parallel array by one node.
            self.left.push(node);
            self.right.push(node);
            self.up.push(0);
            self.down.push(0);
            self.col.push(c);
            self.size.push(0); // unused for non-headers
            self.color.push(color);
            self.node_option.push(opt);
            // Splice `node` into the bottom of item list c (just above c).
            let last = self.up[c];
            self.down[last] = node;
            self.up[node] = last;
            self.down[node] = c;
            self.up[c] = node;
            self.size[c] += 1;
            // Link horizontally into this option's circular row.
            match first {
                None => first = Some(node),
                Some(f) => {
                    let l = self.left[f];
                    self.right[l] = node;
                    self.left[node] = l;
                    self.right[node] = f;
                    self.left[f] = node;
                }
            }
        }
        opt
    }

    /// `hide(p)`: unlink every *other* node of p's option from its item
    /// list — except nodes already purified (`color < 0`), which purify
    /// left linked on purpose; skipping them here (and again in `unhide`)
    /// is what keeps hide/unhide exact inverses.
    fn hide(&mut self, p: usize) {
        let mut q = self.right[p];
        while q != p {
            if self.color[q] >= 0 {
                self.down[self.up[q]] = self.down[q];
                self.up[self.down[q]] = self.up[q];
                self.size[self.col[q]] -= 1;
            }
            q = self.right[q];
        }
    }

    /// `unhide(p)`: exact inverse of `hide(p)`, walking the row the other
    /// way and skipping the same purified nodes.
    fn unhide(&mut self, p: usize) {
        let mut q = self.left[p];
        while q != p {
            if self.color[q] >= 0 {
                self.size[self.col[q]] += 1;
                self.down[self.up[q]] = q;
                self.up[self.down[q]] = q;
            }
            q = self.left[q];
        }
    }

    /// `cover(i)` for a primary item: remove i from the root ring, then
    /// hide every option in i's list (each has become unusable).
    fn cover(&mut self, i: usize) {
        self.right[self.left[i]] = self.right[i];
        self.left[self.right[i]] = self.left[i];
        let mut p = self.down[i];
        while p != i {
            self.hide(p);
            p = self.down[p];
        }
    }

    /// `uncover(i)`: exact reverse of `cover(i)`.
    fn uncover(&mut self, i: usize) {
        let mut p = self.up[i];
        while p != i {
            self.unhide(p);
            p = self.up[p];
        }
        self.right[self.left[i]] = i;
        self.left[self.right[i]] = i;
    }

    /// `purify(p)`: p is a secondary node with color c = color[p] > 0,
    /// being committed. Fix item i = col[p] to color c: walk i's list;
    /// nodes of the *same* color become purified (color <- -1, left
    /// linked — their options remain choosable and this item now costs
    /// them nothing); options of a *different* color are hidden. Knuth
    /// also records c on the header, a bookkeeping touch we keep.
    fn purify(&mut self, p: usize) {
        let c = self.color[p];
        let i = self.col[p];
        self.color[i] = c;
        let mut q = self.down[i];
        while q != i {
            if self.color[q] == c {
                if q != p {
                    self.color[q] = -1;
                }
            } else {
                self.hide(q);
            }
            q = self.down[q];
        }
    }

    /// `unpurify(p)`: exact reverse of `purify(p)` — restore the -1
    /// markers to color c and unhide the different-colored options,
    /// walking the list in the opposite direction.
    fn unpurify(&mut self, p: usize) {
        let c = self.color[p];
        let i = self.col[p];
        let mut q = self.up[i];
        while q != i {
            if self.color[q] < 0 {
                self.color[q] = c;
            } else if q != p {
                self.unhide(q);
            }
            q = self.up[q];
        }
        self.color[i] = 0;
    }

    /// `commit(j)`: the color-aware generalization of "cover the other
    /// items of the chosen option" (Algorithm C's replacement for
    /// Algorithm X's plain cover):
    /// - primary node (color 0): `cover` its item;
    /// - secondary node with a live color (> 0): `purify` its item;
    /// - purified node (−1): do nothing — an equal-color commitment
    ///   already fixed this item.
    fn commit(&mut self, j: usize) {
        match self.color[j] {
            0 => {
                let c = self.col[j];
                self.cover(c);
            }
            c if c > 0 => self.purify(j),
            _ => {} // purified: nothing to do, and nothing to undo later
        }
    }

    /// `uncommit(j)`: exact reverse of `commit(j)`.
    fn uncommit(&mut self, j: usize) {
        match self.color[j] {
            0 => {
                let c = self.col[j];
                self.uncover(c);
            }
            c if c > 0 => self.unpurify(j),
            _ => {}
        }
    }

    /// Choose the active primary item of minimum size (Knuth's MRV
    /// heuristic, as in Module 09).
    fn choose_item(&self) -> usize {
        let mut best = self.right[self.root];
        let mut best_size = self.size[best];
        let mut c = self.right[best];
        while c != self.root {
            if self.size[c] < best_size {
                best = c;
                best_size = self.size[c];
            }
            c = self.right[c];
        }
        best
    }

    /// The recursive heart of Algorithm 7.2.2.1C (steps C2–C8 recast as
    /// recursion, exactly as Module 09 recast Algorithm X):
    ///
    /// ```text
    /// C2. [Solved?]      If the root ring is empty, visit the solution.
    /// C3. [Choose i.]    An uncovered primary item of minimum size.
    /// C4. [Cover i.]     cover(i); dead end if its list was empty.
    /// C5. [Try option.]  For each option r in i's list: commit every
    ///                    other node of r's row (left to right), recurse,
    /// C6/C7.             then uncommit them right to left and move on
    ///                    to the next option.
    /// C8. [Backtrack.]   uncover(i).
    /// ```
    fn search(&mut self, partial: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        // C2. [Solved?]
        if self.right[self.root] == self.root {
            let mut sol: Vec<usize> = partial.iter().map(|&r| self.node_option[r]).collect();
            sol.sort_unstable();
            out.push(sol);
            return;
        }
        // C3. [Choose i.]
        let i = self.choose_item();
        if self.size[i] == 0 {
            return; // an uncovered primary item nothing can cover: dead end
        }
        // C4. [Cover i.]
        self.cover(i);
        let mut r = self.down[i];
        while r != i {
            // C5. [Try the option containing node r.]
            partial.push(r);
            let mut j = self.right[r];
            while j != r {
                self.commit(j);
                j = self.right[j];
            }
            self.search(partial, out);
            // C6. [Retract]: uncommit in reverse horizontal order.
            let mut j = self.left[r];
            while j != r {
                self.uncommit(j);
                j = self.left[j];
            }
            partial.pop();
            // C7. [Next option.]
            r = self.down[r];
        }
        // C8. [Backtrack.]
        self.uncover(i);
    }

    /// Every solution, each as a sorted list of option indices. The
    /// structure is fully restored afterwards: solving twice gives the
    /// same answer.
    pub fn solve_all(&mut self) -> Vec<Vec<usize>> {
        let mut out = Vec::new();
        let mut partial = Vec::new();
        self.search(&mut partial, &mut out);
        out
    }

    /// The number of solutions.
    pub fn count_solutions(&mut self) -> u64 {
        self.solve_all().len() as u64
    }
}

// ============================================================================
// Tests: worked examples from §7.1.4 and §7.2.2.1
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Build the family whose members are exactly `sets`, via the algebra.
    fn family(z: &mut Zdd, sets: &[&[u32]]) -> Ref {
        let mut f = z.empty();
        for s in sets {
            let mut one = z.unit();
            for &v in *s {
                let sv = z.single(v);
                one = z.join(one, sv);
            }
            f = z.union(f, one);
        }
        f
    }

    #[test]
    fn empty_family_vs_family_of_empty_set() {
        let z = Zdd::new();
        assert_ne!(z.empty(), z.unit());
        assert_eq!(z.count_sets(z.empty()), 0);
        assert_eq!(z.count_sets(z.unit()), 1);
        assert!(!z.contains_set(z.empty(), &[]));
        assert!(z.contains_set(z.unit(), &[]));
        assert_eq!(z.sets(z.empty()), Vec::<Vec<u32>>::new());
        assert_eq!(z.sets(z.unit()), vec![Vec::<u32>::new()]);
    }

    #[test]
    fn canonicity_same_family_two_ways() {
        // The lesson's running family {{0,1}, {2}} built two ways.
        let mut z = Zdd::new();
        let f1 = family(&mut z, &[&[0, 1], &[2]]);
        // Other way: build a superset family and subtract the excess.
        let a = family(&mut z, &[&[0, 1], &[2], &[0]]);
        let junk = family(&mut z, &[&[0]]);
        let f2 = z.diff(a, junk);
        assert_eq!(f1, f2, "canonicity: same family, same Ref");
        assert_eq!(z.count_sets(f1), 2);
        assert_eq!(z.sets(f1), vec![vec![0, 1], vec![2]]);
        assert!(z.contains_set(f1, &[2]) && z.contains_set(f1, &[1, 0]));
        assert!(!z.contains_set(f1, &[0]) && !z.contains_set(f1, &[]));
    }

    #[test]
    fn zero_suppression_holds_arena_wide() {
        let mut z = Zdd::new();
        let f = family(&mut z, &[&[0, 2], &[1], &[1, 3], &[0, 1, 2, 3], &[]]);
        let g = family(&mut z, &[&[2], &[0, 3]]);
        let _ = z.union(f, g);
        let _ = z.join(f, g);
        let _ = z.diff(f, g);
        for i in 2..z.len() {
            assert_ne!(
                z.hi(Ref(i as u32)),
                z.empty(),
                "node {i} violates zero-suppression"
            );
        }
    }

    #[test]
    fn join_hand_examples() {
        // {{a},{b}} ⊔ {{c}} = {{a,c},{b,c}} with a,b,c = 0,1,2.
        let mut z = Zdd::new();
        let ab = family(&mut z, &[&[0], &[1]]);
        let c = family(&mut z, &[&[2]]);
        let j = z.join(ab, c);
        assert_eq!(z.sets(j), vec![vec![0, 2], vec![1, 2]]);
        // {∅} is the identity, ∅ annihilates.
        let (u, e) = (z.unit(), z.empty());
        assert_eq!(z.join(ab, u), ab);
        assert_eq!(z.join(ab, e), e);
    }

    #[test]
    fn sparsity_singletons_are_linear() {
        // The family {{i} : 0 <= i < 64} needs one node per variable.
        let mut z = Zdd::new();
        let mut f = z.empty();
        for i in 0..64 {
            let s = z.single(i);
            f = z.union(f, s);
        }
        assert_eq!(z.count_sets(f), 64);
        assert_eq!(z.node_count(f), 64 + 2);
        // And {{0}} is 3 nodes no matter how large the universe is.
        let s0 = z.single(0);
        assert_eq!(z.node_count(s0), 3);
    }

    #[test]
    fn graph_counts_match_module_13_and_the_classics() {
        // Independent sets: P_4 -> F_6 = 8, C_5 -> L_5 = 11 (Module 13's
        // numbers); matchings of P_4 = F_5 = 5, of C_5 = L_5 = 11.
        let p4 = [(0, 1), (1, 2), (2, 3)];
        let c5 = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)];
        assert_eq!(independent_sets_zdd(4, &p4), 8);
        assert_eq!(independent_sets_zdd(5, &c5), 11);
        assert_eq!(matchings_zdd(4, &p4), 5);
        assert_eq!(matchings_zdd(5, &c5), 11);
        // The lesson's hand trace: matchings of P_3 (edges 01, 12) = 3.
        assert_eq!(matchings_zdd(3, &[(0, 1), (1, 2)]), 3);
        // Telephone numbers: matchings of K_n for n = 0..5: 1,1,2,4,10,26.
        for (n, want) in [1u128, 1, 2, 4, 10, 26].into_iter().enumerate() {
            let mut edges = Vec::new();
            for u in 0..n {
                for v in (u + 1)..n {
                    edges.push((u, v));
                }
            }
            assert_eq!(matchings_zdd(n, &edges), want, "K_{n}");
        }
    }

    #[test]
    fn xcc_reproduces_the_plain_dlx_example() {
        // Knuth's §7.2.2.1 exact-cover example (Module 09, stage 3): items
        // a..g primary, unique solution = options {0, 3, 4}.
        let mut x = Xcc::new(7, 0);
        x.add_option(&[2, 4], &[]);
        x.add_option(&[0, 3, 6], &[]);
        x.add_option(&[1, 2, 5], &[]);
        x.add_option(&[0, 3, 5], &[]);
        x.add_option(&[1, 6], &[]);
        x.add_option(&[3, 4, 6], &[]);
        assert_eq!(x.solve_all(), vec![vec![0, 3, 4]]);
        assert_eq!(x.count_solutions(), 1, "structure restored");
    }

    #[test]
    fn color_semantics() {
        // Two options meeting at a secondary item: same color compatible.
        let mut same = Xcc::new(2, 1);
        same.add_option(&[0], &[(0, 7)]);
        same.add_option(&[1], &[(0, 7)]);
        assert_eq!(same.solve_all(), vec![vec![0, 1]]);
        // Different colors: incompatible.
        let mut diff = Xcc::new(2, 1);
        diff.add_option(&[0], &[(0, 7)]);
        diff.add_option(&[1], &[(0, 8)]);
        assert_eq!(diff.count_solutions(), 0);
    }

    #[test]
    fn latin_squares_of_order_3() {
        // Items: cell(r,c), row-symbol(r,s), col-symbol(c,s) — 27 primary.
        // Option (r,c,s) covers one of each. There are 12 Latin squares.
        let mut x = Xcc::new(27, 0);
        for r in 0..3 {
            for c in 0..3 {
                for s in 0..3 {
                    x.add_option(&[3 * r + c, 9 + 3 * r + s, 18 + 3 * c + s], &[]);
                }
            }
        }
        assert_eq!(x.count_solutions(), 12);
    }
}
