# AGENTS.md – Grand-Coach

Canonical guide for coding agents (Codex, Claude Code and others).
`CLAUDE.md` only imports this file; edit agent instructions here.

Skat trainer for PC (Linux, Windows 11): Skat per ISkO against two AI
opponents, plus an optional coach that explains the line of thought (What to
think about? Which options? How do they compare?). Focus: card counting and
putting yourself in the opponents' shoes.

## Memory – read first
- `docs/vision.md`: goals, features, scope, architecture
- `docs/architecture.md`: design decisions inside `skat-core`
- `docs/decisions.md`: decision log (D-xx) and open points
- `docs/roadmap.md`: milestones and current status (update after each milestone)
- `docs/third-party.md`: origin and licence of third-party files
- `docs/glossary.md`: German Skat terms → English
- `CONTRIBUTING.md`: branching, commits, pull requests

New fundamental decisions go into `docs/decisions.md` as a new D entry.
Keep all documents short.

## Language
- **Repository language is English**: code, comments, commit messages, docs,
  PR titles and descriptions.
- **Chat with the user in German.**
- `README.md`: German first, English section second.
- Skat terms stay German in code and docs; see `docs/glossary.md`.

## Tech stack
Rust (Cargo workspace), GUI with egui. Later additionally Godot via
godot-rust/gdext (D-11).

## Architecture (crates; only `skat-core` exists so far)
```
crates/skat-core      rule engine: pure, deterministic (seed), no IO, no GUI
crates/skat-analysis  card counter, opponent model, option evaluation (Monte Carlo, double-dummy)
crates/skat-ai        opponent AI with strength levels
crates/skat-coach     structured explanations (text keys + parameters), no texts
crates/skat-app       application interface: session, commands, events, view models
crates/gui-egui       egui frontend
tools/cardgen         generates card SVGs into assets/cards/
```
Dependencies only point downwards: core ← analysis ← ai/coach ← app ← gui.
Translations live in `assets/i18n/`.

## Hard rules
- **Coach and AI do not compute separately.** Both use `skat-analysis`; the
  coach only verbalises what was computed there.
- **Separate information sets.** AI and coach get one player's view, never the
  full state. Exceptions (open mode, post-game analysis) are visible in the
  type system.
- **Frontends use `skat-app` only.** No egui or Godot types below `gui-*`.
  API types are plain data (serde-capable) so Godot can attach later without
  core changes.
- **No user-facing text in logic.** German/English texts live in translation
  files under `assets/i18n/`.
- **Determinism.** All randomness goes through a passed-in seed/RNG so tests,
  replays and "same deal again" work.
- Rule questions are decided by the ISkO; Kontra/Re follows D-03. No Ramsch,
  no Bock.

## Build and test
Verified commands (Rust stable, Cargo workspace at the repository root):

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --all-features   # includes the serde round-trip tests
```

`cargo test` includes 10,000 seeded random games
(`crates/skat-core/tests/random_games.rs`). For documentation changes, run
`git diff --check` and check relative links.
Document new commands here and in `CONTRIBUTING.md` only once verified.

Local Fedora 44 Distrobox setup is documented in `CONTRIBUTING.md` and was
verified with Rust 1.99.1. `cargo fetch --locked` prepares the dependency
cache; `cargo build --locked --offline`, `cargo test --locked --offline`
and `cargo test --locked --offline --all-features` also pass locally.
There is no executable or GUI target yet.

## Testing
- Rule engine and analysis are test-driven; rule cases get unit tests with
  fixed deals and known positions.
- The rule-engine milestone also requires thousands of seeded random games
  without rule violations (see `docs/roadmap.md`).

## Conventions
- Branching, Conventional Commits and PRs: see `CONTRIBUTING.md` (D-17).
  Feature branches from `main`, changes only via PR. Update affected vision,
  roadmap and decision documents in the same PR.
- In CI via `.github/workflows/implement-issue.yml` (D-27): follow the
  workflow prompt; do not push or open the PR yourself.
- Every source file starts with the SPDX header:
  ```rust
  // SPDX-License-Identifier: EUPL-1.2
  ```
  For scripts with `#` comments: `# SPDX-License-Identifier: EUPL-1.2`.
- Generated or drawn content under `assets/` and `docs/` is CC BY-SA 4.0
  (`LICENSE-ASSETS`). SVG files get an `SPDX-License-Identifier: CC-BY-SA-4.0`
  comment where practical.
- Third-party graphics or libraries only with a compatible licence. Record
  every third-party file in `docs/third-party.md` before committing it. No NC
  or ND licences, no files of unclear origin.
