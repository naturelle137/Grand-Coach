# Didactic coach work packages

Status: planned, 2026-10-31. Concept and implementation comparison:
[coach-didactics.md](coach-didactics.md). Tracking feature:
[#39](https://github.com/naturelle137/grand-coach/issues/39).

## Assignments and dependencies

Each issue is a bounded Claude Code or Codex assignment with its own
acceptance criteria, relevant files, exclusions and prerequisites. S means
one focused PR; M means one agent and normally one or two focused PRs,
not a promised elapsed time. Split an M package further if its inspected
implementation requires a broad architectural change.

| Package | Feature request | Outcome | Size | New-package prerequisites |
|---------|-----------------|---------|------|---------------------------|
| A | [#41](https://github.com/naturelle137/grand-coach/issues/41) | Compact debriefing, estimates off by default, honest fallback | S | None |
| B | [#42](https://github.com/naturelle137/grand-coach/issues/42) | Shared evidence/provenance contract, no new strategy engine | M | None |
| C | [#43](https://github.com/naturelle137/grand-coach/issues/43) | Two exact lessons: remaining Trumpf and proven voids | M | B |
| D | [#44](https://github.com/naturelle137/grand-coach/issues/44) | Two narrow card-play mechanisms with conditions/counterexamples | M | B, C |
| E | [#45](https://github.com/naturelle137/grand-coach/issues/45) | Pre-play structure and legal-value lessons | M | B |
| F | [#46](https://github.com/naturelle137/grand-coach/issues/46) | One optional focus, staged hints and teaching-based moment selection | M | A, C, D |
| G | [#47](https://github.com/naturelle137/grand-coach/issues/47) | Historical own hand and current trick in moment cards | M | A, B |
| H | [#48](https://github.com/naturelle137/grand-coach/issues/48) | Optional retrieval questions and explicit assistance levels | M | C, F; M6 quiz APIs |
| I | [#49](https://github.com/naturelle137/grand-coach/issues/49) | Local assistance-aware descriptive progress | M | B, H; coordinate M8 storage |
| J | [#50](https://github.com/naturelle137/grand-coach/issues/50) | Twelve legal practice cases and varied unaided transfer | M | D, G, H, I; M7/M8 foundations |
| K | [#51](https://github.com/naturelle137/grand-coach/issues/51) | Expert/learner validation protocol and empty result templates | S | None to prepare; later features to execute |

Start A, B and the protocol part of K independently. Merge B before C/E;
merge A/B before G. Then D, followed by F. H/I/J follow the existing
counting, replay and training prerequisites. E can proceed separately from
the card-play track. API consumers wait until their prerequisite has
merged; concurrent agents should not invent competing public contracts.

The inspected local D-49 commit `9eda0d9` supplies staged hints and move
reviews and was ahead of inspected main `edc0db0`.
[PR #38](https://github.com/naturelle137/grand-coach/pull/38) merged during
preparation; this documentation was rebased onto main `f733d70`. The
prerequisite is now available. Reinspect its merged APIs before coding.

## Delivery slices

1. **Readable review (A):** immediate relief from the screenshot's overload.
   No invented replacement explanations; existing comparisons stay in details.
2. **Teaching foundation (B/C/D/F/G; E separately):** exact lessons, supported
   mechanisms, focus and understandable historical context. Engineering
   readiness needs these tests; educational validation still needs humans.
3. **Independent practice (H/I/J):** selective retrieval, local observations
   and new tasks without help, after the milestone-owned foundations exist.
4. **Validation (K protocol, then human execution):** prepare early, review
   actual lesson versions and exercise independent transfer when available.

A package's PR should demonstrate the changed player experience, include
its independent fixtures, record the actual gate result and update the
relevant coach/analysis/decision docs. GUI packages include screenshots per
D-45. New lesson definitions need positive, boundary and counterexample
cases, not just snapshots of the implementation's own output.

## Existing issue ownership

| Existing issue | Keeps ownership | New work relationship |
|----------------|-----------------|-----------------------|
| [#10 M5](https://github.com/naturelle137/grand-coach/issues/10) | Live coach lifecycle and remaining interactive smoke tests | A–G extend didactics; none declares M5 complete |
| [#11 M6](https://github.com/naturelle137/grand-coach/issues/11) | Counting aid and baseline in-game quiz | C supplies lesson evidence; H adds retrieval/fading through that API |
| [#12 M7](https://github.com/naturelle137/grand-coach/issues/12) | Replay/save/load, open mode and same-deal attempts | G adds immediate permitted snapshots; J reuses reconstruction |
| [#13 M8](https://github.com/naturelle137/grand-coach/issues/13) | Baseline learning history/training and release packaging | I/J refine progress/practice; reuse storage/session APIs, no parallel subsystem |
| [#34](https://github.com/naturelle137/grand-coach/issues/34), [#35](https://github.com/naturelle137/grand-coach/issues/35) | Optional learned evaluators | Independent; neither a prerequisite nor an explanation source by itself |

Do not silently rewrite or close existing milestone issues. At implementation
start reconcile delivered APIs and overlaps. New checklists track only the
additional outcomes; parent completion still requires its full existing scope.

## Shared acceptance boundaries

- Analysis computes all strategic facts/mechanisms; the coach only selects
  and verbalises structured IDs/parameters. No separate counting or search.
- Frontends use `skat-app`; plain serde-capable data, Fluent German/English.
- Preserve D-49 before live moves, including implicit recommendation leaks.
- Historical debriefing uses the information available at the decision,
  even after actual hidden cards become available elsewhere.
- Preserve seeds, budgets, cancellation, stale-result rejection, warnings
  and ordinary play. Repeated help requests do not restart search.
- Record tests through `scripts/check.sh`; tick only verified work.
- Distinguish automated verification, expert review and observed learning.
  K can close after protocol delivery; #39's human-validation item stays
  open until execution. Agents must not fabricate that evidence.
