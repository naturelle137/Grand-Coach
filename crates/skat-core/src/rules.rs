// SPDX-License-Identifier: EUPL-1.2

//! Game types, contracts, Trumpf, card order, Bedienen and Stich winners.

use crate::card::{Card, CardSet, Rank, Suit};

/// The three game families.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GameType {
    /// Suit game: Jacks plus the named suit are Trumpf.
    Farbe(Suit),
    /// Grand: only the four Jacks are Trumpf.
    Grand,
    /// Null: no Trumpf, the declarer must lose every trick.
    Null,
}

/// What a card counts as when following suit (Bedienen).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Follow {
    /// The card is Trumpf.
    Trumpf,
    /// The card belongs to this side suit (Fehlfarbe).
    Farbe(Suit),
}

/// Plain-suit order in suit games and Grand: A, 10, K, Q, 9, 8, 7.
const PLAIN_ORDER: [Rank; 7] = [
    Rank::Ass,
    Rank::Zehn,
    Rank::Koenig,
    Rank::Dame,
    Rank::Neun,
    Rank::Acht,
    Rank::Sieben,
];

/// Order in Null: A, K, Q, J, 10, 9, 8, 7.
const NULL_ORDER: [Rank; 8] = [
    Rank::Ass,
    Rank::Koenig,
    Rank::Dame,
    Rank::Bube,
    Rank::Zehn,
    Rank::Neun,
    Rank::Acht,
    Rank::Sieben,
];

const BUBEN: CardSet = CardSet::from_bits(
    1 << (Rank::Bube as u32)
        | 1 << (8 + Rank::Bube as u32)
        | 1 << (16 + Rank::Bube as u32)
        | 1 << (24 + Rank::Bube as u32),
);

impl GameType {
    /// The four Jacks.
    pub const BUBEN: CardSet = BUBEN;

    /// Base value (Grundwert); `None` for Null, whose values are fixed.
    pub const fn base_value(self) -> Option<u16> {
        match self {
            GameType::Farbe(suit) => Some(suit.base_value()),
            GameType::Grand => Some(24),
            GameType::Null => None,
        }
    }

    /// All Trumpf cards of this game type.
    pub fn trumps(self) -> CardSet {
        match self {
            GameType::Farbe(suit) => BUBEN | suit.cards(),
            GameType::Grand => BUBEN,
            GameType::Null => CardSet::EMPTY,
        }
    }

    /// `true` if the card is Trumpf.
    pub fn is_trump(self, card: Card) -> bool {
        self.trumps().contains(card)
    }

    /// Class a card belongs to for Bedienen.
    pub fn follow(self, card: Card) -> Follow {
        if self.is_trump(card) {
            Follow::Trumpf
        } else {
            Follow::Farbe(card.suit())
        }
    }

    /// All cards of a follow class.
    pub fn cards_of(self, follow: Follow) -> CardSet {
        match follow {
            Follow::Trumpf => self.trumps(),
            Follow::Farbe(suit) => suit.cards() - self.trumps(),
        }
    }

    /// Trumpf cards from highest to lowest (Jacks first, then the trump suit).
    /// Used for Spitzen. Empty for Null.
    pub fn trump_order(self) -> Vec<Card> {
        let mut order: Vec<Card> = match self {
            GameType::Null => return Vec::new(),
            _ => Suit::ALL.iter().map(|&suit| Card::bube(suit)).collect(),
        };
        if let GameType::Farbe(suit) = self {
            order.extend(PLAIN_ORDER.iter().map(|&rank| Card::new(suit, rank)));
        }
        order
    }

    /// Strength of a card within its follow class; higher wins.
    fn strength(self, card: Card) -> u8 {
        if self == GameType::Null {
            return 8 - position(&NULL_ORDER, card.rank());
        }
        if card.rank() == Rank::Bube {
            return 20 - card.suit().index();
        }
        8 - position(&PLAIN_ORDER, card.rank())
    }

    /// `true` if `card` beats `best`, the card currently winning a trick in
    /// which `led` was played first.
    pub fn beats(self, card: Card, best: Card, led: Card) -> bool {
        let rank = |c: Card| {
            let class = self.follow(c);
            let tier = if class == Follow::Trumpf {
                2
            } else if class == self.follow(led) {
                1
            } else {
                0
            };
            (tier, self.strength(c))
        };
        rank(card) > rank(best)
    }
}

fn position<const N: usize>(order: &[Rank; N], rank: Rank) -> u8 {
    order.iter().position(|&r| r == rank).unwrap() as u8
}

/// Cards from `hand` that may be played to a trick led with `led`
/// (`None` when leading): Bedienen is mandatory, otherwise any card.
pub fn legal_cards(game: GameType, hand: CardSet, led: Option<Card>) -> CardSet {
    let Some(led) = led else {
        return hand;
    };
    let following = hand & game.cards_of(game.follow(led));
    if following.is_empty() {
        hand
    } else {
        following
    }
}

/// Index (0, 1 or 2) of the winning card in a complete trick, given in play
/// order starting with the led card.
pub fn trick_winner(game: GameType, cards: [Card; 3]) -> usize {
    let mut best = 0;
    for i in 1..3 {
        if game.beats(cards[i], cards[best], cards[0]) {
            best = i;
        }
    }
    best
}

/// The declarer's announcement after bidding (Ansage). Whether the game is a
/// Hand game was already decided in the Skat phase and is not part of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Announcement {
    /// Game type.
    pub game: GameType,
    /// Schneider announced (suit and Grand Hand games only).
    pub schneider: bool,
    /// Schwarz announced (suit and Grand Hand games only, implies Schneider).
    pub schwarz: bool,
    /// Ouvert. Suit and Grand: Hand only, implies Schneider and Schwarz.
    pub ouvert: bool,
}

impl Announcement {
    /// Plain announcement without Schneider, Schwarz or Ouvert.
    pub const fn plain(game: GameType) -> Announcement {
        Announcement {
            game,
            schneider: false,
            schwarz: false,
            ouvert: false,
        }
    }

    /// `true` if this announcement is allowed for a Hand game (`hand`) or a
    /// game with Skat pickup (ISkO 2.1.3, 2.1.4, 5.2.6).
    pub const fn is_valid(self, hand: bool) -> bool {
        match self.game {
            GameType::Null => !self.schneider && !self.schwarz,
            GameType::Farbe(_) | GameType::Grand => {
                if !hand {
                    !self.schneider && !self.schwarz && !self.ouvert
                } else {
                    (!self.schwarz || self.schneider) && (!self.ouvert || self.schwarz)
                }
            }
        }
    }

    /// All valid announcements for a Hand game or a game with pickup.
    pub fn all(hand: bool) -> Vec<Announcement> {
        let mut games: Vec<GameType> = Suit::ALL.iter().map(|&s| GameType::Farbe(s)).collect();
        games.push(GameType::Grand);
        let mut all = Vec::new();
        for game in games {
            all.push(Announcement::plain(game));
            if hand {
                all.push(Announcement {
                    game,
                    schneider: true,
                    schwarz: false,
                    ouvert: false,
                });
                all.push(Announcement {
                    game,
                    schneider: true,
                    schwarz: true,
                    ouvert: false,
                });
                all.push(Announcement {
                    game,
                    schneider: true,
                    schwarz: true,
                    ouvert: true,
                });
            }
        }
        all.push(Announcement::plain(GameType::Null));
        all.push(Announcement {
            ouvert: true,
            ..Announcement::plain(GameType::Null)
        });
        all
    }
}

/// The game being played: announcement plus the Hand decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Contract {
    /// Game type.
    pub game: GameType,
    /// Played without picking up the Skat.
    pub hand: bool,
    /// Schneider announced.
    pub schneider: bool,
    /// Schwarz announced.
    pub schwarz: bool,
    /// Ouvert.
    pub ouvert: bool,
}

impl Contract {
    /// Contract from an announcement and the Hand decision.
    pub const fn new(announcement: Announcement, hand: bool) -> Contract {
        Contract {
            game: announcement.game,
            hand,
            schneider: announcement.schneider,
            schwarz: announcement.schwarz,
            ouvert: announcement.ouvert,
        }
    }

    /// Fixed value of a Null game (ISkO 5.1.2); `None` for other games.
    pub const fn null_value(self) -> Option<u16> {
        match (self.game, self.hand, self.ouvert) {
            (GameType::Null, false, false) => Some(23),
            (GameType::Null, true, false) => Some(35),
            (GameType::Null, false, true) => Some(46),
            (GameType::Null, true, true) => Some(59),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trump_counts() {
        assert_eq!(GameType::Farbe(Suit::Herz).trumps().len(), 11);
        assert_eq!(GameType::Grand.trumps().len(), 4);
        assert!(GameType::Null.trumps().is_empty());
        assert_eq!(GameType::Farbe(Suit::Pik).trump_order().len(), 11);
    }

    #[test]
    fn announcement_counts() {
        assert_eq!(Announcement::all(false).len(), 7);
        assert_eq!(Announcement::all(true).len(), 22);
        for hand in [false, true] {
            assert!(Announcement::all(hand).iter().all(|a| a.is_valid(hand)));
        }
    }
}
