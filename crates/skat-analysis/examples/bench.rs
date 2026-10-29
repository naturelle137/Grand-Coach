// SPDX-License-Identifier: EUPL-1.2

//! Latency benchmark of [`skat_analysis::analyze`] for every decision phase
//! (see `docs/analysis.md`). Positions come from seeded games in which the
//! continuation policy and double-dummy card play decide every seat.
//!
//! ```sh
//! cargo run --locked --release -p skat-analysis --example bench
//! ```
//!
//! Optional arguments: number of deals per phase (default 10) and the
//! sample budget (default `Budget::INTERACTIVE`).

use std::time::{Duration, Instant};

use skat_analysis::{Budget, Status, analyze, policy};
use skat_core::{Action, Game, Phase, PlayerView, RuleConfig};

/// Decision kinds measured.
const KINDS: [&str; 8] = [
    "Reizen",
    "SkatAufnahme",
    "Druecken",
    "Ansage",
    "Kontra/Re",
    "Spiel trick 1",
    "Spiel trick 4",
    "Spiel trick 8",
];

fn kind(view: &PlayerView) -> Option<usize> {
    let trick = view.tricks.len();
    Some(match view.phase {
        Phase::Reizen => 0,
        Phase::SkatAufnahme => 1,
        Phase::Druecken => 2,
        Phase::Ansage => 3,
        Phase::Spiel
            if view
                .legal_actions
                .iter()
                .any(|a| matches!(a, Action::Kontra | Action::Re | Action::KeinRe)) =>
        {
            4
        }
        Phase::Spiel if trick == 0 => 5,
        Phase::Spiel if trick == 3 => 6,
        Phase::Spiel if trick == 7 => 7,
        _ => return None,
    })
}

/// A fixed, cheap move for advancing the benchmark games.
fn advance(view: &PlayerView) -> Action {
    let legal = &view.legal_actions;
    let wanted = match view.phase {
        Phase::Reizen => policy::bid(policy::choose_game(view.hand).1, view.bid, legal),
        Phase::SkatAufnahme => policy::hand_or_pickup(view.hand),
        Phase::Druecken => Action::Discard(policy::discard(view.hand)),
        Phase::Ansage => policy::announce(
            view.hand,
            view.own_skat.unwrap_or_default(),
            view.bid,
            view.hand_game == Some(true),
            legal,
        ),
        _ => legal
            .iter()
            .copied()
            .find(|a| matches!(a, Action::Play(_) | Action::KeinRe | Action::KeinKontra))
            .unwrap_or(legal[0]),
    };
    if legal.contains(&wanted) {
        wanted
    } else {
        legal[0]
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let deals: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or(10);
    let mut budget = Budget::INTERACTIVE;
    if let Some(samples) = args.next().and_then(|a| a.parse().ok()) {
        budget.samples = samples;
    }
    if let Some(nodes) = args.next().and_then(|a| a.parse().ok()) {
        budget.nodes_per_task = nodes;
    }
    let mut times: Vec<Vec<Duration>> = vec![Vec::new(); KINDS.len()];
    let mut incomplete = vec![0; KINDS.len()];
    let mut nodes: Vec<Vec<u64>> = vec![Vec::new(); KINDS.len()];
    let mut seed = 0;
    while times.iter().any(|t| t.len() < deals) && seed < 10_000 {
        seed += 1;
        let mut game = Game::new(seed, RuleConfig { kontra_re: true });
        while let Some(seat) = game.current_seat() {
            let view = game.view(seat);
            if let Some(k) = kind(&view)
                && times[k].len() < deals
                && !(k == 0 && view.bidding.len() > 1)
            {
                let start = Instant::now();
                let analysis = analyze(&view, budget, seed, None).expect("consistent view");
                times[k].push(start.elapsed());
                nodes[k].push(analysis.effort.nodes);
                if analysis.status != Status::Complete {
                    incomplete[k] += 1;
                }
            }
            game.apply(advance(&view)).unwrap();
        }
    }
    println!(
        "budget: {} samples, {} nodes per task; {} threads",
        budget.samples,
        budget.nodes_per_task,
        std::thread::available_parallelism().map_or(1, |n| n.get())
    );
    println!(
        "{:<14} {:>5} {:>9} {:>9} {:>9} {:>10} {:>12}",
        "decision", "n", "median", "p90", "max", "incomplete", "max nodes"
    );
    for (k, name) in KINDS.iter().enumerate() {
        let t = &mut times[k];
        t.sort();
        if t.is_empty() {
            println!("{name:<14} {:>5}", 0);
            continue;
        }
        let at = |q: f64| t[((t.len() - 1) as f64 * q).round() as usize];
        println!(
            "{name:<14} {:>5} {:>9.0?} {:>9.0?} {:>9.0?} {:>10} {:>12}",
            t.len(),
            at(0.5),
            at(0.9),
            t[t.len() - 1],
            incomplete[k],
            nodes[k].iter().max().copied().unwrap_or(0)
        );
    }
}
