//! Reference implementations for the TAOCP-in-Rust course.
//!
//! One module per course module. Each algorithm keeps Knuth's step labels
//! (E1, E2, ...) as comments in a step-faithful implementation; idiomatic
//! variants may follow. Unit tests reproduce worked examples from the text.
//!
//! Students: don't read these until you've earned the stage. `./grade` never
//! needs you to open this crate — it exists so the course can prove its own
//! test suites are satisfiable (`./grade verify`).

pub mod m01_algorithms;
pub mod m02_math;
pub mod m03_structures;
pub mod m04_random;
pub mod m05_arithmetic;
pub mod m06_sorting;
pub mod m07_searching;
pub mod m08_generation;
pub mod m09_backtrack;
pub mod m10_sat;
pub mod m11_btree_trie;
pub mod m12_spectral;
pub mod m13_bits_bdds;
pub mod m14_cdcl;
pub mod m15_external;
pub mod m16_spectral_hd;
pub mod m17_zdd_xcc;
pub mod m18_mmix;
pub mod m19_float;
pub mod m20_networks;
pub mod m21_boolean;
pub mod m22_hamilton;
