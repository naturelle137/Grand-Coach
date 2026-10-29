// SPDX-License-Identifier: EUPL-1.2

//! Baseline opponent: fixed rules of thumb on top of `skat-analysis`. It has
//! no randomness, so the same view always gives the same action.

use skat_analysis::hand::{TRUMP_GAMES, game_value, null_risks, strength, trump_count};
use skat_analysis::{Knowledge, TrickState};
use skat_core::{
    Action, Announcement, Card, CardSet, Contract, GameType, Phase, PlayerView, reizwerte,
};

use crate::Opponent;

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

/// A deliberately simple, deterministic opponent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BaselineOpponent;

impl Opponent for BaselineOpponent {
    fn decide(&self, view: &PlayerView) -> Option<Action> {
        let legal = &view.legal_actions;
        if legal.is_empty() {
            return None;
        }
        let wanted = match view.phase {
            Phase::Reizen => bid(view),
            Phase::SkatAufnahme => {
                let (game, _) = choose_game(view.hand);
                if strength(game, view.hand) >= HAND {
                    Action::Hand
                } else {
                    Action::TakeSkat
                }
            }
            Phase::Druecken => Action::Discard(discard(view.hand)),
            Phase::Ansage => announce(view),
            Phase::Spiel => play(view),
            Phase::Beendet | Phase::Eingepasst => return None,
        };
        if legal.contains(&wanted) {
            Some(wanted)
        } else {
            legal.first().copied()
        }
    }
}

/// Best game for `cards` and the highest bid the opponent goes to (0: pass).
fn choose_game(cards: CardSet) -> (GameType, u16) {
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

fn bid(view: &PlayerView) -> Action {
    let (_, limit) = choose_game(view.hand);
    let legal = &view.legal_actions;
    if legal.contains(&Action::Hold) {
        return if view.bid <= limit {
            Action::Hold
        } else {
            Action::Pass
        };
    }
    let next = reizwerte().into_iter().find(|&v| v > view.bid);
    match next {
        Some(value) if value <= limit && legal.contains(&Action::Bid(value)) => Action::Bid(value),
        _ => Action::Pass,
    }
}

/// Two cards to put away: never Aces or trumps if avoidable; prefer short
/// suits and unguarded Tens (their Augen count for the declarer).
fn discard(cards: CardSet) -> CardSet {
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

fn announce(view: &PlayerView) -> Action {
    let hand_game = view.hand_game == Some(true);
    let cards = view.hand | view.own_skat.unwrap_or_default();
    let (preferred, _) = choose_game(cards);
    let fits = |game: GameType| {
        let contract = Contract::new(Announcement::plain(game), hand_game);
        contract
            .null_value()
            .or_else(|| game_value(game, cards, hand_game))
            .is_some_and(|v| v >= view.bid)
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
    if game == GameType::Null
        && null_risks(view.hand) == 0
        && view.legal_actions.contains(&Action::Announce(ouvert))
    {
        return Action::Announce(ouvert);
    }
    Action::Announce(Announcement::plain(game))
}

fn play(view: &PlayerView) -> Action {
    let legal = &view.legal_actions;
    if legal.contains(&Action::KeinKontra) {
        return Action::KeinKontra;
    }
    if legal.contains(&Action::Re) {
        let game = view.contract.map_or(GameType::Grand, |c| c.game);
        let cards = view.hand | view.own_skat.unwrap_or_default();
        return if strength(game, cards) >= RE {
            Action::Re
        } else {
            Action::KeinRe
        };
    }
    let contract = view.contract.expect("contract during play");
    if legal.contains(&Action::Kontra)
        && contract.game != GameType::Null
        && trump_count(contract.game, view.hand) >= KONTRA_TRUMPS
    {
        return Action::Kontra;
    }
    let cards: Vec<Card> = legal
        .iter()
        .filter_map(|a| match a {
            Action::Play(card) => Some(*card),
            _ => None,
        })
        .collect();
    Action::Play(choose_card(view, contract.game, &cards))
}

/// Number of cards of the same follow class that `card` beats when led.
fn power(game: GameType, card: Card) -> i32 {
    game.cards_of(game.follow(card))
        .iter()
        .filter(|&other| game.beats(card, other, card))
        .count() as i32
}

fn lowest(game: GameType, cards: impl Iterator<Item = Card>) -> Option<Card> {
    cards.min_by_key(|&c| (game.is_trump(c), c.augen(), power(game, c)))
}

fn choose_card(view: &PlayerView, game: GameType, cards: &[Card]) -> Card {
    let k = Knowledge::from_view(view);
    let trick = TrickState::of(game, &view.current_trick);
    let declarer = view.declarer.expect("declarer during play");
    let me_declarer = view.seat == declarer;
    let all = cards.iter().copied();
    let winners: Vec<Card> = all.clone().filter(|&c| trick.beaten_by(game, c)).collect();
    let losers: Vec<Card> = all.clone().filter(|&c| !trick.beaten_by(game, c)).collect();
    let last = view.current_trick.len() == 2;
    let fallback = lowest(game, all.clone()).expect("at least one legal card");

    if game == GameType::Null {
        let highest_loser = losers.iter().copied().max_by_key(|&c| power(game, c));
        return match (trick.led, me_declarer) {
            (None, _) => fallback,
            (Some(_), true) => highest_loser.unwrap_or(fallback),
            (Some(_), false) if trick.best_seat == Some(declarer) => {
                highest_loser.unwrap_or(fallback)
            }
            (Some(_), false) => fallback,
        };
    }

    if trick.led.is_none() {
        let trumps: Vec<Card> = all.clone().filter(|&c| game.is_trump(c)).collect();
        if me_declarer && !trumps.is_empty() && !k.trumps_out().is_empty() {
            return *trumps.iter().max_by_key(|&&c| power(game, c)).unwrap();
        }
        let master = all
            .clone()
            .filter(|&c| !game.is_trump(c) && k.is_master(c))
            .max_by_key(|&c| c.augen());
        return master.unwrap_or(fallback);
    }

    let partner_wins = !me_declarer && trick.best_seat.is_some_and(|s| s != declarer);
    if partner_wins {
        let safe = last || trick.best.is_some_and(|b| k.is_master(b));
        if safe {
            return all
                .clone()
                .max_by_key(|&c| (!game.is_trump(c), c.augen()))
                .unwrap_or(fallback);
        }
        return lowest(game, losers.into_iter()).unwrap_or(fallback);
    }
    let cheapest_winner = winners
        .iter()
        .copied()
        .min_by_key(|&c| (game.is_trump(c), power(game, c)));
    match cheapest_winner {
        Some(card) if last || k.is_master(card) || game.is_trump(card) => card,
        _ => lowest(game, losers.into_iter()).unwrap_or(fallback),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::ChaCha8Rng;
    use rand_chacha::rand_core::{Rng, SeedableRng};
    use skat_core::{Game, RuleConfig, Seat};

    /// Plays `seed` with the baseline opponent on every seat.
    fn self_play(seed: u64, kontra_re: bool) -> Game {
        let mut game = Game::new(seed, RuleConfig { kontra_re });
        while let Some(seat) = game.current_seat() {
            let view = game.view(seat);
            let action = BaselineOpponent.decide(&view).expect("legal action");
            assert!(
                view.legal_actions.contains(&action),
                "seed {seed}: {action:?}"
            );
            game.apply(action).unwrap();
        }
        game
    }

    #[test]
    fn self_play_finishes_legally_and_varies() {
        let mut games = std::collections::BTreeMap::new();
        let mut kontras = 0;
        for seed in 0..500 {
            let game = self_play(seed, true);
            if let Some(c) = game.contract() {
                *games.entry(format!("{:?}", c.game)).or_insert(0) += 1;
            }
            if game.kontra() != skat_core::KontraLevel::None {
                kontras += 1;
            }
        }
        // All four suits and Grand; Null needs a rare hand (NULL_RISKS).
        assert!(games.len() >= 5, "{games:?}");
        assert!(kontras > 0);
    }

    #[test]
    fn decisions_depend_only_on_the_view() {
        let mut rng = ChaCha8Rng::seed_from_u64(9);
        for seed in 0..200 {
            let mut game = Game::new(seed, RuleConfig { kontra_re: true });
            while let Some(seat) = game.current_seat() {
                let view = game.view(seat);
                let action = BaselineOpponent.decide(&view).unwrap();
                assert_eq!(BaselineOpponent.decide(&view.clone()), Some(action));
                // Mix in random legal actions so more positions are reached.
                let action = if rng.next_u32() % 4 == 0 {
                    let i = rng.next_u32() as usize % view.legal_actions.len();
                    view.legal_actions[i]
                } else {
                    action
                };
                game.apply(action).unwrap();
            }
        }
    }

    #[test]
    fn no_decision_without_turn() {
        let game = Game::new(1, RuleConfig::default());
        let waiting = Seat::ALL
            .into_iter()
            .find(|&s| Some(s) != game.current_seat())
            .unwrap();
        assert_eq!(BaselineOpponent.decide(&game.view(waiting)), None);
    }
}
