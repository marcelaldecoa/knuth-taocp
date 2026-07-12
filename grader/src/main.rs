//! Staged grader for the TAOCP-in-Rust course.
//!
//! Invoke through the `./grade` wrapper at the repository root:
//!
//! ```text
//! ./grade                 show course progress
//! ./grade 6               grade module 06, stage by stage (stops at first failure)
//! ./grade 6 --stage 3     grade a single stage of module 06
//! ./grade watch 6         re-grade module 06 on every save to its lab.rs
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
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
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

fn load_progress(root: &Path) -> BTreeSet<String> {
    fs::read_to_string(root.join(PROGRESS_FILE))
        .map(|t| t.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
        .unwrap_or_default()
}

fn save_progress(root: &Path, progress: &BTreeSet<String>) {
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

/// The reference solution file stem for a module: `module-01-algorithms`
/// maps to `m01_algorithms` (i.e. `reference/src/m01_algorithms.rs`).
fn reference_stem(module: &Module) -> String {
    format!("m{}", module.dir.trim_start_matches("module-").replace('-', "_"))
}

/// Wall-clock cap for a single stage run, so a student's accidental infinite
/// loop ends with a diagnosis instead of hanging `./grade` (and `watch`)
/// forever. Generous by default because `cargo test` includes compilation;
/// override with `TAOCP_STAGE_TIMEOUT_SECS`.
fn stage_timeout() -> std::time::Duration {
    let secs = std::env::var("TAOCP_STAGE_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(300);
    std::time::Duration::from_secs(secs)
}

/// The `N passed` count from a `test result:` summary line, if this is one.
fn parse_passed_count(line: &str) -> Option<usize> {
    let t = line.trim_start();
    if !t.starts_with("test result:") {
        return None;
    }
    let before = t.split(" passed").next()?;
    before
        .rsplit(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}

/// Run one stage's test target. Returns (passed, captured output).
fn run_stage(root: &PathBuf, module: &Module, test_target: &str, solutions: bool) -> (bool, String) {
    use std::io::Read;
    use std::process::Stdio;
    use std::time::{Duration, Instant};

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
    } else {
        // A lab crate must never pull the reference in by default (say, via a
        // stray `default = ["solutions"]` in its Cargo.toml) — that would make
        // every stage pass with the stub untouched. Labs define no other
        // features, so this is safe to pass unconditionally.
        cmd.arg("--no-default-features");
    }
    cmd.env("CARGO_TERM_COLOR", "never").env("RUST_BACKTRACE", "0");
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return (false, format!("failed to invoke cargo: {e}")),
    };
    // Drain both pipes on threads so a chatty test can't fill a pipe buffer
    // and deadlock against the timeout loop below.
    let mut child_out = child.stdout.take();
    let mut child_err = child.stderr.take();
    let out_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(ref mut s) = child_out {
            let _ = s.read_to_end(&mut buf);
        }
        buf
    });
    let err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(ref mut s) = child_err {
            let _ = s.read_to_end(&mut buf);
        }
        buf
    });

    let deadline = Instant::now() + stage_timeout();
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    // Best effort: killing cargo can orphan the test binary,
                    // but it unblocks the grader with a clear diagnosis.
                    timed_out = true;
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return (false, format!("failed to wait on cargo: {e}"));
            }
        }
    };

    let mut text = String::from_utf8_lossy(&out_reader.join().unwrap_or_default()).into_owned();
    text.push_str(&String::from_utf8_lossy(&err_reader.join().unwrap_or_default()));

    if timed_out || status.is_none() {
        text.push_str(&format!(
            "\nstage timed out after {}s — look for an infinite loop in your code \
             (or raise TAOCP_STAGE_TIMEOUT_SECS if this machine is just slow)",
            stage_timeout().as_secs()
        ));
        return (false, text);
    }
    if !status.map(|s| s.success()).unwrap_or(false) {
        return (false, text);
    }
    // A green run that executed zero tests proves nothing — treat it as a
    // failure (this catches an emptied stage-test file).
    let ran: usize = text.lines().filter_map(parse_passed_count).sum();
    if ran == 0 {
        text.push_str(
            "\nstage reported success but ran 0 tests — restore the stage test file \
             (git checkout -- labs/<module>/tests/)",
        );
        return (false, text);
    }
    (true, text)
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
            if !solutions {
                progress.insert(stage_key(module, stage.test_target));
            }
            println!("    {}", style.green("✓ passed"));
        } else {
            if !solutions {
                progress.remove(&stage_key(module, stage.test_target));
            }
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
                "    {} course/{}/README.md — the stage {} lesson",
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
            println!(
                "    {} ./grade {} --stage {} --hint",
                style.yellow("Hint:"),
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
    // Progress records what the STUDENT's code earned. `--solutions` runs
    // (and `verify`, which grades with solutions on) exercise the reference,
    // so they must neither mark stages complete nor overwrite the record.
    if !solutions {
        save_progress(root, progress);
    }
    (passed, total)
}

fn print_status(root: &Path, style: &Style) {
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
    println!("  {} ./grade next          jump to your next unsolved stage", style.dim("run:"));
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

/// The first module that still has an unsolved stage, or `None` if the course
/// is complete. Used by `./grade next`.
fn next_unsolved_module(progress: &BTreeSet<String>) -> Option<&'static Module> {
    MODULES
        .iter()
        .find(|m| m.stages.iter().any(|s| !progress.contains(&stage_key(m, s.test_target))))
}

/// The "you passed — here's where to look next" footer, shared by the module
/// grading path and `./grade next`.
fn report_module_pass(root: &Path, style: &Style, m: &Module, only_stage: bool) {
    println!(
        "{}",
        style.green(&format!("Module {} — all graded stages pass.", m.id))
    );
    println!(
        "  {} course/{}/WALKTHROUGH.md — how the reference is built",
        style.dim("Deepen:"),
        m.dir
    );
    println!(
        "  {} reference/src/{}.rs — Knuth's step-faithful solution, compare with yours",
        style.dim("Compare:"),
        reference_stem(m),
    );
    if !only_stage {
        println!("  Next: {}", next_hint(&load_progress(root)));
    }
}

/// Recursively collect files with the given extension, skipping build/VCS dirs.
fn collect_files(dir: &std::path::Path, ext: &str, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if matches!(name, "target" | ".git" | ".taocp" | "node_modules") {
                continue;
            }
            collect_files(&path, ext, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

/// Extract the targets of Markdown links `[text](target)` from `text`.
fn markdown_link_targets(text: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut i = 0;
    while let Some(rel) = text[i..].find("](") {
        let open = i + rel + 2; // index just past "]("
        if let Some(close) = text[open..].find(')') {
            let target = text[open..open + close].trim().to_string();
            if !target.is_empty() {
                targets.push(target);
            }
            i = open + close + 1;
        } else {
            break;
        }
    }
    targets
}

/// Strip inline Markdown (`` ` ``, `*`, `_`, and `[label](url)` → `label`) from a
/// heading so it can be turned into an anchor slug.
fn strip_inline_markdown(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '`' | '*' | '_' => {}
            '[' => {
                // Emit the link label, then swallow an optional "(url)".
                for l in chars.by_ref() {
                    if l == ']' {
                        break;
                    }
                    out.push(l);
                }
                if chars.peek() == Some(&'(') {
                    for u in chars.by_ref() {
                        if u == ')' {
                            break;
                        }
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// The GitHub-style anchor slug of a heading's text: lowercase, drop
/// punctuation, spaces become hyphens (Unicode letters/digits are kept).
fn heading_slug(raw: &str) -> String {
    let cleaned = strip_inline_markdown(raw);
    let mut slug = String::new();
    for ch in cleaned.trim().chars() {
        if ch == ' ' || ch == '\t' {
            slug.push('-');
        } else if ch == '-' || ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if ch.is_alphanumeric() {
            slug.extend(ch.to_lowercase());
        }
        // everything else (punctuation) is dropped
    }
    slug
}

/// Every heading anchor slug defined in a Markdown document.
fn heading_slugs(text: &str) -> BTreeSet<String> {
    let mut slugs = BTreeSet::new();
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with('#') {
            let hashes = t.chars().take_while(|&c| c == '#').count();
            if (1..=6).contains(&hashes) {
                let rest = t[hashes..].trim();
                if !rest.is_empty() {
                    slugs.insert(heading_slug(rest));
                }
            }
        }
    }
    slugs
}

/// Check that every *relative* Markdown link in the repo's `.md` files resolves
/// to a file that exists, and that any `#anchor` pointing at a Markdown file
/// matches a real heading there. External (http, mailto) links are skipped.
/// Keeps the course's promise of precise, self-contained cross-references
/// honest as modules evolve.
fn check_doc_links(root: &Path, style: &Style) -> bool {
    print!("  documentation links … ");
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let mut files = Vec::new();
    collect_files(root, "md", &mut files);

    // Read every doc once; index heading slugs by canonical path for anchors.
    let mut contents: Vec<(PathBuf, String)> = Vec::new();
    let mut slug_index: HashMap<PathBuf, BTreeSet<String>> = HashMap::new();
    for file in &files {
        if let Ok(text) = fs::read_to_string(file) {
            if let Ok(canon) = fs::canonicalize(file) {
                slug_index.insert(canon, heading_slugs(&text));
            }
            contents.push((file.clone(), text));
        }
    }

    let mut broken: Vec<(PathBuf, String, &'static str)> = Vec::new();
    let mut checked = 0usize;
    for (file, text) in &contents {
        let base = file.parent().unwrap_or(root);
        for target in markdown_link_targets(text) {
            let lower = target.to_ascii_lowercase();
            if lower.starts_with("http://")
                || lower.starts_with("https://")
                || lower.starts_with("mailto:")
                || lower.starts_with("tel:")
            {
                continue;
            }
            let (path_part, anchor) = match target.split_once('#') {
                Some((p, a)) => (p, Some(a)),
                None => (target.as_str(), None),
            };
            let path_part = path_part.split('?').next().unwrap_or(path_part);
            checked += 1;

            // Resolve the target file: empty path means "this same file".
            let target_file = if path_part.is_empty() {
                fs::canonicalize(file).ok()
            } else {
                let resolved = base.join(path_part);
                match fs::canonicalize(&resolved) {
                    Ok(c) => Some(c),
                    Err(_) => {
                        broken.push((file.clone(), target.clone(), "missing file"));
                        continue;
                    }
                }
            };

            // Validate a #anchor only against Markdown targets we indexed.
            if let Some(anchor) = anchor {
                if anchor.is_empty() {
                    continue;
                }
                if let Some(tf) = &target_file {
                    if let Some(slugs) = slug_index.get(tf) {
                        if !slugs.contains(&anchor.to_ascii_lowercase()) {
                            broken.push((file.clone(), target.clone(), "missing anchor"));
                        }
                    }
                }
            }
        }
    }

    if broken.is_empty() {
        println!(
            "{} {}",
            style.green("✓"),
            style.dim(&format!("({} links in {} files)", checked, files.len()))
        );
        true
    } else {
        println!("{}", style.red("✗"));
        for (file, target, why) in &broken {
            let shown = file.strip_prefix(root).unwrap_or(file);
            println!(
                "    {} {} → {} ({})",
                style.red("broken:"),
                style.dim(&shown.display().to_string()),
                target,
                why
            );
        }
        false
    }
}

/// Structural invariants of the manifest against the filesystem: every stage
/// has its test file and at least one hint, and every module ships its assets
/// and a reference solution. Cheap, and it catches the pedagogical gaps that a
/// green test run would not. (The website's manifest.json mirror is guarded
/// separately by the manifest-drift CI check.)
fn check_structure(root: &Path, style: &Style) -> bool {
    print!("  course structure … ");
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let mut problems: Vec<String> = Vec::new();
    for m in MODULES {
        let cdir = root.join("course").join(m.dir);
        let ldir = root.join("labs").join(m.dir);
        for asset in ["README.md", "WALKTHROUGH.md", "exercises.md", "hints.md"] {
            if !cdir.join(asset).exists() {
                problems.push(format!("module {}: missing course/{}/{}", m.id, m.dir, asset));
            }
        }
        let reference = root
            .join("reference")
            .join("src")
            .join(format!("{}.rs", reference_stem(m)));
        if !reference.exists() {
            problems.push(format!("module {}: missing reference/src/{}.rs", m.id, reference_stem(m)));
        }
        let hints_text = fs::read_to_string(cdir.join("hints.md")).unwrap_or_default();
        for (i, stage) in m.stages.iter().enumerate() {
            let n = i + 1;
            let test = ldir.join("tests").join(format!("{}.rs", stage.test_target));
            if !test.exists() {
                problems.push(format!(
                    "module {} stage {}: missing labs/{}/tests/{}.rs",
                    m.id, n, m.dir, stage.test_target
                ));
            }
            if parse_hints(&hints_text, n).is_empty() {
                problems.push(format!(
                    "module {} stage {}: no hint in hints.md (## Stage {})",
                    m.id, n, n
                ));
            }
        }
    }
    if problems.is_empty() {
        println!("{} {}", style.green("✓"), style.dim(&format!("({} modules)", MODULES.len())));
        true
    } else {
        println!("{}", style.red("✗"));
        for p in &problems {
            println!("    {} {}", style.red("gap:"), style.dim(p));
        }
        false
    }
}

/// Course integrity check: every lab test must pass when the lab crate
/// re-exports the reference solutions, and the reference crate's own unit
/// tests (Knuth's worked examples) must pass too.
fn verify(root: &PathBuf, style: &Style, verbose: bool) -> bool {
    println!();
    println!("{}", style.bold("Course self-check: reference solutions vs. lab test suites"));
    let mut ok = true;

    ok &= check_structure(root, style);
    ok &= check_doc_links(root, style);

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
    // `verify` grades with `solutions: true`, and `grade_module` neither
    // records nor persists progress on solutions runs — so the student's
    // `.taocp/progress` is untouched by this self-check.
    println!();
    if ok {
        println!("{}", style.green("verify: every stage passes against the reference solutions."));
    } else {
        println!("{}", style.red("verify: FAILURES — the course itself is broken, see above."));
    }
    ok
}

/// Parse the graduated hints for one stage out of a module's `hints.md`.
/// Returns the hints in order (hint 1 = gentlest). Format: a `## Stage <k>`
/// heading, then lines beginning `<n>.` up to the next `##`.
fn load_hints(root: &Path, module: &Module, stage_1based: usize) -> Vec<String> {
    let path = root.join("course").join(module.dir).join("hints.md");
    match fs::read_to_string(&path) {
        Ok(text) => parse_hints(&text, stage_1based),
        Err(_) => Vec::new(),
    }
}

/// Pure hints parser (filesystem-free, so it can be unit-tested): pull the
/// numbered items under the `## Stage <k>` heading out of `text`.
fn parse_hints(text: &str, stage_1based: usize) -> Vec<String> {
    let mut in_stage = false;
    let mut hints: Vec<String> = Vec::new();
    let mut current = String::new();
    let want = format!("## stage {}", stage_1based);
    for line in text.lines() {
        let lower = line.trim_start().to_ascii_lowercase();
        if lower.starts_with("## ") {
            // A new heading: entering our stage, or leaving it.
            let entering = lower.starts_with(&want)
                && lower[want.len()..]
                    .chars()
                    .next()
                    .map(|c| !c.is_ascii_digit())
                    .unwrap_or(true);
            if in_stage && !current.trim().is_empty() {
                hints.push(current.trim().to_string());
                current.clear();
            }
            in_stage = entering;
            continue;
        }
        if in_stage {
            let t = line.trim_start();
            // A new numbered item starts a new hint: digits, a dot, then a
            // space (or end of line). Requiring the space keeps a wrapped
            // continuation line like "3.14 is the…" or "2.5x speedup…" from
            // being misread as the start of hint 3 (resp. 2).
            let starts_item = t
                .split_once('.')
                .map(|(n, rest)| {
                    !n.is_empty()
                        && n.chars().all(|c| c.is_ascii_digit())
                        && rest.chars().next().map(|c| c == ' ').unwrap_or(true)
                })
                .unwrap_or(false);
            if starts_item {
                if !current.trim().is_empty() {
                    hints.push(current.trim().to_string());
                    current.clear();
                }
                // Drop the "N." prefix.
                let rest = t.split_once('.').map(|(_, r)| r).unwrap_or("").trim_start();
                current.push_str(rest);
            } else if !current.is_empty() {
                current.push(' ');
                current.push_str(t);
            }
        }
    }
    if in_stage && !current.trim().is_empty() {
        hints.push(current.trim().to_string());
    }
    hints
}

/// Show hints for a module/stage. `which` is 1-based; None = show the first
/// and say how many more exist.
fn show_hints(root: &Path, style: &Style, module: &Module, stage: usize, which: Option<usize>) -> ExitCode {
    if stage == 0 || stage > module.stages.len() {
        eprintln!("module {} has stages 1..={}", module.id, module.stages.len());
        return ExitCode::FAILURE;
    }
    let s = &module.stages[stage - 1];
    let hints = load_hints(root, module, stage);
    println!();
    println!(
        "{} — stage {}: {}",
        style.bold(&format!("Module {} hints", module.id)),
        stage,
        style.bold(s.title)
    );
    if hints.is_empty() {
        println!("  {}", style.dim("(no hints written for this stage yet)"));
        return ExitCode::SUCCESS;
    }
    let n = hints.len();
    let show = which.unwrap_or(1).clamp(1, n);
    for (i, hint) in hints.iter().take(show).enumerate() {
        println!();
        println!("  {} {}", style.yellow(&format!("Hint {}/{}:", i + 1, n)), hint);
    }
    if show < n {
        println!();
        println!(
            "  {}",
            style.dim(&format!(
                "Need more? ./grade {} --stage {} --hint {}",
                module.id.trim_start_matches('0'),
                stage,
                show + 1
            ))
        );
    } else {
        println!();
        println!(
            "  {}",
            style.dim("That's the last hint. After the stage is green, read the WALKTHROUGH.md.")
        );
    }
    ExitCode::SUCCESS
}

/// `./grade doctor` — diagnose the environment and workspace.
fn doctor(root: &Path, style: &Style) -> ExitCode {
    println!();
    println!("{}", style.bold("Course doctor — checking your setup"));
    let mut ok = true;
    let mut check = |label: &str, good: bool, detail: &str| {
        if good {
            println!("  {} {}  {}", style.green("✓"), label, style.dim(detail));
        } else {
            ok = false;
            println!("  {} {}  {}", style.red("✗"), label, detail);
        }
    };

    // Toolchain.
    let tool_ver = |bin: &str| {
        Command::new(bin)
            .arg("--version")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    match tool_ver("cargo") {
        Some(v) => check("cargo present", true, &v),
        None => check("cargo present", false, "install Rust from https://rustup.rs"),
    }
    match tool_ver("rustc") {
        Some(v) => check("rustc present", true, &v),
        None => check("rustc present", false, "install Rust from https://rustup.rs"),
    }

    // Workspace compiles.
    print!("  {} checking workspace compiles… ", style.dim("·"));
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let build = Command::new("cargo")
        .current_dir(root)
        .args(["check", "-q", "--workspace"])
        .env("CARGO_TERM_COLOR", "never")
        .output();
    println!("\r                                      \r");
    match build {
        Ok(o) if o.status.success() => check("workspace compiles", true, "all lab stubs build"),
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            let first = err.lines().find(|l| l.contains("error")).unwrap_or("see `cargo check`");
            check("workspace compiles", false, first);
        }
        Err(e) => check("workspace compiles", false, &format!("could not run cargo: {e}")),
    }

    // Did the student accidentally edit plumbing?
    let mut edited_plumbing = Vec::new();
    for m in MODULES {
        let lib = root.join("labs").join(m.dir).join("src").join("lib.rs");
        if let Ok(t) = fs::read_to_string(&lib) {
            if !t.contains("You never need to edit this file")
                && !t.contains("you never need to edit this file")
            {
                edited_plumbing.push(m.dir);
            }
        }
    }
    check(
        "lab plumbing intact",
        edited_plumbing.is_empty(),
        &if edited_plumbing.is_empty() {
            "src/lib.rs untouched (as intended)".to_string()
        } else {
            format!("edited src/lib.rs in: {} — restore from git", edited_plumbing.join(", "))
        },
    );

    // A lab must not smuggle the reference in by default: a `default =
    // ["solutions"]` line in a lab Cargo.toml would make every stage pass
    // with the stub untouched. (Grading also passes --no-default-features,
    // but catch the edit explicitly so the student sees why.)
    let mut default_features = Vec::new();
    for m in MODULES {
        let toml = root.join("labs").join(m.dir).join("Cargo.toml");
        if let Ok(t) = fs::read_to_string(&toml) {
            let mut in_features = false;
            for line in t.lines() {
                let l = line.trim();
                if l.starts_with('[') {
                    in_features = l == "[features]";
                } else if in_features
                    && l.starts_with("default")
                    && l["default".len()..].trim_start().starts_with('=')
                {
                    default_features.push(m.dir);
                }
            }
        }
    }
    check(
        "lab features intact",
        default_features.is_empty(),
        &if default_features.is_empty() {
            "no lab enables `solutions` by default".to_string()
        } else {
            format!(
                "`default` feature declared in: {} — remove it (labs must not enable `solutions` by default)",
                default_features.join(", ")
            )
        },
    );

    // Progress file readable/writable.
    let prog_dir = root.join(".taocp");
    let writable = fs::create_dir_all(&prog_dir).is_ok();
    check(
        "progress dir writable",
        writable,
        &format!("{}", prog_dir.display()),
    );

    println!();
    if ok {
        println!("{}", style.green("doctor: everything looks healthy. Run ./grade to begin."));
        ExitCode::SUCCESS
    } else {
        println!("{}", style.red("doctor: problems found above — fix them, then re-run ./grade doctor."));
        ExitCode::FAILURE
    }
}

/// `./grade bench <module>` — run a module's growth-curve benchmark, if it
/// ships one (`labs/<dir>/examples/bench.rs`), against the reference impl.
fn bench(root: &PathBuf, style: &Style, module: &Module) -> ExitCode {
    let example = root.join("labs").join(module.dir).join("examples").join("bench.rs");
    println!();
    println!(
        "{}",
        style.bold(&format!("Benchmark — Module {}: {}", module.id, module.title))
    );
    if !example.exists() {
        println!(
            "  {}",
            style.dim("This module has no bench (not every algorithm has a growth curve to plot).")
        );
        return ExitCode::SUCCESS;
    }
    println!(
        "  {}",
        style.dim("Timing the reference implementation; compare against the lesson's asymptotics.")
    );
    println!();
    let status = Command::new("cargo")
        .current_dir(root)
        .args([
            "run",
            "-q",
            "-p",
            module.lab_crate,
            "--example",
            "bench",
            "--features",
            "solutions",
            "--release",
        ])
        .env("CARGO_TERM_COLOR", "never")
        .status();
    match status {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        _ => {
            eprintln!("bench failed to run");
            ExitCode::FAILURE
        }
    }
}

/// `./grade watch <module> [-s <stage>]` — re-grade a module every time its
/// `lab.rs` is saved. Std-only: polls the file's modified-time (no external
/// crates, no OS-specific inotify). Runs until interrupted with Ctrl-C, so it
/// never returns.
fn watch(
    root: &PathBuf,
    style: &Style,
    module: &Module,
    only_stage: Option<usize>,
    solutions: bool,
    verbose: bool,
) -> ExitCode {
    use std::time::{Duration, SystemTime};

    let lab = root.join("labs").join(module.dir).join("src").join("lab.rs");
    let mtime = |p: &Path| -> Option<SystemTime> { fs::metadata(p).and_then(|m| m.modified()).ok() };

    println!(
        "{} module {} — re-grading on every save to {}",
        style.bold("watch:"),
        module.id,
        style.dim(&format!("labs/{}/src/lab.rs", module.dir)),
    );
    println!("{}", style.dim("  edit, save, watch it re-run — Ctrl-C to stop."));

    let mut last = mtime(&lab);
    loop {
        println!();
        println!("{}", style.dim(&"─".repeat(60)));
        let mut progress = load_progress(root);
        let (passed, total) =
            grade_module(root, style, module, only_stage, solutions, verbose, &mut progress);
        let graded = if only_stage.is_some() { 1 } else { total };
        println!();
        if passed >= graded {
            report_module_pass(root, style, module, only_stage.is_some());
        }
        println!("{}", style.dim("  watching for changes…"));

        // Block until lab.rs changes on disk (or Ctrl-C ends the process).
        loop {
            std::thread::sleep(Duration::from_millis(400));
            let now = mtime(&lab);
            if now != last {
                last = now;
                break;
            }
        }
    }
}

fn main() -> ExitCode {
    let root = repo_root();
    let style = Style::new();
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut positional: Vec<String> = Vec::new();
    let mut only_stage: Option<usize> = None;
    let mut hint: Option<usize> = None;
    let mut hint_flag = false;
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
            "--hint" => {
                hint_flag = true;
                // Optional number: `--hint` or `--hint 2`.
                if let Some(v) = args.get(i + 1).and_then(|v| v.parse::<usize>().ok()) {
                    hint = Some(v);
                    i += 1;
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
        Some("doctor") => doctor(&root, &style),
        Some("manifest") => {
            emit_manifest_json();
            ExitCode::SUCCESS
        }
        Some("next") => match next_unsolved_module(&progress) {
            None => {
                println!();
                println!("{}", style.green("Every stage is complete — congratulations!"));
                ExitCode::SUCCESS
            }
            Some(m) => {
                let (p, t) = grade_module(&root, &style, m, None, solutions, verbose, &mut progress);
                println!();
                if p >= t {
                    report_module_pass(&root, &style, m, false);
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
        },
        Some("bench") => match positional.get(1).and_then(|q| find_module(q)) {
            Some(m) => bench(&root, &style, m),
            None => {
                eprintln!("usage: ./grade bench <module>   e.g. ./grade bench 6");
                ExitCode::FAILURE
            }
        },
        Some("watch") => match positional.get(1).and_then(|q| find_module(q)) {
            Some(m) => watch(&root, &style, m, only_stage, solutions, verbose),
            None => {
                eprintln!("usage: ./grade watch <module> [-s <stage>]   e.g. ./grade watch 6");
                ExitCode::FAILURE
            }
        },
        Some("hint") | Some("hints") => {
            // `./grade hint <module> <stage> [n]`
            let m = positional.get(1).and_then(|q| find_module(q));
            let st = positional.get(2).and_then(|v| v.parse::<usize>().ok());
            match (m, st.or(only_stage)) {
                (Some(m), Some(st)) => {
                    show_hints(&root, &style, m, st, hint.or(positional.get(3).and_then(|v| v.parse().ok())))
                }
                _ => {
                    eprintln!("usage: ./grade hint <module> <stage> [n]   e.g. ./grade hint 6 3");
                    ExitCode::FAILURE
                }
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
                // `./grade 6 --stage 3 --hint` short-circuits to hints.
                if hint_flag {
                    let st = only_stage.unwrap_or(1);
                    return show_hints(&root, &style, m, st, hint);
                }
                let (p, t) =
                    grade_module(&root, &style, m, only_stage, solutions, verbose, &mut progress);
                let graded = if only_stage.is_some() { 1 } else { t };
                println!();
                if p >= graded {
                    report_module_pass(&root, &style, m, only_stage.is_some());
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

/// Emit the course manifest as JSON on stdout, grouped by TAOCP volume in
/// curriculum order. The website consumes this to generate the course sidebar
/// and the interactive course map, so those can never drift from this single
/// source of truth. Accepts (and ignores) a trailing `--json` for readability.
fn emit_manifest_json() {
    // Volume display names, keyed by the prefix of `Module::source`.
    fn volume_name(key: &str) -> &'static str {
        match key {
            "Vol. 1" => "Fundamental Algorithms",
            "Vol. 2" => "Seminumerical Algorithms",
            "Vol. 3" => "Sorting and Searching",
            "Vol. 4A" => "Combinatorial Algorithms, Part 1",
            "Vol. 4B" => "Combinatorial Algorithms, Part 2",
            "Toward Vol. 4C" => "Pre-fascicles",
            _ => "Other",
        }
    }
    // The volume key is the source text up to the first comma.
    fn volume_key(source: &str) -> &str {
        source.split(',').next().unwrap_or(source).trim()
    }

    // Group modules by volume, preserving first-appearance (= manifest) order,
    // which reproduces the canonical shelf ordering (Vol 1, 2, 3, 4A, 4B, 4C).
    let mut order: Vec<&str> = Vec::new();
    let mut groups: HashMap<&str, Vec<&Module>> = HashMap::new();
    for m in MODULES {
        let key = volume_key(m.source);
        if !order.contains(&key) {
            order.push(key);
        }
        groups.entry(key).or_default().push(m);
    }

    let mut out = String::from("{\n  \"volumes\": [\n");
    for (vi, key) in order.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"key\": {},\n", json_str(key)));
        out.push_str(&format!("      \"name\": {},\n", json_str(volume_name(key))));
        out.push_str("      \"modules\": [\n");
        let mods = &groups[key];
        for (mi, m) in mods.iter().enumerate() {
            out.push_str("        {\n");
            out.push_str(&format!("          \"id\": {},\n", json_str(m.id)));
            out.push_str(&format!("          \"dir\": {},\n", json_str(m.dir)));
            out.push_str(&format!("          \"title\": {},\n", json_str(m.title)));
            out.push_str(&format!("          \"source\": {},\n", json_str(m.source)));
            out.push_str("          \"stages\": [\n");
            for (si, s) in m.stages.iter().enumerate() {
                out.push_str("            { ");
                out.push_str(&format!("\"title\": {}, ", json_str(s.title)));
                out.push_str(&format!("\"algorithm\": {}, ", json_str(s.algorithm)));
                out.push_str(&format!("\"test_target\": {} ", json_str(s.test_target)));
                out.push('}');
                out.push_str(if si + 1 < m.stages.len() { ",\n" } else { "\n" });
            }
            out.push_str("          ]\n        }");
            out.push_str(if mi + 1 < mods.len() { ",\n" } else { "\n" });
        }
        out.push_str("      ]\n    }");
        out.push_str(if vi + 1 < order.len() { ",\n" } else { "\n" });
    }
    out.push_str("  ]\n}\n");
    print!("{out}");
}

/// Minimal JSON string escaping (the course carries zero dependencies, so no serde).
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn print_help() {
    println!(
        "\
The TAOCP-in-Rust course grader.

USAGE:
    ./grade                    show progress across all modules
    ./grade <module>           grade a module stage by stage (e.g. ./grade 3)
    ./grade <module> -s <n>    grade a single stage
    ./grade watch <module>     re-grade a module every time you save its lab.rs
    ./grade next               jump to the module with your next unsolved stage
    ./grade all                grade every module
    ./grade verify             self-check: run all lab tests against the
                               built-in reference solutions
    ./grade hint <m> <stage>   show a graduated hint (add a number for the next)
    ./grade bench <module>     run a module's growth-curve benchmark
    ./grade doctor             diagnose your toolchain and workspace
    ./grade manifest           print the course structure as JSON (for the website)
    ./grade reset              clear recorded progress

FLAGS:
    --stage, -s <n>    run only stage n of the chosen module
    --hint [n]         show hint n for the chosen stage (with -s), gentlest first
    --solutions        run lab tests against the reference solutions
    --verbose, -v      show full cargo output on failure

EXAMPLES:
    ./grade 6                  start Module 06 (sorting)
    ./grade 6 -s 3 --hint      stuck on stage 3? get a nudge
    ./grade 6 -s 3 --hint 2    the next, more specific hint
    ./grade watch 6            re-grade on every save while you work
    ./grade bench 6            watch the sorts' growth curves
"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_stem_maps_dir_to_file() {
        let m = Module {
            id: "11",
            dir: "module-11-btree-trie",
            lab_crate: "lab-11-btree-trie",
            title: "T",
            source: "S",
            stages: &[],
        };
        assert_eq!(reference_stem(&m), "m11_btree_trie");
    }

    #[test]
    fn every_manifest_module_maps_to_a_known_stem() {
        // Guards the dir → mNN_slug convention against a typo in the manifest.
        for m in MODULES {
            let stem = reference_stem(m);
            assert!(stem.starts_with('m') && stem.contains('_'), "bad stem: {stem}");
        }
    }

    #[test]
    fn markdown_links_are_extracted() {
        let t = "see [a](x.md) and [b](../y.md#h) but not a bare (paren).";
        assert_eq!(markdown_link_targets(t), vec!["x.md", "../y.md#h"]);
        assert!(markdown_link_targets("no links here").is_empty());
    }

    #[test]
    fn parse_hints_does_not_split_on_decimal_continuations() {
        // A wrapped line starting with a decimal number ("3.14 is…") must not
        // be misread as the start of hint 3.
        let text = "## Stage 1\n1. First hint about pi:\n3.14159 is the constant you want.\n2. Second hint.\n";
        let hints = parse_hints(text, 1);
        assert_eq!(hints.len(), 2, "continuation line split a hint: {hints:?}");
        assert!(hints[0].contains("3.14159"));
        assert!(hints[1].starts_with("Second"));
    }

    #[test]
    fn parse_passed_count_reads_summary_lines() {
        assert_eq!(parse_passed_count("test result: ok. 7 passed; 0 failed; 0 ignored"), Some(7));
        assert_eq!(parse_passed_count("test result: ok. 0 passed; 0 failed"), Some(0));
        assert_eq!(parse_passed_count("running 7 tests"), None);
    }

    #[test]
    fn find_module_rejects_numeric_junk() {
        // "./grade 0" must not grade module 01 via a dir-substring match, and
        // "-1" must not pick up module 11.
        assert!(find_module("0").is_none());
        assert!(find_module("-1").is_none());
        assert_eq!(find_module("6").map(|m| m.id), Some("06"));
        assert_eq!(find_module("06").map(|m| m.id), Some("06"));
        assert_eq!(find_module("sort").map(|m| m.id), Some("06"));
        assert_eq!(find_module("module-01-algorithms").map(|m| m.id), Some("01"));
        assert_eq!(find_module("module-01").map(|m| m.id), Some("01"));
    }

    #[test]
    fn heading_slugs_follow_github_rules() {
        assert_eq!(heading_slug("Pacing and difficulty"), "pacing-and-difficulty");
        assert_eq!(heading_slug("5. Reading Knuth's notation"), "5-reading-knuths-notation");
        assert_eq!(heading_slug("The `todo!()` convention"), "the-todo-convention");
        assert_eq!(heading_slug("A — B"), "a--b"); // em-dash dropped, two spaces → two hyphens
    }

    #[test]
    fn heading_slugs_collects_all_levels() {
        let doc = "# Top\n\nprose\n\n## Sub One\n\n### Deep\n";
        let slugs = heading_slugs(doc);
        assert!(slugs.contains("top"));
        assert!(slugs.contains("sub-one"));
        assert!(slugs.contains("deep"));
    }

    #[test]
    fn parse_hints_pulls_numbered_items_for_the_stage() {
        let md = "\
## Stage 1: A\n\n1. first hint.\n2. second hint.\n\n## Stage 2: B\n\n1. other stage.\n";
        let h1 = parse_hints(md, 1);
        assert_eq!(h1.len(), 2);
        assert_eq!(h1[0], "first hint.");
        assert_eq!(h1[1], "second hint.");
        let h2 = parse_hints(md, 2);
        assert_eq!(h2, vec!["other stage."]);
        assert!(parse_hints(md, 3).is_empty());
    }

    #[test]
    fn next_unsolved_is_first_gap_then_none_when_complete() {
        let empty = BTreeSet::new();
        assert_eq!(next_unsolved_module(&empty).map(|m| m.id), Some("01"));
        let mut all = BTreeSet::new();
        for m in MODULES {
            for s in m.stages {
                all.insert(stage_key(m, s.test_target));
            }
        }
        assert!(next_unsolved_module(&all).is_none());
    }
}
