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
- `docs/testing.md`: test levels and the verification gate (D-30)
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

## Architecture (crates; `skat-coach` follows in M5)

```text
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
- **Green gate only.** Never commit or push with a failing
  `scripts/check.sh`, never bypass the git hooks (`--no-verify`, changing
  `core.hooksPath`), never weaken checks or tests to get green.
- Rule questions are decided by the ISkO; Kontra/Re follows D-03. No Ramsch,
  no Bock.

## Build and test

All checks run through one script (D-30, `docs/testing.md`):

```sh
scripts/check.sh          # full gate: lint + test
scripts/check.sh lint     # whitespace, SPDX, rustfmt, markdownlint, shellcheck, actionlint
scripts/check.sh test     # clippy and tests, with and without features
scripts/check.sh deep     # 1,000,000 seeded random games (weekly in CI)
```

The git hooks in `.githooks/` run `quick` before each commit and `full`
before each push (tests only when code files changed); Claude Code
activates them via `.claude/settings.json`. CI always runs `lint`, and
`test` on Linux and Windows plus an MSRV check when code changed; its job
`ci-ok` is required to merge into `main`. Locally missing linters are
skipped with a warning; CI runs them all. `cargo test` includes 10,000 seeded random
games (`crates/skat-core/tests/random_games.rs`). For documentation
changes, also check relative links (not automated).
Document new commands here and in `CONTRIBUTING.md` only once verified.

Local Fedora 44 Distrobox setup is documented in `CONTRIBUTING.md` and was
verified with Rust 1.99.1. `cargo fetch --locked` prepares the dependency
cache; `cargo build --locked --offline`, `cargo test --locked --offline`
and `cargo test --locked --offline --all-features` also pass locally.

The desktop app is the binary `grand-coach` of `crates/gui-egui`
(verified on Linux/Wayland; Windows 11 not yet, see `docs/testing.md`):

```sh
cargo run --locked --release -p gui-egui      # start the game
GRAND_COACH_SEED=42 cargo run -p gui-egui     # replay a series seed
cargo run --locked -p cardgen                 # regenerate assets/cards/
```

On Linux the window needs EGL/GL and Wayland or X11 client libraries
(Fedora: `mesa-libEGL mesa-libGL libwayland-egl libxkbcommon`). After
changing `tools/cardgen`, regenerate and commit `assets/cards/`; a test
fails otherwise.

## Testing

- Rule engine and analysis are test-driven; rule cases get unit tests with
  fixed deals and known positions.
- The rule-engine milestone also requires thousands of seeded random games
  without rule violations (see `docs/roadmap.md`).

## Conventions

- Branching, Conventional Commits and PRs: see `CONTRIBUTING.md` (D-17).
  Feature branches from `main`, changes only via PR. Update affected vision,
  roadmap and decision documents in the same PR.
- In local sessions: once changes pass `scripts/check.sh`,
  commit them (Conventional Commits) and push the feature branch. Never
  commit failing changes, never push to `main`.
- To implement an issue, follow the skill
  `.claude/skills/implement-issue/SKILL.md` (D-29). Tick an issue's
  checklist items only when they are done and verified.
- In CI via `.github/workflows/implement-issue.yml` (D-27): the workflow
  runs that skill; follow its CI notes and do not push, open the PR or edit
  the issue yourself.
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
