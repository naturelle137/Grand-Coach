// SPDX-License-Identifier: EUPL-1.2

//! Fairness and coverage of every strength level and of the Goldfish
//! (M4; Mephisto sees every card by design and is tested in `variants.rs`):
//!
//! - changing the concealed cards while the deciding seat's view stays the
//!   same leaves the analysis inputs and the decision unchanged, including
//!   positions with a known Skat (own discard) and revealed Ouvert cards,
//! - every level decides every phase legally: Reizen, Hand or pickup,
//!   Drücken, Ansage (Hand and Ouvert games), Kontra, Re and card play.

use std::collections::BTreeSet;

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};
use skat_ai::{BaselineOpponent, Level, Opponent, Profile};
use skat_core::{Action, Card, CardSet, Deal, Game, Phase, PlayerView, RuleConfig, Seat};

const RULES: RuleConfig = RuleConfig { kontra_re: true };

/// A game history steered towards rare decisions: Hand, Ouvert, Kontra
/// and Re are chosen often; everything else follows the rules of thumb.
fn history(seed: u64) -> (Deal, Vec<Action>) {
    let mut game = Game::new(seed, RULES);
    let deal = original_deal(&game);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut actions = Vec::new();
    while let Some(seat) = game.current_seat() {
        let view = game.view(seat);
        let legal = &view.legal_actions;
        let coin = rng.next_u32().is_multiple_of(2);
        let ouvert = legal
            .iter()
            .copied()
            .rfind(|a| matches!(a, Action::Announce(x) if x.ouvert));
        let action = match view.phase {
            Phase::SkatAufnahme if coin => Action::Hand,
            Phase::Ansage if coin && ouvert.is_some() => ouvert.unwrap(),
            Phase::Spiel if coin && legal.contains(&Action::Kontra) => Action::Kontra,
            Phase::Spiel if coin && legal.contains(&Action::Re) => Action::Re,
            _ => BaselineOpponent.decide(&view).unwrap(),
        };
        game.apply(action).unwrap();
        actions.push(action);
    }
    (deal, actions)
}

fn original_deal(game: &Game) -> Deal {
    let all = game.omniscient_view();
    Deal::new(all.original_hands, all.original_skat).unwrap()
}

/// The game after the first `n` actions of a history on `deal`; `None` if
/// one of them is illegal there.
fn replay(deal: Deal, actions: &[Action]) -> Option<Game> {
    let mut game = Game::from_deal(deal, RULES);
    for &a in actions {
        game.apply(a).ok()?;
    }
    Some(game)
}

/// Concealed state of a game: every hand and the current Skat.
fn concealed(game: &Game) -> ([CardSet; 3], CardSet) {
    let all = game.omniscient_view();
    (all.hands, all.skat)
}

/// Swaps cards `x` and `y` wherever they are in `set`.
fn swap(set: CardSet, x: Card, y: Card) -> CardSet {
    let pair = CardSet::single(x) | CardSet::single(y);
    if (set & pair).len() == 1 {
        (set - pair) | (pair - set)
    } else {
        set
    }
}

/// Histories that differ from `deal` and `actions` only in cards `seat`
/// cannot see: two unseen cards swap places in the deal (and in a discard
/// they are part of, which only the declarer sees). Kept only if every
/// action stays legal, the seat's view stays identical and the current
/// concealed cards differ.
fn alternatives(
    deal: Deal,
    actions: &[Action],
    seat: Seat,
    limit: usize,
) -> Vec<(Deal, Vec<Action>)> {
    let game = replay(deal, actions).unwrap();
    let view = game.view(seat);
    let mut seen = view.hand | view.own_skat.unwrap_or_default();
    seen |= view.declarer_cards.unwrap_or_default();
    for trick in &view.tricks {
        seen |= trick.cards.iter().copied().collect();
    }
    seen |= view.current_trick.cards.iter().flatten().copied().collect();
    let unseen: Vec<Card> = (CardSet::ALL - seen).into_iter().collect();
    let mut out = Vec::new();
    for (i, &x) in unseen.iter().enumerate() {
        for &y in &unseen[i + 1..] {
            let hands = Seat::ALL.map(|s| swap(deal.hand(s), x, y));
            let alt = Deal::new(hands, swap(deal.skat(), x, y)).unwrap();
            let alt_actions: Vec<Action> = actions
                .iter()
                .map(|&a| match a {
                    Action::Discard(cards) => Action::Discard(swap(cards, x, y)),
                    other => other,
                })
                .collect();
            let fits = replay(alt, &alt_actions)
                .is_some_and(|g| g.view(seat) == view && concealed(&g) != concealed(&game));
            if fits {
                out.push((alt, alt_actions));
                if out.len() == limit {
                    return out;
                }
            }
        }
    }
    out
}

/// Whether `p` analyses this decision (`docs/ai.md`).
fn searched(p: Profile, view: &PlayerView) -> bool {
    p.budget.is_some()
        && match view.phase {
            Phase::Reizen => p.search_bidding,
            Phase::SkatAufnahme | Phase::Druecken | Phase::Ansage => p.search_declaring,
            _ => true,
        }
}

/// Kind of a decision, for coverage.
fn kind(view: &PlayerView) -> &'static str {
    let legal = &view.legal_actions;
    match view.phase {
        Phase::Reizen => "Reizen",
        Phase::SkatAufnahme => "SkatAufnahme",
        Phase::Druecken => "Druecken",
        Phase::Ansage => "Ansage",
        Phase::Spiel if legal.contains(&Action::Kontra) => "Kontra",
        Phase::Spiel if legal.contains(&Action::Re) => "Re",
        Phase::Spiel if view.own_skat.is_some() => "play, Skat known",
        Phase::Spiel if view.declarer_cards.is_some() && view.declarer != Some(view.seat) => {
            "play against Ouvert"
        }
        Phase::Spiel if view.contract.is_some_and(|c| c.hand) => "play in a Hand game",
        Phase::Spiel => "play",
        Phase::Beendet | Phase::Eingepasst => "none",
    }
}

const KINDS: [&str; 10] = [
    "Reizen",
    "SkatAufnahme",
    "Druecken",
    "Ansage",
    "Kontra",
    "Re",
    "play, Skat known",
    "play against Ouvert",
    "play in a Hand game",
    "play",
];

/// Checks one level on the first decision of every kind in seeded steered
/// games (`per_kind` positions per kind, `alts` alternative deals each).
fn check_level(level: Level, per_kind: usize, alts: usize) {
    check_profile(&format!("{level:?}"), level.profile(), per_kind, alts);
}

/// Like [`check_level`] for any profile.
fn check_profile(level: &str, profile: Profile, per_kind: usize, alts: usize) {
    let mut done: std::collections::BTreeMap<&str, usize> = Default::default();
    let mut swapped = BTreeSet::new();
    for seed in 0..400 {
        if KINDS
            .iter()
            .all(|k| done.get(k).copied().unwrap_or(0) >= per_kind)
        {
            break;
        }
        let (deal, actions) = history(seed);
        for n in 0..actions.len() {
            let game = replay(deal, &actions[..n]).unwrap();
            let seat = game.current_seat().unwrap();
            let view = game.view(seat);
            let k = kind(&view);
            let count = done.entry(k).or_insert(0);
            if *count >= per_kind {
                continue;
            }
            *count += 1;
            let opponent = Opponent {
                profile,
                seed: seed.wrapping_mul(7919) ^ 0x5eed,
            };
            let decision = opponent.decide(&view, None).unwrap();
            assert!(
                view.legal_actions.contains(&decision.action),
                "{level:?} seed {seed}: illegal {decision:?}"
            );
            assert!(!decision.fallback, "{level:?} seed {seed} {k}");
            // The analysis sees exactly the view, budget and seed; equal
            // inputs give equal results (deterministic, tested in
            // skat-analysis), so equal inputs and decisions are checked.
            let input = opponent.analysis_input(&view);
            assert_eq!(input.is_some(), searched(profile, &view), "{level:?} {k}");
            let forgotten = opponent.forgotten(&view);
            for (alt, alt_actions) in alternatives(deal, &actions[..n], seat, alts) {
                let other = replay(alt, &alt_actions).unwrap();
                assert_ne!(concealed(&other), concealed(&game));
                let alt_view = other.view(seat);
                assert_eq!(alt_view, view);
                assert_eq!(opponent.analysis_input(&alt_view), input);
                assert_eq!(opponent.forgotten(&alt_view), forgotten);
                assert_eq!(
                    opponent.decide(&alt_view, None),
                    Some(decision),
                    "{level:?} seed {seed} {k}"
                );
                swapped.insert(k);
            }
        }
    }
    for k in KINDS {
        assert!(done.get(k).copied().unwrap_or(0) > 0, "{level:?}: no {k}");
    }
    // Positions with nothing left to swap (e.g. late in play) are fine;
    // every kind before the last tricks has concealed cards to swap.
    for k in [
        "Reizen",
        "Druecken",
        "Kontra",
        "play, Skat known",
        "play against Ouvert",
    ] {
        assert!(swapped.contains(k), "{level:?}: never swapped at {k}");
    }
}

#[test]
fn rookie_is_fair_and_covers_every_phase() {
    check_level(Level::Rookie, 6, 3);
}

#[test]
#[ignore = "slow: CI only (docs/testing.md)"]
fn goldfish_is_fair_and_covers_every_phase() {
    check_profile("Goldfish", Profile::GOLDFISH, 1, 1);
}

#[test]
fn easy_is_fair_and_covers_every_phase() {
    check_level(Level::Easy, 6, 3);
}

#[test]
#[ignore = "slow: CI only (docs/testing.md)"]
fn medium_is_fair_and_covers_every_phase() {
    check_level(Level::Medium, 2, 2);
}

#[test]
#[ignore = "slow: CI only (docs/testing.md)"]
fn strong_is_fair_and_covers_every_phase() {
    check_level(Level::Strong, 1, 1);
}
