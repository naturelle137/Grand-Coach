# Testing and verification gate

How Grand-Coach makes sure nothing with failing checks reaches `main`
(D-30). One script, `scripts/check.sh`, defines the checks; git hooks, CI
and agents only call it, so they cannot drift apart.

## Enforcement layers

| Layer | Runs | Tier | Binding? |
|-------|------|------|----------|
| Claude Code hooks (`.claude/settings.json`) | agent sessions, local and CI | – | activate the git hooks; block `--no-verify` and changing `core.hooksPath` |
| `pre-commit` (`.githooks/`) | every commit | `quick` | local feedback; can be skipped by humans |
| `pre-push` (`.githooks/`) | every push | `full --base` | local feedback; can be skipped by humans |
| `implement-issue.yml` | after the agent, before the PR | `full --base` | red gate → draft PR with a warning |
| CI `ci.yml`, job `ci-ok` | every PR and push to `main` | `lint`, `test` on Linux and Windows, MSRV | **required status check on `main`; the only hard gate** |
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

- `lint` (seconds, no compilation): whitespace, SPDX headers, rustfmt,
  markdownlint (`.markdownlint-cli2.yaml`), shellcheck, actionlint.
- `test`: clippy and `cargo test`, each with and without `--all-features`,
  all `--locked`.
- `quick` (pre-commit): `lint`, plus clippy when code files are staged.
- `full` (pre-push, default): `lint` and `test`. With `--base REV`, `test`
  is skipped when no code file changed since REV.
- `deep`: the seeded random games with `SKAT_RANDOM_GAMES` games (default
  1,000,000) in release mode.

**Code files** (changes trigger `test`): `*.rs`, `Cargo.toml`,
`Cargo.lock`, `rust-toolchain*`, `assets/`, `scripts/check.sh` and
`.github/workflows/ci.yml`. Docs-only changes run only `lint`. CI decides
this in the job `changes` instead of a `paths:` filter, so `ci-ok` is
always reported and never blocks a docs PR.

Linters missing locally are skipped with a warning; CI sets
`CHECK_STRICT=1`, where a missing linter fails. Local install (Fedora):
`sudo dnf install nodejs ShellCheck` (markdownlint runs via `npx`),
actionlint from its [releases](https://github.com/rhysd/actionlint/releases).

## Test levels

| Level | Here | Where | Tier |
|-------|------|-------|------|
| Static analysis | rustfmt, clippy, markdown, shell and workflow linters | `scripts/check.sh` | lint, test |
| Unit | functions and types inside a crate | `#[cfg(test)]` modules | test |
| Component (service) | one crate through its public API, fixed deals | `crates/*/tests/` | test |
| Interface / contract | serde round-trips, determinism (seed → deal, replay), later `skat-app` commands/events/view models as frozen JSON | `tests/serde.rs`, golden files | test |
| System | whole games end to end: 10,000 seeded random games against an independent rule model; later AI-vs-AI series and a GUI smoke run | `tests/random_games.rs` | test (10k), deep (1M) |

New crates follow the same pattern; anything that takes minutes belongs in
`deep`, everything else in `test`.

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
