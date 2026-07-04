# Exercises — Module 20 (Optimum Sorting and Sorting Networks)

Self-contained problems on this module's material — the information-theoretic
lower bound $\lceil \lg n! \rceil$, Ford–Johnson merge insertion and its count
$F(n)$, sorting networks, and the zero-one principle. You can work every one
**without the books**: each states the problem in full, gives a **hint** to peek
at when stuck, and a worked **answer sketch** to check against after you try.
Numeric answers here are reproduced by the code you write in the lab (or a few
lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise, its number in §5.3 is
noted for readers who own Volume 3.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | $\lceil \lg n! \rceil$ vs. the binary-insertion count $\sum \lceil \lg k \rceil$ | 10 | ⬜ |
| 2 | ▶ Prove merge insertion costs $F(n) = \sum \lceil \lg(3k/4) \rceil$ | 22 | ⬜ |
| 3 | Hand-trace merge insertion at $n = 11$ (26 comparisons) | 20 | ⬜ |
| 4 | Where does $F(n) = \lceil \lg n! \rceil$ fail, for $n \le 20$? | 24 | ⬜ |
| 5 | ▶ Prove the zero-one principle (Theorem Z) | 20 | ⬜ |
| 6 | Batcher's odd–even merge correctness via the 0-1 principle | 22 | ⬜ |
| 7 | ▶ Depth $\ge \lceil \lg n \rceil$ and comparators $\ge \lceil \lg n! \rceil$ | 28 | ⬜ |
| 8 | Minimum-comparator networks for $n \le 10$ | 30 | ⬜ |

## Problems

### 1. $\lceil \lg n! \rceil$ vs. the binary-insertion count $\sum \lceil \lg k \rceil$ (rating 10 · cf. 5.3.1–2)

**Problem.** Binary insertion sort inserts the $k$-th element into a sorted run
of $k-1$ elements using a binary search, which costs $\lceil \lg k \rceil$
comparisons. So its worst case is $B(n) = \sum_{k=1}^{n} \lceil \lg k \rceil$.
One might guess this equals the information-theoretic lower bound
$\lceil \lg n! \rceil$. Compute both for $n \le 8$, decide whether the identity
holds, and reconcile the two quantities.

**Hint.** Compare the two tables term by term. For the inequality direction, use
$\lceil \lg(ab) \rceil \le \lceil \lg a \rceil + \lceil \lg b \rceil$, then take
$a \cdot b \cdots = n!$.

**Answer sketch.** The values are

| $n$ | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|
| $\lceil \lg n! \rceil$ | 0 | 1 | 3 | 5 | 7 | 10 | 13 | 16 |
| $B(n) = \sum \lceil \lg k \rceil$ | 0 | 1 | 3 | 5 | 8 | 11 | 14 | 17 |

They agree for $n \le 4$ but **diverge from $n = 5$ on** ($B(5) = 8 > 7$), so the
identity is *false*. What is always true is $B(n) \ge \lceil \lg n! \rceil$:
since $\lceil \lg(ab) \rceil \le \lceil \lg a \rceil + \lceil \lg b \rceil$,
$$\lceil \lg n! \rceil = \Big\lceil \lg \textstyle\prod_{k=1}^n k \Big\rceil
\le \sum_{k=1}^{n} \lceil \lg k \rceil = B(n).$$
So binary insertion is *never below* the lower bound and matches it only when no
comparison is "wasted." A comparison is wasted exactly when it searches a range
whose size is not one less than a power of two — this is the very inefficiency
merge insertion (Problems 2–3) is engineered to avoid. (A closed form falls out
of the sum: $B(n) = n\lceil \lg n \rceil - 2^{\lceil \lg n \rceil} + 1$, which
you can check against the table.)

### 2. ▶ Prove merge insertion costs $F(n) = \sum \lceil \lg(3k/4) \rceil$ (rating M22 · cf. 5.3.1–3)

**Problem.** Algorithm M (Ford–Johnson merge insertion) sorts $n$ elements. Show
that its worst-case comparison count is
$$F(n) = \sum_{k=1}^{n} \left\lceil \lg \tfrac{3k}{4} \right\rceil,$$
equivalently $F(n) = F(n-1) + \lceil \lg(3n/4) \rceil$ with $F(1) = 0$.

**Hint.** Track one increment. When the algorithm has already placed the first
$k-1$ elements of the main chain and inserts the next partner $b_i$, how large is
the range it binary-searches, and why is that range's size always $2^j - 1$ for
the current Jacobsthal group $j$? Sum the per-insertion costs $j$ over the
grouping.

**Answer sketch.** The recurrence follows from the module's "budget" accounting.
The Jacobsthal grouping (group boundaries $t_j = (2^{j+1} + (-1)^j)/3 = 1, 3, 5,
11, 21, \dots$, inserting each group from its top index down) is chosen so that
**every insertion in group $j$ binary-searches a range of size $2^j - 1$**, which
costs exactly $\lceil \lg 2^j \rceil = j$ comparisons — none wasted. Charging the
$k$-th element folded into the chain the cost of the group it lands in gives the
increment $F(k) - F(k-1) = \lceil \lg(3k/4) \rceil$; the $3k/4$ inside the
ceiling is exactly the fraction that turns the Jacobsthal group index into a
per-$k$ formula. Summing telescopes to $F(n) = \sum_{k=1}^n \lceil \lg(3k/4)
\rceil$. A quick numeric check of the increments confirms the shape: the added
cost for $k = 1,\dots,11$ is $0,1,2,2,2,3,3,3,3,3,4$, giving running totals
$0,1,3,5,7,10,13,16,19,22,26$ — precisely the $F(n)$ table in the lesson. (The
full proof that the grouping *achieves* these range sizes for every $k$ is
Algorithm M's invariant; the instructive step is seeing why $2^j - 1$ is the
"perfect fit" that makes $\lceil \lg(m+1) \rceil$ tight.)

### 3. Hand-trace merge insertion at $n = 11$ (rating 20 · cf. 5.3.1–4)

**Problem.** Carry out Algorithm M by hand on 11 elements and confirm it uses
exactly $F(11) = 26$ comparisons in the worst case, matching
$\lceil \lg 11! \rceil = 26$.

**Hint.** Break the 26 into its phases: pairing comparisons, the recursive sort
of the larger halves, and the group-by-group binary insertions. Use the
per-element increments from Problem 2.

**Answer sketch.** Pair the 11 elements into $\lfloor 11/2 \rfloor = 5$ pairs
with one left over; comparing each pair costs **5** comparisons and yields larger
elements $a_1,\dots,a_5$ and their smaller partners $b_1,\dots,b_5$ (plus the
leftover). Recursively merge-insertion-sort the 5 larger elements: that costs
$F(5) = 7$. The main chain is $b_1 a_1 a_2 a_3 a_4 a_5$ — $b_1$ sits in front for
free. Now binary-insert the remaining smaller elements $b_2, b_3, b_4, b_5$ and
the leftover in Jacobsthal order (groups $(b_3, b_2)$, then $(b_5, b_4)$, with
the leftover folded in at its group), each insertion hitting a range of size
$2^j - 1$. Adding the insertion costs to $5 + 7$ reaches **26** total. Since
$\lceil \lg 11! \rceil = 26$ as well, merge insertion is **comparison-optimal at
$n = 11$**: $F(11) = S(11) = \lceil \lg 11! \rceil = 26$. (Contrast $n = 12$,
where $F(12) = 30$ but $\lceil \lg 12! \rceil = 29$ — the first size where the
counting bound is provably loose.)

### 4. Where does $F(n) = \lceil \lg n! \rceil$ fail, for $n \le 20$? (rating 24 · cf. 5.3.1–8)

**Problem.** Merge insertion meets the lower bound for small $n$. Tabulate
$F(n)$ and $\lceil \lg n! \rceil$ for $1 \le n \le 20$, identify the first $n$
where they differ, and explain why a *loose* lower bound at that $n$ does not
contradict the lower-bound theorem.

**Hint.** Compute both columns. Watch for the first strict inequality; then
recall what the theorem of §1 actually asserts (a floor on *every* algorithm, not
the exact truth).

**Answer sketch.** The two columns agree for $n \le 11$, **first differ at
$n = 12$** ($F(12) = 30$ vs. $\lceil \lg 12! \rceil = 29$), stay apart through
$n = 19$, and **coincide again at $n = 20$ and $n = 21$**:

| $n$ | 11 | 12 | 13 | 14 | … | 19 | 20 | 21 |
|---|---|---|---|---|---|---|---|---|
| $\lceil \lg n! \rceil$ | 26 | 29 | 33 | 37 | … | 57 | 62 | 66 |
| $F(n)$ | 26 | 30 | 34 | 38 | … | 58 | 62 | 66 |

No contradiction: the theorem says every comparison sort needs **at least**
$\lceil \lg n! \rceil$ comparisons — a *lower* bound. The true minimum $S(n)$ can
exceed it, and at $n = 12$ it does ($S(12) = 30$, achieved by merge insertion).
A binary tree of height 29 has $2^{29} \approx 5.4 \times 10^8$ leaf-slots,
comfortably more than $12! = 4.8 \times 10^8$, yet those leaves cannot be *packed*
into a consistent comparison tree — the looseness is a real feature of the
problem's geometry, not an arithmetic slack. (The re-coincidence at $n = 20, 21$
shows the gap is not monotone; $S(n)$ is known only for small $n$ and is an open
problem in general.)

### 5. ▶ Prove the zero-one principle (Theorem Z) (rating M20 · cf. 5.3.4–1)

**Problem.** Prove Theorem Z: a comparator network sorts **all** inputs (from any
ordered set) if and only if it sorts all $2^n$ inputs consisting only of 0s
and 1s.

**Hint.** "Only if" is immediate. For "if," suppose the network fails on some
input $x$ with unsorted output $y$; build a monotone threshold map $f$ that turns
$y$'s inversion into a 0-1 counterexample. You need the fact that a single
comparator commutes with any nondecreasing $f$.

**Answer sketch.** ($\Rightarrow$) Trivial: 0-1 inputs are among all inputs.
($\Leftarrow$) The crux is that **comparators commute with monotone maps**: a
comparator outputs $\min(a,b)$ and $\max(a,b)$, and any nondecreasing $f$
satisfies $f(\min(a,b)) = \min(f(a),f(b))$ and $f(\max(a,b)) = \max(f(a),f(b))$;
applying this comparator by comparator, running the network on
$f(x) = (f(x_1),\dots,f(x_n))$ yields $f(y)$, where $y$ is the network's output
on $x$. Now suppose the network fails on some $x$, so its output $y$ has an
inversion $y_k > y_{k+1}$. Take the threshold function
$$f(v) = \begin{cases} 0 & v < y_k,\\ 1 & v \ge y_k, \end{cases}$$
which is nondecreasing. Then the network on the 0-1 input $f(x)$ outputs $f(y)$,
which has $f(y_k) = 1$ immediately before $f(y_{k+1}) = 0$ — unsorted. So $f(x)$
is a 0-1 input the network fails to sort, contradicting the hypothesis. Hence the
network sorts every input. $\blacksquare$ (This is a *reduction by a well-chosen
map*: a bad general input is compressed to a bad 0-1 input.)

### 6. Batcher's odd–even merge correctness via the 0-1 principle (rating 22 · cf. 5.3.4–6)

**Problem.** Use the zero-one principle to prove Batcher's odd–even *merge*
sorts. It suffices to show the merge combines two already-sorted 0-1 halves into
one sorted 0-1 sequence. Then conclude the full network (recursive
sort-halves-then-merge) sorts.

**Hint.** A sorted 0-1 sequence is a block of 0s then a block of 1s, described by
one number: its count of 1s. Track how the counts of 1s in the even-indexed and
odd-indexed subsequences of a sorted half relate, and what the final "cleanup"
comparator sweep fixes.

**Answer sketch.** By Theorem Z (Problem 5) we need only handle 0-1 inputs. A
sorted 0-1 half of length $m$ is fully described by its number of 1s. Split each
half into even- and odd-indexed subsequences; the number of 1s in the even
subsequence and in the odd subsequence of a sorted 0-1 half differ by at most 1,
so after recursively merging the evens together and the odds together, the two
merged streams are each sorted (0s then 1s) and their 1-counts differ by at most
a small bounded amount. The interleaved result is therefore sorted **except**
possibly for a single adjacent out-of-order pair, which is exactly what the final
sweep of neighbour comparators corrects — leaving one sorted 0-1 sequence. Since
the merge sorts all 0-1 inputs, by Theorem Z it merges all inputs. The full
network sorts each half (by induction) and then merges, so it sorts every 0-1
input and hence — again by Theorem Z — every input. This is why the module proves
correctness over 0-1 sequences rather than unwinding the $n!$-way recursion: the
counting argument on 1-counts is the whole content. (You verify it concretely in
Stage 4 by enumerating all $2^n$ binary strings with `sorts_all_zero_one`.)

### 7. ▶ Depth $\ge \lceil \lg n \rceil$ and comparators $\ge \lceil \lg n! \rceil$ (rating M28 · cf. 5.3.4–10)

**Problem.** Prove two lower bounds for any sorting network on $n$ wires: (a) its
**depth** is at least $\lceil \lg n \rceil$; (b) its **number of comparators** is
at least $\lceil \lg n! \rceil$.

**Hint.** For (a), bound how many input wires can influence a given output wire
after $d$ parallel steps. For (b), view the whole network as a comparison sort
and count the outcomes it can distinguish — each comparator is one yes/no branch.

**Answer sketch.** **(a) Depth.** Say a comparator step "grows" the set of input
wires an output can depend on. Before any step, each output wire depends on the 1
input on that wire. A parallel step joins the dependency sets of the two wires of
each comparator, so it can at most **double** the size of any wire's dependency
set. After $d$ steps a wire depends on at most $2^d$ inputs. To sort, the wire
holding the minimum must be able to depend on **all $n$** inputs (any of them
could be the smallest), so $2^d \ge n$, giving $d \ge \lceil \lg n \rceil$.
**(b) Comparators.** A network with $C$ comparators is a comparison sort: at each
comparator the two values either swap or not — a binary outcome. Tracing a single
input through the network selects one of at most $2^C$ swap/no-swap patterns, and
inputs that are ordered differently must follow distinguishable patterns to be
sorted correctly. So the network distinguishes at most $2^C$ outcomes but must
handle $n!$ orderings: $2^C \ge n!$, hence $C \ge \lceil \lg n! \rceil$. This is
the **same counting argument** as the comparison-sort lower bound of §1, applied
to comparators. Both bounds are often loose (see Problem 8): Batcher's network on
$n = 8$ uses 19 comparators against a bound of $\lceil \lg 8! \rceil = 16$, and
depth 6 against $\lceil \lg 8 \rceil = 3$.

### 8. Minimum-comparator networks for $n \le 10$ (rating 30 · cf. 5.3.4–38)

**Problem.** Let $\hat S(n)$ be the fewest comparators in any sorting network on
$n$ wires. Using the counting lower bound $\hat S(n) \ge \lceil \lg n! \rceil$
(Problem 7b) and Batcher's construction as an upper bound, investigate $\hat
S(n)$ for $n \le 10$: how tight is the lower bound, and what are the established
optimal values?

**Hint.** Tabulate the counting bound $\lceil \lg n! \rceil$ and Batcher's counts
where they apply, then compare to the known optima. Ask the same question the
lesson asks about $S(n)$: is the cheap bound the truth?

**Answer sketch.** The counting bound $\lceil \lg n! \rceil$ gives $0,1,3,5,7,10,
13,16,19,22$ for $n = 1,\dots,10$, but the **true minima** — established by
decades of construction and, for $n = 9, 10$, exhaustive computer search — are
$$\hat S(n) = 0,\,1,\,3,\,5,\,9,\,12,\,16,\,19,\,25,\,29 \quad (n = 1,\dots,10).$$
So the bound is **tight only for $n \le 4$** and grows loose afterward (gaps
$0,0,0,0,2,2,3,3,6,7$) — the very same "cheap bound vs. hard truth" phenomenon
the lesson highlights for $S(n)$ at $n = 12$. Batcher's odd–even network hits the
optimum at the powers of two in range ($n = 2$: 1; $n = 4$: 5; $n = 8$: 19), so
it is not just asymptotically good but *exactly* optimal for those small $n$;
non-power-of-two optima like $\hat S(5) = 9$ and $\hat S(9) = 25$ require
hand-crafted constructions. The exact values of $\hat S(n)$ are known only for
small $n$ and remain a research frontier for larger $n$ — a comparator-count
analogue of the open problem of computing $S(n)$. (These optima are verifiable
published results; the module derives the *lower bound* and one *upper bound*
here, and the gap between them is the point of the exercise.)

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
