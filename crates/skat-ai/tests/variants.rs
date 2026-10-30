// SPDX-License-Identifier: EUPL-1.2

//! Rookie, Goldfish and Mephisto (D-41): how often the Rookie deviates
//! from the rules of thumb, what the Goldfish forgets, and that Mephisto,
//! deciding with every card known, never loses a game it declares.

use std::collections::BTreeMap;

use skat_ai::{BaselineOpponent, Level, Mephisto, Opponent, Profile};
use skat_core::{Action, Game, Phase, PlayerView, RuleConfig};

const RULES: RuleConfig = RuleConfig { kontra_re: true };

/// Every decision of seeded games played by the rules of thumb.
fn positions(seeds: std::ops::Range<u64>) -> Vec<PlayerView> {
    let mut out = Vec::new();
    for seed in seeds {
        let mut game = Game::new(seed, RULES);
        while let Some(seat) = game.current_seat() {
            let view = game.view(seat);
            game.apply(BaselineOpponent.decide(&view).unwrap()).unwrap();
            out.push(view);
        }
    }
    out
}

fn kind(view: &PlayerView) -> &'static str {
    match view.phase {
        Phase::Reizen => "bid",
        Phase::Druecken => "discard",
        Phase::Ansage => "announce",
        Phase::Spiel
            if view
                .legal_actions
                .iter()
                .any(|a| !matches!(a, Action::Play(_))) =>
        {
            "kontra"
        }
        Phase::Spiel => "card",
        _ => "other",
    }
}

#[test]
fn the_rookie_deviates_sometimes_but_not_always() {
    let mut seen: BTreeMap<&str, (u32, u32)> = BTreeMap::new();
    let mut bidders = (0u32, 0u32);
    for (i, view) in positions(0..300).iter().enumerate() {
        let rookie = Opponent::new(Level::Rookie, i as u64 % 7);
        let action = rookie.decide(view, None).unwrap().action;
        assert!(view.legal_actions.contains(&action));
        assert_eq!(rookie.decide(view, None).unwrap().action, action);
        let rules = BaselineOpponent.decide(view).unwrap();
        // Only choices with a real alternative count.
        let alternatives = view.legal_actions.len() > 1;
        if alternatives {
            let entry = seen.entry(kind(view)).or_default();
            entry.0 += 1;
            entry.1 += u32::from(action != rules);
        }
        if view.phase == Phase::Reizen && view.bidding.len() < 2 {
            bidders.0 += 1;
            bidders.1 += u32::from(action != rules);
        }
    }
    let rate = |k: &str| {
        let (n, d) = seen[k];
        f64::from(d) / f64::from(n)
    };
    // Cards: 12 % random legal cards, some of which match the rules.
    assert!((0.04..0.2).contains(&rate("card")), "{seen:?}");
    // Announcement: 40 % the second-best game where one covers the bid.
    assert!((0.1..0.5).contains(&rate("announce")), "{seen:?}");
    assert!((0.05..0.4).contains(&rate("discard")), "{seen:?}");
    // Bidding is misjudged in some games, never in all.
    let first_calls = f64::from(bidders.1) / f64::from(bidders.0);
    assert!((0.05..0.5).contains(&first_calls), "{bidders:?}");
}

#[test]
fn the_goldfish_forgets_only_older_tricks() {
    let goldfish = Opponent {
        profile: Profile::GOLDFISH,
        seed: 11,
    };
    let strong = Opponent::new(Level::Strong, 11);
    let mut forgot = 0;
    let mut differs = 0;
    let views: Vec<PlayerView> = positions(0..20)
        .into_iter()
        .filter(|v| v.phase == Phase::Spiel)
        .collect();
    for view in &views {
        let forgotten = goldfish.forgotten(view);
        assert!(strong.forgotten(view).is_empty());
        if view.tricks.len() < 2 {
            assert!(forgotten.is_empty());
            continue;
        }
        let older: skat_core::CardSet = view.tricks[..view.tricks.len() - 1]
            .iter()
            .flat_map(|t| t.cards)
            .collect();
        assert!(forgotten.is_subset(older));
        assert_eq!(goldfish.forgotten(view), forgotten);
        if !forgotten.is_empty() {
            forgot += 1;
        }
        if (3..7).contains(&view.tricks.len()) && differs < 1 {
            let g = goldfish.decide(view, None).unwrap();
            assert!(view.legal_actions.contains(&g.action));
            if g.action != strong.decide(view, None).unwrap().action {
                differs += 1;
            }
        }
    }
    assert!(forgot > views.len() / 4, "{forgot} of {}", views.len());
    assert_eq!(differs, 1, "forgetting never changed a card");
}

/// Default number of Mephisto games; `SKAT_MEPHISTO_GAMES` overrides it for
/// the scheduled deep run (see `docs/testing.md`).
const MEPHISTO_GAMES: u64 = 3;

fn mephisto_games() -> u64 {
    match std::env::var("SKAT_MEPHISTO_GAMES") {
        Ok(n) => n.parse().expect("SKAT_MEPHISTO_GAMES must be a number"),
        Err(_) => MEPHISTO_GAMES,
    }
}

/// Mephisto on every seat: it only takes a game it wins double-dummy, and
/// as declarer it plays that out against any defence. With every card
/// known and a node budget instead of a time limit this is exact and
/// reproducible, so a lost game is a bug, not bad luck. Only a game in
/// which the rules of thumb decided (exhausted budget) may be lost; such
/// games are counted and stay rare.
#[test]
#[ignore = "slow: CI only (docs/testing.md)"]
fn mephisto_never_loses_a_game_it_declares() {
    let games = mephisto_games();
    let (mut declared, mut fallbacks) = (0, 0);
    for seed in 0..games {
        let mut game = Game::new(seed, RULES);
        let mut fallback = false;
        while let Some(seat) = game.current_seat() {
            let d = Mephisto.decide(&game, None).unwrap();
            fallback |= d.fallback;
            assert!(game.view(seat).legal_actions.contains(&d.action));
            game.apply(d.action).unwrap();
        }
        if let Some(result) = game.result() {
            declared += 1;
            assert!(result.won || fallback, "seed {seed}: {result:?}");
        }
        fallbacks += u64::from(fallback);
        assert_eq!(Mephisto.decide(&game, None), None);
    }
    eprintln!("{games} Mephisto games: {declared} declared, {fallbacks} with a fallback");
    assert!(declared > 0);
    // At most 1 in 20 games, so none in the default run.
    assert!(fallbacks * 20 <= games, "{fallbacks} of {games} games");
}
