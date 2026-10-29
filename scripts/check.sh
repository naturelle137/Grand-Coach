#!/usr/bin/env bash
# SPDX-License-Identifier: EUPL-1.2
# Verification gate shared by git hooks, CI and agents (D-30, docs/testing.md).
#
# Usage: scripts/check.sh [TIER] [--base REV]
#   lint   whitespace, SPDX headers, rustfmt, markdownlint, shellcheck,
#          actionlint (seconds, no compilation)
#   test   clippy and all tests, each with and without --all-features
#   quick  pre-commit: lint, plus clippy when code files are staged
#   full   pre-push (default): lint and test
#   deep   scheduled CI: SKAT_RANDOM_GAMES seeded random games (default
#          1,000,000), release build
#   code-changed REV  exit 0 if code files changed since REV, 1 if not
#
# With --base REV, `full` skips `test` when no code file changed since REV.
# Linters missing locally are skipped with a warning; with CHECK_STRICT=1
# (set in CI) a missing linter fails.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

tier=${1:-full}
base=
if [ "${2:-}" = --base ]; then
    base=${3:?--base needs a revision}
fi

# Files whose change requires compiling and testing (docs/testing.md).
code_re='\.rs$|(^|/)Cargo\.(toml|lock)$|^rust-toolchain|^assets/|^scripts/check\.sh$|^\.github/workflows/ci\.yml$'
markdownlint_version=0.23.3

step() {
    printf '\n==> %s\n' "$*"
    "$@"
}

# Code changed since the merge base with REV, committed or not? An unknown
# REV counts as changed.
code_changed() {
    local mb files
    mb=$(git merge-base "$1" HEAD 2>/dev/null) || return 0
    files=$(git diff --name-only "$mb" HEAD && git diff --name-only HEAD)
    grep -qE "$code_re" <<<"$files"
}

have() {
    command -v "$1" > /dev/null && return 0
    if [ "${CHECK_STRICT:-}" = 1 ]; then
        echo "error: $1 is not installed" >&2
        exit 1
    fi
    echo "warning: $1 is not installed; skipped here, CI runs it" >&2
    return 1
}

# Every source file needs an SPDX header in its first lines (AGENTS.md).
spdx() {
    local missing=() f
    while IFS= read -r f; do
        [ -f "$f" ] || continue
        head -n 5 "$f" | grep -q 'SPDX-License-Identifier' || missing+=("$f")
    done < <(git ls-files -- '*.rs' '*.toml' '*.sh' '*.py' '*.yml' '*.yaml' '.githooks/*')
    if [ ${#missing[@]} -gt 0 ]; then
        printf 'missing SPDX header: %s\n' "${missing[@]}" >&2
        return 1
    fi
}

markdown() {
    local globs=('**/*.md' '#target')
    if command -v markdownlint-cli2 > /dev/null; then
        markdownlint-cli2 "${globs[@]}"
    elif have npx; then
        npx --yes "markdownlint-cli2@$markdownlint_version" "${globs[@]}"
    fi
}

shell_scripts() {
    local files
    mapfile -t files < <(git ls-files -- '*.sh' '.githooks/*')
    if have shellcheck; then
        shellcheck "${files[@]}"
    fi
}

workflows() {
    if have actionlint; then
        actionlint
    fi
}

lint() {
    step git diff --check "$(git hash-object -t tree /dev/null)"
    step spdx
    step cargo fmt --check
    step markdown
    step shell_scripts
    step workflows
}

clippy() {
    step cargo clippy --locked --all-targets -- -D warnings
    step cargo clippy --locked --all-targets --all-features -- -D warnings
}

run_tests() {
    clippy
    step cargo test --locked
    step cargo test --locked --all-features
}

case $tier in
    lint)
        lint
        ;;
    test)
        run_tests
        ;;
    quick)
        lint
        if git diff --cached --name-only | grep -qE "$code_re"; then
            clippy
        fi
        ;;
    full)
        lint
        if [ -z "$base" ] || code_changed "$base"; then
            run_tests
        else
            printf '\n==> no code changes since %s; tests skipped\n' "$base"
        fi
        ;;
    deep)
        export SKAT_RANDOM_GAMES=${SKAT_RANDOM_GAMES:-1000000}
        step cargo test --locked --release -p skat-core --test random_games
        ;;
    code-changed)
        code_changed "${2:?code-changed needs a revision}"
        exit
        ;;
    *)
        echo "usage: $0 [lint|test|quick|full|deep] [--base REV] | code-changed REV" >&2
        exit 2
        ;;
esac
printf '\n==> check %s passed\n' "$tier"
