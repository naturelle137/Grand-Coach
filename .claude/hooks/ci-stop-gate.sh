#!/usr/bin/env bash
# SPDX-License-Identifier: EUPL-1.2
# Claude Code Stop hook, active only in the implement-issue workflow (D-27):
# the agent must not finish while work is uncommitted, nothing was committed
# or the gate is red. Exit code 2 sends stderr back to Claude, which then
# continues. After MAX_BLOCKS refusals the agent may stop; the workflow then
# commits the leftovers and opens a draft PR.
set -uo pipefail
[ "${GITHUB_ACTIONS:-}" = "true" ] && [ -n "${BASE_SHA:-}" ] && [ -n "${RUNNER_TEMP:-}" ] || exit 0
cat > /dev/null # hook input is not needed

MAX_BLOCKS=5
counter="$RUNNER_TEMP/stop-gate-blocks"
blocks=$(cat "$counter" 2>/dev/null || echo 0)
if [ "$blocks" -ge "$MAX_BLOCKS" ]; then
    exit 0
fi
block() {
    echo $((blocks + 1)) > "$counter"
    {
        echo "Not done yet (stop gate, attempt $((blocks + 1)) of $MAX_BLOCKS):"
        printf '%s\n' "$@"
        echo "Fix this and finish again. Wait for every command to complete;"
        echo "do not run checks in the background."
    } >&2
    exit 2
}

cd "$(git rev-parse --show-toplevel)" || exit 0

if [ -n "$(git status --porcelain)" ]; then
    block "- Uncommitted changes in the working tree:" \
        "$(git status --short | head -n 20)" \
        "  Commit them. If the pre-commit hook fails, read its output, fix the" \
        "  lint or clippy findings and commit again. Never skip the hooks."
fi

if [ "$(git rev-parse HEAD)" = "$BASE_SHA" ]; then
    if [ -s "$RUNNER_TEMP/no-change-reason.md" ]; then
        exit 0
    fi
    block "- No commits since $BASE_SHA. Implement the issue and commit. If it" \
        "  genuinely needs no change, explain why in \$RUNNER_TEMP/no-change-reason.md" \
        "  (public: it is posted to the issue)."
fi

log="$RUNNER_TEMP/stop-gate-check.log"
if ! scripts/check.sh push --base "$BASE_SHA" > "$log" 2>&1; then
    block "- scripts/check.sh push --base $BASE_SHA fails; last lines:" \
        "$(tail -n 40 "$log")" \
        "  Fix the cause (never weaken a check or test) and commit the fix."
fi

missing=()
for f in pr-title.txt pr-body.md; do
    [ -s "$RUNNER_TEMP/$f" ] || missing+=("\$RUNNER_TEMP/$f")
done
if [ ${#missing[@]} -gt 0 ]; then
    block "- Missing: ${missing[*]} (see the CI notes in your prompt)."
fi
exit 0
