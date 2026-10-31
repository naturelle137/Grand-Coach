# Live coach: `skat-coach`

How the live coach of M5 explains a decision (D-48). Details are in the
code (`crates/skat-coach`, `crates/skat-app/src/coach.rs`).

## Evidence only from the analysis

`skat_coach::explain(view, analysis)` turns one `Analysis` of the human's
`PlayerView` into a structured `Explanation` (D-13); `advise` runs the
analysis and the explanation in one call. The coach ranks, groups and
labels; it computes no strategy, counts no cards and models no opponent.

| Part | Source in the analysis | Kind |
|------|------------------------|------|
| Considerations `Known` | `observations` (voids, trumps out, Augen, own Skat, bids, Kontra eligibility) | exact fact |
| Considerations `Inferred` | `inferences`: expected trumps (Jacks before the announcement) per hidden place, with min/max | estimate; `certain` only if the hard constraints prove it |
| Options, values | `options[].estimate`, ranked by `skat_analysis::utility` (the opponents' utility) | estimate with standard error |
| Reasons | differences between two estimates | estimate |
| Assumptions, basis | `assumptions`, `effort` | model |

Win chances are those of the deciding seat's party; the coach labels the
party (own side, declarer, defenders). Points are Seeger-Fabian list
points and, for defenders, the declarer's game points; Augen are never
estimated. Null and Schwarz objectives come from the contract.

`skat-analysis` gained `inferences` for M5: the opponent model's
distribution of trumps over the hidden places, from the same weighted
distributions as the estimates.

## Policies (D-48)

- **Equivalent**: within 1 point (`TIE_POINTS`) or within the noise margin,
  2 combined standard errors of the expected score (`MARGIN_SE`).
- **Weak**: at least 10 Seeger-Fabian points (`WEAK_POINTS`) below the best
  option *and* beyond the noise margin. Card play and Kontra/Re estimates
  have standard errors of about 3–4 points, so weak means a clear loss
  there; before card play (17–20 points, `analysis.md`) only gross
  mistakes qualify, which matches what 24 samples can tell apart.
- Drücken options dropped after screening are `Screened`, never weak.
- **Important decision**: the analysis is complete, at least two options
  are compared, and a *natural* option is weak. Natural: every option
  except jump bids above the next bid value and Schneider/Schwarz
  announcements.
- **Warning**: only with a complete analysis and a weak chosen action.

## No telling ahead (D-49)

Before the move a frontend gets `Hints` (`skat_coach::hints`), never the
ranked `Explanation`. Each `RequestAdvice` reveals one more `Stage`:

1. `Considerations`: objective, sure facts, likely inferences.
2. `Situation`: the legal options in the view's order and a `Situation`
   (`Forced`, `OneStandsOut`, `SeveralEqual`, `Unclear` when the analysis
   is incomplete) plus the important-decision mark. No option is named as
   better.
3. `Figures`: every option's estimate in the same neutral order, no
   judgement; only with "show figures" (otherwise stage 2 is the last).

After the move `review` judges the chosen option; `MoveReview::note` is the
short note (`Good`, `AboutEqual`, `SomewhatWeaker`, `ClearlyWeaker`,
`Forced`, `Uncertain`). At the end of the game `moments` picks from the
complete, not forced reviews at most `BAD_MOMENTS` (3) worse or weak
choices losing at least `MOMENT_POINTS` (5 Seeger-Fabian points), by
expected loss, and at most `GOOD_MOMENTS` (2) important
decisions decided best or equivalent, by the loss of the costliest natural
weak option avoided (the `trap`); they are shown in game order with the
better option, reasons, figures and what was known then. The warning
(`Warning`) carries only the chosen option and its loss.

## Session lifecycle (`skat-app`)

- Levels (D-08, D-49): **off** (no hints, notes or debriefing, no help
  action), **on request** (default; each `RequestAdvice` one stage),
  **important** (stage 2 on its own when important, help action too),
  **every move** (stage 1 on its own). Warning off by default (D-09),
  independent of the level; "show figures" hides estimates but keeps the
  reasoning and exact facts.
- With the level not off, every own decision is analysed for the note
  (`CoachView::last_move`) and the debriefing (`CoachView::debriefing`,
  once the game is over). A decision played before its analysis finished
  is queued with its view; `coach_job` hands out the current decision
  first, then the oldest queued one (`CoachJob::is_review`), and the
  frontend cancels a running review when the current decision needs an
  analysis. A cancelled review stays queued; the next game drops the
  queue. `Session::coach_result` exposes the full analysis for tools and
  tests only.
- `Session::coach_job` hands out the analysis of the human's view when one
  is due; the frontend runs it off the UI thread. `finish_coach` accepts it
  only for the same decision (game number, actions applied) and budget, or
  for a queued one. Cancelled or failed analyses show as
  unavailable, asking again recomputes; an analysis out of budget shows as
  provisional without recommendation, importance or warning.
- Warning flow: with warnings on, `Act` holds the action until the
  analysis is there (`Checking`; "play without check" proceeds), then plays
  it or holds it with a `Warning`. `ProceedAnyway` plays it exactly once;
  `Reconsider` withdraws it, and choosing it again at the same position is
  not warned twice. Turning warnings off withdraws a held action. Nothing
  held is ever played behind the player's back.
- The coach seed comes from stream 2 of the series seed, mixed with the
  game number and the view; opponents and deals are untouched. Same view,
  seed and budget give the same explanation.

## Latency

The coach runs one `analyze` with `Budget::INTERACTIVE` per human
decision; `explain` is a single pass over the options. Its latency is the
analysis latency, with the interactive target and reference measurement
in [analysis.md](analysis.md#latency-target-and-benchmark).
`skat-app/examples/coach_latency.rs` measures it as the frontend sees it:
a session with the coach on "every move", time from `coach_job` to an
accepted `finish_coach`, the human following the coach's best option,
Medium opponents, Kontra/Re on, series seeds 1, 2, …, 10 positions per
kind:

```sh
cargo run --locked --release -p skat-app --example coach_latency
```

Recorded on 2026-10-30 on a GitHub Actions runner (AMD EPYC 7763,
4 vCPUs, 16 GB; series seeds 1–71, about 45 minutes), not on the
reference hardware with 20 threads:

| Decision | Median | p90 | Max | Incomplete |
|----------|--------|-----|-----|------------|
| Reizen | 5.5 s | 9.9 s | 26.6 s | 1 |
| Hand/pickup | 3.3 s | 4.5 s | 5.0 s | 0 |
| Drücken | 15.9 s | 36.2 s | 42.0 s | 0 |
| Announcement | 6.8 s | 8.5 s | 12.8 s | 0 |
| Kontra/Re | 0.6 s | 0.9 s | 3.0 s | 0 |
| Card play, trick 1 | 0.7 s | 3.9 s | 14.7 s | 0 |
| Card play, trick 4 | 10 ms | 38 ms | 135 ms | 0 |
| Card play, trick 8 | 1 ms | 1 ms | 1 ms | 0 |

Card play and Kontra/Re meet the interactive target even on four
vCPUs; the decisions before card play exceed it there. The analysis
spreads its work over all available threads, so fewer threads mean
longer waits. The positions also differ from the analysis benchmark:
every bidding decision counts, not only opening calls, and the human
bids as the coach suggests. While the coach computes, the panel shows a translated
"thinking" status and the player can act at any time; the analysis then
continues for the note, unless the next own decision needs one first. The GUI test `window_stays_responsive_while_the_coach_analyses`
checks that frames keep returning quickly during a full interactive
analysis and that acting works meanwhile. The interactive smoke test in
the playable application on the reference hardware is still open
(issue #10).
