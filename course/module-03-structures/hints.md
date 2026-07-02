# Hints — Module 03: Information Structures

Graduated hints, three per stage. Reach for hint 1 first; drop to 3 only when stuck.

## Stage 1: Stacks and queues in sequential storage

1. A sequential stack is a base plus a top pointer T; a queue is a circular buffer with front F and rear R moving mod M. Knuth's OVERFLOW/UNDERFLOW are *conditions to report*, not crashes — so push/enqueue return `Err(Overflow)` when full and pop/dequeue return `Err(Underflow)` when empty. For the railway shunting problem, the key theorem (§2.2.1) is that a permutation is stack-realizable iff it avoids the 3-1-2 pattern.
2. Back the stack with a `Vec` whose length plays the role of T; check `len == capacity` before pushing. For the queue, store `front` and an explicit `len` (not two bare pointers, which can't tell full from empty) and compute the rear index as `(front + len) % capacity`. For `stack_permutable`, greedily simulate one siding: push arrivals until the wanted car is on top, then it must be the top or the answer is `false`.
3. Queue enqueue: `let r = (self.front + self.len) % cap; self.buf[r] = Some(x); self.len += 1;`. Dequeue: take `buf[front]`, advance `front = (front+1) % cap`, `len -= 1`. Shunting: `for &want in perm { while next_arrival <= want { stack.push(next_arrival); next_arrival += 1; } if stack.pop() != Some(want) { return false; } }` — plus an early `false` if any `want` is outside `1..=n`.

## Stage 2: Linked allocation with an AVAIL list

1. Model Knuth's linked memory as two parallel arrays — INFO and LINK indexed by `usize` — with the null link Λ being a sentinel (`LAMBDA = usize::MAX`). The AVAIL list is itself a stack of free cells threaded through their LINK fields; `allocate` is `P <= AVAIL` and `free` is `AVAIL <= P`.
2. On `allocate`, if `avail != LAMBDA` reuse the head cell (`avail <- link[avail]`); only when AVAIL is empty do you `push` a fresh cell onto the backing `Vec`. On `free`, thread the cell back: `link[p] <- avail; avail <- p`. This guarantees freed cells are reused before new memory is drawn — check via `cells_in_memory()`.
3. `delete_after(p)`: `let q = link[p]; if q == LAMBDA { return None; } link[p] = link[q]; Some(self.free(q))`. `push_front(first, info)`: `let p = self.allocate(info); link[p] = first; p`. Store INFO as `Option<T>` so a cell on AVAIL holds `None` and `free` can `.take()` the value out.

## Stage 3: Topological sorting

1. Algorithm 2.2.3T is Kahn's method: repeatedly output an object with no remaining predecessors. Maintain `COUNT[k]` = number of unmet predecessors of `k`, and a successor list per object; objects reach the output queue exactly when their count drops to zero. A leftover object at the end means a cycle.
2. First pass over the relations: for each `(j, k)`, `count[k] += 1` and append `k` to `suc[j]`. Seed a FIFO queue with every object whose count is 0 (scan `1..=n` in order for determinism). Then repeatedly pop the front, output it, and decrement each successor's count, enqueueing any that hit zero.
3. Use a `VecDeque` for the queue. `while let Some(f) = queue.pop_front() { output.push(f); for &k in &suc[f] { count[k] -= 1; if count[k] == 0 { queue.push_back(k); } } }`. Return `Some(output)` iff `output.len() == n`, else `None`. Treat any `(j, j)` pair as an immediate cycle → `None`.

## Stage 4: Traversing binary trees

1. Algorithm 2.3.1T does inorder traversal with an *explicit stack*, not recursion: go left as far as possible pushing nodes, then pop-visit-go-right. Preorder and postorder are small variations on the same skeleton (§2.3.1 exercises 12–13). Reconstructing a tree from preorder+inorder works because the first preorder element is the root, which splits the inorder sequence into left/right subtrees.
2. Inorder: an inner `while p != LAMBDA { push(p); p = llink[p]; }` then pop and `visit; p = rlink[p]`. Preorder: visit at the moment of *pushing* instead of popping. Postorder is cleanly done as a two-stack / reversed walk: push root, pop-and-emit, push left then right children, finally reverse the output.
3. `from_traversals`: recursively, `root = pre[0]`; find `split = position of root in inorder`; left subtree from `pre[1..=split]` & `ino[..split]`, right from `pre[split+1..]` & `ino[split+1..]`; `add_node(root, left, right)`. Build children first (bottom-up) so you have their indices before creating the parent.

## Stage 5: Threaded binary trees

1. A fully threaded tree replaces every Λ link with a *tagged thread* to the node's inorder predecessor (left) or successor (right), plus a list head at index 0. The payoff: Algorithm 2.3.1S computes the inorder successor with **no stack and no recursion**. Insertion must patch the one thread that used to point where the new node now sits.
2. `successor(p)`: if RLINK(p) is a thread (RTAG set), it points straight at the successor; otherwise take one RLINK then descend LLINKs while LTAG is clear to reach the leftmost node of the right subtree. For `insert_right(p, x)`, the new node inherits P's right link/tag and gets a left thread back to P; then P's right becomes the new node; then if the new node has a real right subtree, retarget its successor's left thread.
3. `successor`: `let mut q = rlink[p]; if rtag[p] { return q; } while !ltag[q] { q = llink[q]; } q`. Iterate from `head()` calling the inline successor until you return to the head to get the full inorder sequence — Λ is `LAMBDA`, the head is node 0 with RLINK(head) = head. `insert_left` is the mirror image and is what appends at the end of the inorder order when applied to the head.
