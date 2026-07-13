//! The course manifest: every module and every stage, in order.
//!
//! This table is the single source of truth for the grader. A stage's
//! `test_target` names an integration-test file in the module's lab crate
//! (`labs/<dir>/tests/<test_target>.rs`).

pub struct Module {
    /// Two-digit id, e.g. "01".
    pub id: &'static str,
    /// Directory under labs/ and course/, e.g. "module-01-algorithms".
    pub dir: &'static str,
    /// Cargo package name of the lab crate.
    pub lab_crate: &'static str,
    pub title: &'static str,
    /// Where this lives in TAOCP.
    pub source: &'static str,
    pub stages: &'static [Stage],
}

pub struct Stage {
    /// Integration-test file name (without .rs) in the lab crate.
    pub test_target: &'static str,
    pub title: &'static str,
    /// Knuth's algorithm label(s), if any — e.g. "Algorithm 1.1E".
    pub algorithm: &'static str,
}

pub const MODULES: &[Module] = &[
    Module {
        id: "01",
        dir: "module-01-algorithms",
        lab_crate: "lab-01-algorithms",
        title: "The Notion of an Algorithm",
        source: "Vol. 1, §1.1",
        stages: &[
            Stage {
                test_target: "stage_01_euclid",
                title: "Euclid's algorithm, step by step",
                algorithm: "Algorithm 1.1E",
            },
            Stage {
                test_target: "stage_02_euclid_f",
                title: "Avoiding trivial replacements",
                algorithm: "Algorithm 1.1F (ex. 1.1-3)",
            },
            Stage {
                test_target: "stage_03_extended_euclid",
                title: "Extended Euclid: certifying the gcd",
                algorithm: "Algorithm 1.2.1E",
            },
            Stage {
                test_target: "stage_04_lame",
                title: "Counting divisions; Lamé's worst case",
                algorithm: "Theorem 4.5.3F (Lamé)",
            },
        ],
    },
    Module {
        id: "02",
        dir: "module-02-math",
        lab_crate: "lab-02-math",
        title: "Mathematical Preliminaries",
        source: "Vol. 1, §1.2",
        stages: &[
            Stage {
                test_target: "stage_01_sums",
                title: "Sums in closed form",
                algorithm: "§1.2.3",
            },
            Stage {
                test_target: "stage_02_binomials",
                title: "Binomial coefficients",
                algorithm: "§1.2.6",
            },
            Stage {
                test_target: "stage_03_fibonacci",
                title: "Fibonacci numbers, fast",
                algorithm: "§1.2.8",
            },
            Stage {
                test_target: "stage_04_harmonic",
                title: "Harmonic numbers, exactly and asymptotically",
                algorithm: "§1.2.7",
            },
            Stage {
                test_target: "stage_05_find_max",
                title: "Analysis of an algorithm: finding the maximum",
                algorithm: "Algorithm 1.2.10M",
            },
        ],
    },
    Module {
        id: "03",
        dir: "module-03-structures",
        lab_crate: "lab-03-structures",
        title: "Information Structures",
        source: "Vol. 1, Ch. 2",
        stages: &[
            Stage {
                test_target: "stage_01_stack_queue",
                title: "Stacks and queues in sequential storage",
                algorithm: "§2.2.1–2.2.2",
            },
            Stage {
                test_target: "stage_02_linked_list",
                title: "Linked allocation with an AVAIL list",
                algorithm: "§2.2.3",
            },
            Stage {
                test_target: "stage_03_toposort",
                title: "Topological sorting",
                algorithm: "Algorithm 2.2.3T",
            },
            Stage {
                test_target: "stage_04_tree_traversal",
                title: "Traversing binary trees",
                algorithm: "Algorithm 2.3.1T",
            },
            Stage {
                test_target: "stage_05_threaded_tree",
                title: "Threaded binary trees",
                algorithm: "Algorithm 2.3.1S",
            },
        ],
    },
    Module {
        id: "04",
        dir: "module-04-random",
        lab_crate: "lab-04-random",
        title: "Random Numbers",
        source: "Vol. 2, Ch. 3",
        stages: &[
            Stage {
                test_target: "stage_01_lcg",
                title: "The linear congruential method",
                algorithm: "§3.2.1",
            },
            Stage {
                test_target: "stage_02_chi_square",
                title: "The chi-square test",
                algorithm: "§3.3.1",
            },
            Stage {
                test_target: "stage_03_shuffle",
                title: "Shuffling",
                algorithm: "Algorithm 3.4.2P",
            },
            Stage {
                test_target: "stage_04_reservoir",
                title: "Reservoir sampling",
                algorithm: "Algorithm 3.4.2R",
            },
        ],
    },
    Module {
        id: "05",
        dir: "module-05-arithmetic",
        lab_crate: "lab-05-arithmetic",
        title: "Arithmetic",
        source: "Vol. 2, Ch. 4",
        stages: &[
            Stage {
                test_target: "stage_01_bignum_add",
                title: "Multiple-precision addition and subtraction",
                algorithm: "Algorithms 4.3.1A, 4.3.1S",
            },
            Stage {
                test_target: "stage_02_bignum_mul",
                title: "Classical multiplication",
                algorithm: "Algorithm 4.3.1M",
            },
            Stage {
                test_target: "stage_03_karatsuba",
                title: "Faster multiplication by divide and conquer",
                algorithm: "§4.3.3",
            },
            Stage {
                test_target: "stage_04_binary_gcd",
                title: "The binary gcd algorithm",
                algorithm: "Algorithm 4.5.2B",
            },
            Stage {
                test_target: "stage_05_primality",
                title: "Probabilistic primality testing",
                algorithm: "Algorithm 4.5.4P (Miller–Rabin lineage)",
            },
        ],
    },
    Module {
        id: "06",
        dir: "module-06-sorting",
        lab_crate: "lab-06-sorting",
        title: "Sorting",
        source: "Vol. 3, Ch. 5",
        stages: &[
            Stage {
                test_target: "stage_01_insertion",
                title: "Straight insertion; inversions",
                algorithm: "Algorithm 5.2.1S, §5.1.1",
            },
            Stage {
                test_target: "stage_02_shellsort",
                title: "Shellsort: diminishing increments",
                algorithm: "Algorithm 5.2.1D",
            },
            Stage {
                test_target: "stage_03_quicksort",
                title: "Quicksort: partition exchange",
                algorithm: "Algorithm 5.2.2Q",
            },
            Stage {
                test_target: "stage_04_heapsort",
                title: "Heapsort",
                algorithm: "Algorithm 5.2.3H",
            },
            Stage {
                test_target: "stage_05_natural_merge",
                title: "Natural merge sort",
                algorithm: "Algorithm 5.2.4N",
            },
            Stage {
                test_target: "stage_06_radix",
                title: "Radix sorting",
                algorithm: "Algorithm 5.2.5R",
            },
        ],
    },
    Module {
        id: "07",
        dir: "module-07-searching",
        lab_crate: "lab-07-searching",
        title: "Searching",
        source: "Vol. 3, Ch. 6",
        stages: &[
            Stage {
                test_target: "stage_01_binary_search",
                title: "Binary search",
                algorithm: "Algorithm 6.2.1B",
            },
            Stage {
                test_target: "stage_02_bst",
                title: "Binary search trees",
                algorithm: "Algorithm 6.2.2T",
            },
            Stage {
                test_target: "stage_03_avl",
                title: "Balanced trees (AVL)",
                algorithm: "Algorithm 6.2.3A",
            },
            Stage {
                test_target: "stage_04_hashing",
                title: "Hashing with open addressing",
                algorithm: "Algorithms 6.4L, 6.4D",
            },
        ],
    },
    Module {
        id: "08",
        dir: "module-08-generation",
        lab_crate: "lab-08-generation",
        title: "Combinatorial Generation",
        source: "Vol. 4A, §7.2.1",
        stages: &[
            Stage {
                test_target: "stage_01_gray_codes",
                title: "Gray binary code",
                algorithm: "Algorithm 7.2.1.1G",
            },
            Stage {
                test_target: "stage_02_lex_permutations",
                title: "Permutations in lexicographic order",
                algorithm: "Algorithm 7.2.1.2L",
            },
            Stage {
                test_target: "stage_03_plain_changes",
                title: "Plain changes: adjacent transpositions",
                algorithm: "Algorithm 7.2.1.2P",
            },
            Stage {
                test_target: "stage_04_combinations",
                title: "Generating combinations",
                algorithm: "Algorithm 7.2.1.3T",
            },
            Stage {
                test_target: "stage_05_partitions",
                title: "Partitions of an integer",
                algorithm: "Algorithm 7.2.1.4P",
            },
        ],
    },
    Module {
        id: "09",
        dir: "module-09-backtrack",
        lab_crate: "lab-09-backtrack",
        title: "Backtracking and Dancing Links",
        source: "Vol. 4B, §7.2.2–7.2.2.1",
        stages: &[
            Stage {
                test_target: "stage_01_n_queens",
                title: "Basic backtrack: n queens",
                algorithm: "Algorithm 7.2.2B",
            },
            Stage {
                test_target: "stage_02_walker_bitwise",
                title: "Backtracking with bitwise state",
                algorithm: "Walker's method, §7.2.2",
            },
            Stage {
                test_target: "stage_03_dancing_links",
                title: "Exact cover via dancing links",
                algorithm: "Algorithm 7.2.2.1X",
            },
            Stage {
                test_target: "stage_04_sudoku",
                title: "Sudoku as an exact-cover problem",
                algorithm: "§7.2.2.1 application",
            },
        ],
    },
    MODULE_10,
    Module {
        id: "11",
        dir: "module-11-btree-trie",
        lab_crate: "lab-11-btree-trie",
        title: "Multiway Trees and Digital Searching",
        source: "Vol. 3, §6.2.4 & §6.3",
        stages: &[
            Stage {
                test_target: "stage_01_btree",
                title: "B-tree search and insertion with node splitting",
                algorithm: "§6.2.4",
            },
            Stage {
                test_target: "stage_02_btree_analysis",
                title: "B-tree invariants and the height bound",
                algorithm: "§6.2.4 analysis",
            },
            Stage {
                test_target: "stage_03_trie",
                title: "Digital searching: tries",
                algorithm: "Algorithm 6.3T",
            },
            Stage {
                test_target: "stage_04_patricia",
                title: "Patricia: compressed binary tries",
                algorithm: "Algorithm 6.3P",
            },
        ],
    },
    Module {
        id: "12",
        dir: "module-12-spectral",
        lab_crate: "lab-12-spectral",
        title: "The Spectral Test",
        source: "Vol. 2, §3.3.4",
        stages: &[
            Stage {
                test_target: "stage_01_lattice",
                title: "The lattice structure of linear congruential sequences",
                algorithm: "§3.3.4 preliminaries",
            },
            Stage {
                test_target: "stage_02_spectral_2d",
                title: "The two-dimensional spectral test, exactly",
                algorithm: "Gauss–Lagrange reduction",
            },
            Stage {
                test_target: "stage_03_spectral_3d",
                title: "Short dual vectors in three dimensions",
                algorithm: "Algorithm 3.3.4S (bounded search)",
            },
            Stage {
                test_target: "stage_04_merit",
                title: "Figures of merit: judging real generators",
                algorithm: "§3.3.4 ratings",
            },
        ],
    },
    Module {
        id: "13",
        dir: "module-13-bits-bdds",
        lab_crate: "lab-13-bits-bdds",
        title: "Bitwise Tricks and Binary Decision Diagrams",
        source: "Vol. 4A, §7.1.3–7.1.4",
        stages: &[
            Stage {
                test_target: "stage_01_bit_tricks",
                title: "Ruler function, sideways addition, Gosper's hack",
                algorithm: "§7.1.3",
            },
            Stage {
                test_target: "stage_02_bdd_build",
                title: "Building reduced ordered BDDs",
                algorithm: "§7.1.4",
            },
            Stage {
                test_target: "stage_03_bdd_count",
                title: "Model counting and the ordering problem",
                algorithm: "Algorithm 7.1.4C",
            },
            Stage {
                test_target: "stage_04_bdd_apps",
                title: "BDDs at work: independent sets and queens",
                algorithm: "§7.1.4 applications",
            },
        ],
    },
    Module {
        id: "14",
        dir: "module-14-cdcl",
        lab_crate: "lab-14-cdcl",
        title: "Conflict-Driven Clause Learning",
        source: "Vol. 4B, §7.2.2.2, Algorithm C",
        stages: &[
            Stage {
                test_target: "stage_01_watched_literals",
                title: "Lazy data structures: two watched literals",
                algorithm: "§7.2.2.2 (Algorithm C's engine)",
            },
            Stage {
                test_target: "stage_02_trail",
                title: "The trail: decisions, levels, reasons",
                algorithm: "§7.2.2.2",
            },
            Stage {
                test_target: "stage_03_clause_learning",
                title: "Conflict analysis: learning from failure",
                algorithm: "Algorithm 7.2.2.2C (first UIP)",
            },
            Stage {
                test_target: "stage_04_cdcl_solver",
                title: "The complete CDCL solver",
                algorithm: "Algorithm 7.2.2.2C",
            },
        ],
    },
    Module {
        id: "15",
        dir: "module-15-external",
        lab_crate: "lab-15-external",
        title: "External Sorting",
        source: "Vol. 3, §5.4",
        stages: &[
            Stage {
                test_target: "stage_01_replacement_selection",
                title: "Replacement selection: runs twice the memory",
                algorithm: "Algorithm 5.4.1R",
            },
            Stage {
                test_target: "stage_02_multiway_merge",
                title: "k-way merging with a tree of losers",
                algorithm: "§5.4.1",
            },
            Stage {
                test_target: "stage_03_polyphase",
                title: "Polyphase merge: Fibonacci on tape",
                algorithm: "Algorithm 5.4.2D",
            },
            Stage {
                test_target: "stage_04_external_sort",
                title: "The full pipeline, with I/O accounted",
                algorithm: "§5.4 synthesis",
            },
        ],
    },
    Module {
        id: "16",
        dir: "module-16-spectral-hd",
        lab_crate: "lab-16-spectral-hd",
        title: "The Spectral Test in Higher Dimensions",
        source: "Vol. 2, §3.3.4, Algorithm S",
        stages: &[
            Stage {
                test_target: "stage_01_dual_basis",
                title: "Dual lattice bases in dimension t",
                algorithm: "§3.3.4, steps S1–S3",
            },
            Stage {
                test_target: "stage_02_reduction",
                title: "Basis reduction by unimodular transformations",
                algorithm: "§3.3.4, steps S5–S7",
            },
            Stage {
                test_target: "stage_03_exhaustive",
                title: "The certified exhaustive search",
                algorithm: "§3.3.4, steps S8–S10",
            },
            Stage {
                test_target: "stage_04_spectral_t",
                title: "ν_t and μ_t for real generators, t ≤ 6",
                algorithm: "Algorithm 3.3.4S",
            },
        ],
    },
    Module {
        id: "17",
        dir: "module-17-zdd-xcc",
        lab_crate: "lab-17-zdd-xcc",
        title: "ZDDs and Exact Covering with Colors",
        source: "Vol. 4A, §7.1.4 & Vol. 4B, §7.2.2.1",
        stages: &[
            Stage {
                test_target: "stage_01_zdd_build",
                title: "Zero-suppressed decision diagrams",
                algorithm: "§7.1.4 (ZDD reduction rule)",
            },
            Stage {
                test_target: "stage_02_zdd_ops",
                title: "The family algebra: union, intersection, join",
                algorithm: "§7.1.4",
            },
            Stage {
                test_target: "stage_03_zdd_paths",
                title: "Counting structures in graphs with ZDDs",
                algorithm: "§7.1.4 applications",
            },
            Stage {
                test_target: "stage_04_xcc",
                title: "Exact cover with colors",
                algorithm: "Algorithm 7.2.2.1C",
            },
        ],
    },
    Module {
        id: "18",
        dir: "module-18-mmix",
        lab_crate: "lab-18-mmix",
        title: "MMIX: Knuth's Machine",
        source: "Vol. 1, Fascicle 1",
        stages: &[
            Stage {
                test_target: "stage_01_registers_loadstore",
                title: "Machine state, memory, loads and stores",
                algorithm: "MMIX basics",
            },
            Stage {
                test_target: "stage_02_arithmetic",
                title: "Arithmetic: signed, unsigned, overflow, DIV",
                algorithm: "MMIX operations",
            },
            Stage {
                test_target: "stage_03_branches",
                title: "Comparisons, branches, and loops",
                algorithm: "MMIX control flow",
            },
            Stage {
                test_target: "stage_04_programs",
                title: "Programs: Euclid and FindMax on the metal",
                algorithm: "Fascicle 1 programs",
            },
        ],
    },
    Module {
        id: "19",
        dir: "module-19-float",
        lab_crate: "lab-19-float",
        title: "Floating-Point Arithmetic",
        source: "Vol. 2, §4.2",
        stages: &[
            Stage {
                test_target: "stage_01_representation",
                title: "Representation: pack, unpack, normalize",
                algorithm: "§4.2.1",
            },
            Stage {
                test_target: "stage_02_add_sub",
                title: "Addition and subtraction with rounding",
                algorithm: "Algorithm 4.2.1A",
            },
            Stage {
                test_target: "stage_03_mul_div",
                title: "Multiplication and division",
                algorithm: "Algorithm 4.2.1M",
            },
            Stage {
                test_target: "stage_04_error_analysis",
                title: "Error analysis: ulps and compensated summation",
                algorithm: "§4.2.2",
            },
        ],
    },
    Module {
        id: "20",
        dir: "module-20-networks",
        lab_crate: "lab-20-networks",
        title: "Optimum Sorting and Sorting Networks",
        source: "Vol. 3, §5.3",
        stages: &[
            Stage {
                test_target: "stage_01_decision_trees",
                title: "Comparison lower bounds by decision trees",
                algorithm: "§5.3.1",
            },
            Stage {
                test_target: "stage_02_merge_insertion",
                title: "Merge insertion (Ford–Johnson)",
                algorithm: "Algorithm 5.3.1M",
            },
            Stage {
                test_target: "stage_03_networks",
                title: "Sorting networks: Batcher's odd-even merge",
                algorithm: "Algorithm 5.3.4M",
            },
            Stage {
                test_target: "stage_04_zero_one",
                title: "The zero-one principle and bitonic sorting",
                algorithm: "Theorem 5.3.4Z",
            },
        ],
    },
    Module {
        id: "21",
        dir: "module-21-boolean",
        lab_crate: "lab-21-boolean",
        title: "Boolean Functions and Optimal Evaluation",
        source: "Vol. 4A, §7.1.1–7.1.2",
        stages: &[
            Stage {
                test_target: "stage_01_truth_tables",
                title: "Truth tables and normal forms",
                algorithm: "§7.1.1",
            },
            Stage {
                test_target: "stage_02_boolean_chains",
                title: "Boolean chains and combinational cost",
                algorithm: "§7.1.2",
            },
            Stage {
                test_target: "stage_03_median_threshold",
                title: "Median, threshold, and symmetric functions",
                algorithm: "§7.1.1",
            },
            Stage {
                test_target: "stage_04_optimal_chains",
                title: "Optimum chains for small functions",
                algorithm: "§7.1.2",
            },
        ],
    },
    Module {
        id: "22",
        dir: "module-22-hamilton",
        lab_crate: "lab-22-hamilton",
        title: "Hamiltonian Paths and Cycles",
        source: "Toward Vol. 4C, §7.2.2.4 (pre-fascicles)",
        stages: &[
            Stage {
                test_target: "stage_01_backtrack",
                title: "Hamiltonian paths by backtracking",
                algorithm: "§7.2.2.4",
            },
            Stage {
                test_target: "stage_02_warnsdorff",
                title: "Warnsdorff's heuristic: the knight's tour",
                algorithm: "§7.2.2.4",
            },
            Stage {
                test_target: "stage_03_gray_cycles",
                title: "Hamiltonian cycles on the hypercube are Gray codes",
                algorithm: "§7.2.1.1 ↔ §7.2.2.4",
            },
            Stage {
                test_target: "stage_04_held_karp",
                title: "Held–Karp: shortest Hamiltonian path by bitmask DP",
                algorithm: "§7.2.2.4 (dynamic programming)",
            },
        ],
    },
    Module {
        id: "23",
        dir: "module-23-csp",
        lab_crate: "lab-23-csp",
        title: "Constraint Satisfaction",
        source: "Toward Vol. 4C, §7.2.2.3 (Fascicle 7)",
        stages: &[
            Stage {
                test_target: "stage_01_model",
                title: "The CSP model and basic backtracking",
                algorithm: "§7.2.2.3 & Algorithm 7.2.2B",
            },
            Stage {
                test_target: "stage_02_lookahead",
                title: "Forward checking and MRV ordering",
                algorithm: "§7.2.2.3 lookahead",
            },
            Stage {
                test_target: "stage_03_ac3",
                title: "Arc consistency (AC-3)",
                algorithm: "§7.2.2.3 consistency",
            },
            Stage {
                test_target: "stage_04_sat_encoding",
                title: "Translating CSP to SAT",
                algorithm: "§7.2.2.3 ↔ §7.2.2.2",
            },
        ],
    },
];

const MODULE_10: Module = Module {
    id: "10",
        dir: "module-10-sat",
        lab_crate: "lab-10-sat",
        title: "Satisfiability",
        source: "Vol. 4B, §7.2.2.2",
        stages: &[
            Stage {
                test_target: "stage_01_cnf",
                title: "Conjunctive normal form and DIMACS",
                algorithm: "§7.2.2.2 representations",
            },
            Stage {
                test_target: "stage_02_unit_propagation",
                title: "Unit propagation",
                algorithm: "§7.2.2.2",
            },
            Stage {
                test_target: "stage_03_dpll",
                title: "A DPLL solver",
                algorithm: "Algorithm 7.2.2.2D",
            },
            Stage {
                test_target: "stage_04_sat_applications",
                title: "Encoding problems into SAT",
                algorithm: "§7.2.2.2 encodings",
            },
        ],
};

/// Find a module by "1", "01", a "module-…" dir prefix, or a substring of its
/// slug (the part after the numeric prefix, e.g. "sorting").
pub fn find_module(query: &str) -> Option<&'static Module> {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return None;
    }
    let as_id = q.trim_start_matches('0');
    // A purely numeric query must match a module id exactly: the substring
    // fallback would otherwise let "./grade 0" grade module 01 (via
    // "module-01-…") and similar junk queries match arbitrary modules.
    let numeric = q.chars().all(|c| c.is_ascii_digit());
    MODULES.iter().find(|m| {
        let slug = m.dir.splitn(3, '-').nth(2).unwrap_or("");
        m.id == q
            || (numeric && !as_id.is_empty() && m.id.trim_start_matches('0') == as_id)
            || (!numeric && q.starts_with("module-") && m.dir.starts_with(q.as_str()))
            || (!numeric && !q.starts_with("module-") && slug.contains(q.as_str()))
    })
}
