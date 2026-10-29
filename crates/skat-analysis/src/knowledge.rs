// SPDX-License-Identifier: EUPL-1.2

//! What one seat knows about the cards (a minimal card counter).

use skat_core::{Card, CardSet, GameType, PlayerView, Seat};

/// Card knowledge of one seat, derived only from its [`PlayerView`] (D-23).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Knowledge {
    /// The viewing seat.
    pub seat: Seat,
    /// The game being played, once announced.
    pub game: Option<GameType>,
    /// Own cards.
    pub own: CardSet,
    /// Cards played so far, including the trick in progress.
    pub played: CardSet,
    /// Cards whose place is known without being own or played: the own
    /// discarded Skat and the revealed cards of an Ouvert declarer.
    pub known_elsewhere: CardSet,
}

impl Knowledge {
    /// Derives the knowledge of `view.seat`.
    pub fn from_view(view: &PlayerView) -> Knowledge {
        let mut played: CardSet = view.tricks.iter().flat_map(|t| t.cards).collect();
        played |= view.current_trick.cards.iter().flatten().copied().collect();
        let mut known_elsewhere = view.own_skat.unwrap_or_default();
        if view.declarer != Some(view.seat) {
            known_elsewhere |= view.declarer_cards.unwrap_or_default();
        }
        Knowledge {
            seat: view.seat,
            game: view.contract.map(|c| c.game),
            own: view.hand,
            played,
            known_elsewhere,
        }
    }

    /// Cards this seat has not seen: in other hands or in the Skat.
    pub fn unseen(&self) -> CardSet {
        CardSet::ALL - self.own - self.played - self.known_elsewhere
    }

    /// Trumps not yet played and not held by this seat.
    pub fn trumps_out(&self) -> CardSet {
        self.game.map_or(CardSet::EMPTY, |g| {
            g.trumps() & (CardSet::ALL - self.own - self.played)
        })
    }

    /// `true` if no card of the same follow class (Trumpf or the side suit)
    /// still out elsewhere can beat `card` when it is led. A side-suit card
    /// may still be trumped.
    pub fn is_master(&self, card: Card) -> bool {
        let Some(game) = self.game else {
            return false;
        };
        let elsewhere = (CardSet::ALL - self.own - self.played) & game.cards_of(game.follow(card));
        elsewhere.iter().all(|other| !game.beats(other, card, card))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skat_core::{Action, Announcement, Game, Phase, Rank, RuleConfig, Suit};

    /// Plays a seeded game to the first card of the play phase with a fixed
    /// policy: first legal action, Grand announced.
    fn game_in_play(seed: u64) -> Option<Game> {
        let mut game = Game::new(seed, RuleConfig::default());
        while game.phase() != Phase::Spiel {
            let action = match game.phase() {
                Phase::Reizen => {
                    if game.bid() == 0 {
                        Action::Bid(18)
                    } else {
                        Action::Pass
                    }
                }
                Phase::Ansage => Action::Announce(Announcement::plain(GameType::Grand)),
                Phase::Beendet | Phase::Eingepasst => return None,
                _ => game.legal_actions()[0],
            };
            game.apply(action).ok()?;
        }
        Some(game)
    }

    #[test]
    fn unseen_cards_exclude_own_and_played() {
        let mut game = game_in_play(3).expect("seed 3 reaches play");
        let first = game.legal_actions()[0];
        game.apply(first).unwrap();
        for seat in Seat::ALL {
            let k = Knowledge::from_view(&game.view(seat));
            assert!(k.unseen().is_disjoint(k.own));
            assert!(k.unseen().is_disjoint(k.played));
            assert_eq!(k.played.len(), 1);
            let skat_known =
                game.declarer() == Some(seat) && game.view(seat).hand_game == Some(false);
            assert_eq!(k.known_elsewhere.len(), if skat_known { 2 } else { 0 });
            assert_eq!(
                k.unseen().len() + k.own.len() + k.played.len() + k.known_elsewhere.len(),
                32
            );
            assert_eq!(k.game, Some(GameType::Grand));
        }
    }

    #[test]
    fn master_cards() {
        let own =
            CardSet::single(Card::bube(Suit::Kreuz)) | Card::new(Suit::Herz, Rank::Zehn).into();
        let mut k = Knowledge {
            seat: Seat::Vorhand,
            game: Some(GameType::Grand),
            own,
            played: CardSet::EMPTY,
            known_elsewhere: CardSet::EMPTY,
        };
        assert!(k.is_master(Card::bube(Suit::Kreuz)));
        assert!(!k.is_master(Card::new(Suit::Herz, Rank::Zehn)));
        k.played.insert(Card::new(Suit::Herz, Rank::Ass));
        assert!(k.is_master(Card::new(Suit::Herz, Rank::Zehn)));
        assert_eq!(k.trumps_out().len(), 3);
    }
}
