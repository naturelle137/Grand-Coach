// SPDX-License-Identifier: EUPL-1.2

//! Baseline opponent: fixed rules of thumb on top of `skat-analysis`. It has
//! no randomness, so the same view always gives the same action.

use skat_analysis::hand::power;
use skat_analysis::{Knowledge, TrickState, policy};
use skat_core::{Action, Card, GameType, Phase, PlayerView};

/// A deliberately simple, deterministic opponent: the shared continuation
/// policy of `skat-analysis` for the decisions before card play, plus rules
/// of thumb for the cards.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BaselineOpponent;

impl BaselineOpponent {
    /// Chooses one of `view.legal_actions`; `None` if the list is empty,
    /// i.e. it is not this seat's turn.
    pub fn decide(&self, view: &PlayerView) -> Option<Action> {
        let legal = &view.legal_actions;
        if legal.is_empty() {
            return None;
        }
        let wanted = match view.phase {
            Phase::Reizen => {
                let (_, limit) = policy::choose_game(view.hand);
                policy::bid(limit, view.bid, legal)
            }
            Phase::SkatAufnahme => policy::hand_or_pickup(view.hand),
            Phase::Druecken => Action::Discard(policy::discard(view.hand)),
            Phase::Ansage => policy::announce(
                view.hand,
                view.own_skat.unwrap_or_default(),
                view.bid,
                view.hand_game == Some(true),
                legal,
            ),
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

fn play(view: &PlayerView) -> Action {
    let legal = &view.legal_actions;
    if legal.contains(&Action::KeinKontra) {
        return Action::KeinKontra;
    }
    let game = view.contract.map_or(GameType::Grand, |c| c.game);
    if legal.contains(&Action::Re) {
        let cards = view.hand | view.own_skat.unwrap_or_default();
        return if policy::wants_re(game, cards) {
            Action::Re
        } else {
            Action::KeinRe
        };
    }
    if legal.contains(&Action::Kontra) && policy::wants_kontra(game, view.hand) {
        return Action::Kontra;
    }
    let cards: Vec<Card> = legal
        .iter()
        .filter_map(|a| match a {
            Action::Play(card) => Some(*card),
            _ => None,
        })
        .collect();
    Action::Play(choose_card(view, game, &cards))
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
