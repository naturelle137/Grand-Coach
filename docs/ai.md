# Opponent AI: `skat-ai`

Strength levels of M4 (D-07, D-40): profiles, information boundary, the
calibration protocol and its results. Details are in the code
(`crates/skat-ai`).

## Calibration protocol

Written down before the first validation run. The tuning suite had
already been used for pilot runs at that point; the validation suite had
not.

- **Runner**: `cargo run --locked --release -p skat-ai --example
  calibrate -- <suite> <A> <B> <deals> <parallel> <first>`, headless,
  with the real engine, analysis and opponents (`skat_ai::arena`).
- **Rules**: ISkO with Kontra/Re on (D-03, D-26).
- **Suites**: deal seeds come from `ChaCha8Rng` streams with separate
  keys for tuning and validation, so the two never share deals. Tuning
  uses only the tuning suite. Each validation attempt uses a fresh block
  of validation deals (attempt 1: deals 0–199, attempt 2: 200–399, …);
  every attempt is reported.
- **Opponent seeds**: from a third key mixed with suite, deal index, game
  and seat; never derived from a deal seed.
- **Schedule**: each deal is played six times (`arena::SEATINGS`): A
  alone on Vorhand, Mittelhand and Hinterhand against two copies of B,
  then B alone on each seat. Each profile holds each seat, and so each
  dealer position, equally often, both as the lone player and as a pair
  of defenders.
- **Metric**: Seeger-Fabian points per seat-game, Einpassen included
  with 0 points for every seat. Per deal, D = mean of A's 9 seat-games −
  mean of B's 9 seat-games. Declarer and defender games, win rates and
  points per role are reported as supporting information.
- **Sample size**: 200 deals (1,200 games) for each adjacent pair (Easy
  vs. Medium, Medium vs. Strong) on the validation suite.
- **Uncertainty**: deals are independent; standard error of the mean of D
  over deals and a 95 % interval (normal approximation, mean ± 1.96
  standard errors).
- **Pass criterion**: for each adjacent pair, the 95 % interval of
  stronger − weaker lies entirely above 0. Otherwise the result is
  reported as inconclusive (or reversed) and calibration continues on the
  tuning suite before a new validation attempt.

## Levels

| Level | Before card play | Card play, Kontra/Re | Budget |
|-------|------------------|----------------------|--------|
| Easy | rules of thumb | rules of thumb | no search |
| Medium | rules of thumb | `analyze` | 6 samples, 10⁸ nodes per task |
| Strong | rules of thumb | `analyze` | `Budget::INTERACTIVE` (24 samples) |

- The rules of thumb are `BaselineOpponent`: the shared continuation
  policy of `skat-analysis` (`policy`) plus simple card rules.
- An analysed decision picks the option with the highest utility: the
  seat's expected Seeger-Fabian points; a defender also counts half of the
  declarer's expected game points (so Kontra and denying Schneider count).
  Only options evaluated on the most distributions compete. Exact ties go
  to the rules-of-thumb action (a 0.5-point bonus), since double-dummy
  values only resolve scoring classes.
- `Profile` also supports selection noise (`temperature`) and analysed
  bidding and declaring. The shipped levels use neither: on the tuning
  suite, analysed bidding passed too often (the double-dummy defenders are
  perfect, so the analysis is pessimistic for the declarer; M3 itself
  rates pre-play estimates as coarse, D-39), and noise only weakened play.
- No level forgets observed history: every analysed decision uses the full
  view, as `skat-analysis` does.

## Information boundary and determinism

- `Opponent::decide(view, cancel)` takes one seat's `PlayerView` (D-23).
  Its analysis input (`Opponent::analysis_input`) is that view, the
  profile's budget and a seed mixed from the opponent's seed and public
  facts of the view.
- `skat-app` draws opponent seeds from stream 1 of the series seed, the
  deals from stream 0; the arena uses separate keys. No opponent gets a
  deal seed.
- Same view, opponent seed and profile give the same decision, unless a
  `Cancel` stops the analysis. Fallback: with partial results the
  finished distributions decide; without any estimate (cancelled before
  the first distribution, exhausted node budget, inconsistent view) the
  rules of thumb decide. Every choice is a legal action of the view.
- The frontend computes opponent moves on a background thread
  (`Session::opponent_job`, `OpponentJob::run`, `Session::finish_opponent`);
  a move for an outdated position is rejected. Starting a new series
  cancels the computation. There is no deadline, so series stay
  reproducible.

Tests: `crates/skat-ai/tests/fairness.rs` swaps concealed cards (hands,
Skat, also in Hand and Ouvert games) while the deciding seat's view stays
identical and checks equal analysis inputs and decisions for every level,
including a known own Skat and revealed Ouvert cards;
`tests/levels.rs` covers profiles, reproducibility, complete games and the
fallbacks.

## Calibration results

Validation attempt 1 on 2026-10-29, deals 0–199 of the validation suite
(Easy vs. Strong: 0–99), reference hardware of
[analysis.md](analysis.md#latency-target-and-benchmark), 10 deals in
parallel. Differences in Seeger-Fabian points per seat-game, stronger
minus weaker:

| Pair | Deals | Difference | Std. error | 95 % interval | Verdict |
|------|-------|------------|------------|---------------|---------|
| Medium − Easy | 200 | +16.18 | 2.26 | [+11.74, +20.62] | pass |
| Strong − Medium | 200 | +6.76 | 1.75 | [+3.33, +10.19] | pass |
| Strong − Easy | 100 | +22.65 | 3.28 | [+16.23, +29.08] | (supporting) |

Supporting results per profile (each pair: 588 declarer and 1,176
defender seat-games per profile; 24 of 1,200 games thrown in):

| Pair: profile | Points/seat-game | Declarer won | Points/declarer game | Defender won | Points/defender game |
|---------------|------------------|--------------|----------------------|--------------|----------------------|
| Easy (vs. Medium) | 8.83 | 62.2 % | 4.0 | 28.7 % | 11.5 |
| Medium (vs. Easy) | 25.01 | 75.5 % | 49.8 | 33.5 % | 13.4 |
| Medium (vs. Strong) | 22.84 | 71.9 % | 50.1 | 24.7 % | 9.9 |
| Strong (vs. Medium) | 29.61 | 77.0 % | 69.6 | 26.3 % | 10.5 |

Both levels bid by the same rules, so seatings with equal bidding produce
equal declarers; the steps come from card play and Kontra/Re. Wall time:
4 min (Easy–Medium), 23 min (Medium–Strong), 9 min (Easy–Strong).

Tuning runs before (tuning suite, 30–60 deals each) compared analysed
bidding, analysed declaring, 4/6/24 samples and temperature 4; they are
not part of the evidence above.

Reproduce:

```sh
cargo run --locked --release -p skat-ai --example calibrate -- validation easy medium 200 10 0
cargo run --locked --release -p skat-ai --example calibrate -- validation medium strong 200 10 0
cargo run --locked --release -p skat-ai --example calibrate -- validation easy strong 100 10 0
```

## Latency

`cargo run --locked --release -p skat-ai --example latency [positions]`
times `Opponent::decide` on the benchmark positions of
[analysis.md](analysis.md#latency-target-and-benchmark) (reference
hardware, quiet machine). Decisions by rules of thumb take well under
1 ms. Analysed decisions, 30 positions per kind:

| Level | Decision | Median | p90 | Max |
|-------|----------|--------|-----|-----|
| Medium | Kontra | 0.13 s | 0.51 s | 1.5 s |
| Medium | Card play, trick 1 | 0.25 s | 1.1 s | 4.0 s |
| Medium | Card play, trick 4 / 8 | 4 ms / 1 ms | 10 ms / 1 ms | 19 ms / 2 ms |
| Strong | Kontra | 0.33 s | 2.5 s | 3.5 s |
| Strong | Card play, trick 1 | 0.74 s | 2.2 s | 5.4 s |
| Strong | Card play, trick 4 / 8 | 6 ms / 1 ms | 21 ms / 1 ms | 47 ms / 2 ms |

All levels meet the M3 target (card play and Kontra/Re: median ≤ 1 s,
p90 ≤ 3 s). With only the first 10 positions per kind, Strong's first
trick had a median of 1.08 s (p90 2.1 s), slightly above the target; the
larger sample is reported as the measurement. Every decision completed.
