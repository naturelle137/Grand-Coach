#!/usr/bin/env bash
# SPDX-License-Identifier: EUPL-1.2
# Verification gate shared by git hooks, CI and agents (D-30, docs/testing.md).
#
# Usage: scripts/check.sh [quick|full|deep]
#   quick  pre-commit: whitespace and SPDX of the staged files, rustfmt and
#          clippy (cargo steps only when Rust files are staged)
#   full   pre-push and CI (default): whitespace and SPDX of the whole tree,
#          rustfmt, clippy and all tests, with and without features
#   deep   scheduled CI: the seeded random games with SKAT_RANDOM_GAMES games
#          (default 1,000,000), release build
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

tier=${1:-full}
empty_tree=$(git hash-object -t tree /dev/null)

step() {
    printf '\n==> %s\n' "$*"
    "$@"
}

# Every source file needs an SPDX header in its first lines (AGENTS.md).
spdx() {
    local missing=() f
    while IFS= read -r f; do
        [ -f "$f" ] || continue
        head -n 5 "$f" | grep -q 'SPDX-License-Identifier' || missing+=("$f")
    done < <("$@" -- '*.rs' '*.toml' '*.sh' '*.py' '*.yml' '*.yaml' '.githooks/*')
    if [ ${#missing[@]} -gt 0 ]; then
        printf 'missing SPDX header: %s\n' "${missing[@]}" >&2
        return 1
    fi
}

cargo_lint() {
    step cargo fmt --check
    step cargo clippy --locked --all-targets -- -D warnings
    step cargo clippy --locked --all-targets --all-features -- -D warnings
}

case $tier in
    quick)
        step git diff --cached --check
        step spdx git diff --cached --name-only --diff-filter=d
        if git diff --cached --name-only | grep -qE '\.rs$|(^|/)Cargo\.(toml|lock)$'; then
            cargo_lint
        fi
        ;;
    full)
        step git diff --check "$empty_tree"
        step spdx git ls-files
        cargo_lint
        step cargo test --locked
        step cargo test --locked --all-features
        ;;
    deep)
        export SKAT_RANDOM_GAMES=${SKAT_RANDOM_GAMES:-1000000}
        step cargo test --locked --release -p skat-core --test random_games
        ;;
    *)
        echo "usage: $0 [quick|full|deep]" >&2
        exit 2
        ;;
esac
printf '\n==> check %s passed\n' "$tier"
