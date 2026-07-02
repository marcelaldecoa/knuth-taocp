# Module 21 — Boolean Functions and Optimal Evaluation

> **Source:** *The Art of Computer Programming*, Vol. 4A, §7.1.1 (Boolean basic
> functions) and §7.1.2 (Boolean evaluation and optimum chains).
> **Lab:** `labs/module-21-boolean` · **Grade it:** `./grade 21`
>
> Self-contained: you can finish the module from this lesson alone. If you own
> Vol. 4A, read §7.1.1–7.1.2 alongside — this lesson tells you where to look.

A Boolean function turns a handful of yes/no inputs into a single yes/no
answer. That sounds small, but it is the entire subject matter of digital
hardware, of SAT solvers, of cryptographic circuits, and of a counting problem
so hard that its ninth value was only computed in 2023. This module builds the
data structures and the search that let you *manipulate* Boolean functions, and
then asks the optimization question Knuth cares most about: **what is the
cheapest circuit that computes a given function?**

By the end you will represent functions as integers, convert to and from normal
forms, build and cost straight-line circuits ("Boolean chains"), recognize the
special functions (median, threshold, symmetric, monotone, self-dual), and run
an exact search for the minimum gate count `C(f)`.

---

## 1. The truth-table-as-integer trick

An `n`-variable Boolean function `f(x₁, …, xₙ)` has `2ⁿ` possible inputs, and on
each it returns 0 or 1. Write the inputs in binary — the integer `i` stands for
the assignment `x_j = (i >> (j−1)) & 1` — and the function is completely
described by the `2ⁿ`-bit string of its outputs. For `n ≤ 6` that string fits in
a single 64-bit word:

```
table bit i  =  f(i)
```

This is the representation Knuth uses everywhere in Volume 4A, and the reason is
irresistible: **operations on functions become bitwise operations on words.**

| function-level operation | truth-table operation |
|---|---|
| `¬f` | `!table` (masked to `2ⁿ` bits) |
| `f ∧ g` | `table_f & table_g` |
| `f ∨ g` | `table_f \| table_g` |
| `f ⊕ g` | `table_f ^ table_g` |
| "does f ever equal 1?" | `table != 0` |
| "how many inputs give 1?" | `table.count_ones()` |

A single 64-bit AND evaluates a conjunction on all `2ⁿ` rows of the truth table
*at once*. There is no faster way to do bulk Boolean algebra on a normal CPU,
and it is why the whole module fits in one `u64` field:

```rust
pub struct BoolFunc { pub n: u32, pub table: u64 }
```

**Worked example — AND of two variables.** With `x₁` = bit 0 and `x₂` = bit 1,
`x₁ ∧ x₂` is 1 only at input `i = 0b11 = 3`, so `table = 0b1000 = 8`. Reading it
back: `eval(3) = (8 >> 3) & 1 = 1`, and `eval(0) = eval(1) = eval(2) = 0`. The
number of 1-inputs is `count_ones(8) = 1`.

### How many functions are there?

Each of the `2ⁿ` rows can independently be 0 or 1, so there are exactly

```
    2^(2ⁿ)   Boolean functions of n variables.
```

That is 2, 4, 16, 256, 65536, 4.3 billion, 1.8×10¹⁹ for `n = 0,1,2,3,4,5,6`. The
double exponential is the central fact of the theory: functions are absurdly
numerous, so **most of them cannot have a small circuit** — there are not enough
small circuits to go around. We make that precise in §5.

---

## 2. Normal forms: DNF and CNF

Two canonical ways to write any function fall straight out of the truth table.

A **minterm** is a product (AND) of literals — one per variable — that is true
at exactly one input. The row `i` where `f = 1` corresponds to the minterm using
`+j` when `x_j = 1` in `i` and `−j` (meaning `¬x_j`) when `x_j = 0`. Since a
function is 1 exactly on its 1-rows, it equals the OR of its minterms — the
**disjunctive normal form (DNF)**:

```
    f  =  ⋁ (minterm of each 1-row)
```

Dually, a **maxterm** is a clause (OR of literals) false at exactly one input.
The rows where `f = 0` give the maxterms (negate every literal of that
assignment so the clause vanishes there), and `f` equals their AND — the
**conjunctive normal form (CNF)**:

```
    f  =  ⋀ (maxterm of each 0-row)
```

We store a term or clause as a `Vec<i32>` of signed literals: `+j` for `x_j`,
`−j` for `¬x_j`, with variables numbered from 1.

**Worked example — OR of two variables.** `x₁ ∨ x₂` is true on rows 1, 2, 3 and
false only on row 0.

| row `i` | `x₁ x₂` | `f` | contributes |
|---|---|---|---|
| 0 | 0 0 | 0 | maxterm `(x₁ ∨ x₂)` = clause `[+1, +2]` |
| 1 | 1 0 | 1 | minterm `x₁¬x₂` = term `[+1, −2]` |
| 2 | 0 1 | 1 | minterm `¬x₁x₂` = term `[−1, +2]` |
| 3 | 1 1 | 1 | minterm `x₁x₂` = term `[+1, +2]` |

So the DNF has **3** terms (= number of minterms = `count_ones`) and the CNF has
**1** clause (= number of 0-rows). Notice the size asymmetry: OR has a tiny CNF
and a larger DNF. In general one normal form can be exponentially larger than
the other, which is exactly why choosing the representation matters.

**Sizes.** `to_dnf(f).len() == f.num_minterms() == popcount(table)`, and
`to_cnf(f).len() == 2ⁿ − popcount(table)`. Both normal forms reconstruct the
function exactly (`from_dnf` / `from_cnf` are the inverses), and terms need not
be full minterms — a shorter product term like `[+1]` covers a whole subcube
(here, both rows with `x₁ = 1`).

**De Morgan, for free.** Complementing the table (`!table`, masked) complements
the function, and the two laws `¬(f ∧ g) = ¬f ∨ ¬g` and `¬(f ∨ g) = ¬f ∧ ¬g` are
just bitwise identities on words. In the lab you check `¬(x₁ ∧ x₂)` equals
`(¬x₁) ∨ (¬x₂)` by comparing whole truth tables — a proof by exhaustive
evaluation that `assert_eq!` performs in one line.

---

## 3. Boolean chains: circuits as straight-line programs

A **Boolean chain** (Knuth's term, §7.1.2) is a straight-line program that
computes a function with 2-input gates. Its values are numbered:

- values `0 .. n−1` are the inputs `x₁ … xₙ`;
- value `n + k` is produced by step `k`, a gate `op(value[left], value[right])`
  where `left, right` are strictly earlier value indices.

The result is whichever value you nominate as the output. **Cost** is simply the
number of gates. This is the "size" measure of a circuit; it models a chip's
gate count, an FPGA's LUT usage, or a crypto circuit's operation budget.

### The sixteen gates, encoded as nibbles

There are exactly `2^(2²) = 16` two-input Boolean operations. We encode each as
a 4-bit truth table `op`: the output for inputs `(a, b)` is bit `2a + b` of `op`.

| index `2a+b` | `(a,b)` |
|---|---|
| 0 | (0,0) |
| 1 | (0,1) |
| 2 | (1,0) |
| 3 | (1,1) |

So `AND` (true only at `(1,1)`) is `0b1000 = 8`; `XOR` (true at `(0,1)` and
`(1,0)`) is `0b0110 = 6`; `OR` is `0b1110 = 14`; and `NOT` of the left operand,
ignoring the right, is `NOTL = 0b0011 = 3`. This uniform nibble encoding is what
makes the optimum-cost search in §6 a few bitwise instructions per gate.

**Worked example — XOR from AND/OR/NOT.** XOR is not in the `{AND, OR, NOT}`
basis directly, but `x₁ ⊕ x₂ = (x₁ ∨ x₂) ∧ ¬(x₁ ∧ x₂)`:

```text
value 0: x1                       (input)
value 1: x2                       (input)
value 2: OR (0, 1)   = x1 ∨ x2
value 3: AND(0, 1)   = x1 ∧ x2
value 4: NOTL(3, 3)  = ¬(x1 ∧ x2)
value 5: AND(2, 4)   = (x1∨x2) ∧ ¬(x1∧x2)   ← output
```

Four gates. Trace it on `x₁=1, x₂=1`: value 2 = 1, value 3 = 1, value 4 = 0,
value 5 = 0 — correct, `1 ⊕ 1 = 0`. Of course the *full* basis contains XOR as a
single gate, so with all sixteen operations available this same function costs
just 1. **The basis you allow changes the cost**; that tension is the whole
point of §6.

`chain_computes(chain, f)` certifies a chain by evaluating it on all `2ⁿ` inputs
and comparing to `f`'s table — the only honest test of a straight-line program.

---

## 4. Special functions worth a name (§7.1.1)

Some functions are important enough that Knuth gives them their own notation.

**Median / majority.** For an odd number of inputs, `⟨x₁ … xₙ⟩` is the value
held by more than half of them — the median bit. Equivalently it is a
**threshold**: majority is true iff at least `⌈n/2⌉` inputs are 1. The carry out
of a full adder is exactly `majority(x₁, x₂, x₃)`.

**Threshold.** `[x₁ + ⋯ + xₙ ≥ k]` — true when at least `k` inputs are 1. Median
is the special case `k = (n+1)/2`.

**Symmetric functions.** A function is *symmetric* if its value depends only on
*how many* inputs are 1, not which. Such a function is pinned down by the
`n + 1` bits `w₀, …, wₙ`, where `w_j` is the value when exactly `j` inputs are
true. That is a spectacular compression: `n + 1` bits instead of `2ⁿ`. Majority,
threshold, and parity are all symmetric. In code:

```rust
symmetric_function(n, &weights)   // value at x = weights[popcount(x)]
```

**Monotone functions.** `f` is *monotone (nondecreasing)* if flipping any input
from 0 to 1 never flips the output from 1 to 0 — formally `x ⊆ y ⇒ f(x) ≤ f(y)`
under the bitwise-subset order. AND, OR, majority, and every threshold are
monotone; XOR is not (raising one input can toggle the answer either way). It
suffices to check single-bit steps: for every `x` and every clear bit `b`,
`f(x) ≤ f(x | bit b)`.

**Self-dual functions.** `f` is *self-dual* if `f(¬x) = ¬f(x)` for every input —
complementing all inputs complements the output. Every projection `x_j` is
self-dual, and the median of an odd number of inputs is the archetype (flip all
the votes and the majority flips too). AND is not self-dual.

### Dedekind's problem — a famous hard count

How many *monotone* Boolean functions of `n` variables are there? This is the
**Dedekind number** `M(n)`, and it grows ferociously:

```
   n:    0   1   2   3    4     5        6            7                8                    9
 M(n):   2   3   6  20  168  7581  7828354  2414682040998  56130437228687557907788  286386577668298411128469151667598498812366
```

Dedekind posed it in 1897. Each new value has been a milestone: `M(8)` took a
supercomputer in 1991, and **`M(9)` was computed only in 2023** — twice,
independently, by Christian Jäkel and by Van Hirtum et al., using months of FPGA
and GPU time. There is no known closed form. In the lab you compute `M(0..4)` the
brute-force way — enumerate all `2^(2ⁿ)` functions and count the monotone ones —
reproducing `2, 3, 6, 20, 168`. At `n = 4` that is 65536 functions, which your
laptop dispatches in well under a second; the wall you hit at `n = 6` is the
same wall the researchers spent decades scaling.

---

## 5. Combinational complexity C(f) and the counting bound

The **combinational complexity** `C(f)` is the minimum number of gates in any
Boolean chain (over a chosen basis) that computes `f`. It is the "size" of the
cheapest circuit. Two functions of the same `n` can have wildly different `C`:
a projection has `C = 0`, XOR-of-two has `C = 1`, and the hardest functions sit
near the ceiling we now derive.

**Shannon's counting theorem (the pessimist's masterpiece).** *Almost every*
Boolean function of `n` variables requires roughly `2ⁿ / n` gates.

*Proof sketch (counting circuits vs. functions).* A chain of `r` gates over the
16-operation basis is described by choosing, for each gate, an operation (16
ways) and two earlier operands (at most `(n + r)²` ways). So the number of
chains with `r` gates is at most `(16 (n+r)²)ʳ`, which is `2^{O(r log r)}`. To
compute even a constant fraction of the `2^(2ⁿ)` functions we need
`2^{O(r log r)} ≥ 2^(2ⁿ) · const`, forcing `r = Ω(2ⁿ / n)`. Conversely Lupanov
showed `O(2ⁿ / n)` gates always suffice. So the *typical* function needs
`≈ 2ⁿ / n` gates — exponentially many. ∎ (sketch)

The moral is the double exponential from §1 biting back: there are
`2^(2ⁿ)` functions but only `2^{O(r log r)}` small circuits, so small circuits
are precious and most functions are inherently expensive. Yet the functions we
actually *want* — adders, comparators, the symmetric functions — are the rare
cheap ones, which is why hardware is possible at all.

**Size vs. depth.** Two different cost measures live on the same chain. *Size*
is the gate count (what `C(f)` measures). *Depth* is the longest path from an
input to the output — the number of gate delays, i.e. latency. The XOR chain in
§3 has size 4 but depth 3 (`AND → NOTL → AND`). Minimizing size saves area and
power; minimizing depth saves time. They trade off against each other, and real
synthesis tools juggle both.

---

## 6. Searching for the optimum chain (§7.1.2)

Computing `C(f)` exactly is a search problem. The lab's `optimal_cost` runs a
breadth-first search — but *over which graph?* Getting this right is the subtle
heart of the module.

**The tempting shortcut that is wrong.** Grow a set `R_c` of "functions
reachable with `c` gates" by `R_0 = {constants, projections}` and
`R_c = R_{c−1} ∪ { g(a,b) : a, b ∈ R_{c−1} }`. This *undercounts*, because it
pretends the two operands `a` and `b` are simultaneously available for free. But
each operand may itself cost gates, and their subcircuits do not always share.
For **majority-of-three** the shortcut reports 3 — yet no 3-gate chain exists;
the true cost is 4. The shortcut computes a *lower bound*, not the answer.

**The correct search — BFS over states.** A real chain keeps *every*
intermediate value available, so sharing is automatic. Model a **state** as the
*set of functions computed so far*. Start from the free set (the two constants
and the `n` projections). One move appends a gate:

```text
    state  →  state ∪ { g(a, b) }      for a, b ∈ state, g ∈ basis     (cost +1)
```

BFS over states, deduplicating each state by its sorted contents. The first
state that contains `f`'s truth table is reached at depth `C(f)`. Because the
whole *set* is carried forward, a later gate may reuse any earlier value — that
is exactly what "sharing" means, and it is why this search, unlike the shortcut,
is correct.

**Worked derivation — majority-of-three needs four gates.**

```text
value 0,1,2 : x1, x2, x3          (free inputs)
value 3     : AND(0, 1) = x1 ∧ x2
value 4     : OR (0, 1) = x1 ∨ x2
value 5     : AND(2, 4) = x3 ∧ (x1 ∨ x2)
value 6     : OR (3, 5) = (x1∧x2) ∨ (x3∧(x1∨x2))   ← majority, output
```

Check the three cases: if `x₁ = x₂ = 1` value 3 fires; if exactly one of
`x₁, x₂` is 1 then value 4 = 1 and value 5 = `x₃`, so the output is 1 iff `x₃`
adds the second vote; if `x₁ = x₂ = 0` everything is 0. Four gates, and the
state-BFS confirms no chain of three suffices. (For `n = 3` the search shows
*every* function has `C ≤ 4` over the full basis.)

**Cost values you will pin.** Over the full 16-gate basis: constants and
projections cost 0; every one of the 16 two-variable functions costs at most 1
(each *is* a gate), with `C = 0` only for the 4 degenerate ones (two constants,
two projections — note a *negated* input still costs one NOT gate); `C(XOR₂) =
1`; `C(x₁ ⊕ x₂ ⊕ x₃) = 2`; `C(majority₃) = 4`.

**Staying fast.** Keep `n ≤ 3`: there are only 256 functions of 3 variables, and
the state-BFS finishes each query in a fraction of a second. At `n = 4` the
65536 functions blow the state space up — the same combinatorial explosion that
pushes *exact* synthesis of real circuits onto SAT solvers.

---

## 7. Stage-by-stage lab guide

Open `labs/module-21-boolean/src/lab.rs`. Each stage has a test file
`tests/stage_NN_*.rs`; `./grade 21` runs them in order and stops at the first
failure.

### Stage 1 — `BoolFunc` and normal forms

Implement the struct methods. `from_closure` tabulates a rule into a `u64`;
`eval` reads one bit; `to_dnf`/`to_cnf` walk the rows emitting signed-literal
terms; `from_dnf`/`from_cnf` evaluate the formula on every input. The tests
check eval-matches-table, that DNF size equals the minterm count, exhaustive
DNF/CNF round-trips for all functions of `n ≤ 4`, the constant/tautology edge
cases, and De Morgan via `complement`. Watch the masking so `n = 6` (all 64
bits) does not overflow.

### Stage 2 — Boolean chains

Implement `apply_gate` (index bit `2a+b` of the op), the `Chain` builder
(`new`, `gate`, `set_output`), `eval_chain`, `chain_cost`, and `chain_computes`.
Build XOR from AND/OR/NOT (cost 4) and majority-of-three by hand (cost 4), and
verify the sum (`x₁⊕x₂⊕x₃`) and carry (`majority`) of a full adder. The key test
evaluates a chain over all `2ⁿ` inputs and matches its target's table.

### Stage 3 — median, threshold, symmetric, monotone, self-dual

Implement the five functions. `majority` is `2·popcount > len`; `threshold` is
`popcount ≥ k`; `symmetric_function` reads `weights[popcount(x)]`. `is_monotone`
checks single-bit raises; `is_self_dual` checks `f(x) ≠ f(¬x)`. The capstone
test reproduces the Dedekind numbers `2, 3, 6, 20, 168` by enumerating all
`2^(2ⁿ)` functions for `n ≤ 4`.

### Stage 4 — optimum chains

Implement `full_basis` (`0..16`), `standard_basis` (`{AND, OR, NOTL}`), and the
state-BFS `optimal_cost`. Tests pin `C = 0` for projections/constants,
`C(XOR₂) = 1`, `C ≤ 1` for all 2-variable functions, `C(XOR₃) = 2`, and
`C(majority₃) = 4` — each matched against a hand-built chain of the same cost.

---

## 8. Check your understanding

1. Why does `to_dnf(f).len()` always equal `popcount(f.table)`, and what is the
   analogous formula for `to_cnf`?
2. The function `x₁ ∨ x₂` has a 3-term DNF but a 1-clause CNF. Give a function
   whose CNF is much larger than its DNF. (Hint: complement the situation.)
3. Encode NAND as a nibble in the `2a+b` convention, and check that
   `NOTL(a, a)` computes `¬a`.
4. Why is majority-of-three self-dual but AND not? Argue from the definition
   `f(¬x) = ¬f(x)`.
5. Explain in one sentence why the "reachable functions" shortcut undercounts
   `C(majority₃)`, and what the state-BFS fixes.

## 9. Exercises from the text

Ratings are Knuth's (00 immediate · 10 a minute · 20 up to an hour · 30 hours ·
40 term project · 50 open research). ▶ marks especially instructive problems.
Log attempts in `exercises.md`.

| Ex. (§7.1.1–7.1.2) | Rating | Statement (paraphrased) |
|---|---|---|
| 7.1.1–2 | 10 | How many Boolean functions of `n` variables are self-dual? Count them. |
| ▶7.1.1–5 | 20 | Show every symmetric function of `n` variables is determined by `n+1` bits; how many symmetric functions are there? |
| 7.1.1–16 | 22 | Prove `f` is monotone iff it has a DNF using only *positive* literals. |
| ▶7.1.2–1 | 15 | Exhibit an optimum (4-gate) chain for the median of three, and prove no 3-gate chain works. |
| 7.1.2–2 | 20 | Find `C(f)` for the full adder's two outputs; can they share gates? |
| ▶7.1.2–23 | 30 | Discuss why `C(f)` for a random `f` is `≈ 2ⁿ/n` (Shannon/Lupanov). |
| 7.1.1–ex (Dedekind) | 40 | Compute `M(5)`; describe the pipeline used for `M(8)` and `M(9)`. |

## In the real world

Everything here is the daily bread of **electronic design automation (EDA)**.
Logic-synthesis tools such as Berkeley's **ABC** take a Boolean specification and
minimize its gate count and depth before it is etched onto silicon — literally
running smarter versions of the stage-4 search on functions with millions of
gates, using AND-inverter graphs and cut-based rewriting because exact `C(f)` is
out of reach at scale. **FPGA** toolchains solve a cousin problem — *technology
mapping* — packing a chain into `k`-input lookup tables (LUTs), where the "cost"
is LUT count rather than raw gates. Cryptographers cost their primitives in
**gates**, especially XORs: an AES or SHA circuit's area and its resistance to
side channels are measured in the very units of §3, and lightweight ciphers are
designed to minimize `C`. When the function is small but the optimum must be
*exact*, engineers reach for **SAT-based exact synthesis** — encode "is there a
chain of `r` gates computing `f`?" as a CNF and let a SAT solver decide, deepening
`r` until it says yes. That is the industrial descendant of your state-BFS, and
it is why the CNF of §2 and the search of §6 are the same subject.

## Why it's done this way

- **Truth-table-as-integer** because a 64-bit register runs Boolean algebra on
  all `2ⁿ` rows in one instruction; every function operation collapses to a
  bitwise op, which is both faster and simpler than any tree of `Rc<RefCell>`
  nodes.
- **The nibble gate encoding** because it makes "apply gate to two truth
  tables" a handful of `&`/`|`/`!` operations, turning the optimum search's
  inner loop into straight-line bit twiddling.
- **State-BFS, not the frontier shortcut**, because circuit cost is about
  *sharing*, and only a state that carries the whole computed set forward
  models sharing correctly. Choosing the wrong graph gives a plausible,
  confidently-wrong answer — a lesson worth more than the code.

## Proof techniques you practiced

- **Proof by exhaustive evaluation** — De Morgan and `chain_computes` verify an
  identity by checking all `2ⁿ` rows; when the domain is finite, "try everything"
  is a rigorous proof, and the truth-table word makes it cheap.
- **Counting / pigeonhole** — Shannon's bound counts circuits against functions;
  the double exponential `2^(2ⁿ)` forces most functions to be hard. You met the
  same double exponential concretely in Dedekind's `M(n)`.
- **Invariant of a search** — the state-BFS is correct because each state is
  *exactly* the set of functions realizable by some chain of the given cost;
  keeping that invariant true is what distinguishes it from the broken shortcut.
- **Extremal / worst-case construction** — the 4-gate majority chain is an
  optimum *witness*; pairing an explicit chain with a lower-bound search is how
  you prove a cost is exactly right, upper and lower bound meeting.

## 10. Where this leads

- **BDDs and ZDDs** (Modules 12–13, 17) give a canonical, often-compact form for
  the same functions, where equality and the Boolean operations are graph
  operations instead of `u64` operations — the scalable successor to the truth
  table.
- **SAT and CDCL** (Modules 10, 14) run the CNF of §2 at industrial scale; exact
  synthesis feeds circuits *into* a SAT solver, closing the loop.
- **The counting bound** reappears whenever a resource is scarcer than the
  demands on it — a theme from Kolmogorov complexity to lower bounds in
  complexity theory.
- **Dedekind's problem** remains open past `n = 9`: a standing invitation to the
  intersection of combinatorics, hardware, and heroic computation.
