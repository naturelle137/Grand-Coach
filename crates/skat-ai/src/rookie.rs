// SPDX-License-Identifier: EUPL-1.2

//! Seeded mistakes on top of the rules of thumb (level Rookie, D-41).
//!
//! Each kind of decision deviates from the rules of thumb with its own
//! probability; otherwise the rules decide. Every deviation is a legal
//! action, and its randomness comes from the opponent's seed and the view.

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};
use skat_analysis::hand::{TRUMP_GAMES, game_value, strength};
use skat_analysis::policy;
use skat_core::{Action, Announcement, Card, CardSet, GameType, Phase, PlayerView, reizwerte};

/// How often each kind of decision deviates from the rules of thumb.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mistakes {
    /// Per game: the bid limit is moved one to three bidding values up or
    /// down (bidding too much or too little).
    pub bid: f64,
    /// Per announcement: the second-best trump game that still covers the
    /// bid is announced instead of the best.
    pub game: f64,
    /// Per Drücken: two random cards other than Jacks are discarded.
    pub discard: f64,
    /// Per Kontra or Re decision: the rules' choice is reversed.
    pub kontra: f64,
    /// Per card: a random legal card is played.
    pub card: f64,
}

impl Mistakes {
    /// No mistakes: the plain rules of thumb.
    pub const NONE: Mistakes = Mistakes {
        bid: 0.0,
        game: 0.0,
        discard: 0.0,
        kontra: 0.0,
        card: 0.0,
    };

    /// `true` if no decision deviates.
    pub fn is_none(&self) -> bool {
        *self == Mistakes::NONE
    }
}

/// Uniform number in [0, 1).
fn unit(rng: &mut ChaCha8Rng) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
}

fn below(rng: &mut ChaCha8Rng, n: usize) -> usize {
    (rng.next_u64() % n as u64) as usize
}

/// The decision with mistakes. `rules` is the rules-of-thumb action,
/// `game_seed` is constant for the seat's whole bidding (so a misjudged
/// limit stays misjudged), `seed` is the decision's own seed.
pub(crate) fn decide(
    view: &PlayerView,
    rules: Action,
    mistakes: &Mistakes,
    game_seed: u64,
    seed: u64,
) -> Action {
    let legal = &view.legal_actions;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let wanted = match view.phase {
        Phase::Reizen => {
            let mut game_rng = ChaCha8Rng::seed_from_u64(game_seed);
            if unit(&mut game_rng) >= mistakes.bid {
                return rules;
            }
            let limit = policy::choose_game(view.hand).1;
            let values = reizwerte();
            let steps = 1 + below(&mut game_rng, 3);
            let up = game_rng.next_u64().is_multiple_of(2);
            let position = values.iter().position(|&v| v >= limit);
            let moved = match (limit, up) {
                (0, true) => values[steps - 1],
                (0, false) => 0,
                (_, true) => {
                    let i = position.map_or(values.len() - 1, |i| i + steps);
                    values[i.min(values.len() - 1)]
                }
                (_, false) => match position {
                    Some(i) if i >= steps => values[i - steps],
                    _ => 0,
                },
            };
            policy::bid(moved, view.bid, legal)
        }
        Phase::Ansage if unit(&mut rng) < mistakes.game => second_game(view, rules),
        Phase::Druecken if unit(&mut rng) < mistakes.discard => {
            let cards: Vec<Card> = (view.hand - GameType::BUBEN).into_iter().collect();
            if cards.len() < 2 {
                return rules;
            }
            let a = below(&mut rng, cards.len());
            let mut b = below(&mut rng, cards.len() - 1);
            if b >= a {
                b += 1;
            }
            Action::Discard(CardSet::single(cards[a]) | CardSet::single(cards[b]))
        }
        Phase::Spiel => {
            let flip = |a: Action| match a {
                Action::Kontra => Action::KeinKontra,
                Action::KeinKontra => Action::Kontra,
                Action::Re => Action::KeinRe,
                Action::KeinRe => Action::Re,
                other => other,
            };
            let kontra = matches!(
                rules,
                Action::Kontra | Action::KeinKontra | Action::Re | Action::KeinRe
            );
            if kontra && unit(&mut rng) < mistakes.kontra {
                flip(rules)
            } else if !kontra && unit(&mut rng) < mistakes.card {
                legal[below(&mut rng, legal.len())]
            } else {
                rules
            }
        }
        _ => rules,
    };
    if legal.contains(&wanted) {
        wanted
    } else {
        rules
    }
}

/// The second-best trump game by the rules' strength measure that still
/// covers the bid; the rules' choice if there is none.
fn second_game(view: &PlayerView, rules: Action) -> Action {
    let Action::Announce(chosen) = rules else {
        return rules;
    };
    let cards = view.hand | view.own_skat.unwrap_or_default();
    let hand_game = view.hand_game == Some(true);
    TRUMP_GAMES
        .into_iter()
        .filter(|&g| g != chosen.game)
        .filter(|&g| game_value(g, cards, hand_game).is_some_and(|v| v >= view.bid))
        .max_by_key(|&g| strength(g, cards))
        .map_or(rules, |g| Action::Announce(Announcement::plain(g)))
}
