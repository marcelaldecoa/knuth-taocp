# Exercises — Module 05 (Arithmetic)

Self-contained problems on this module's material — the classical algorithms
(addition, multiplication), Karatsuba's divide-and-conquer multiply, the binary
gcd, and probabilistic primality/factoring. You can work every one **without the
books**: each states the problem in full, gives a **hint** to peek at when stuck,
and a worked **answer sketch** to check against after you try. Computational
answers here are reproduced by the code you write in the lab (or a few lines at a
REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise its section is noted for
readers who own Volume 2.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | The carry in Algorithm A never exceeds $1$ | 15 | ⬜ |
| 2 | ▶ Exact digit-multiplication count of Algorithm M | 25 | ⬜ |
| 3 | Karatsuba's constant and the empirical crossover | 30 | ⬜ |
| 4 | ▶ Binary gcd vs. Euclid on Fibonacci pairs | 28 | ⬜ |
| 5 | Mini-project: Pollard's rho factoring | 35 | ⬜ |

## Problems

### 1. The carry in Algorithm A never exceeds $1$ (rating 15 · cf. §4.3.1)

**Problem.** Numbers are stored in radix $b$ (the lab uses $b = 2^{32}$) as digit
strings with $0 \le u_j, v_j < b$. Algorithm A adds them digit by digit: at
position $j$ it forms $t = u_j + v_j + k$, writes $w_j = t \bmod b$, and carries
$k \leftarrow \lfloor t/b \rfloor$ into the next position. Prove that the carry
$k$ is always $0$ or $1$, so a single extra limb $w_n$ suffices to hold the whole
sum.

**Hint.** Bound $t$ from above in the worst case. What is the largest possible
value of $u_j + v_j + k$ if you already know $k \le 1$? Set up an induction on
$j$.

**Answer sketch.** Induct on $j$. Initially $k = 0 \le 1$. Suppose the incoming
carry satisfies $k \le 1$. Then

$$
t = u_j + v_j + k \le (b-1) + (b-1) + 1 = 2b - 1 < 2b,
$$

so the outgoing carry $\lfloor t/b \rfloor \le \lfloor (2b-1)/b \rfloor = 1$. The
bound is preserved, so $k \in \{0, 1\}$ at every step. (Check: base $10$ gives
$t \le 19 < 20$; base $2^{32}$ gives $t \le 2^{33} - 1 < 2^{33}$, both with carry
$\le 1$.) Because the final carry is at most $1$, one extra digit $w_n \in
\{0,1\}$ always holds it — this is why an $n$-digit + $n$-digit sum needs at most
$n+1$ digits, and why hardware chains a single carry line.

### 2. ▶ Exact digit-multiplication count of Algorithm M (rating 25 · cf. §4.3.1)

**Problem.** Algorithm M multiplies an $m$-digit number $u$ by an $n$-digit
number $v$ with two nested loops: for each digit $v_i$ of $v$ it runs across every
digit $u_j$ of $u$, forming the product $u_j \cdot v_i$. Count the *exact* number
of single-digit multiplications Algorithm M performs, as a function of $m$ and
$n$, and argue it does not depend on the digit *values*. What does this say about
the asymptotic cost?

**Hint.** The inner statement $t \leftarrow u_j \cdot v_i + w_{i+j} + k$ runs once
for every pair $(i, j)$. How many such pairs are there? Are any pairs skipped for
particular inputs?

**Answer sketch.** The outer loop runs over $i = 0, \dots, n-1$ ($n$ passes) and
for each, the inner loop runs over $j = 0, \dots, m-1$ ($m$ digit products), with
no early exits. So Algorithm M always performs exactly

$$
m \cdot n \text{ single-digit multiplications,}
$$

regardless of the operands' values — every digit of $u$ meets every digit of $v$
exactly once. (The additions/carries add only $O(mn)$ lower-cost operations.)
Hence "schoolbook" multiplication is $\Theta(mn)$, i.e. $\Theta(n^2)$ for
equal-size operands. This exact, value-independent count is precisely the
$\Theta(n^2)$ baseline that Karatsuba (Problem 3) must beat, and knowing it to the
digit is what lets you tune the Karatsuba cutover.

### 3. Karatsuba's constant and the empirical crossover (rating 30 · cf. §4.3.3)

**Problem.** Karatsuba multiplies two $n$-digit numbers with **three** half-size
products instead of four, giving the recurrence $T(n) = 3\,T(n/2) + O(n)$.
(a) Solve the recurrence: what is the exponent $\alpha$ in $T(n) = \Theta(n^\alpha)$,
and how many *leaf* (single-digit) products does the pure recursion make for
$n = 2^k$? (b) Explain why, despite the smaller exponent, real implementations
(and your `big_mul_karatsuba`) switch to the classical $\Theta(n^2)$ method below
a cutoff of a few dozen limbs — and describe how you would locate that crossover
empirically.

**Hint.** Unroll the recursion tree: each internal node spawns $3$ children and
the depth is $\log_2 n$. Count the leaves. For (b), compare *constants*, not just
exponents: which method has the smaller hidden constant, and at what size does
$n^{\log_2 3}$ finally undercut $n^2$ once constants are included?

**Answer sketch.** (a) Recursion depth is $k = \log_2 n$ and each level triples
the number of subproblems, so there are $3^k = 3^{\log_2 n} = n^{\log_2 3}$
leaves. By the recursion-tree (master) argument the leaves dominate the $O(n)$
merge work, giving

$$
T(n) = \Theta\!\left(n^{\log_2 3}\right) = \Theta\!\left(n^{1.585}\right),
\qquad \log_2 3 = 1.58496\ldots
$$

(Verified leaf counts for $n = 2^k$: $n{=}4 \to 9$, $n{=}8 \to 27$, $n{=}1024 \to
59049$, versus classical $n^2 = 16, 64, 1048576$ — the classical/Karatsuba ratio
grows from $1.78$ to $17.8$.) (b) Karatsuba's smaller exponent hides a *larger*
constant: three recursive calls plus several $O(n)$ additions/shifts and
recursion overhead per level. For small $n$ that overhead exceeds the $n^2$
digit-products classical multiply saves, so $n^2 \cdot c_1 < n^{1.585} \cdot c_2$
until $n$ is a few dozen limbs. To find the crossover empirically: time both
`big_mul` and `big_mul_karatsuba` on random inputs of increasing size, and set
the cutoff at the size where Karatsuba first wins consistently. Below it,
`big_mul_karatsuba` should delegate to the classical routine.

### 4. ▶ Binary gcd vs. Euclid on Fibonacci pairs (rating 28 · cf. §4.5.2)

**Problem.** Consecutive Fibonacci numbers are Euclid's worst case: every quotient
is $1$, so the algorithm makes the most division steps possible for inputs of that
size. (a) Show that Euclid's algorithm on $(F_{n+1}, F_n)$ makes exactly $n-1$
division steps, and that $\gcd(F_{n+1}, F_n) = 1$. (b) Run the binary gcd
(Stein's algorithm — shifts and subtractions only) on the same pairs and compare
the step counts. Which grows faster, and why is each still $O(\log uv)$?

**Hint.** For (a), one Euclid step on $(F_{n+1}, F_n)$ produces
$(F_n, F_{n-1})$ because $F_{n+1} \bmod F_n = F_{n-1}$; induct down to the base
case. For (b), remember $F_n$ grows like $\phi^n$, so
$\log_2 F_n \approx n \log_2 \phi \approx 0.694\,n$ — both algorithms' step counts
are linear in $n$, hence logarithmic in the operands.

**Answer sketch.** (a) Since $0 < F_{n-1} < F_n$ and $F_{n+1} = F_n + F_{n-1}$, we
have $F_{n+1} \bmod F_n = F_{n-1}$, so one division step sends $(F_{n+1}, F_n) \to
(F_n, F_{n-1})$. Iterating drives the pair down to $(F_2, F_1) = (1, 1) \to (1, 0)$,
taking exactly $n-1$ steps and leaving gcd $1$. (Verified: $(F_4, F_3) = (3,2)$
takes $2$; $(F_{11}, F_{10}) = (89, 55)$ takes $9$; $(F_{20}, F_{19}) = (6765,
4181)$ takes $18$ — always $n-1$, always gcd $1$.) (b) The binary gcd on the same
pairs makes noticeably *fewer* shift-and-subtract iterations — e.g. $9$ Euclid
divisions vs. $4$ Stein subtractions on $(89, 55)$ — because it removes factors of
$2$ in bulk and never divides. The exact count depends on the variant, but both
are $\Theta(\log uv)$: Euclid's $n-1$ and Stein's iterations are each linear in
$n$, and $\log_2(F_{n+1} F_n) \approx 1.39\,n$. The instructive point: Fibonacci
inputs make Euclid do its maximum work, yet division-free binary gcd is unbothered
— trading expensive divisions for cheap shifts is the whole idea.

### 5. Mini-project: Pollard's rho factoring (rating 35 · cf. §4.5.4)

**Problem.** Miller–Rabin tells you a number is composite but not what its factors
are. Implement **Pollard's rho** and use it to factor a 60-bit semiprime.
Pollard's rho iterates $x \leftarrow f(x)$ with $f(x) = (x^2 + c) \bmod n$,
detects the cycle with Floyd's tortoise-and-hare, and returns
$\gcd(|x_i - x_{2i}|, n)$ as a nontrivial factor. (a) Explain why this finds a
factor $p$ in expected $O(p^{1/2}) = O(n^{1/4})$ iterations. (b) Factor
$n = 1073741834516192789$ and report the two primes.

**Hint.** (a) Reduce the iteration modulo an unknown prime factor $p$ of $n$: by
the birthday paradox the sequence $x_i \bmod p$ repeats after about
$\sqrt{p}$ steps, and when $x_i \equiv x_{2i} \pmod p$ but $x_i \not\equiv x_{2i}
\pmod n$, the gcd exposes $p$. (b) Use fast modular arithmetic (`mul_mod` via
u128) from Stage 5, and re-run with a different constant $c$ if the gcd comes back
as $n$ itself.

**Answer sketch.** (a) Let $p$ be the smallest prime factor of $n$. The map
$x \mapsto (x^2 + c) \bmod p$ behaves like a random function on the $p$ residues,
so by the birthday bound the reduced sequence $\{x_i \bmod p\}$ collides after
$\Theta(\sqrt p)$ steps. Floyd's cycle finder locates such a collision using
$O(\sqrt p)$ iterations; at that point $p \mid (x_i - x_{2i})$ but typically
$n \nmid (x_i - x_{2i})$, so $\gcd(|x_i - x_{2i}|, n)$ is a proper factor. Since
$p \le \sqrt n$, the expected cost is $O(n^{1/4})$ gcd/step operations — for a
60-bit $n$ that is on the order of $n^{1/4} \approx 3.2 \times 10^4$ iterations.
(b) With $f(x) = x^2 + 1$ and $x_0 = 2$, Floyd's method exposes a factor of

$$
n = 1073741834516192789 = 1000000007 \times 1073741827
$$

in about $27{,}000$ iterations (both factors are prime, verifiable with
`is_prime_u64`) — consistent with the $O(n^{1/4})$ estimate. This is the same
$\sqrt{1}$/random-map machinery as Miller–Rabin, turned from a *detector* of
compositeness into a *producer* of factors.

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
