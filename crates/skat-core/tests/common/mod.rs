// SPDX-License-Identifier: EUPL-1.2

//! Test helpers: short card notation and fixed deals.
//!
//! Notation: suit `C` Kreuz, `S` Pik, `H` Herz, `D` Karo; rank `7 8 9 T J Q K A`.

#![allow(dead_code)]

use skat_core::{Action, Card, CardSet, Deal, Event, Game, Rank, RuleConfig, Suit};

pub fn c(s: &str) -> Card {
    let mut chars = s.chars();
    let suit = match chars.next() {
        Some('C') => Suit::Kreuz,
        Some('S') => Suit::Pik,
        Some('H') => Suit::Herz,
        Some('D') => Suit::Karo,
        _ => panic!("bad suit in {s}"),
    };
    let rank = match chars.next() {
        Some('7') => Rank::Sieben,
        Some('8') => Rank::Acht,
        Some('9') => Rank::Neun,
        Some('T') => Rank::Zehn,
        Some('J') => Rank::Bube,
        Some('Q') => Rank::Dame,
        Some('K') => Rank::Koenig,
        Some('A') => Rank::Ass,
        _ => panic!("bad rank in {s}"),
    };
    assert!(chars.next().is_none(), "bad card {s}");
    Card::new(suit, rank)
}

pub fn cards(s: &str) -> CardSet {
    s.split_whitespace().map(c).collect()
}

pub fn deal(vh: &str, mh: &str, hh: &str, skat: &str) -> Deal {
    Deal::new([cards(vh), cards(mh), cards(hh)], cards(skat)).expect("valid test deal")
}

/// Standard test deal.
/// VH: all Jacks and six Kreuz; MH: Pik and three Herz; HH: Herz and Karo.
pub fn standard_deal() -> Deal {
    deal(
        "CJ SJ HJ DJ CA CT CK CQ C9 C8",
        "SA ST SK SQ S9 S8 S7 HA HT HK",
        "HQ H9 H8 H7 DA DT DK DQ D9 D8",
        "C7 D7",
    )
}

pub fn game(deal: Deal, kontra_re: bool) -> Game {
    Game::from_deal(deal, RuleConfig { kontra_re })
}

/// Applies actions, panicking with the index on the first error.
pub fn run(game: &mut Game, actions: &[Action]) -> Vec<Event> {
    let mut events = Vec::new();
    for (i, &action) in actions.iter().enumerate() {
        match game.apply(action) {
            Ok(mut e) => events.append(&mut e),
            Err(err) => panic!("action {i} {action:?} rejected: {err:?}"),
        }
    }
    events
}

/// Plays card notations in order.
pub fn play(game: &mut Game, cards: &str) -> Vec<Event> {
    let actions: Vec<Action> = cards
        .split_whitespace()
        .map(|s| Action::Play(c(s)))
        .collect();
    run(game, &actions)
}

/// Asserts that `action` is rejected with `expected` and the state is unchanged.
pub fn assert_rejected(game: &mut Game, action: Action, expected: skat_core::RuleError) {
    let before = game.clone();
    assert_eq!(game.apply(action), Err(expected));
    assert_eq!(*game, before, "state changed after rejected {action:?}");
    assert!(
        !game.legal_actions().contains(&action),
        "{action:?} listed as legal"
    );
}
