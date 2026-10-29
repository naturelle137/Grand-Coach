#!/usr/bin/env bash
# SPDX-License-Identifier: EUPL-1.2
# Claude Code PreToolUse hook (Bash): agents must not skip or redirect the
# git hooks of the verification gate (D-30). Exit code 2 blocks the call.
input=$(cat)
block() {
    echo "Blocked: $1. Fix the failing check instead; see docs/testing.md." >&2
    exit 2
}
if grep -qE -- '--no-verify|git +commit[^"|;&]* -[a-zA-Z]*n[a-zA-Z]*( |")' <<<"$input"; then
    block "do not skip git hooks (--no-verify / commit -n)"
fi
if grep -qE -- '(-c +|config +(--(local|global|system|worktree|unset|unset-all|replace-all) +)*)core\.hooksPath' <<<"$input" &&
    ! grep -qE -- 'core\.hooksPath[ =]+\.githooks' <<<"$input"; then
    block "core.hooksPath must stay .githooks"
fi
exit 0
