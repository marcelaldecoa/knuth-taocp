# Hints — Module 15: External Sorting

## Stage 1: Replacement selection: runs twice the memory

1. Replacement selection (Algorithm 5.4.1R) produces initial runs whose *expected* length is 2P on random input — the snow-plow argument of E. F. Moore. The mechanism: keep P records in a priority structure ordered by `(run number, key)`; an arrival smaller than the last key output cannot extend the current run, so it is "frozen" (tagged with the next run number) and sleeps in memory until the run ends.
2. A `BinaryHeap<Reverse<(usize, i64)>>` is a perfectly good selection structure — `Reverse` turns Rust's max-heap into a min-heap on `(RN, KEY)`. Fill it with the first P records (all tag 0), then repeatedly pop the smallest, emit its key, read one new record, and decide its tag by comparing against the key you just output.
3. Pop `Reverse((rn, key))`: if `rn != current_rn`, close the current run and start run `rn`; push `key` onto the current run; then if input remains, read `x`, and `push(Reverse((if x < key { rn+1 } else { rn }, x)))`. When the heap empties, push the final run. Panic (message containing "at least one") if `p == 0`; return no runs for empty input.

## Stage 2: k-way merging with a tree of losers

1. Merging k runs, a heap pays up to *two* comparisons per level per record (it compares against both children); a *tree of losers* pays exactly one — each internal node stores the loser of the match played there, which is precisely the only opponent a climbing candidate can meet, so one leaf-to-root walk of ⌈lg k⌉ comparisons restores the tournament. That is how merging reaches the information-theoretic rate `n·⌈lg k⌉ + k`.
2. Pad k to `kk = k.next_power_of_two()` so every leaf sits at depth exactly ⌈lg k⌉. Internal nodes occupy array slots `1..kk`; leaf `j` is conceptual slot `kk + j`; parent of `i` is `i/2`. Represent an exhausted or padding front as +∞ (an `Option<i64>` whose `None` loses to everything) so end-of-run needs no special case. Build the initial tournament bottom-up, then pop-and-replay.
3. `beats(a,b)`: `None` loses; `Some` beats `None`; two `Some` values compare (`x <= y`) and *this* is the only place you increment the comparison counter — +∞ checks are free. `play(node)` recursively finds the winner and records the loser at `loser[node]`; `loser[0]` holds the champion. `pop`: output `key(loser[0])`, advance its run, then climb from `(kk+w)/2` to the root swapping in the current loser whenever it `beats` the candidate.

## Stage 3: Polyphase merge: Fibonacci on tape

1. Polyphase merging keeps all but one tape active every phase by distributing the initial runs in a *generalized Fibonacci* pattern, so that after each phase exactly one tape empties and becomes the next output. The number of phases grows like log of the run count — the point of the whole scheme is to beat balanced merging's tape count.
2. `polyphase_distribution`: start at level 0 = `(1, 0, ..., 0)` on T-1 input tapes and climb levels by `a1' = a1+a2, a2' = a1+a3, ..., a_{T-1}' = a1` until the total reaches `num_runs`; return that level. The shortfall to the perfect total is made up of *dummy runs* — empty `Vec`s that merge for free. `polyphase_merge`: deal runs per the distribution (one tape empty), then repeatedly (T-1)-way-merge until the shortest input tape empties.
3. Distribution loop: `while a.iter().sum() < num_runs { b[i] = a[0] + a[i+1] for i in 0..t-1; b[t-1] = a[0]; a = b }`. Merge: each phase, `out = the empty tape`; `m = min length among input tapes`; do `m` merges of one run popped from each input tape (via your stage-2 `merge_runs`) onto `out`; count one phase; stop when `<= 1` run remains total. A perfect level-n distribution finishes in exactly n phases.

## Stage 4: The full pipeline, with I/O accounted

1. The §5.4 cost model bills an external sort by *records moved*, not comparisons: run formation is one full pass (n reads + n writes), and each merge phase reads and writes only the records it touches — polyphase's advantage is that a phase usually touches part of the file, so `records_written <= n·(1 + phases)`. A file that collapses to a single run costs exactly one pass.
2. `external_sort` glues stage 1 to stage 3 while feeding one shared `IoStats`. Charge `records_read += n` and `records_written += n` for run formation, then run the polyphase phases charging each (T-1)-way merge: reads = sum of input-run lengths, writes = merged length. Reuse the polyphase engine so the accounting lives in one place.
3. `io.records_read += input.len()`, form runs with `replacement_selection(input, memory)`, `io.records_written += input.len()`, then run the polyphase core threading `&mut io` so every merge adds its moved-record counts. Verify: already-sorted input (one run) gives `records_read == records_written == n` and zero merge phases. Panic as before for `memory == 0` or `tapes < 3`.
