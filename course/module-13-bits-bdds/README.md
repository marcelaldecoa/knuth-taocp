# Module 13 — Bitwise Tricks and Binary Decision Diagrams

> **Source:** *The Art of Computer Programming*, Vol. 4A, §7.1.3 ("Bitwise
> Tricks and Techniques") and §7.1.4 ("Binary Decision Diagrams").
> **Lab:** `labs/module-13-bits-bdds` · **Grade it:** `./grade 13`
>
> This lesson is self-contained: you can complete the module without the
> book. If you own Vol. 4A, read §7.1.3 up to the sideways-addition
> discussion and the first third of §7.1.4 first.

This module is two lessons in how to *represent* Boolean information so that
computation becomes cheap. At the word level (§7.1.3), a 64-bit register is a
tiny SIMD machine, and arithmetic on it — `+`, `−`, `&`, `|`, `^`, `×` — acts
on all 64 bits at once; the art is making carries and masks do combinatorial
work. At the function level (§7.1.4), a Boolean function of dozens of
variables becomes a *reduced ordered binary decision diagram* (BDD), a DAG on
which equivalence testing is pointer comparison and model counting is a
linear-time walk. By the end you will have re-derived three famous bit
tricks from two's-complement algebra, proved the BDD canonicity theorem, and
*measured* — not just believed — the exponential effect of variable ordering.

---

## 1. Bit twiddling is algebra (§7.1.3)

Nothing in this section is a "hack" in the pejorative sense. Every trick is a
small theorem about the binary representation, and each proof follows one
recipe: **write the operand in the form the trick exploits, then compute.**

### 1.1 Two's complement and the rightmost 1-bit

A 64-bit word x represents an integer mod 2^64, and negation is defined by
the identity

    −x = x̄ + 1        (because x + x̄ = 111…1 = 2^64 − 1 ≡ −1).

**Lemma (rightmost-bit extraction).** If x ≠ 0, write x in binary as

    x = α 1 0^a

where a = number of trailing zeros and α is the remaining 63 − a high bits.
Then x & (−x) = 2^a: the AND isolates exactly the rightmost 1-bit.

*Proof.* Complement each region: x̄ = ᾱ 0 1^a. Add 1: the carry ripples
through the a trailing ones and stops at the 0, giving

    −x = x̄ + 1 = ᾱ 1 0^a.

Now AND with x = α 1 0^a: the high region gives α & ᾱ = 0, the middle bit
gives 1 & 1 = 1, the low region gives 0. Result: 0…010^a = 2^a. ∎

The same three-line computation proves a whole family. With x = α 1 0^a we
have x − 1 = α 0 1^a, hence:

| expression   | result (binary) | effect                                   |
|--------------|-----------------|------------------------------------------|
| `x & (x−1)`  | α 0 0^a         | *drop* the rightmost 1                    |
| `x \| (x−1)` | α 1 1^a         | *smear* the rightmost 1 to the right      |
| `x ^ (x−1)`  | 0 1 1^a         | mask of the rightmost 1 and all below     |
| `x & (−x)`   | 0 1 0^a         | *extract* the rightmost 1                 |

Hand-check every row on x = 01011000 (so α = 0101, a = 3) before you code
stage 1; the tests use exactly this example.

Two boundary conventions, fixed by the wrapping arithmetic: for x = 0 the
extraction gives 0 & 0 = 0 (nothing to extract), and the smear gives
0 | (2^64 − 1) = all ones. Rust makes you say `wrapping_neg`/`wrapping_sub`
out loud — the algorithms genuinely rely on mod-2^64 arithmetic, and the
type system usefully refuses to let that pass unremarked.

### 1.2 Sideways addition, derived

ν(x), the number of 1-bits of x ("population count", "sideways addition"
because you sum *across* the word), is the fundamental quantity of §7.1.3.
The elegant computation is divide and conquer **in place**: first make every
2-bit field hold the count of its own two bits, then every 4-bit field, then
every byte, then total the bytes.

The pair step could be done with two masks and an add, but one subtraction
is cheaper. For a 2-bit field with value v ∈ {0, 1, 2, 3}:

| v (binary) | ⌊v/2⌋ | v − ⌊v/2⌋ | ν(v) |
|---|---|---|---|
| 00 | 0 | 0 | 0 |
| 01 | 0 | 1 | 1 |
| 10 | 1 | 1 | 1 |
| 11 | 1 | 2 | 2 |

So ν(v) = v − ⌊v/2⌋ on each field, and `(x >> 1) & 0x5555…` computes all
64/2 floors at once — the mask stops the shift from leaking a neighbor's low
bit into a field. That is the whole SWAR idea ("SIMD within a register").

**Algorithm S** (*Sideways addition, §7.1.3*). Given a 64-bit word x, return
ν(x).

```text
S1. [Pairs.]    x <- x − ((x >> 1) & 0x5555555555555555).
                (Each 2-bit field now holds its own ν, by the table above.)
S2. [Nibbles.]  x <- (x & 0x3333333333333333) + ((x >> 2) & 0x3333333333333333).
                (Sums of adjacent pairs; both operands masked because 2+2
                 needs 3 bits and could carry across a field boundary.)
S3. [Bytes.]    x <- (x + (x >> 4)) & 0x0f0f0f0f0f0f0f0f.
                (One mask *after* the add suffices: each nibble holds ≤ 4,
                 so nibble sums ≤ 8 fit in 4 bits — the garbage that the
                 unmasked add smears into high nibbles is masked away.)
S4. [Total.]    return (x · 0x0101010101010101) >> 56.
                (If the bytes are b7…b0, the product's top byte is
                 b7+b6+…+b0: the multiply is eight shifted adds, and no
                 byte carry occurs because the total is ≤ 64 < 256.)
```

Trace one byte of it, x = 10110110 (ν = 5):

| step | value | fields |
|---|---|---|
| start | 10110110 | — |
| S1 | 01100101 | 01·10·01·01 = counts 1,2,1,1 |
| S2 | 00110010 | 0011·0010 = counts 3,2 |
| S3 | 00000101 | one byte: 5 ✓ |

Count the cost: 4 ands, 3 shifts, 1 subtract, 2 adds, 1 multiply — about a
dozen instructions for 64 bits, *branchless*, no table, no loop. (Hardware
`POPCNT` does it in one instruction today; §4 below tells that story. You
will still write Algorithm S, because it *is* the algorithm inside the
hardware and inside every compiler's fallback path.)

### 1.3 The ruler function and de Bruijn multiplication

ρ(x) = the number of trailing zeros of x — the exponent a of the lemma in
§1.1. The sequence ρ(1), ρ(2), ρ(3), … = 0, 1, 0, 2, 0, 1, 0, 3, … rises
like the tick marks on a ruler, whence the name. ρ(0) is *undefined* — there
is no rightmost 1-bit — and your implementation must say so loudly
(definiteness, as always since Module 01).

You already know how to reduce ρ to a simpler problem: b = x & (−x) = 2^a,
so it suffices to find the exponent of a power of two. The beautiful step is
next. A **de Bruijn cycle** B(2, 6) is a cyclic sequence of 64 bits in which
every 6-bit pattern occurs exactly once as a window. Take a 64-bit constant
D whose left-shifts realize those windows in the top 6 bits — this works
when the cycle *starts* with six zeros, because shifting left feeds zeros in
at the right, which then just replays the cycle's opening zeros. Then:

    b · D = 2^a · D = D << a,

so `(b · D) >> 58` is the a-th window — and since all 64 windows are
distinct, a 64-entry table inverts the map. Multiply, shift, look up: three
instructions.

See it in miniature with B(2, 3): the 8-bit constant D = 00011101 (0x1D).

| a | D << a | top 3 bits |
|---|---|---|
| 0 | 00011101 | 000 |
| 1 | 00111010 | 001 |
| 2 | 01110100 | 011 |
| 3 | 11101000 | 111 |
| 4 | 11010000 | 110 |
| 5 | 10100000 | 101 |
| 6 | 01000000 | 010 |
| 7 | 10000000 | 100 |

All eight windows distinct — so a 3-bit table lookup recovers a. The lab
uses the 64-bit constant D = 0x03f79d71b4cb0a89 (any valid B(2, 6) constant
beginning with six 0 bits works, and there are 2^26 of them; Knuth discusses
the trick and the count in §7.1.3). Build the table once from the identity
`table[(D << a) >> 58] = a` — don't copy a table you can't check.

*Why a multiply?* Because multiplication by a power of two **is** a shift,
and the de Bruijn property turns "which shift was it?" into a perfect hash.
This is the module's first taste of a theme: arithmetic instructions can be
repurposed as data-movement instructions.

An alternative with no magic constant: binary search with masks. If
x & 0xFFFFFFFF ≠ 0 the rightmost 1 is in the low 32 bits, else add 32 to the
answer and shift; repeat with 16, 8, 4, 2, 1. Six branchless rounds. Either
route is accepted in stage 1; both are honest, neither is an intrinsic.

### 1.4 Gosper's hack: the next subset of the same size

Combinatorics wants to enumerate all C(n, w) words with exactly w bits set —
all w-element subsets of an n-set — and §7.1.3 (with HAKMEM item 175, R. W.
Gosper, 1972) does it in increasing numeric order with seven operations per
step ("snoob": smallest next of same bits).

**What must the successor look like?** Write x with its rightmost *run* of
ones exposed:

    x = α 0 1^b 0^a        (b ≥ 1 ones, then a ≥ 0 zeros).

To get a *larger* word we must set some 0-bit to 1; to get the *smallest*
larger word we choose the lowest 0-bit that can be "paid for" by ones below
it — the 0 just above the run. Setting it, we now have one 1 too many above,
so we delete the run's b ones and re-add b − 1 ones — and to keep the result
minimal we pack them at the very bottom:

    next = α 1 0^{a+1} 1^{b−1}.

*Minimality, properly:* any y > x agrees with x above some bit position t
and has y_t = 1 > x_t = 0. If t is above the run's top there would be a
cheaper choice at the run's top + 1; if t is below, y < x. Given that forced
bit, the remaining b − 1 ones must sit as low as possible. This exchange
argument is the proof; the algorithm is its constructive reading.

**Algorithm G** (*Gosper's hack*). Given x ≠ 0, return the next larger word
of the same weight.

```text
G1. [Isolate.]      u <- x & (−x).           (u = 2^a, the run's lowest bit.)
G2. [Carry.]        v <- x + u.              (Adding 2^a to α01^b0^a carries
                                              through the run: v = α10^{b+a}.
                                              One 1 popped out on top; the
                                              run vanished.)
G3. [Redistribute.] return v | (((x ^ v) / u) >> 2).
                    (x ^ v = 01^{b+1}0^a: the vacated run plus the new bit,
                     b + 1 ones in all. Divide by u = 2^a to right-justify:
                     1^{b+1}. We owe only b − 1 ones — of those b + 1, one
                     is the bit v already keeps, and one is the surplus that
                     the carry promoted — so shift right by 2 and OR in the
                     remaining 1^{b−1} at the very bottom.)
```

Trace x = 01011100 (α = 010, b = 3, a = 2; weight 4):

| step | value | note |
|---|---|---|
| G1 | u = 00000100 | 2^2 |
| G2 | v = 01100000 | run collapsed, one bit up |
| x ^ v | 00111100 | run + new bit |
| / u | 00001111 | right-justified |
| >> 2 | 00000011 | the b − 1 = 2 owed ones |
| G3 | **01100011** | next weight-4 word ✓ |

The division is exact (x ^ v ends in exactly a zeros), and on modern CPUs
`/ u` can be replaced by a shift by ρ(u) — but the division form is the
classic one, and it is branch-free.

**Domain.** G2 overflows precisely when the rightmost run of x touches bit
63 — i.e. when x is the largest 64-bit word of its weight, which has no
successor. Also x = 0 has no rightmost run at all. Both are *inputs the
algorithm is not defined for*; stage 1 requires a panic for 0 and documented
behavior for the other. When you enumerate C(12, 3) subsets in the tests,
the loop stops before ever leaving the domain — that's the caller's duty,
stated in the contract.

---

## 2. Binary decision diagrams (§7.1.4)

### 2.1 From decision tree to DAG

Fix a variable order x0 < x1 < … < x_{n−1}. A **binary decision diagram**
is a rooted DAG with two *sinks* ⊥ and ⊤ and internal *branch nodes*; each
branch node is labeled with a variable x_v and has two outgoing edges LO
(taken when x_v = 0) and HI (taken when x_v = 1). It is **ordered** if
variables strictly increase along every root-to-sink path (levels may be
*skipped* — that will matter), and **reduced** if

- **(R1)** no node has LO = HI (such a test decides nothing), and
- **(R2)** no two nodes agree in (variable, LO, HI) (duplicates merged).

A reduced ordered BDD is a *ROBDD*; from here on, "BDD" means ROBDD. The
function of a node is defined by the obvious walk: to evaluate, start at the
root and follow branches; you reach a sink in ≤ n steps. B(f) denotes the
number of nodes (sinks included) reachable from f's root.

Here is the median (2-out-of-3 majority) ⟨x0 x1 x2⟩, LO edges drawn left:

```text
             (x0)
           0/    \1
          (x1)   (x1)
         0/  \1  0/  \1
         ⊥  (x2)(x2)  ⊤
            0/\1 0/\1
            ⊥  ⊤ ⊥ ⊤     — after merging duplicate (x2) nodes: 6 nodes total
```

The two x2-nodes drawn are the *same* node (both are (x2, ⊥, ⊤)); rule R2
merges them, giving B = 6: one x0-node, two x1-nodes, one x2-node, 2 sinks.

### 2.2 The canonicity theorem — the module's central theorem

**Theorem (Bryant 1986; §7.1.4).** Fix the variable order. Every Boolean
function f of x0, …, x_{n−1} is represented by *exactly one* reduced
ordered BDD.

*Proof, by induction on n (the number of variables that may appear).*

*Base n = 0.* The only functions are the constants 0 and 1, and the only
reduced diagrams with no branch nodes are the bare sinks ⊥ and ⊤. Unique,
and distinct from each other.

*Step.* Assume the theorem for functions of x1, …, x_{n−1} (one fewer
variable). Let f be a function of x0, …, x_{n−1} and let f0 = f|x0=0 and
f1 = f|x0=1 be its cofactors — both functions of x1, …, x_{n−1} only, so
each has a unique ROBDD by the induction hypothesis.

*Case 1: f0 = f1.* Then f does not depend on x0. No ROBDD for f can have a
root labeled x0: such a root's LO and HI subgraphs would be ROBDDs for f0
and f1 (the subgraph rooted at any node is itself reduced and ordered, and
computes the corresponding cofactor), and by the induction hypothesis equal
functions have equal — *identical* — ROBDDs, so LO = HI would violate R1.
Hence every ROBDD for f is an ROBDD for the common cofactor, unique by
induction.

*Case 2: f0 ≠ f1.* Then f depends on x0, so the root must be labeled x0 —
a root labeled x_j with j ≥ 1 would make the whole diagram a function
independent of x0 (ordering: x0 can never appear below x_j). The root's LO
and HI subgraphs are ROBDDs of f0 and f1, unique by induction, and they are
non-identical since f0 ≠ f1 — so R1 is satisfied, and by R2 there is exactly
one node with this (x0, LO, HI) triple. The whole diagram is determined. ∎

**Making the theorem executable: hash-consing.** Keep all nodes in one
arena and a *unique table* mapping (var, LO, HI) → node index. Route every
node creation through one function `mk(v, lo, hi)` that (R1) returns `lo`
when `lo == hi` and (R2) returns the existing index when the triple is
already in the table. Then the induction above runs *inside your program*:
equal functions are constructed as the *same index*, and

    f ≡ g   ⟺   Ref(f) == Ref(g)   —   equivalence testing in O(1).

Satisfiability is `f != ⊥`; tautology is `f == ⊤`. Stage 2's tests take the
theorem literally: they build the same function two syntactically alien ways
and `assert_eq!` the `Ref`s.

### 2.3 Synthesis: apply, with memoization

You rarely build a BDD from a truth table; you *combine* BDDs with Boolean
operations, using the **Shannon expansion**. For any binary operation ⋄ and
the topmost (smallest) variable v appearing in f or g:

    f ⋄ g = ( x̄_v ∧ (f0 ⋄ g0) ) ∨ ( x_v ∧ (f1 ⋄ g1) ),

where f0, f1 are f's cofactors on x_v — which are just `low(f)`, `high(f)`
if f's root tests v, and f itself otherwise (f doesn't depend on x_v; its
cofactors are trivial). The operation ⋄ commutes with cofactoring, which is
all this identity says.

**Algorithm A** (*Apply, §7.1.4*). Given nodes f, g in a shared arena and
⋄ ∈ {∧, ∨, ⊕}, compute the node of f ⋄ g.

```text
A1. [Trivial?]  If the answer is forced by sink identities — 0∧g=0, 1∧g=g,
                1∨g=1, 0∨g=g, 0⊕g=g, f⋄f ∈ {f, 0} — return it.
A2. [Memo.]     If the memo table holds (⋄, f, g), return that.
A3. [Expand.]   v <- min(var(f), var(g)); split f and g as above;
                r <- mk(v, apply(⋄, f0, g0), apply(⋄, f1, g1)).
A4. [Cache.]    memo[(⋄, f, g)] <- r; return r.
```

**Theorem (cost of apply).** With the memo table, Algorithm A performs
O(B(f) · B(g)) node expansions.

*Proof sketch.* Every recursive call's arguments are (a node of f's DAG, a
node of g's DAG) — cofactoring only ever moves each argument to one of its
own descendants or leaves it in place. So there are at most B(f)·B(g)
distinct argument pairs; the memo table ensures each is *expanded* at most
once (step A3), and each expansion does O(1) work plus hash operations
(amortized O(1)). Without the memo, the recursion tree can be exponential —
the DAG's sharing is exactly what memoization exploits. ∎

Note also what A3 + `mk` guarantee together: the result is reduced and
ordered *by construction*, so canonicity is maintained through every
operation — including ¬f, which is simply f ⊕ ⊤ (one memoized O(B(f))
sweep). De Morgan's laws hold as *pointer equalities* in your arena.

### 2.4 Counting models (Algorithm 7.1.4C) — and the 2^skip lemma

A BDD with 500 nodes can represent a function with 2^40 satisfying
assignments; counting them by enumeration is hopeless, but counting on the
DAG is one pass. The only subtlety is that reduced BDDs *skip* levels: an
edge from a node at level v to a node at level w > v + 1 passes over
variables x_{v+1}, …, x_{w−1} without testing them.

Define the *level* ℓ(p) of a branch node as its variable index, and pin the
sinks at ℓ = n. For each node p let s(p) = the number of assignments to the
variables x_{ℓ(p)}, …, x_{n−1} that satisfy p's function. (This is
well-defined: p's function depends only on those variables.)

**Lemma (skip weighting).** s(⊥) = 0, s(⊤) = 1, and for a branch node p at
level v with children l = LO(p), h = HI(p):

    s(p) = 2^{ℓ(l)−v−1} · s(l) + 2^{ℓ(h)−v−1} · s(h).

*Proof, by induction on v from n − 1 down to 0.* Split the assignments
counted by s(p) on the value of x_v. Those with x_v = 0 satisfy p iff they
satisfy l's function, which depends only on x_{ℓ(l)}, …, x_{n−1}. The
variables x_{v+1}, …, x_{ℓ(l)−1} — the *skipped* ones — are therefore
unconstrained: every assignment counted by s(l) (well-defined by induction,
since ℓ(l) > v) extends to exactly 2^{ℓ(l)−v−1} assignments of
x_{v+1}, …, x_{n−1}, each giving one assignment counted by s(p) with
x_v = 0, and all are distinct. The x_v = 1 half is symmetric with h. ∎

**Algorithm C** (*Count solutions, Algorithm 7.1.4C*). Compute s(p) for
every node reachable from f, memoized (bottom-up or top-down, one visit per
node); return

    #models(f over n vars) = 2^{ℓ(f)} · s(f)

— the final factor pays for the variables *above* the root, unconstrained
by the same argument. Cost: O(B(f)) arithmetic operations.

Hand-trace on the median BDD of §2.1. Name the nodes: Z = (x2, ⊥, ⊤),
A = (x1, ⊥, Z), B = (x1, Z, ⊤), root R = (x0, A, B); sinks live at level 3.
Note the two edges that *skip* level 2: A's LO edge (level 1 → ⊥ at level 3
jumps over x2) and B's HI edge (level 1 → ⊤, likewise).

| node | level | s, factor by factor |
|---|---|---|
| ⊥ | 3 | 0 |
| ⊤ | 3 | 1 |
| Z | 2 | 2^{3−2−1}·s(⊥) + 2^{3−2−1}·s(⊤) = 1·0 + 1·1 = 1 |
| A | 1 | 2^{3−1−1}·s(⊥) + 2^{2−1−1}·s(Z) = 2·0 + 1·1 = 1 |
| B | 1 | 2^{2−1−1}·s(Z) + 2^{3−1−1}·s(⊤) = 1·1 + 2·1 = 3 |
| R | 0 | 2^{1−0−1}·s(A) + 2^{1−0−1}·s(B) = 1 + 3 = 4 |

Answer: 2^{ℓ(R)}·s(R) = 2^0·4 = **4** ✓ — the models are 011, 101, 110,
111. Sanity-check B's row against the picture: from B, taking x1 = 1 goes
straight to ⊤ *without testing x2*, so both 110 and 111 are models — that
HI edge is worth 2, not 1. Forgetting exactly that factor (getting 3) is
the classic stage-3 bug; the reduction rule R1 deleted the node (x2, ⊤, ⊤),
and the skip factor is where its two assignments went.

### 2.5 The ordering problem, made rigorous

Everything so far is order-relative. How much can the order matter?
Exponentially — and you will measure it in stage 3 before proving it here.

Consider k independent "couples": the function

    f  =  (x_{a1} ∧ x_{b1}) ∨ (x_{a2} ∧ x_{b2}) ∨ … ∨ (x_{ak} ∧ x_{bk})

over 2k distinct variables.

**Good order** (couples adjacent): a1 b1 a2 b2 … ak bk. Reading the
variables in this order, you only ever need to remember "has some earlier
couple already succeeded?" (if yes you're done) and "is the current couple's
first member true?" — constant knowledge. The BDD has one a-node and one
b-node per couple: **B(f) = 2k + 2**. (Check the k = 2 case by hand.)

**Bad order** (all firsts, then all seconds): a1 a2 … ak b1 b2 … bk. Now,
after reading all the a's, the *entire subset* S = { i : x_{ai} = 1 } is
relevant future knowledge — every couple whose first member was true is
still "pending."

**Theorem (width lower bound).** With the bad order, every ordered BDD for
f (reduced or not) has at least 2^k nodes.

*Proof.* For each of the 2^k assignments α to the a-variables, let f_α be
the residual function of the b-variables. Explicitly f_α = ⋁_{i ∈ S(α)}
x_{bi} where S(α) = { i : α(a_i) = 1 }. These 2^k functions are *pairwise
distinct*: if S(α) ≠ S(α′), pick i in one but not the other; the b-assignment
"x_{bi} = 1, all other b's = 0" satisfies exactly one of f_α, f_α′.

Now walk the BDD from the root following α (taking the branch α dictates at
every a-node; nodes for skipped a-variables are simply not encountered).
The walk stops at the first node whose variable is a b-variable (or a sink),
at or below the a/b boundary; call it p(α). The function computed from p(α)
by the remaining walk is exactly f_α — this is the defining semantics of a
decision diagram: the residual function is determined by the current node
alone, not by how you got there. Therefore α ↦ p(α) maps 2^k *distinct*
functions to nodes, and since a node has one function, p is injective on
distinct residuals: the BDD contains at least 2^k distinct nodes. ∎

For k = 8: at least 256 nodes versus 18 — and in your lab the bad-order BDD
comes out at exactly 512 nodes while the good order gives exactly 2·8 + 2 =
18, with **the model count identical (2^16 − 3^8 = 58975) in both**, since
the function never changed. The general lesson: a BDD level is a *memory*,
and its width is the number of distinguishable pasts — the same
"distinguishing set" argument that gives lower bounds in communication
complexity and automata theory (Myhill–Nerode). Finding the optimal order is
NP-hard (Bollig–Wegener 1996), so real BDD packages use heuristics and
dynamic reordering ("sifting", Rudell 1993); Knuth covers both in §7.1.4.

### 2.6 A paragraph on ZDDs

If BDDs are canonical forms for Boolean *functions*, **zero-suppressed**
decision diagrams (ZDDs, Minato 1993) are canonical forms for *families of
sparse sets*, and Knuth is their most enthusiastic expositor (§7.1.4, and
his annual "Christmas tree lectures"). One rule changes: instead of deleting
nodes with LO = HI, delete nodes whose **HI edge points to ⊥** — i.e. a
variable not mentioned is *absent from the set* rather than *free*. When the
objects being represented are families of small subsets of a large universe
(paths in a graph, exact covers, chess-piece placements), that convention
compresses dramatically: the family of all 5-element subsets of a 100-element
universe wastes no nodes saying "the other 95 elements are absent." The
algorithms of this module — mk with a unique table, memoized apply, weighted
counting — all carry over with adjusted skip factors (a skipped ZDD level
multiplies by 1, not 2, on the "absent" interpretation — work out why after
stage 3; it is a two-line change and a good exercise).

---

## 3. Why it's done this way

**Why SWAR instead of a loop or a table?** A byte-table popcount costs 8
dependent loads; the naive loop costs up to 64 iterations. Algorithm S is a
dozen *independent-ish* register operations — no memory traffic, no branch
misprediction, and it vectorizes: the same code counts 2×64 bits in a 128-bit
register. The masks 0x5555…/0x3333…/0x0f0f… are the binary expansion of the
divide-and-conquer recursion itself; nothing about them is ad hoc.

**Why does `ruler` panic on 0 but `extract_rightmost_one` returns 0?**
Because the algebra says so: x & (−x) is a total function with a natural
value at 0, while "the position of the rightmost 1-bit of 0" is a
description with no referent. The course rule since Module 01: an
algorithm's input specification is part of the algorithm.

**Why an arena of `u32` indices instead of `Rc<RefCell<Node>>`?** Three
reasons. Canonicity *requires* a global identity map (the unique table) — a
pointer-per-node design just rebuilds the arena badly. Knuth's own memory
model (links as small integers into arrays) maps exactly onto `Vec<Node>` +
`u32`, so the code stays step-faithful. And it is faster: 12-byte nodes
packed contiguously, no reference counting, `Ref` is `Copy`.

**Why reduce during construction (hash-consing) rather than build-then
reduce?** Bryant's key insight: if `mk` maintains R1+R2 as invariants, then
*every intermediate result* of apply is canonical, so the memo table and the
trivial-case tests in A1 (`f == g`, `f == ⊥`, …) work by index comparison.
A build-then-reduce design loses the O(1) equivalence test exactly where
apply needs it most. The invariant-maintenance discipline is the same one
you used for heaps and B-trees — stated once, enforced at the single point
of creation.

**Why count on the DAG instead of enumerating?** Because the DAG *is* a
dynamic-programming table that the reduction rules built for you: equal
subproblems were merged by R2, so one s-value per node is exhaustive and
non-redundant. The 2^skip factors are the price of the compression R1
performed — the lemma of §2.4 is literally an accounting identity.

---

## 4. In the real world

**Bit tricks are production infrastructure, not folklore.**
- *Chess engines* live on 64-bit "bitboards" (one bit per square — the fit
  is a happy accident of 8×8). Move generators in engines like Stockfish
  iterate attack sets with exactly stage 1's loop: `b = x & (−x)` to peel a
  square, de Bruijn-multiply bitscan to name it, `x &= x − 1` to drop it.
  Before hardware bit-scan was fast and universal, the de Bruijn trick (the
  Leiserson–Prokop–Randall formulation, 1998) was *the* portable bitscan,
  and it still is on targets without the instruction.
- *Hardware grew these functions because software proved them hot.* x86's
  BMI1 extension literally ships stage 1 as instructions: `BLSI` is
  x & (−x), `BLSR` is x & (x − 1), `BLSMSK` is x ^ (x − 1), `TZCNT` is
  `ruler`, and `POPCNT` is Algorithm S in silicon. Population count has been
  a "military instruction" since the CDC 6600 era (cryptanalysts count
  matching bits: ν(a ⊕ b) is Hamming distance), and today it powers bitmap
  index intersections in databases (Roaring bitmaps), chemical-fingerprint
  similarity (Tanimoto), and genomics k-mer counting.
- *Compilers know the idioms.* LLVM and GCC pattern-match the SWAR popcount
  and the `x & (x−1)` loop and replace them with `POPCNT`/`TZCNT` when the
  target has them — write the algebra, get the instruction. Gosper's hack
  remains the standard way to enumerate fixed-size subsets in bitmask
  dynamic programming (TSP over subsets, scheduling), where "next 3-subset
  of 12" is exactly your stage 1 test.

**BDDs run verification and EDA.**
- *Symbolic model checking.* McMillan's 1992 thesis showed that sets of
  states of a finite-state system — and the transition relation itself —
  can be BDDs, letting fixed-point reachability run on state spaces of size
  10^20 and beyond without enumerating a single state. After the 1994
  Pentium FDIV recall, formal verification with BDDs (and later SAT) became
  standard practice at hardware companies; equivalence checking of a
  circuit against its RTL reference — canonicity again: build both, compare
  roots — is a routine sign-off step in every chip's flow.
- *Tools you can touch:* CUDD (the classic C BDD package with reordering by
  sifting), the model checkers NuSMV/nuXmv, ABC for synthesis and
  verification. Knuth's own §7.1.4 programs (BDD14/BDD15) are on his site.
- *ZDDs* count and optimize over combinatorial families in network
  reliability, and Minato's group's Graphillion library
  ("one graph, 10^60 paths") is ZDDs end to end. Knuth's dancing-links
  exact-cover solvers (Module 09) and ZDDs are two faces of one idea:
  represent the solution *family*, not one solution.

---

## 5. Stage-by-stage lab guide

Open `labs/module-13-bits-bdds/src/lab.rs`; each stage has a test file
`tests/stage_NN_*.rs`, and `./grade 13` runs them in order. Ground rule for
stage 1: **no `count_ones`, `trailing_zeros`, `leading_zeros`** or cousins —
the tests use them as oracles, your job is to *be* them.

### Stage 1 — `ruler`, `sideways_addition`, `extract_rightmost_one`, `smear_right`, `next_same_weight`

All five are direct transcriptions of §1 of this lesson. Rust notes:
`wrapping_neg`/`wrapping_sub`/`wrapping_add`/`wrapping_mul` everywhere the
algebra is mod 2^64 (debug builds trap plain overflow, and the traps would
be right: the wrap is semantic, say so). `ruler` must panic on 0 with a
message containing "undefined"; `next_same_weight` must panic on 0, and its
overflow domain (largest word of its weight) is documented, not checked.
Build the de Bruijn table from the identity, or use the mask binary search.
The Gosper test enumerates every 3-subset of a 12-set in increasing order —
if your G3 shift is off by one, the *sequence* (not just one value) breaks,
which makes the bug easy to see with `--nocapture` printing.

### Stage 2 — `Bdd`: mk, variable, apply (and/or/xor/not), eval, node_count

Write `mk` first and route *everything* through it; the two reduction rules
live there and nowhere else. Then `apply` per Algorithm A with one memo
table keyed (op, f, g) — normalize the key order for commutative ops if you
like (halves the table; not required). `not(f) = xor(f, ⊤)`. The tests
audit your whole arena for R1/R2/ordering violations and check canonicity
as `Ref` equality on distributivity, De Morgan, hand-built xor, tautologies,
and 30 random formulas negated two structurally different ways. If a
canonicity assert fails, the bug is almost always in `mk` (forgot a rule) or
in A3 (split a node that doesn't test v).

### Stage 3 — `count_models`

Algorithm C with the 2^skip weighting, `u128` arithmetic, memoized per node
(a local `HashMap<Ref, u128>` under `&self` is fine). Do the §2.4 hand
trace — including its deliberate mistake — before coding. Then run the
ordering experiment: same function, two orders, counts equal, sizes 18
versus > 256. Look at the numbers; you will remember them longer than the
theorem.

### Stage 4 — `independent_set_count`, `queens_bdd_count`

Both are "conjoin constraints, then count": independent sets are
⋀ ¬(x_u ∧ x_v) over edges; queens adds at-least-one-per-row disjunctions
over n² cell variables. The tests anchor the counts to closed forms —
Fibonacci F_{n+2} for paths, Lucas L_n for cycles, 2^n and n + 1 for the
empty and complete graphs, and 2/10/4 solutions for 4/5/6-queens. The
queens BDD's growth is why the tests cap n at 6: with the row-major order
each level must remember which columns and diagonals are still free —
that's the §2.5 width argument aimed at you. (Modules 09 and 10 solved the
same puzzle by backtracking and SAT; now you have the third classic method,
and it is the only one of the three that *counts* solutions without
visiting them one by one.)

---

## 6. Check your understanding

Answer before moving on (no code needed).

1. In step S1 of sideways addition, why is the mask applied to `x >> 1` and
   not to `x`? *(Hint: which neighbor's bit can leak into a 2-bit field
   after the shift, and does the subtrahend or the minuend need protecting?)*
2. The de Bruijn constant for `ruler` must begin with six 0 bits. Which
   inputs x would be mis-ranked if it didn't? *(Hint: for which a does
   D << a pull "wrapped" bits into the top window — and what fills the
   bottom in a real shift?)*
3. In the canonicity proof, point to the exact sentence where rule R1 is
   used, and the exact sentence where R2 is used. What breaks if the unique
   table is keyed on (LO, HI) without the variable? *(Hint: two functions
   as simple as x3 and x5.)*
4. The BDD of f = x2 over n = 6 variables is one branch node and two sinks.
   Compute count_models(f, 6) by the skip lemma, factor by factor. *(Hint:
   2^2 above the root; what do the two sink edges each skip?)*
5. Building a BDD for a CNF formula and testing `f != ⊥` decides SAT, and
   apply is polynomial per operation. Why is P = NP still safe? *(Hint:
   polynomial in **what**? Where did the ordering experiment say the size
   can go?)*

## 7. Exercises from the text

Ratings are Knuth's scale (00 immediate · 10 a minute · 20 up to an hour ·
30 hours · 40 term project). ▶ marks especially instructive ones. Exercise
numbers follow the Vol. 4A first edition; if your printing differs, match
by content. Log your attempts in `course/module-13-bits-bdds/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 7.1.3-(warmup) | 10 | Prove the four rightmost-bit identities of §1.1's table from x = α10^a, including what each does when x = 0. |
| ▶7.1.3-20 | 20 | Derive Gosper's hack yourself from the α01^b0^a picture, then prove the division in G3 is always exact. |
| 7.1.3-(swar) | 22 | Adapt Algorithm S to count bits in each *byte* separately (stop after S3) and use it to compute ν for a 512-bit block with one final multiply per word. |
| ▶7.1.3-(debruijn) | 25 | The number of B(2,n) de Bruijn cycles is 2^(2^(n−1)−n) (de Bruijn 1946). Verify the formula by hand for n = 3 (it gives 2 — find both cycles), and conclude there are 2^26 usable 64-bit `ruler` constants. |
| 7.1.4-(median) | 15 | Draw the ROBDD of ⟨x1x2x3⟩ under both orders x1<x2<x3 and x3<x1<x2; verify B = 6 both times (symmetric functions are order-insensitive — prove it). |
| ▶7.1.4-(canon) | 25 | Write out the canonicity induction for n = 2 explicitly: list all 16 functions and their unique diagrams. |
| 7.1.4-(count) | 20 | Prove the ZDD variant of the skip lemma (skipped level ⇒ factor 1 on the LO-interpretation) and adapt Algorithm C. |
| ▶7.1.4-(hidden) | 30 | The *hidden weighted bit* function HWB(x) = x_{ν(x)} (0 if x = 0): show every variable order gives exponential BDDs (Bryant 1991) — read the proof and reproduce its distinguishing-set core in your own words. |

## 8. Proof techniques you practiced

- **Representation-shape arguments** (write x = α10^a, compute both sides):
  carried every §1.1 identity and Gosper's G2 — the bit-level analogue of
  "write n = qd + r" from Module 01.
- **Exhaustive case analysis on a small domain:** the 4-row table proving
  ν(v) = v − ⌊v/2⌋; small enough to be a *complete* proof, not an example.
- **Bijection/perfect-hash counting:** the 64 de Bruijn windows are distinct
  because a de Bruijn cycle *is* a bijection between shifts and windows —
  pigeonhole with equality.
- **Exchange argument for minimality:** Gosper's successor is smallest
  because any other same-weight increase can be improved by moving a bit
  lower — the greedy-correctness pattern from sorting networks and Huffman.
- **Structural induction on variables:** the canonicity theorem — the
  module's central proof, induction where the induction hypothesis is
  *uniqueness*, used twice (once per cofactor case).
- **Invariant maintenance at a single choke point:** R1 + R2 live only in
  `mk`, so every reachable arena state is reduced — the same discipline as
  heap-order in Module 06 and B-tree balance in Module 11.
- **Downward induction on levels with an accounting identity:** the 2^skip
  lemma — each edge's factor exactly pays for the choices reduction erased.
- **Distinguishing-set (fooling-set) lower bound:** 2^k pairwise-distinct
  residual functions force 2^k nodes — your first *unconditional* size
  lower bound in the course, kin to Myhill–Nerode and communication
  complexity.

## 9. Where this leads

- **Module 14 (CDCL)** attacks the same objects — Boolean constraints —
  from the search side. The trade is fundamental: BDDs pay exponential
  *space* up front for polynomial *queries* forever after (knowledge
  compilation); SAT solvers pay nothing up front and gamble exponential
  *time* per query. #SAT, weighted model counting, and d-DNNF compilers sit
  between the two camps.
- **Backwards:** Module 09's dancing links enumerate exact covers one at a
  time; ZDDs represent all of them at once — Knuth's §7.2.2.1 ties the two
  together explicitly.
- **Deeper into §7.1.4:** dynamic variable reordering (sifting), companion
  algorithms (restriction, composition, existential quantification — the
  model-checking toolkit), and BDD-based dynamic programming for
  optimization (shortest paths in the DAG = minimum-cost satisfying
  assignments).
- **The bit-tricks thread** continues wherever words meet combinatorics:
  §7.1.3's broadword chess problems, bit-parallel string matching
  (Shift-Or, Myers' bit-vector edit distance), and succinct data structures
  whose rank/select primitives are, at bottom, popcount with bookkeeping.
