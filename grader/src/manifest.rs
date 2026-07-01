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
    Module {
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
    },
];

/// Find a module by "1", "01", "module-01-...", or a substring of its dir.
pub fn find_module(query: &str) -> Option<&'static Module> {
    let q = query.trim().to_ascii_lowercase();
    let as_id = q.trim_start_matches('0');
    MODULES.iter().find(|m| {
        m.id == q
            || m.id.trim_start_matches('0') == as_id && !as_id.is_empty()
            || m.dir == q
            || m.dir.contains(q.as_str())
    })
}
