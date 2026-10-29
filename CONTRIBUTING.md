# Contributing to Grand-Coach

Thanks for your interest in contributing!

## Before you start

- Read the [Code of Conduct](CODE_OF_CONDUCT.md). It applies to all
  participation.
- Get an overview: [docs/vision.md](docs/vision.md) (goals and architecture),
  [docs/roadmap.md](docs/roadmap.md) (where the project stands) and
  [docs/decisions.md](docs/decisions.md) (what has already been decided).
- For larger changes, please open an issue first so we can discuss the idea.

## Language

The repository language is English (code, comments, commits, docs, PRs).
Skat terms stay German; see the [glossary](docs/glossary.md).

## Development setup

Install Rust stable (minimum 1.88) via [rustup](https://rustup.rs). From the
repository root, run before every pull request:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --all-features
```

For documentation-only changes, `git diff --check` is enough. New Rust
dependencies must have a compatible licence and be listed in
[docs/third-party.md](docs/third-party.md).

## Branching

- `main` is the only long-lived branch.
- Work on a feature branch created from `main`, e.g. `feat/bidding-values`
  or `fix/null-ouvert-scoring`.
- Changes reach `main` only through a pull request; no direct pushes.

## Commit messages

We use [Conventional Commits](https://www.conventionalcommits.org/):
`type(scope): description`, scope optional.

| Type | Use for |
|------|---------|
| `feat` | a new feature |
| `fix` | a bug fix |
| `docs` | documentation only |
| `style` | formatting, no logic change |
| `refactor` | code change that neither fixes a bug nor adds a feature |
| `test` | adding or correcting tests |
| `chore` | build, tooling, dependencies, repository maintenance |

Example: `docs: add roadmap with milestones`

## Pull requests

- Keep PRs small and focused; review your own diff first.
- Describe what changes and why; reference issues where they exist.
- If a change affects a decision, the vision or the roadmap, update the
  matching file in `docs/` in the same PR.

## Automated implementation (Claude)

Maintainers can let Claude implement a single issue (D-27): Actions →
"Implement issue" → enter the issue number, or
`gh workflow run implement-issue.yml -f issue=<n>`. Claude works on the
branch `claude/issue-<n>` and opens a regular PR to `main`, which is reviewed
like any other PR. Requires the repository secret `CLAUDE_CODE_OAUTH_TOKEN`
(created with `claude setup-token`).

## Licences

By contributing you agree that code is licensed under the EUPL-1.2 and other
content under CC BY-SA 4.0; see the licence section of the
[README](README.md#licence) for the details, including SPDX headers and
third-party files.
