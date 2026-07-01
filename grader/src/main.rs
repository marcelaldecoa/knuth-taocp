//! Staged grader for the TAOCP-in-Rust course.
//!
//! Invoke through the `./grade` wrapper at the repository root:
//!
//! ```text
//! ./grade                 show course progress
//! ./grade 6               grade module 06, stage by stage (stops at first failure)
//! ./grade 6 --stage 3     grade a single stage of module 06
//! ./grade all             grade every module in order
//! ./grade verify          course self-check: lab tests against reference solutions
//! ./grade reset           forget recorded progress
//! ```
//!
//! Like CodeCrafters, stages are ordered and each stage is a small, concrete
//! milestone. A stage passes when its integration-test file in the module's
//! lab crate is green. Progress is remembered in `.taocp/progress`.

mod manifest;

use manifest::{find_module, Module, MODULES};
use std::collections::BTreeSet;
use std::fs;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

const PROGRESS_FILE: &str = ".taocp/progress";

struct Style {
    on: bool,
}

impl Style {
    fn new() -> Self {
        Style {
            on: std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
        }
    }
    fn paint(&self, code: &str, s: &str) -> String {
        if self.on {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }
    fn green(&self, s: &str) -> String {
        self.paint("32;1", s)
    }
    fn red(&self, s: &str) -> String {
        self.paint("31;1", s)
    }
    fn yellow(&self, s: &str) -> String {
        self.paint("33", s)
    }
    fn bold(&self, s: &str) -> String {
        self.paint("1", s)
    }
    fn dim(&self, s: &str) -> String {
        self.paint("2", s)
    }
}

fn repo_root() -> PathBuf {
    // The wrapper script runs us from the repo root; fall back to the
    // directory containing Cargo.toml with [workspace] if invoked elsewhere.
    let cwd = std::env::current_dir().expect("cannot read current directory");
    let mut dir = cwd.as_path();
    loop {
        let candidate = dir.join("Cargo.toml");
        if candidate.exists() {
            if let Ok(text) = fs::read_to_string(&candidate) {
                if text.contains("[workspace]") {
                    return dir.to_path_buf();
                }
            }
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => return cwd,
        }
    }
}

fn load_progress(root: &PathBuf) -> BTreeSet<String> {
    fs::read_to_string(root.join(PROGRESS_FILE))
        .map(|t| t.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
        .unwrap_or_default()
}

fn save_progress(root: &PathBuf, progress: &BTreeSet<String>) {
    let path = root.join(PROGRESS_FILE);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut text: String = progress.iter().map(|l| format!("{l}\n")).collect();
    if text.is_empty() {
        text = String::new();
    }
    let _ = fs::write(path, text);
}

fn stage_key(module: &Module, test_target: &str) -> String {
    format!("{}/{}", module.lab_crate, test_target)
}

/// Run one stage's test target. Returns (passed, captured output).
fn run_stage(root: &PathBuf, module: &Module, test_target: &str, solutions: bool) -> (bool, String) {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(root)
        .arg("test")
        .arg("-q")
        .arg("-p")
        .arg(module.lab_crate)
        .arg("--test")
        .arg(test_target);
    if solutions {
        cmd.arg("--features").arg("solutions");
    }
    cmd.env("CARGO_TERM_COLOR", "never").env("RUST_BACKTRACE", "0");
    match cmd.output() {
        Ok(out) => {
            let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&out.stderr));
            (out.status.success(), text)
        }
        Err(e) => (false, format!("failed to invoke cargo: {e}")),
    }
}

/// Trim cargo/test noise down to the informative tail.
fn failure_excerpt(output: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = output
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with("Compiling")
                && !t.starts_with("Finished")
                && !t.starts_with("Running")
                && !t.starts_with("warning:")
        })
        .collect();
    let start = lines.len().saturating_sub(max_lines);
    lines[start..].join("\n")
}

fn print_stage_header(style: &Style, idx: usize, total: usize, title: &str, algorithm: &str) {
    println!(
        "  {} {}  {}",
        style.bold(&format!("Stage {}/{}", idx, total)),
        style.dim("·"),
        style.bold(title),
    );
    println!("    {}", style.dim(algorithm));
}

/// Grade a module CodeCrafters-style: stages in order, stop at first failure.
/// Returns the number of passing stages.
fn grade_module(
    root: &PathBuf,
    style: &Style,
    module: &Module,
    only_stage: Option<usize>,
    solutions: bool,
    verbose: bool,
    progress: &mut BTreeSet<String>,
) -> (usize, usize) {
    let total = module.stages.len();
    println!();
    println!(
        "{}",
        style.bold(&format!(
            "── Module {}: {} ({}) {}",
            module.id,
            module.title,
            module.source,
            "─".repeat(20)
        ))
    );

    let mut passed = 0;
    for (i, stage) in module.stages.iter().enumerate() {
        let n = i + 1;
        if let Some(only) = only_stage {
            if n != only {
                continue;
            }
        }
        print_stage_header(style, n, total, stage.title, stage.algorithm);
        let (ok, output) = run_stage(root, module, stage.test_target, solutions);
        if ok {
            passed += 1;
            progress.insert(stage_key(module, stage.test_target));
            println!("    {}", style.green("✓ passed"));
        } else {
            progress.remove(&stage_key(module, stage.test_target));
            println!("    {}", style.red("✗ failed"));
            let excerpt = if verbose {
                output.clone()
            } else {
                failure_excerpt(&output, 25)
            };
            for line in excerpt.lines() {
                println!("      {}", style.dim(line));
            }
            println!();
            println!(
                "    {} course/{}/README.md — stage {} walkthrough",
                style.yellow("Read:"),
                module.dir,
                n
            );
            println!(
                "    {} labs/{}/src/lab.rs — your code",
                style.yellow("Edit:"),
                module.dir
            );
            println!(
                "    {} ./grade {} --stage {}",
                style.yellow("Retry:"),
                module.id.trim_start_matches('0'),
                n
            );
            if only_stage.is_none() {
                // CodeCrafters behavior: later stages stay locked.
                let remaining = total - n;
                if remaining > 0 {
                    println!(
                        "    {}",
                        style.dim(&format!(
                            "({} later stage{} not run — fix this one first)",
                            remaining,
                            if remaining == 1 { "" } else { "s" }
                        ))
                    );
                }
                break;
            }
        }
    }
    save_progress(root, progress);
    (passed, total)
}

fn print_status(root: &PathBuf, style: &Style) {
    let progress = load_progress(root);
    println!();
    println!("{}", style.bold("The Art of Computer Programming — a hands-on course in Rust"));
    println!("{}", style.dim("Progress is recorded when you run `./grade <module>`."));
    println!();
    let mut done_total = 0;
    let mut all_total = 0;
    for m in MODULES {
        let done = m
            .stages
            .iter()
            .filter(|s| progress.contains(&stage_key(m, s.test_target)))
            .count();
        let total = m.stages.len();
        done_total += done;
        all_total += total;
        let bar: String = (0..total)
            .map(|i| if i < done { '█' } else { '░' })
            .collect();
        let bar = if done == total {
            style.green(&bar)
        } else if done > 0 {
            style.yellow(&bar)
        } else {
            style.dim(&bar)
        };
        println!(
            "  {}  {}  {}/{}  Module {} · {} {}",
            bar,
            if done == total { style.green("✓") } else { " ".to_string() },
            done,
            total,
            m.id,
            m.title,
            style.dim(&format!("({})", m.source)),
        );
    }
    println!();
    println!(
        "  {} of {} stages complete. Next: {}",
        done_total,
        all_total,
        style.bold(&next_hint(&progress)),
    );
    println!();
    println!("  {} ./grade <module>      e.g. ./grade 1", style.dim("run:"));
    println!("  {} ./grade all           grade everything", style.dim("run:"));
    println!("  {} ./grade verify        self-check labs against reference", style.dim("run:"));
}

fn next_hint(progress: &BTreeSet<String>) -> String {
    for m in MODULES {
        for (i, s) in m.stages.iter().enumerate() {
            if !progress.contains(&stage_key(m, s.test_target)) {
                return format!(
                    "./grade {} (stage {}: {})",
                    m.id.trim_start_matches('0'),
                    i + 1,
                    s.title
                );
            }
        }
    }
    "all done — congratulations!".to_string()
}

/// Course integrity check: every lab test must pass when the lab crate
/// re-exports the reference solutions, and the reference crate's own unit
/// tests (Knuth's worked examples) must pass too.
fn verify(root: &PathBuf, style: &Style, verbose: bool) -> bool {
    println!();
    println!("{}", style.bold("Course self-check: reference solutions vs. lab test suites"));
    let mut ok = true;

    print!("  reference unit tests … ");
    let out = Command::new("cargo")
        .current_dir(root)
        .args(["test", "-q", "-p", "taocp-reference"])
        .env("CARGO_TERM_COLOR", "never")
        .output();
    match out {
        Ok(o) if o.status.success() => println!("{}", style.green("✓")),
        Ok(o) => {
            ok = false;
            println!("{}", style.red("✗"));
            let mut text = String::from_utf8_lossy(&o.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&o.stderr));
            let excerpt = if verbose { text } else { failure_excerpt(&text, 30) };
            for line in excerpt.lines() {
                println!("    {}", style.dim(line));
            }
        }
        Err(e) => {
            ok = false;
            println!("{} ({e})", style.red("✗"));
        }
    }

    let mut dummy = BTreeSet::new();
    for m in MODULES {
        let (passed, total) = grade_module(root, style, m, None, true, verbose, &mut dummy);
        if passed != total {
            ok = false;
        }
    }
    // `verify` must not overwrite the student's progress record.
    println!();
    if ok {
        println!("{}", style.green("verify: every stage passes against the reference solutions."));
    } else {
        println!("{}", style.red("verify: FAILURES — the course itself is broken, see above."));
    }
    ok
}

fn main() -> ExitCode {
    let root = repo_root();
    let style = Style::new();
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut positional: Vec<String> = Vec::new();
    let mut only_stage: Option<usize> = None;
    let mut solutions = false;
    let mut verbose = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--stage" | "-s" => {
                i += 1;
                only_stage = args.get(i).and_then(|v| v.parse().ok());
                if only_stage.is_none() {
                    eprintln!("--stage needs a number");
                    return ExitCode::FAILURE;
                }
            }
            "--solutions" => solutions = true,
            "--verbose" | "-v" => verbose = true,
            "--help" | "-h" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other => positional.push(other.to_string()),
        }
        i += 1;
    }

    let mut progress = load_progress(&root);

    match positional.first().map(String::as_str) {
        None => {
            print_status(&root, &style);
            ExitCode::SUCCESS
        }
        Some("help") => {
            print_help();
            ExitCode::SUCCESS
        }
        Some("reset") => {
            save_progress(&root, &BTreeSet::new());
            println!("progress cleared.");
            ExitCode::SUCCESS
        }
        Some("verify") => {
            if verify(&root, &style, verbose) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Some("all") => {
            let mut all_ok = true;
            for m in MODULES {
                let (p, t) =
                    grade_module(&root, &style, m, None, solutions, verbose, &mut progress);
                if p != t {
                    all_ok = false;
                }
            }
            println!();
            print_status(&root, &style);
            if all_ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Some(query) => match find_module(query) {
            Some(m) => {
                let (p, t) =
                    grade_module(&root, &style, m, only_stage, solutions, verbose, &mut progress);
                let graded = if only_stage.is_some() { 1 } else { t };
                println!();
                if p >= graded {
                    println!(
                        "{}",
                        style.green(&format!("Module {} — all graded stages pass.", m.id))
                    );
                    if only_stage.is_none() {
                        println!("  Next: {}", next_hint(&load_progress(&root)));
                    }
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            None => {
                eprintln!("no module matches {query:?} — try ./grade with no arguments to list modules");
                ExitCode::FAILURE
            }
        },
    }
}

fn print_help() {
    println!(
        "\
The TAOCP-in-Rust course grader.

USAGE:
    ./grade                    show progress across all modules
    ./grade <module>           grade a module stage by stage (e.g. ./grade 3)
    ./grade <module> -s <n>    grade a single stage
    ./grade all                grade every module
    ./grade verify             self-check: run all lab tests against the
                               built-in reference solutions
    ./grade reset              clear recorded progress

FLAGS:
    --stage, -s <n>    run only stage n of the chosen module
    --solutions        run lab tests against the reference solutions
    --verbose, -v      show full cargo output on failure
"
    );
}
