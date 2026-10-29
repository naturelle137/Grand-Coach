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
| Interface / contract | serde round-trips, determinism (seed → deal, replay); `skat-app` commands, events, view models and errors as JSON | `skat-core/tests/serde.rs`, `skat-app/tests/series.rs` | test |
| Analysis | card counting, voids, visibility and hidden-card distributions on fixed positions; double-dummy against hand-solved positions and an exhaustive minimax; every decision type, information isolation, reproducibility, cancellation, Überreizen, Kontra/Re, Monte Carlo against exact enumeration | `skat-analysis/tests/` | test |
| Performance | latency and estimation quality of the analysis per decision phase ([analysis.md](analysis.md#latency-target-and-benchmark)) | `skat-analysis/examples/bench.rs` | manual |
| AI | strength profiles, information isolation by swapping concealed cards for every level, reproducibility, fallback on cancellation and exhausted budgets, complete games; background opponent moves through `skat-app` | `skat-ai/tests/`, `skat-ai/src/arena.rs`, `skat-app/tests/series.rs` | test |
| Calibration | AI-vs-AI strength ordering and decision latency per level ([ai.md](ai.md)) | `skat-ai/examples/calibrate.rs`, `skat-ai/examples/latency.rs` | manual |
| System | whole games end to end: 10,000 seeded random games against an independent rule model; complete seeded 36-game series with a random human, information boundary of the human's view, opponent self-play | `skat-core/tests/random_games.rs`, `skat-app/tests/`, `skat-app/src/tests.rs`, `skat-ai` | test (10k), deep (1M) |
| GUI | the egui frontend driven headlessly by clicks (`egui_kittest`): series by clicks, every decision and game variant clicked at least once, table elements shown, language and card-face switch, translated errors, every card face renders; translation completeness; committed card SVGs match `tools/cardgen` | `gui-egui/tests/gui.rs`, `gui-egui/src/i18n.rs`, `tools/cardgen/tests/` | test |
| Manual smoke run | launch the window on each platform and play (see `CONTRIBUTING.md`) | – | per milestone |

New crates follow the same pattern; anything that takes minutes belongs in
`deep`, everything else in `test`.

## Platform verification (M2)

Evidence that the first playable version builds and runs. A platform
counts as verified only with an interactive smoke test: launch, play a
series to the score sheet, switch language and card faces.

| Platform | Build | Launch | Interactive smoke test |
|----------|-------|--------|------------------------|
| Linux (Fedora 44 Distrobox, Wayland, Rust 1.99.1) | ✅ `cargo build --locked --release -p gui-egui`, 2026-10-29 | ✅ window opens and renders the first deal (`GRAND_COACH_SCREENSHOT`), 2026-10-29 | ✅ 2026-10-29 |
| Windows | ✅ CI job `test (windows-latest)` (Windows Server) builds everything and passes all tests, including the headless GUI tests, 2026-10-29 | ❌ not verified on Windows 11 | ❌ not verified on Windows 11; open by D-36 |

## Verification vs. validation

- **Verification** ("built right"): everything above. Automated, binding.
- **Validation** ("right thing built"): does the coach help a learner, does
  the AI feel right, are the rules the ones players expect. Not automatable;
  done by the human PR review (D-27) against the issue's intent and by
  playtesting against each milestone's "Done when" in
  [roadmap.md](roadmap.md). Measurable parts move into verification once
  they exist (e.g. AI strength levels via the AI-vs-AI calibration runner,
  M4, [ai.md](ai.md)).

## Later (CD)

Since M2, `test` compiles and tests the frontend on Linux and Windows, so
CI builds the executable on both. With M8: tagged releases build
installable packages after `full` passes. No deployment before that.
