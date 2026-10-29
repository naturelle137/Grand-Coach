// SPDX-License-Identifier: EUPL-1.2

//! State of the trick in progress.

use skat_core::{Card, GameType, Seat, Trick};

/// Who currently wins the trick in progress and with which card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrickState {
    /// The led card; `None` if the trick is empty.
    pub led: Option<Card>,
    /// The card currently winning the trick.
    pub best: Option<Card>,
    /// Seat that played `best`.
    pub best_seat: Option<Seat>,
}

impl TrickState {
    /// Evaluates `trick` under `game`.
    pub fn of(game: GameType, trick: &Trick) -> TrickState {
        let mut state = TrickState {
            led: trick.led(),
            best: None,
            best_seat: None,
        };
        let (Some(leader), Some(led)) = (trick.leader, trick.led()) else {
            return state;
        };
        for (i, card) in trick.cards.iter().enumerate() {
            let Some(card) = *card else { break };
            if state.best.is_none_or(|best| game.beats(card, best, led)) {
                state.best = Some(card);
                state.best_seat = Some(Seat::from_index(leader.index() + i));
            }
        }
        state
    }

    /// `true` if `card` would take over the trick.
    pub fn beaten_by(&self, game: GameType, card: Card) -> bool {
        match (self.best, self.led) {
            (Some(best), Some(led)) => game.beats(card, best, led),
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skat_core::{Rank, Suit};

    #[test]
    fn best_card_and_seat() {
        let herz_ass = Card::new(Suit::Herz, Rank::Ass);
        let herz_sieben = Card::new(Suit::Herz, Rank::Sieben);
        let trick = Trick {
            leader: Some(Seat::Mittelhand),
            cards: [Some(herz_sieben), Some(herz_ass), None],
        };
        let state = TrickState::of(GameType::Grand, &trick);
        assert_eq!(state.best, Some(herz_ass));
        assert_eq!(state.best_seat, Some(Seat::Hinterhand));
        assert!(state.beaten_by(GameType::Grand, Card::bube(Suit::Karo)));
        assert!(!state.beaten_by(GameType::Grand, Card::new(Suit::Herz, Rank::Zehn)));
        let empty = TrickState::of(GameType::Grand, &Trick::default());
        assert_eq!(empty.best, None);
    }
}
