#!/usr/bin/env bash
# One-time setup for Codespaces / Dev Containers: after this runs, the
# student's first command can simply be `./grade`.
set -euo pipefail

# The safety hook that keeps Drive handles/credentials out of git
# (docs/knuth-agents/SETUP.md step 4).
git config core.hooksPath githooks

# Pre-build the grader so the first `./grade` answers in seconds instead of
# compiling. Checking the workspace also proves the stubs compile.
cargo build -q -p grader
cargo check -q --workspace

echo
echo "Ready. Try:  ./grade          (course status)"
echo "       then: ./grade 1        (start Module 01)"
echo "       and:  ./grade watch 1  (re-grade on every save)"
