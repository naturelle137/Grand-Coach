// SPDX-License-Identifier: EUPL-1.2

//! Continuation policy: fixed rules of thumb for the decisions before card
//! play (bidding, Hand or pickup, Drücken, announcement, Kontra/Re).
//!
//! The option evaluation ([`crate::analyze`]) enumerates the options of the
//! decision at hand exactly; every later decision of every seat in a
//! simulated deal follows this policy, and card play follows double-dummy
//! search. The baseline opponent in `skat-ai` plays by the same rules, so
//! AI and coach share one strategic computation. Every rule only looks at
//! the deciding seat's own cards (and public facts such as the bid).

use skat_core::{Action, Announcement, Card, CardSet, Contract, GameType, reizwerte};

use crate::hand::{TRUMP_GAMES, game_value, null_risks, power, strength, trump_count};

/// Strength from which a suit game is bid.
const SUIT_GAME: i32 = 60;
/// Strength from which a Grand is bid (with at least two Jacks).
const GRAND: i32 = 70;
/// Suits that may be unsafe for a Null game (see `null_risks`).
const NULL_RISKS: usize = 1;
/// Strength from which the declarer plays Hand instead of picking up.
const HAND: i32 = 100;
/// Trumps in a defender's hand from which it says Kontra.
const KONTRA_TRUMPS: usize = 5;
/// Strength from which the declarer answers Kontra with Re.
const RE: i32 = 90;

/// Best game for `cards` and the highest bid the policy goes to (0: pass).
pub fn choose_game(cards: CardSet) -> (GameType, u16) {
    let grand_ok =
        (cards & GameType::BUBEN).len() >= 2 && strength(GameType::Grand, cards) >= GRAND;
    if grand_ok {
        let value = game_value(GameType::Grand, cards, false).unwrap_or(0);
        return (GameType::Grand, value);
    }
    let best = TRUMP_GAMES
        .into_iter()
        .filter(|&g| g != GameType::Grand)
        .max_by_key(|&g| (strength(g, cards), trump_count(g, cards)))
        .expect("four suit games");
    if strength(best, cards) >= SUIT_GAME {
        (best, game_value(best, cards, false).unwrap_or(0))
    } else if null_risks(cards) <= NULL_RISKS {
        (GameType::Null, 23)
    } else {
        (best, 0)
    }
}

/// Bidding action for a seat whose highest acceptable bid is `limit`, given
/// the current bid and the legal actions.
pub fn bid(limit: u16, current: u16, legal: &[Action]) -> Action {
    if legal.contains(&Action::Hold) {
        return if current <= limit {
            Action::Hold
        } else {
            Action::Pass
        };
    }
    let next = reizwerte().into_iter().find(|&v| v > current);
    match next {
        Some(value) if value <= limit && legal.contains(&Action::Bid(value)) => Action::Bid(value),
        _ => Action::Pass,
    }
}

/// Hand or pickup for the declarer's ten cards.
pub fn hand_or_pickup(cards: CardSet) -> Action {
    let (game, _) = choose_game(cards);
    if strength(game, cards) >= HAND {
        Action::Hand
    } else {
        Action::TakeSkat
    }
}

/// Two cards to put away: never Aces or trumps if avoidable; prefer short
/// suits and unguarded Tens (their Augen count for the declarer).
pub fn discard(cards: CardSet) -> CardSet {
    let (game, _) = choose_game(cards);
    let mut ranked: Vec<Card> = cards.iter().collect();
    ranked.sort_by_key(|&card| keep_score(game, cards, card));
    ranked[..2].iter().copied().collect()
}

/// Higher means rather keep.
fn keep_score(game: GameType, cards: CardSet, card: Card) -> i32 {
    if game == GameType::Null {
        return -power(game, card);
    }
    if game.is_trump(card) {
        return 1000 + power(game, card);
    }
    let suit = cards & game.cards_of(game.follow(card));
    let has_ass = suit.iter().any(|c| c.augen() == 11);
    match card.augen() {
        11 => 500,
        10 if has_ass => 400,
        augen => suit.len() as i32 * 20 - i32::from(augen),
    }
}

/// Announcement for the declarer's ten cards `hand` plus the Skat it knows
/// (`skat`: the discard, empty in a Hand game), out of `legal`.
pub fn announce(
    hand: CardSet,
    skat: CardSet,
    bid: u16,
    hand_game: bool,
    legal: &[Action],
) -> Action {
    let cards = hand | skat;
    let (preferred, _) = choose_game(cards);
    let fits = |game: GameType| {
        let contract = Contract::new(Announcement::plain(game), hand_game);
        contract
            .null_value()
            .or_else(|| game_value(game, cards, hand_game))
            .is_some_and(|v| v >= bid)
    };
    let game = if fits(preferred) {
        preferred
    } else {
        TRUMP_GAMES
            .into_iter()
            .filter(|&g| fits(g))
            .max_by_key(|&g| strength(g, cards))
            .unwrap_or(GameType::Grand)
    };
    // A Null hand without any unsafe suit is played open.
    let ouvert = Announcement {
        ouvert: true,
        ..Announcement::plain(GameType::Null)
    };
    let wanted = if game == GameType::Null && null_risks(hand) == 0 {
        Action::Announce(ouvert)
    } else {
        Action::Announce(Announcement::plain(game))
    };
    if legal.contains(&wanted) {
        wanted
    } else {
        Action::Announce(Announcement::plain(game))
    }
}

/// Whether a defender holding `hand` says Kontra against `game`.
pub fn wants_kontra(game: GameType, hand: CardSet) -> bool {
    game != GameType::Null && trump_count(game, hand) >= KONTRA_TRUMPS
}

/// Whether the declarer with `cards` (hand plus own Skat) answers Kontra
/// with Re.
pub fn wants_re(game: GameType, cards: CardSet) -> bool {
    strength(game, cards) >= RE
}
