// SPDX-License-Identifier: EUPL-1.2

//! Display order of a hand.

use skat_core::{Card, CardSet, GameType, Rank, Suit};

/// Sorts cards for display: for suit games and Grand (and before the game
/// is known) Jacks first, then the trump suit, then the other suits, each
/// from Ace down; for Null by suit in Null order (A, K, Q, J, 10, …).
pub fn sort_hand(cards: CardSet, game: Option<GameType>) -> Vec<Card> {
    let game = game.unwrap_or(GameType::Grand);
    let mut sorted: Vec<Card> = cards.iter().collect();
    sorted.sort_by_key(|&card| sort_key(game, card));
    sorted
}

fn sort_key(game: GameType, card: Card) -> (u8, u8) {
    const PLAIN: [Rank; 8] = [
        Rank::Ass,
        Rank::Zehn,
        Rank::Koenig,
        Rank::Dame,
        Rank::Bube,
        Rank::Neun,
        Rank::Acht,
        Rank::Sieben,
    ];
    const NULL: [Rank; 8] = [
        Rank::Ass,
        Rank::Koenig,
        Rank::Dame,
        Rank::Bube,
        Rank::Zehn,
        Rank::Neun,
        Rank::Acht,
        Rank::Sieben,
    ];
    let order = if game == GameType::Null {
        &NULL
    } else {
        &PLAIN
    };
    let rank = order.iter().position(|&r| r == card.rank()).unwrap() as u8;
    if game != GameType::Null && card.rank() == Rank::Bube {
        return (0, card.suit().index());
    }
    let trump_suit = match game {
        GameType::Farbe(suit) => Some(suit),
        _ => None,
    };
    let group = if Some(card.suit()) == trump_suit {
        1
    } else {
        2 + Suit::ALL.iter().position(|&s| s == card.suit()).unwrap() as u8
    };
    (group, rank)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jacks_then_trump_suit() {
        let cards: CardSet = [
            Card::new(Suit::Kreuz, Rank::Sieben),
            Card::new(Suit::Herz, Rank::Ass),
            Card::bube(Suit::Karo),
            Card::bube(Suit::Kreuz),
            Card::new(Suit::Herz, Rank::Zehn),
        ]
        .into_iter()
        .collect();
        let sorted = sort_hand(cards, Some(GameType::Farbe(Suit::Herz)));
        assert_eq!(
            sorted,
            vec![
                Card::bube(Suit::Kreuz),
                Card::bube(Suit::Karo),
                Card::new(Suit::Herz, Rank::Ass),
                Card::new(Suit::Herz, Rank::Zehn),
                Card::new(Suit::Kreuz, Rank::Sieben),
            ]
        );
        let null = sort_hand(cards, Some(GameType::Null));
        assert_eq!(null[0], Card::bube(Suit::Kreuz));
        assert_eq!(null[1], Card::new(Suit::Kreuz, Rank::Sieben));
    }
}
