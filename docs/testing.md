# Testing and verification gate

How Grand-Coach makes sure nothing with failing checks reaches `main`
(D-30). One script, `scripts/check.sh`, defines the checks; git hooks, CI
and agents only call it, so they cannot drift apart.

## Enforcement layers

| Layer | Runs | Tier | Binding? |
|-------|------|------|----------|
| Claude Code hooks (`.claude/settings.json`) | agent sessions, local and CI | – | activate the git hooks; block `--no-verify` and changing `core.hooksPath` |
| `pre-commit` (`.githooks/`) | every commit | `quick` | local feedback; can be skipped by humans |
| `pre-push` (`.githooks/`) | every push | `full` | local feedback; can be skipped by humans |
| `implement-issue.yml` | after the agent, before the PR | `full` | red gate → draft PR with a warning |
| CI `ci.yml`, job `ci-ok` | every PR and push to `main` | `full` + MSRV, Linux and Windows | **required status check on `main`; the only hard gate** |
| CI `ci.yml`, job `deep` | weekly and on demand | `deep` | scheduled-run failure mails the maintainer |

Hooks are fast feedback, not security: anyone can skip them. Merge
protection rests on the required check `ci-ok`, which also covers the
agent in GitHub Actions, where no developer PC exists. Changes to the gate
itself (`scripts/check.sh`, `.githooks/`, workflows) are visible in review;
`implement-issue.yml` runs the gate from the base commit so the agent's
branch cannot weaken it.

Activate the git hooks once per clone (Claude Code does this automatically):

```sh
git config core.hooksPath .githooks
```

The hooks check the working tree, not exactly the staged or pushed
snapshot; CI checks the real commit.

## Tiers of `scripts/check.sh`

- `quick` (seconds): whitespace and SPDX headers of staged files; rustfmt
  and clippy only when Rust files are staged.
- `full` (default): whitespace and SPDX of the whole tree, rustfmt, clippy
  and `cargo test`, each with and without `--all-features`, all `--locked`.
- `deep`: the seeded random games with `SKAT_RANDOM_GAMES` games (default
  1,000,000) in release mode.

## Test levels

| Level | Here | Where | Tier |
|-------|------|-------|------|
| Unit | functions and types inside a crate | `#[cfg(test)]` modules | full |
| Component (service) | one crate through its public API, fixed deals | `crates/*/tests/` | full |
| Interface / contract | serde round-trips, determinism (seed → deal, replay), later `skat-app` commands/events/view models as frozen JSON | `tests/serde.rs`, golden files | full |
| System | whole games end to end: 10,000 seeded random games against an independent rule model; later AI-vs-AI series and a GUI smoke run | `tests/random_games.rs` | full (10k), deep (1M) |

New crates follow the same pattern; anything that takes minutes belongs in
`deep`, everything else in `full`.

## Verification vs. validation

- **Verification** ("built right"): everything above. Automated, binding.
- **Validation** ("right thing built"): does the coach help a learner, does
  the AI feel right, are the rules the ones players expect. Not automatable;
  done by the human PR review (D-27) against the issue's intent and by
  playtesting against each milestone's "Done when" in
  [roadmap.md](roadmap.md). Measurable parts move into verification once
  they exist (e.g. AI strength levels via AI-vs-AI series in `deep`, M4).

## Later (CD)

With the first executable (M2): CI builds it on Linux and Windows. With
M8: tagged releases build installable packages after `full` passes. No
deployment before that.
