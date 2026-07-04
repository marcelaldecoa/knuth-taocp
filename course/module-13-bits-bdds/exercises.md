# Exercises — Module 13 (Bitwise Tricks & BDDs)

Self-contained problems on this module's material — the rightmost-bit
identities, Gosper's hack, sideways addition and de Bruijn cycles, BDD
canonicity, model counting, the ordering problem, and ZDDs. You can work every
one **without the books**: each states the problem in full, gives a **hint** to
peek at when stuck, and a worked **answer sketch** to check against after you
try. Computational answers here are reproduced by the code you write in the lab
(or a few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Exercise numbers follow the Vol. 4A first edition; where a printing
differs, match by content.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | The four rightmost-bit identities, including $x = 0$ | 10 | ⬜ |
| 2 | ▶ Re-derive Gosper; prove G3's division exact | 20 | ⬜ |
| 3 | Per-byte counts; a $512$-bit block popcount | 22 | ⬜ |
| 4 | ▶ Count $B(2,n)$ de Bruijn cycles; both $B(2,3)$ by hand | 25 | ⬜ |
| 5 | Median under two orders; symmetric $\Rightarrow$ order-free | 15 | ⬜ |
| 6 | ▶ Canonicity for $n = 2$: all $16$ functions and diagrams | 25 | ⬜ |
| 7 | The ZDD skip lemma (factor $1$) and adapted counting | 20 | ⬜ |
| 8 | ▶ Hidden weighted bit: exponential for every order | 30 | ⬜ |

## Problems

### 1. The four rightmost-bit identities, including $x = 0$ (rating 10 · cf. 7.1.3-warmup)

**Problem.** Write a nonzero word as $x = \alpha\, 1\, 0^a$ ($a$ trailing zeros,
$\alpha$ the high bits). Prove the four identities of the lesson's table — the
binary form and effect of `x & (x-1)`, `x | (x-1)`, `x ^ (x-1)`, and `x & (-x)` —
and state what each does when $x = 0$.

**Hint.** Two facts do all the work: $x - 1 = \alpha\, 0\, 1^a$ (borrow ripples
through the trailing zeros), and $-x = \bar x + 1 = \bar\alpha\, 1\, 0^a$ (the carry
stops at the lowest $1$). Then AND/OR/XOR region by region.

**Answer sketch.** With $x = \alpha\, 1\, 0^a$ and $x - 1 = \alpha\, 0\, 1^a$:

| expression | result | effect |
|---|---|---|
| `x & (x-1)` | $\alpha\, 0\, 0^a$ | drop the rightmost $1$ |
| `x \| (x-1)` | $\alpha\, 1\, 1^a$ | smear the rightmost $1$ downward |
| `x ^ (x-1)` | $0\ldots0\, 1\, 1^a$ | mask of the rightmost $1$ and all below |
| `x & (-x)` | $0\ldots0\, 1\, 0^a$ | extract the rightmost $1$ ($= 2^a$) |

Hand-checked on $x = 01011000$ ($\alpha = 0101$, $a = 3$): drop $= 01010000$,
smear $= 01011111$, xor $= 00001111$, extract $= 00001000$ — all reproduced by a
REPL. **Boundary $x = 0$:** extraction gives $0\, \&\, (-0) = 0\,\&\,0 = 0$
(nothing to extract, a well-defined answer); the smear gives $0\,|\,(0 - 1) =
0\,|\,(2^{64}-1) = $ all ones (the borrow makes $x - 1$ the full mask). Both follow
from the wrapping mod-$2^{64}$ arithmetic, which is why Rust makes you spell out
`wrapping_neg` / `wrapping_sub`.

### 2. ▶ Re-derive Gosper; prove G3's division exact (rating 20 · cf. 7.1.3-20)

**Problem.** Derive Gosper's hack — the next-larger word of the same weight —
from the picture $x = \alpha\, 0\, 1^b\, 0^a$ (rightmost run of $b \ge 1$ ones
above $a \ge 0$ zeros). Show the successor is $\alpha\, 1\, 0^{a+1}\, 1^{b-1}$, and
prove that the division `(x ^ v) / u` in step G3 (with $u = x\,\&\,(-x)$,
$v = x + u$) is always exact.

**Hint.** To grow $x$ minimally, set the lowest $0$ that can be "paid for" — the
$0$ just above the run — then repack the leftover ones as low as possible. For the
division, ask how many trailing zeros $x \oplus v$ has.

**Answer sketch.** *Derivation.* Any larger word must set some $0$-bit; the
cheapest choice is the $0$ immediately above the rightmost run (setting a higher
$0$ costs more, a lower one gives $y < x$). Setting it turns $\alpha\, 0\, 1^b\,
0^a$ into a word with one surplus $1$ above; delete the run's $b$ ones and re-add
$b - 1$ ones packed at the very bottom, giving $\text{next} = \alpha\, 1\, 0^{a+1}\,
1^{b-1}$ — this is the minimal same-weight increase (an exchange argument: the
forced high bit is fixed, the remaining $b - 1$ ones sit as low as possible).
*The algorithm computes exactly this:* $u = x\,\&\,(-x) = 2^a$ isolates the run's
low bit; $v = x + u$ carries through the run to $\alpha\, 1\, 0^{a+b}$; and
$x \oplus v = 0\ldots0\, 1^{b+1}\, 0^a$ exposes the $b + 1$ vacated/new ones over
$a$ trailing zeros. *Exactness:* $x \oplus v$ ends in **exactly $a$** zeros, and
$u = 2^a$, so $(x \oplus v)/u$ is an integer — it right-justifies to $1^{b+1}$;
shifting right by $2$ drops the two ones already accounted for by $v$, leaving the
owed $1^{b-1}$. Traced on $x = 01011100$ ($\alpha = 010$, $b = 3$, $a = 2$):
$u = 00000100$, $v = 01100000$, $x \oplus v = 00111100$, $/u = 00001111$,
$\gg 2 = 00000011$, result $01100011$ — and enumerating this way produces all
$\binom{12}{3} = 220$ three-subsets of a $12$-set in increasing order (checked at a
REPL).

### 3. Per-byte counts; a $512$-bit block popcount (rating 22 · cf. 7.1.3-swar)

**Problem.** Adapt Algorithm S (sideways addition) to leave, in each **byte**, the
population count of that byte — i.e. stop after step S3, before the final
multiply. Then use it to compute $\nu(x)$ (total $1$-bits) of a $512$-bit block
with one multiply per $64$-bit word.

**Hint.** After S3 each byte already holds its own count (each nibble is $\le 4$,
so a byte sum is $\le 8$ and fits). The final multiply
`(x * 0x0101010101010101) >> 56` sums the eight byte-counts of one word into its
top byte.

**Answer sketch.** Steps S1–S3 unchanged:

```text
x -= (x >> 1) & 0x5555555555555555;                              // pairs
x  = (x & 0x3333333333333333) + ((x >> 2) & 0x3333333333333333); // nibbles
x  = (x + (x >> 4)) & 0x0f0f0f0f0f0f0f0f;                        // each BYTE = its popcount
```

Stopping here gives eight independent byte-counts in one word. For a $512$-bit
block = eight $64$-bit words $w_0, \ldots, w_7$: run S1–S3 on each word, then the
final multiply-and-shift per word yields that word's total $\nu(w_i)$, and summing
the eight totals gives the block popcount. (Alternatively, accumulate the eight
post-S3 byte vectors with saturating-safe adds — each byte $\le 8$, eight of them
$\le 64 < 256$, so no byte overflows — and do a single closing multiply.) Verified
at a REPL: the SWAR routine agrees with a reference popcount on $10^4$ random words,
and a random $512$-bit block totals correctly (e.g. $258$ set bits, matching a
naive bit count). About a dozen branchless instructions per word, no table, no
loop — the algorithm inside hardware `POPCNT`.

### 4. ▶ Count $B(2,n)$ de Bruijn cycles; both $B(2,3)$ by hand (rating 25 · cf. 7.1.3-debruijn)

**Problem.** A binary de Bruijn cycle $B(2, n)$ is a cyclic sequence of $2^n$ bits
in which all $2^n$ length-$n$ windows are distinct. The number of such cycles is
$2^{\,2^{n-1} - n}$ (de Bruijn 1946). Verify the formula by hand for $n = 3$ — it
predicts $2$, so exhibit *both* $B(2, 3)$ cycles — and deduce how many usable
$64$-bit `ruler` constants (a $B(2, 6)$ beginning with six $0$ bits) exist.

**Hint.** A valid `ruler` constant must start with six $0$ bits so that left-shifts
feed zeros into the low end and cleanly replay the cycle. Enumerate length-$8$
cyclic binary sequences whose eight $3$-windows are all distinct.

**Answer sketch.** For $n = 3$: $2^{\,2^{2} - 3} = 2^{4-3} = 2^1 = 2$. Enumerating
all length-$8$ cyclic sequences with eight distinct $3$-windows (up to rotation)
yields exactly **two** necklaces:

$$
00011101 \qquad\text{and}\qquad 00010111.
$$

The lesson's constant $D = \texttt{0x1D} = 00011101$ is the first; its left-shifts
produce the eight distinct top-$3$-bit windows $000, 001, 011, 111, 110, 101, 010,
100$ — a perfect hash from shift amount to window, which is exactly why one table
lookup recovers the trailing-zero count. For $n = 6$: $2^{\,2^{5} - 6} =
2^{32 - 6} = 2^{26}$ distinct $B(2, 6)$ cycles; those beginning with six $0$ bits
number $2^{26}$ as well, so there are $2^{26}$ usable $64$-bit `ruler` constants —
`0x03f79d71b4cb0a89` being one. Build the inverse table from the identity
`table[(D << a) >> 58] = a`; never copy a magic table you cannot regenerate.

### 5. Median under two orders; symmetric $\Rightarrow$ order-free (rating 15 · cf. 7.1.4-median)

**Problem.** Draw the ROBDD of the majority function $\langle x_1 x_2 x_3\rangle$
(true when $\ge 2$ inputs are true) under the order $x_1 < x_2 < x_3$ and again
under $x_3 < x_1 < x_2$. Verify $B(f) = 6$ both times, and prove the general fact:
a **symmetric** Boolean function has the same BDD size under every variable order.

**Hint.** A symmetric function's value depends only on how many inputs are $1$, not
which. A change of variable order just relabels the inputs.

**Answer sketch.** Under $x_1 < x_2 < x_3$, the reduced diagram has one $x_1$-node,
two $x_2$-nodes, one $x_3$-node, and the two sinks — $B = 6$ (the duplicate
lower nodes merge under rule R2, as the lesson's median diagram shows). A REPL
build confirms $6$ nodes and $4$ satisfying assignments ($011, 101, 110, 111$).
**Order-independence:** majority is symmetric — it equals $1$ iff the input weight
$\nu(x) \ge 2$, which is unchanged by any permutation $\pi$ of the inputs. Reordering
the variables from $x_1 x_2 x_3$ to $x_3 x_1 x_2$ is exactly applying such a
permutation; the resulting function of the reordered inputs is the *same* symmetric
function, so by the canonicity theorem its ROBDD is isomorphic to the first —
identical node counts, $B = 6$ again. In general, for any symmetric $f$ and any two
orders, the two ROBDDs are relabelings of one another, so $B(f)$ is order-invariant.
(This is the rare happy case; the next problem's cousin — and §2.5's couples
function — show how badly non-symmetric functions depend on order.)

### 6. ▶ Canonicity for $n = 2$: all $16$ functions and diagrams (rating 25 · cf. 7.1.4-canon)

**Problem.** Make the canonicity theorem concrete for $n = 2$: enumerate all $16$
Boolean functions of $x_0, x_1$ (order $x_0 < x_1$) and give each one's unique
reduced ordered BDD, confirming the node count $B(f)$. Group them by structure.

**Hint.** Build each function from its minterms and reduce with rules R1 (drop a
node with LO $=$ HI) and R2 (merge duplicate triples). A function that ignores a
variable has no node for it.

**Answer sketch.** All $16$ diagrams are distinct (that *is* the theorem for
$n = 2$), and they fall into four structural classes by node count (sinks
included), reproduced by a REPL that computes $B(f)$ for every truth table:

| class | functions | $B(f)$ |
|---|---|---|
| constants | $\bot$ ($0000$), $\top$ ($1111$) | $1$ (a bare sink) |
| single literal | $x_0,\ \lnot x_0,\ x_1,\ \lnot x_1$ | $3$ (one branch $+$ $2$ sinks) |
| two-input, non-parity | $\land,\ \lnot(\land),\ \lor,\ \lnot(\lor)$ and the four "one input negated" gates (e.g. $\lnot x_0 \land x_1$) | $4$ (two branches $+$ $2$ sinks) |
| parity | $x_0 \oplus x_1,\ \lnot(x_0 \oplus x_1)$ | $5$ (three branches $+$ $2$ sinks) |

That is $2 + 4 + 8 + 2 = 16$ functions with node counts
$[1,4,4,3,4,3,5,4,4,5,3,4,3,4,4,1]$ across truth tables $0000$–$1111$. Each row of
the canonicity induction is visible here: constants are the base case ($n = 0$);
the literals are functions where a cofactor is constant (Case 1, a variable drops
out); the parity functions are the extreme Case 2 where both cofactors differ and
depend on the deeper variable, forcing the full three-branch shape. No two of the
$16$ share a diagram, so equivalence testing is pointer comparison — exactly the
payoff the theorem promises.

### 7. The ZDD skip lemma (factor $1$) and adapted counting (rating 20 · cf. 7.1.4-count)

**Problem.** In a BDD, a skipped level contributes a factor $2^{\text{skip}}$ to
the model count, because a skipped variable is *free*. Zero-suppressed decision
diagrams (ZDDs) reinterpret a skipped variable as **absent** from the set, not
free. Prove the ZDD version of the skip lemma — a skipped level multiplies by $1$,
not $2$ — and adapt Algorithm C (counting) accordingly.

**Hint.** Compare the reduction rules. A BDD deletes nodes with LO $=$ HI (the
variable is irrelevant $\Rightarrow$ free $\Rightarrow$ two extensions). A ZDD
deletes nodes whose HI edge points to $\bot$ (the variable, if untested, is
$\Rightarrow$ absent $\Rightarrow$ one extension).

**Answer sketch.** Let $s(p)$ count the *sets* (families) accepted from node $p$
over the variables $x_{\ell(p)}, \ldots, x_{n-1}$. In a ZDD, a node deleted by the
zero-suppression rule is one whose HI child is $\bot$: taking that variable to be
$1$ leads to rejection, so **every** accepted set has that variable $= 0$ — the
variable is forced absent, not free. Hence when an edge skips from level $v$ to a
child at level $w > v + 1$, each skipped variable $x_{v+1}, \ldots, x_{w-1}$ is
pinned to $0$ (absent), contributing a factor of $1^{\,w - v - 1} = 1$ rather than
$2^{\,w - v - 1}$. The skip-weighted recurrence becomes

$$
s(\bot) = 0,\quad s(\top) = 1,\qquad s(p) = 1\cdot s(\mathrm{LO}(p)) + 1\cdot s(\mathrm{HI}(p)),
$$

i.e. **drop every $2^{\text{skip}}$ factor** — a two-line change to Algorithm C, and
no top-of-root factor either. This is exactly why ZDDs compress families of sparse
subsets: "the other $95$ of $100$ elements are absent" costs no nodes and no count
factors, whereas the BDD would pay $2^{95}$ of free-variable weight to say the same
thing. (Ground it against §2.6: same `mk`/unique-table/memoized-apply machinery,
adjusted skip factor.)

### 8. ▶ Hidden weighted bit: exponential for every order (rating 30 · cf. 7.1.4-hidden)

**Problem.** The *hidden weighted bit* function is $\mathrm{HWB}(x) = x_{\nu(x)}$
(with $\mathrm{HWB}(0) = 0$): the output is the input bit whose index equals the
input's population count. Bryant (1991) proved that **every** variable order gives
$\mathrm{HWB}$ an exponentially large BDD. Reproduce the core of the argument — the
distinguishing-set idea from §2.5 — in your own words, and explain why this is the
worst case that variable reordering cannot fix.

**Hint.** The §2.5 lower bound found $2^k$ pairwise-distinct residual functions and
concluded $\ge 2^k$ nodes. For $\mathrm{HWB}$ the point is that *no* order lets a
BDD level "forget" enough: after reading half the variables (in any order), the
count $\nu(x)$ is still undetermined, so which bit is selected is still open.

**Answer sketch.** The general lower-bound engine (§2.5): if, cutting the variable
order at some level, the function has $M$ pairwise-distinct residual subfunctions of
the not-yet-read variables, then the BDD has $\ge M$ nodes at that cut — because
the node reached by each partial assignment determines its residual, so distinct
residuals force distinct nodes (a distinguishing/fooling set, kin to
Myhill–Nerode). For $\mathrm{HWB}$, Bryant shows that for *any* order there is a cut
(around the middle) across which exponentially many assignments to the read
variables leave genuinely different residual functions: the selected index
$\nu(x)$ can still land on either side of the cut depending on the unread bits, so
the residual must "remember" both the partial popcount *and* the values of many
read bits that might yet be selected — and these memories are pairwise
distinguishable. That forces $2^{\Omega(n)}$ nodes at the cut, for every order.
Because the bound holds for *all* orders simultaneously, reordering heuristics
(sifting) cannot rescue it — $\mathrm{HWB}$ is the standard witness that some
functions have no good BDD order at all, the definitive limit on the whole
representation. (This problem is a *read-and-reproduce* of Bryant's proof; the
distinguishing-set skeleton above is the transferable core, grounded in §2.5's
$2^k$-residuals argument.)

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
