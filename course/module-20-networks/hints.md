# Hints — Module 20

Graduated hints, three per stage. Try the lesson and your own attempt first;
each hint is more specific than the last. Never peek past the one you need.

## Stage 1: Comparison lower bounds by decision trees

1. The bound is `⌈lg n!⌉` — the height of the shortest binary tree with at least
   `n!` leaves. Think about "smallest `c` with `2^c ≥ n!`", not floating-point
   logarithms (which round wrong near powers of two).
2. Build `n!` in a `u128` (it fits through `n = 34`). Then loop a power of two
   `p = 1` upward, counting steps, until `p ≥ n!`. For `sort_and_count`, any
   correct sort works; binary insertion sort makes the comparison count obvious.
3. `min_comparisons_lower_bound`: `let mut fact=1u128; for k in 2..=n {fact*=k;}
   let (mut c, mut p)=(0u32,1u128); while p<fact {p<<=1; c+=1;} c`. `is_sorted`:
   `a.windows(2).all(|w| w[0] <= w[1])`.

## Stage 2: Merge insertion (Ford–Johnson)

1. This is Algorithm 5.3.1M. Three phases: pair up and compare (larger = `a_i`,
   smaller = `b_i`), recursively sort the `a_i`, then binary-insert the `b_i`
   into the "main chain" in Jacobsthal-number order so no comparison is wasted.
2. Sort *indices* by their key value, not the values themselves — that way each
   larger element keeps a link to its smaller partner through the recursive
   sort. `b_1` (partner of the smallest `a`) goes at the front free. Insert the
   rest in groups bounded by 1, 3, 5, 11, 21, … from the top of each group down.
3. Recurse: `mi_sort(idx, key, &mut comps)`. Pair `idx[0..even_len]`, record
   `partner[hi]=lo`, recurse on the `hi` list to get the main chain, seed
   `chain = [partner[main[0]]] ++ main`, then for each 1-based index `i` in
   `jacobsthal_order(s + odd?1:0)` binary-insert `partner[main[i-1]]` into
   `chain[0 .. pos_of(main[i-1])]` (or the whole chain for the odd straggler),
   counting each comparison. `ford_johnson_sort` just discards the count.

## Stage 3: Sorting networks — Batcher's odd-even merge

1. A network is a `Vec<(usize,usize)>` of comparators; `apply_network` sends the
   smaller value to the first-listed wire (`if a[i] > a[j] { a.swap(i,j) }`).
   Build the comparator list by transcribing Batcher's recursion.
2. `oddeven_sort(lo,n)` sorts the two halves then calls `oddeven_merge(lo,n,1)`.
   `oddeven_merge(lo,n,r)` with `m=2r`: if `m<n`, merge the even and odd
   subsequences (`oddeven_merge(lo,n,m)` and `oddeven_merge(lo+r,n,m)`) then emit
   `(i, i+r)` for `i = lo+r, lo+r+m, …` while `i+r < lo+n`; else emit `(lo, lo+r)`.
3. Depth = ASAP schedule: `ready = vec![0; n]`; per comparator `(i,j)`,
   `layer = max(ready[i], ready[j]); ready[i]=ready[j]=layer+1;` track the max.
   Sanity: n=8 gives 19 comparators, depth 6; n=16 gives 63, depth 10.

## Stage 4: The zero-one principle and bitonic sorting

1. Theorem Z: a network sorts everything iff it sorts all `2^n` binary inputs.
   `sorts_all_zero_one` enumerates every bitmask `0..2^n`, builds the 0-1 array,
   applies the network, and checks it is sorted.
2. For each `mask in 0..(1<<n)`, wire `k` gets `(mask >> k) & 1`; apply the
   network; if any result is unsorted, return false. Bitonic sort is a second
   recursion: sort the first half ascending, the second half descending, then
   bitonic-merge — descending comparators are recorded as `(j, i)` with `j > i`.
3. `bitonic_sort(lo,n,asc)`: `bitonic_sort(lo,m,true); bitonic_sort(lo+m,m,false);
   bitonic_merge(lo,n,asc)`. `bitonic_merge(lo,n,asc)` with `m=n/2`: for
   `i in lo..lo+m` push `(i,i+m)` if `asc` else `(i+m,i)`, then recurse on both
   halves keeping the same direction. Top-level call: `bitonic_sort(0,n,true)`.
