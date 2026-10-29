// SPDX-License-Identifier: EUPL-1.2

//! Simple measures of a hand: game value from Spitzen, trump count, a rough
//! strength score per game type and Null safety. No search, no simulation.

use skat_core::{Card, CardSet, GameType, Rank, Suit, spitzen};

/// The four suit games and Grand, the games with Trumpf.
pub const TRUMP_GAMES: [GameType; 5] = [
    GameType::Farbe(Suit::Kreuz),
    GameType::Farbe(Suit::Pik),
    GameType::Farbe(Suit::Herz),
    GameType::Farbe(Suit::Karo),
    GameType::Grand,
];

/// Value of a suit or Grand game without announced levels, with Spitzen
/// counted from `cards` only (`hand` adds the Hand level). `None` for Null.
pub fn game_value(game: GameType, cards: CardSet, hand: bool) -> Option<u16> {
    let base = game.base_value()?;
    let count = spitzen(game, cards).map_or(0, |s| u16::from(s.count));
    Some(base * (count + 1 + u16::from(hand)))
}

/// Number of trumps in `cards`.
pub fn trump_count(game: GameType, cards: CardSet) -> usize {
    (cards & game.trumps()).len()
}

/// Rough playing strength of `cards` as declarer of `game`, in points of
/// roughly a tenth of a trick: trumps, Jacks, side Aces and guarded Tens.
/// Comparable between the trump games of one hand; 0 for Null.
pub fn strength(game: GameType, cards: CardSet) -> i32 {
    if game == GameType::Null {
        return 0;
    }
    let trumps = cards & game.trumps();
    let mut score = 0;
    for card in trumps {
        score += match card.rank() {
            Rank::Bube => 14 + 4 * i32::from(card.suit() == Suit::Kreuz),
            Rank::Ass | Rank::Zehn => 11,
            _ => 8,
        };
    }
    for suit in Suit::ALL {
        let side = cards & (suit.cards() - game.trumps());
        let ass = side.contains(Card::new(suit, Rank::Ass));
        if ass {
            score += 10;
        }
        if side.contains(Card::new(suit, Rank::Zehn)) {
            score += if ass || side.len() >= 3 { 6 } else { 1 };
        }
        if side.is_empty() && game != GameType::Grand {
            score += 3;
        }
    }
    score
}

/// Number of cards of the same follow class that `card` beats when led.
pub fn power(game: GameType, card: Card) -> i32 {
    game.cards_of(game.follow(card))
        .iter()
        .filter(|&other| game.beats(card, other, card))
        .count() as i32
}

/// Number of suits in which the declarer of a Null game could be forced to
/// take a trick: a suit is safe if its k-th lowest card (from 0) is at most
/// the 2k-th card in Null order 7, 8, 9, 10, J, Q, K, A.
pub fn null_risks(cards: CardSet) -> usize {
    const NULL_RANKS: [Rank; 8] = [
        Rank::Sieben,
        Rank::Acht,
        Rank::Neun,
        Rank::Zehn,
        Rank::Bube,
        Rank::Dame,
        Rank::Koenig,
        Rank::Ass,
    ];
    Suit::ALL
        .iter()
        .filter(|&&suit| {
            let mut positions: Vec<usize> = (cards & suit.cards())
                .iter()
                .map(|c| NULL_RANKS.iter().position(|&r| r == c.rank()).unwrap())
                .collect();
            positions.sort_unstable();
            positions.iter().enumerate().any(|(k, &p)| p > 2 * k)
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(cards: &[(Suit, Rank)]) -> CardSet {
        cards.iter().map(|&(s, r)| Card::new(s, r)).collect()
    }

    #[test]
    fn values_and_counts() {
        let cards = set(&[
            (Suit::Kreuz, Rank::Bube),
            (Suit::Pik, Rank::Bube),
            (Suit::Herz, Rank::Ass),
            (Suit::Herz, Rank::Zehn),
            (Suit::Herz, Rank::Sieben),
        ]);
        assert_eq!(
            game_value(GameType::Farbe(Suit::Herz), cards, false),
            Some(30)
        );
        assert_eq!(game_value(GameType::Grand, cards, true), Some(96));
        assert_eq!(game_value(GameType::Null, cards, false), None);
        assert_eq!(trump_count(GameType::Farbe(Suit::Herz), cards), 5);
        assert!(
            strength(GameType::Farbe(Suit::Herz), cards)
                > strength(GameType::Farbe(Suit::Karo), cards)
        );
    }

    #[test]
    fn null_safety() {
        let safe = set(&[
            (Suit::Herz, Rank::Sieben),
            (Suit::Herz, Rank::Neun),
            (Suit::Pik, Rank::Sieben),
        ]);
        assert_eq!(null_risks(safe), 0);
        let risky = set(&[
            (Suit::Herz, Rank::Ass),
            (Suit::Karo, Rank::Sieben),
            (Suit::Karo, Rank::Koenig),
        ]);
        assert_eq!(null_risks(risky), 2);
    }
}
