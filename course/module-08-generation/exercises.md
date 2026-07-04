# Exercises — Module 08 (Combinatorial Generation, §7.2.1)

Self-contained problems on this module's material — the reflected Gray code and
its rank, lexicographic permutations (Algorithm L) including multisets, plain
changes (Algorithm P), combinations in colex order (Algorithm T) and the
combinatorial number system, and integer partitions with Euler's pentagonal
recurrence. You can work every one **without the books**: each states the
problem in full, gives a **hint** to peek at when stuck, and a worked **answer
sketch** to check against after you try. Computational answers here are
reproduced by the code you write in the lab (or a few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise, its number is noted for
readers who own Volume 4A.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Reflected Gray code is cyclic (last↔first differ in one bit) | 10 | ⬜ |
| 2 | ▶ Prove the prefix-XOR inverts the Gray map (`gray_rank`) | 22 | ⬜ |
| 3 | Hand-trace Algorithm L on the multiset $\{1,2,2,3\}$ | 15 | ⬜ |
| 4 | ▶ Algorithm L visits multiset arrangements in lex order, each once | 26 | ⬜ |
| 5 | Plain changes ends at $2\,1\,3\,4\ldots n$ (Hamiltonian cycle) | 20 | ⬜ |
| 6 | Hand-trace Algorithm T for $(n,k)=(5,3)$; confirm colex order | 15 | ⬜ |
| 7 | ▶ Combinatorial number system: colex rank $=\sum\binom{c_i}{i}$ | 24 | ⬜ |
| 8 | List the 11 partitions of 6 in reverse-lex | 10 | ⬜ |
| 9 | ▶ Derive the pentagonal recurrence for $p(n)$; verify $p(50)$ | M30 | ⬜ |

## Problems

### 1. Reflected Gray code is cyclic (rating 10 · cf. 7.2.1.1–1)

**Problem.** The reflected binary Gray code $\Gamma_n$ lists all $2^n$ binary
$n$-tuples with the closed form $g(k) = k \oplus \lfloor k/2 \rfloor$ for
$k = 0, 1, \dots, 2^n - 1$ (in Rust, `k ^ (k >> 1)`). Consecutive words differ
in exactly one bit. Show the code is also *cyclic*: the last word $g(2^n - 1)$
and the first word $g(0)$ differ in exactly one bit too.

**Hint.** Compute $g(0)$ directly. For the last word, $k = 2^n - 1$ has all $n$
low bits set, so $\lfloor k/2 \rfloor$ has bits $0$ through $n-2$ set. XOR them.

**Answer sketch.** $g(0) = 0 \oplus 0 = 0$, the all-zeros word. For
$k = 2^n - 1$ (bits $0..n-1$ all $1$), $\lfloor k/2 \rfloor = 2^{n-1} - 1$
(bits $0..n-2$ all $1$). Then

$$
g(2^n-1) = (2^n - 1) \oplus (2^{n-1} - 1),
$$

and the low $n-1$ bits cancel, leaving only bit $n-1$: the word
$2^{n-1} = 100\ldots0$. It differs from $g(0) = 0$ in exactly one bit (the top
bit). Hence appending the first word after the last keeps the one-bit-change
property all the way around — $\Gamma_n$ is a Hamiltonian *cycle* on the
$n$-cube. (For $n = 3$ this is $000 \to \cdots \to 100 \to 000$; the closing
flip is bit $2$, matching the README's $\rho(8) = 3$ seam observation.)

### 2. ▶ Prove the prefix-XOR inverts the Gray map (rating 22 · cf. 7.2.1.1–7)

**Problem.** The Gray map is $g(k) = k \oplus \lfloor k/2 \rfloor$. Given a
codeword $w = g(k)$, the lab's `gray_rank` recovers $k$ by the shift-and-XOR

$$
k = w \oplus (w \gg 1) \oplus (w \gg 2) \oplus (w \gg 3) \oplus \cdots
$$

(shifting until zero). Prove this formula really is $g^{-1}$.

**Hint.** Work bit by bit from the top. Bit $n-1$ of $g(k)$ equals bit $n-1$ of
$k$ (nothing is shifted into the top). For a lower bit $i$, expand
$g(k)_i = k_i \oplus k_{i+1}$ and solve for $k_i$ using the already-recovered
$k_{i+1}$.

**Answer sketch.** Write bits with subscripts. From $g(k) = k \oplus (k \gg 1)$,
bit $i$ of the word is $w_i = k_i \oplus k_{i+1}$ (with $k_n = 0$). Solving
downward from the top: $k_{n-1} = w_{n-1}$, and $k_i = w_i \oplus k_{i+1}$ for
each lower $i$. Unrolling this recurrence,

$$
k_i = w_i \oplus w_{i+1} \oplus w_{i+2} \oplus \cdots \oplus w_{n-1}
= \bigoplus_{j \ge i} w_j.
$$

That is precisely bit $i$ of $w \oplus (w \gg 1) \oplus (w \gg 2) \oplus \cdots$:
the term $w \gg t$ contributes $w_{i+t}$ to bit $i$, so summing over all shifts
$t \ge 0$ gives $\bigoplus_{j \ge i} w_j$. Since this holds for every bit, the
prefix-XOR equals $k$, so it inverts $g$. The shifting terminates once
$w \gg t = 0$, i.e. after at most $n$ terms. $\blacksquare$ (Round-trips both
directions are exactly what Stage 1 checks.)

### 3. Hand-trace Algorithm L on the multiset $\{1,2,2,3\}$ (rating 15 · cf. 7.2.1.2–1)

**Problem.** Algorithm L generates the next arrangement in lexicographic order:
L2 finds the pivot $j$ (walk left from the right while $a_j \ge a_{j+1}$); L3
swaps $a_j$ with the rightmost $a_l > a_j$; L4 reverses the suffix after $j$.
Starting from the sorted multiset `1 2 2 3`, list every distinct arrangement it
produces, in order. How many are there, and why?

**Hint.** The $\ge$ comparisons (not $>$) make equal elements behave as an
already-maximal run, so each *distinct* arrangement appears once. The count is
the multinomial $4! / 2!$ because the two $2$s are indistinguishable.

**Answer sketch.** The distinct arrangements, in increasing lexicographic order,
are the $4!/2! = 12$ strings

$$
1223,\ 1232,\ 1322,\ 2123,\ 2132,\ 2213,\ 2231,\ 2312,\ 2321,\ 3122,\ 3212,\ 3221.
$$

Algorithm L produces exactly this sequence: e.g. from `1223`, L2 stops at
$j = 3$ (value $2$, since $a_3 = 2 < a_4 = 3$ is ascending), L3 swaps the $2$
and $3$ giving `1232`, and L4 reverses the length-1 tail (no change). It
terminates at `3221`, the non-increasing (greatest) arrangement, when L2 walks
all the way to $j = 0$. Twelve arrangements, each once.

### 4. ▶ Algorithm L visits multiset arrangements in lex order, each once (rating 26 · cf. 7.2.1.2–11)

**Problem.** Prove: started from the sorted multiset and iterated until L2
terminates, Algorithm L visits every distinct arrangement of the multiset
exactly once, in increasing lexicographic order — where L2 and L3 use $\ge$ (not
$>$) comparisons.

**Hint.** Show two things: (a) each step produces a string *strictly greater*
than the current one, and (b) it produces the *least* string strictly greater —
so nothing between is skipped. Together these give a strictly increasing walk
with no gaps; termination at the greatest arrangement finishes it.

**Answer sketch.** Let the current arrangement be $a = a_1\ldots a_n$. L2 finds
the longest non-increasing suffix $a_{j+1} \ge \cdots \ge a_n$ and sets $j$ just
before it. Because that suffix is non-increasing, no rearrangement of it alone
can increase the string; to get a larger string we must raise position $j$.

*(a) Strictly greater.* L3 swaps $a_j$ with the rightmost suffix element
$a_l > a_j$. Since the suffix is non-increasing, $a_l$ is the **smallest**
element of the suffix that exceeds $a_j$, so the new $a_j$ is larger than the
old — the string strictly increases at position $j$, hence overall.

*(b) Least such.* After the swap the suffix is still non-increasing (we removed
one value and inserted a smaller-or-equal one in sorted position), so L4
reversing it makes it *increasing* — the lexicographically smallest possible
tail for the raised prefix. Raising $a_j$ by the minimum amount and then
minimizing the tail yields exactly the least arrangement strictly greater than
$a$.

Strictly increasing $\Rightarrow$ no arrangement repeats. "Least greater"
$\Rightarrow$ no arrangement is skipped. The use of $\ge$ (not $>$) is what
makes swaps of equal elements never count as an increase, so two orderings
differing only by exchanging equal elements are never both produced. L2
terminates ($j = 0$) exactly at the non-increasing arrangement — the
lexicographically greatest — so all distinct arrangements have been seen. For a
multiset with multiplicities $m_1, \dots, m_r$ the total is the multinomial
$n! / (m_1! \cdots m_r!)$; e.g. $\{1,2,2,3\}$ gives $4!/2! = 12$ (Problem 3).
$\blacksquare$

### 5. Plain changes ends at $2\,1\,3\,4\ldots n$ (rating 20 · cf. 7.2.1.2)

**Problem.** Plain changes (Steinhaus–Johnson–Trotter, the module's Algorithm P)
lists all $n!$ permutations so that consecutive ones differ by a single adjacent
transposition. Starting from $1\,2\,3\ldots n$, show it terminates at
$2\,1\,3\,4\ldots n$, and that this last permutation is itself one adjacent
transposition away from the start — so the whole listing is a Hamiltonian
*cycle* on the permutations.

**Hint.** Use the recursive "weave" picture: element $n$ slides back and forth
across the block of the smaller elements' permutations, one step per visit. The
terminal permutation of $1\ldots n$ is determined by the terminal of $1\ldots(n-1)$
with $n$ parked at one end.

**Answer sketch.** Induct on $n$. *Base* $n = 2$: the list is $1\,2,\ 2\,1$; the
terminal is $2\,1$, and it is one adjacent swap from $1\,2$. *Step:* assume the
plain-changes listing of $1\ldots(n-1)$ starts at $1\,2\ldots(n-1)$ and ends at
$2\,1\,3\,4\ldots(n-1)$. In the weave, $n$ starts at the far right and sweeps
left/right through each successive permutation of the smaller block; across the
whole run it makes an *even* number of full sweeps back so that after the last
smaller-block permutation $2\,1\,3\ldots(n-1)$ it has returned to the far-right
position. Hence the final permutation is $2\,1\,3\,4\ldots(n-1)\,n =
2\,1\,3\,4\ldots n$. This matches the README's stated terminal and is what Stage
3 checks. Finally $2\,1\,3\,4\ldots n$ differs from the start $1\,2\,3\ldots n$
in exactly the adjacent pair $(1,2)$ — a single adjacent transposition — so
closing the list makes it a Hamiltonian cycle. (Direct check $n = 3$: the
sequence is $123, 132, 312, 321, 231, 213$, ending at $2\,1\,3$, one swap from
$1\,2\,3$.) $\blacksquare$

### 6. Hand-trace Algorithm T for $(n,k) = (5,3)$ (rating 15 · cf. 7.2.1.3–1)

**Problem.** Algorithm T lists the $\binom{n}{k}$ combinations (as sets
$\{c_1 < \cdots < c_k\}$ of $\{0,\dots,n-1\}$) in **colexicographic** order:
compare the reversed strings $c_k \ldots c_1$, i.e. group primarily by the
*largest* element. List all $\binom{5}{3} = 10$ combinations for $(n,k)=(5,3)$
in the order T produces, and confirm the largest element is non-decreasing down
the list.

**Hint.** Colex sorts first by $c_3$ (the max), then by $c_2$, then $c_1$. Start
with $\{0,1,2\}$ and enumerate all sets with max $= 2$, then max $= 3$, then
max $= 4$.

**Answer sketch.** In colex order the ten combinations are

$$
\{0,1,2\},\ \{0,1,3\},\ \{0,2,3\},\ \{1,2,3\},\ \{0,1,4\},\ \{0,2,4\},\
\{1,2,4\},\ \{0,3,4\},\ \{1,3,4\},\ \{2,3,4\}.
$$

The maxima run $2,3,3,3,4,4,4,4,4,4$ — non-decreasing, as colex requires; within
each max-group the sets are in colex order of the remaining elements.
$\binom{5}{3} = 10$ matches the list length. (This is exactly the Stage 4 colex
output, and note the first four entries — those using only $\{0,1,2,3\}$ — are a
*prefix* equal to the full $(4,3)$ list, the colex "prefix stability" property.)

### 7. ▶ Combinatorial number system (rating 24 · cf. 7.2.1.3–4)

**Problem.** Prove that in colex order the rank (0-based position) of the
combination $\{c_1 < c_2 < \cdots < c_k\}$ of $\{0,\dots,n-1\}$ is

$$
\operatorname{rank}(\{c_1,\dots,c_k\}) = \binom{c_1}{1} + \binom{c_2}{2}
+ \cdots + \binom{c_k}{k}.
$$

Verify it on two entries of the $(5,3)$ list from Problem 6.

**Hint.** Count how many $k$-combinations come *before* a given one in colex
order. A combination precedes $\{c_1,\dots,c_k\}$ iff at the highest index where
they differ, its element is smaller. Condition on the largest element: how many
$k$-subsets have their maximum $< c_k$?

**Answer sketch.** The rank is the number of $k$-combinations that are colex-less
than $\{c_1,\dots,c_k\}$. Split them by their largest element. Those with
maximum strictly below $c_k$ can use any $k$ elements of $\{0,\dots,c_k-1\}$, and
there are $\binom{c_k}{k}$ of these — all colex-smaller (smaller top element
dominates the colex comparison). The remaining smaller combinations share the
same top element $c_k$ and are colex-less among their lower $k-1$ elements; by
the same argument applied recursively to $\{c_1,\dots,c_{k-1}\}$ (a
$(k-1)$-combination of $\{0,\dots,c_k-1\}$) they number
$\binom{c_1}{1} + \cdots + \binom{c_{k-1}}{k-1}$. Adding the two pieces and
unrolling the recursion gives $\sum_{i=1}^{k} \binom{c_i}{i}$. This is the
*combinatorial number system*: every integer $0 \le N < \binom{n}{k}$ has a
unique such representation with $c_1 < \cdots < c_k$.

*Verification.* For $(5,3)$: $\{0,1,2\}$ has rank
$\binom{0}{1}+\binom{1}{2}+\binom{2}{3} = 0+0+0 = 0$ (first). $\{1,3,4\}$ has
rank $\binom{1}{1}+\binom{3}{2}+\binom{4}{3} = 1+3+4 = 8$, and indeed $\{1,3,4\}$
is the 9th entry (index $8$) of the Problem 6 list. Every entry's formula-rank
equals its position $0,1,\dots,9$. $\blacksquare$

### 8. List the 11 partitions of 6 in reverse-lex (rating 10 · cf. 7.2.1.4–1)

**Problem.** A partition of $n$ writes $n$ as a sum of positive parts in
non-increasing order; $p(6) = 11$. The module's Algorithm P generates partitions
from $[n]$ down to $[1,1,\dots,1]$ in reverse-lexicographic order (each partition
lexicographically less than the one before). List all 11 partitions of 6 in that
order.

**Hint.** Start with the single part `6`, then repeatedly decrease the rightmost
part greater than $1$ and redistribute. Reverse-lex means `6` first,
`1+1+1+1+1+1` last.

**Answer sketch.** In the order Algorithm P produces (reverse-lex):

$$
6,\ 5{+}1,\ 4{+}2,\ 4{+}1{+}1,\ 3{+}3,\ 3{+}2{+}1,\ 3{+}1{+}1{+}1,\
2{+}2{+}2,\ 2{+}2{+}1{+}1,\ 2{+}1{+}1{+}1{+}1,\ 1{+}1{+}1{+}1{+}1{+}1.
$$

That is $11 = p(6)$ partitions, each lexicographically less than its
predecessor. (This matches `partitions(6)` in Stage 5.)

### 9. ▶ Derive the pentagonal recurrence and verify $p(50)$ (rating M30 · cf. 7.2.1.4)

**Problem.** From Euler's generating function
$\sum_{n \ge 0} p(n) x^n = \prod_{k \ge 1} (1 - x^k)^{-1}$ and the pentagonal
number theorem
$\prod_{k \ge 1}(1 - x^k) = \sum_{j=-\infty}^{\infty} (-1)^j x^{j(3j-1)/2}$,
derive the recurrence

$$
p(n) = \sum_{k \ge 1} (-1)^{k+1}\left[p\!\left(n - \tfrac{k(3k-1)}{2}\right)
+ p\!\left(n - \tfrac{k(3k+1)}{2}\right)\right],
$$

with $p(0)=1$ and $p(m)=0$ for $m<0$. Then use it to verify $p(50) = 204226$.

**Hint.** The two products are reciprocals, so their product is $1$. Multiply
$\big(\sum_n p(n)x^n\big)\big(\sum_j (-1)^j x^{g_j}\big) = 1$ and match the
coefficient of $x^n$ for $n \ge 1$ (which must be $0$). Split the $j$ sum into
$j \ge 1$ (exponent $\frac{k(3k-1)}{2}$) and $j \le -1$ (exponent
$\frac{k(3k+1)}{2}$, with $k = -j$).

**Answer sketch.** Because $\prod(1-x^k)^{-1}$ and $\prod(1-x^k)$ are reciprocal,

$$
\Big(\sum_{n \ge 0} p(n) x^n\Big)\Big(\sum_{j=-\infty}^{\infty}(-1)^j
x^{g_j}\Big) = 1, \qquad g_j = \frac{j(3j-1)}{2}.
$$

The coefficient of $x^0$ on the left is $p(0)\cdot 1 = 1$ (matching the RHS), and
for every $n \ge 1$ the coefficient of $x^n$ must vanish:

$$
\sum_{j=-\infty}^{\infty} (-1)^j\, p(n - g_j) = 0.
$$

Peel off the $j=0$ term ($g_0 = 0$, sign $+1$), giving $p(n)$, and move the rest
to the other side. Writing the positive index as $k = j \ge 1$ (exponent
$g_k = \frac{k(3k-1)}{2}$) and the negative index as $k = -j \ge 1$ (exponent
$g_{-k} = \frac{k(3k+1)}{2}$), and noting $(-1)^j = (-1)^k$ so
$-(-1)^k = (-1)^{k+1}$:

$$
p(n) = \sum_{k \ge 1} (-1)^{k+1}\left[p\!\left(n - \tfrac{k(3k-1)}{2}\right)
+ p\!\left(n - \tfrac{k(3k+1)}{2}\right)\right].
$$

The generalized pentagonal exponents are $1, 2, 5, 7, 12, 15, \dots$, and only
$O(\sqrt{n})$ of them are $\le n$, so each $p(n)$ costs $O(\sqrt{n})$.

*Verification.* Tabulating with $p(0)=1$: $p(1)=1$, $p(2)=2$, $p(3)=3$,
$p(4)=5$, $p(5)=7$, $p(6)=11$ (Problem 8), and iterating the recurrence up to
$n = 50$ gives $p(50) = 204226$ — the headline count Stage 5 checks (the same
routine yields $p(100) = 190{,}569{,}292$). $\blacksquare$

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
