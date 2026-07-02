# Hints — Module 08: Combinatorial Generation

## Stage 1: Gray binary code

1. The reflected binary Gray code has a closed form `g(k) = k XOR (k >> 1)`, and the bit that flips from `g(k-1)` to `g(k)` sits at position ρ(k), the number of trailing zeros of k (the ruler function). Algorithm G reproduces exactly that flip sequence *without a counter*, using only a running parity bit.
2. Keep the whole word in one integer and maintain `a_inf`, the parity of how many words you have visited so far. On odd steps you flip bit 0; on even steps you flip the bit just left of the lowest set bit — that "position of the rightmost 1" is `a.trailing_zeros()`. For `gray_rank`, invert `g` by folding the word into itself with shifts and XORs.
3. Loop: push `a`; set `a_inf = 1 - a_inf`; pick `let j = if a_inf == 1 { 0 } else { a.trailing_zeros() + 1 };`; if `j == n` return, else `a ^= 1u64 << j`. For the rank: `while g != 0 { k ^= g; g >>= 1; }`.

## Stage 2: Permutations in lexicographic order

1. The lexicographic successor is found by the Narayana Pandita idea: the longest non-increasing suffix is already "maxed out", so bump the pivot just left of it up minimally, then set the tail to its smallest arrangement. Comparing with `>=` (not `>`) is what makes it emit each *distinct* multiset arrangement exactly once.
2. Work in one `&mut [u32]`. L2 walks an index down from the right while `a[j-1] >= a[j]` to find the pivot; return `false` unchanged if you fall off the left end. L3 finds the rightmost element still exceeding the pivot and swaps; L4 reverses the suffix (which is non-increasing, so reversing = sorting it ascending). `all_permutations` seeds `1..=n` and iterates.
3. With 1-based `j`: `while a[j-1] >= a[j] { j -= 1; if j == 0 { return false; } }`, then `let mut l = n; while a[j-1] >= a[l-1] { l -= 1; }`, `a.swap(j-1, l-1)`, `a[j..].reverse()`, return `true`. Guard `n <= 1` up front.

## Stage 3: Plain changes: adjacent transpositions

1. Steinhaus–Johnson–Trotter weaves the largest movable element back and forth (boustrophedon); every successive permutation differs by one *adjacent* transposition. The state that drives it iteratively is per-element odometer counters `c_j` (how far element j has moved, `0 <= c_j < j`) and directions `o_j = ±1`, plus an offset `s`.
2. Find the largest j with room to move by walking j down from n; compute `q = c_j + o_j`. If `q` is in range, do the swap and go back to visiting; if it hit the top (`q == j`) increase `s` (terminating when `j == 1`); either way, if it did not move, reverse that element's direction and step j down. Mind the 1-based → 0-based conversion in the swap.
3. Inner loop: `let q = c[j] + o[j]; if q >= 0 && q != j { swap a[(j - c[j] + s)-1] with a[(j - q + s)-1]; c[j] = q; break; }`; then `if q == j { if j == 1 { return; } s += 1; }`; then `o[j] = -o[j]; j -= 1;`. Return `vec![vec![]]` for n = 0.

## Stage 4: Generating combinations

1. Algorithm T visits the k-subsets of `{0,...,n-1}` in colexicographic order (sorted by largest element). The common case is O(1): just bump the smallest element `c_1` up by one; only when it collides with `c_2` do you carry. Two sentinels `c_{k+1} = n` and `c_{k+2} = 0` erase the boundary tests.
2. Store the combination in `c[1..=k]` with `c[k+1]`, `c[k+2]` as sentinels; visit `c[1..=k]` ascending. After visiting, if a saved index `j > 0` jump straight to increasing `c_j`; otherwise try the easy `c[1]+1 < c[2]` case, and failing that reset the low run and scan up for the first element with room. Handle `k == 0` and `k == n` directly since T assumes `0 < k < n`; `assert!(k <= n, ...)`.
3. T3: `if c[1] + 1 < c[2] { c[1] += 1; continue; } j = 2;`. T4: `loop { c[j-1] = j-2; let cand = c[j]+1; if cand == c[j+1] { j += 1; } else { x = cand; break; } }`. T5: `if j > k { return out; }`. T6: `c[j] = x; j -= 1;` then visit again. Right after a visit, `if j > 0 { x = j; }` and go to T6.

## Stage 5: Partitions of an integer

1. Algorithm P generates partitions in reverse-lexicographic order from `[n]` down to all-ones, keeping `a_1 >= ... >= a_m` and an index `q` pointing at the rightmost part greater than 1. `partition_count` is separate: use Euler's pentagonal-number recurrence, and `conjugate` uses "part j = number of original parts that are >= j".
2. The generation core: store the final part, then repeatedly visit; the fast path P3/P4 splits a trailing 2 into 1+1; the general path P5/P6 decreases `a_q` by one and redistributes the freed amount `rem` into copies of `x = a_q - 1`. For the count, build `p[0..=n]` bottom-up summing signed terms at the generalized pentagonal offsets `k(3k∓1)/2`; use `i128`/`u64` and `(-1)^{k+1}` signs.
3. Count loop per m: `for k = 1.. { g1 = k*(3k-1)/2; if g1 > m break; sign = if k odd {1} else {-1}; total += sign*p[m-g1]; g2 = k*(3k+1)/2; if g2 <= m { total += sign*p[m-g2]; } }`. Conjugate: `(1..=p[0]).map(|j| p.iter().take_while(|&&x| x >= j).count()).collect()`, asserting parts are positive and non-increasing (message contains "non-increasing").
