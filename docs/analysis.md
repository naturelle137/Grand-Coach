# Analysis: `skat-analysis`

How the shared analysis of M3 evaluates decisions (D-37, D-38). AI (M4)
and coach (M5) use it; neither computes strategy of its own. Details are
in the code (`crates/skat-analysis`).

## Information boundary

- `analyze(view, budget, seed, cancel)` takes one seat's `PlayerView` and
  nothing else (D-23). It cannot reach the real hidden cards.
- Sampled deals (`World`) and fully specified card-play positions
  (`double_dummy::Position`) are hypothetical simulation inputs.
- The only bridges from a real game to full knowledge are
  `Position::from_omniscient(&OmniscientView)` and
  `World::from_omniscient(&OmniscientView)`, for open mode, post-game
  analysis (M7) and the Mephisto opponent (D-41). Their names and argument
  type make every use visible.
- `analyze_known(view, world, budget, cancel)` evaluates every option in
  one given distribution, exactly. `analyze_forgetful(view, forgotten,
  budget, seed, cancel)` is `analyze` for a seat that does not remember
  where some played cards went (Goldfish, [ai.md](ai.md#goldfish)).
- Results are plain data (serde behind the feature `serde`): estimates,
  structured observations, inferences and assumptions, no texts (D-13).
- `inferences`: per hidden place (other seat, unknown Skat) the weighted
  mean, minimum and maximum number of trumps (Jacks before the
  announcement; none in Null) over the finished distributions; `certain`
  only when the hard constraints prove the count. The coach shows them as
  estimates ([coach.md](coach.md)).
- `utility(view, estimate)` is the value opponents choose by and the coach
  ranks by: own expected Seeger-Fabian points, for a defender minus half
  the declarer's expected game points.

## Knowledge (card counter)

Exact facts from the view (`Knowledge`): own cards, played cards, Augen
won per seat, trumps out, the own discard (declarer only, after Drücken),
the Ouvert declarer's cards (after the reveal) and voids proven by not
following suit (the whole follow class: Trumpf or one side suit).

## Hidden cards (opponent model)

- **Hard constraints** (never violated): every unseen card in exactly one
  place, hand and Skat sizes, voids. The Skat is a place of its own; it
  is known only to a declarer who discarded. While the declarer holds
  twelve cards there is no Skat place.
- **Uniform sampling**: `HiddenCards` counts all consistent distributions
  exactly (dynamic programming over groups of cards with the same allowed
  places) and draws each with probability 1 / count. If there are no more
  distributions than the sample budget, all are enumerated (exact mode).
- **Bidding evidence** (heuristic, fallible): each sample is weighted by
  how well the other seats' bids fit their hypothetical cards under the
  continuation policy: factor 0.25 for a bid or hold above the policy
  limit, 0.5 for a pass below it. Weights are never zero, so evidence
  never excludes a distribution the hard constraints allow; conflicting
  evidence only multiplies down. Every estimate reports the effective
  sample size. Discards are not used as evidence yet.

## Evaluation

For every distribution and every legal option the game is played to the
end:

1. the option is the only enumerated decision,
2. later decisions before card play follow the **continuation policy**
   (`policy`, the baseline opponent's rules, shared with `skat-ai`): bid
   limit from the best game, Hand only with very strong cards, discard,
   announcement, Kontra with five trumps, Re with a strong hand. Each rule
   sees only the deciding seat's own cards in that distribution,
3. card play is **double-dummy**,
4. the result is settled by `skat-core` (`settle`, `seeger_fabian`),
   including Überreizen and Kontra/Re.

Option semantics: `Bid(v)` means "say `v` now and go on up to `v`";
`Hold` means "hold and go on up to the policy limit or the current bid";
Kontra and Re in play use the position before the next card. Later Kontra
by the other defender follows the policy.

**Double-dummy**: the declarer maximises, the defenders minimise the
declarer's game points. For a fixed contract these depend only on the
outcome class (no trick, ≤ 30, 31–60, 61–89, ≥ 90 Augen, every trick) and
never decrease from class to class, so the solver finds the class with
null-window alpha-beta searches (61, then 90 or 31), a forced-all-tricks
search and a forced-no-trick search, independent of the contract. Null:
can the declarer avoid every trick. A transposition table, equivalent
cards and a sure-tricks bound keep it fast. Tests compare it with a plain
exhaustive minimax over `settle` on small positions.

## Results

Every option of `PlayerView::legal_actions` gets an `Estimate`, from the
deciding seat's point of view:

| Field | Meaning |
|-------|---------|
| `win_chance` | probability that the seat's party wins the game (thrown in: not won) |
| `expected_score` | expected Seeger-Fabian points of the seat for this game (0 if thrown in) |
| `expected_game_points` | expected declarer list score (`GameResult::score`, negative if lost, with Kontra/Re) |
| `thrown_in` | probability that all pass |
| `score_std_error`, `samples`, `exact` | uncertainty; `exact` means every distribution was enumerated |

Augen are not estimated; the solver only resolves the scoring classes.
Kontra changes only the declarer's points: a defender's Seeger-Fabian
points (40 if the declarer loses) do not depend on it, so compare
`expected_game_points` for Kontra.

`Status` is `Complete`, `BudgetExhausted` or `Cancelled` (estimates only
over finished distributions; a distribution with an interrupted task is
dropped for every option), or `NoDecision`.

**Drücken** has 66 options, each its own deal to solve: every option is
screened on 3 distributions, the best 6 are evaluated on all. `samples`
shows which options had fewer.

## Budgets and determinism

- `Budget { samples, nodes_per_task }`: a task evaluates every option on
  one distribution (Drücken: one option). Work is counted in search nodes,
  never time. Each task gets a fresh solver, so results and node counts do
  not depend on thread scheduling: same view, seed and budget give the
  same `Analysis`.
- Tasks run on all cores (`std::thread::scope`); results are combined in
  task order.
- `Cancel` stops an analysis from outside; `Cancel::with_deadline` adds a
  time limit for interactive use (then only the status is reproducible).
- `Budget::INTERACTIVE` = 24 samples, 100,000,000 nodes per task.

## Latency target and benchmark

Target with `Budget::INTERACTIVE`, release build, on the reference
hardware (Intel Core i9-10900K, 10 cores / 20 threads, 32 GB, Fedora 44
Distrobox): every benchmark analysis complete, and

| Decision | Median | p90 |
|----------|--------|-----|
| card play, Kontra/Re | ≤ 1 s | ≤ 3 s |
| Reizen, Hand/pickup, announcement | ≤ 3 s | ≤ 5 s |
| Drücken | ≤ 8 s | ≤ 10 s |

Benchmark positions: seeded games 1, 2, … (Kontra/Re on) in which the
policy decides before card play and the first legal card is played; the
first 10 positions of each kind: an opening bidding decision (first or
second call), Hand or pickup, Drücken, announcement, Kontra decision (the
benchmark games never say Kontra, so no Re decision), card play at tricks
1, 4 and 8.

```sh
cargo run --locked --release -p skat-analysis --example bench
```

Achieved on 2026-10-29 (10 positions each; `se`: median standard error of
the best option's expected score; `regret`: mean Seeger-Fabian points lost,
as valued by one run, when choosing the best option of a second run with
another seed):

| Decision | Median | p90 | Max | Incomplete | se | regret |
|----------|--------|-----|-----|------------|----|--------|
| Reizen | 0.9 s | 4 s | 4 s | 0 | 4.1 | 6.3 |
| Hand/pickup | 0.9 s | 1 s | 4 s | 0 | 19.4 | 0.0 |
| Drücken | 6 s | 8 s | 10 s | 0 | 17.0 | 15.0 |
| Announcement | 3 s | 3 s | 5 s | 0 | 19.5 | 2.2 |
| Kontra | 0.3 s | 1 s | 3 s | 0 | 2.7 | 0.0 |
| Card play, trick 1 | 0.7 s | 1 s | 3 s | 0 | 3.7 | 1.5 |
| Card play, trick 4 | 5 ms | 17 ms | 37 ms | 0 | 3.9 | 0.2 |
| Card play, trick 8 | 1 ms | 2 ms | 2 ms | 0 | 0.0 | 0.3 |

All rows meet the target. Quality: card play and Kontra/Re estimates are tight;
before card play the standard errors (17–20 points) and the Drücken
regret show that 24 samples separate only clearly different options.
More samples or a faster solver improve this; see the open points in
`decisions.md`. Monte Carlo against exact enumeration is tested in
`tests/analysis.rs` (tolerance: 4 standard errors plus 1 point).
