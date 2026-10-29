// SPDX-License-Identifier: EUPL-1.2

//! Cards, suits, ranks and the `CardSet` bitset (D-20).

use core::fmt;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Sub, SubAssign};

/// One of the four suits, in descending order of base value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Suit {
    /// Clubs (Eichel in the German deck), base value 12.
    Kreuz,
    /// Spades (Grün), base value 11.
    Pik,
    /// Hearts (Rot), base value 10.
    Herz,
    /// Diamonds (Schellen), base value 9.
    Karo,
}

impl Suit {
    /// All suits, from Kreuz down to Karo. This is also the Jack order.
    pub const ALL: [Suit; 4] = [Suit::Kreuz, Suit::Pik, Suit::Herz, Suit::Karo];

    /// Index `0..4` (Kreuz = 0).
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Base value (Grundwert) of a suit game in this suit.
    pub const fn base_value(self) -> u16 {
        match self {
            Suit::Kreuz => 12,
            Suit::Pik => 11,
            Suit::Herz => 10,
            Suit::Karo => 9,
        }
    }

    /// All eight cards of this suit, including its Jack.
    pub const fn cards(self) -> CardSet {
        CardSet(0xFF << (self.index() * 8))
    }
}

/// Card rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Rank {
    /// Seven.
    Sieben,
    /// Eight.
    Acht,
    /// Nine.
    Neun,
    /// Ten.
    Zehn,
    /// Jack (Unter).
    Bube,
    /// Queen (Ober).
    Dame,
    /// King.
    Koenig,
    /// Ace (Daus).
    Ass,
}

impl Rank {
    /// All ranks in index order.
    pub const ALL: [Rank; 8] = [
        Rank::Sieben,
        Rank::Acht,
        Rank::Neun,
        Rank::Zehn,
        Rank::Bube,
        Rank::Dame,
        Rank::Koenig,
        Rank::Ass,
    ];

    /// Index `0..8` (Sieben = 0).
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Card points (Augen) of a card of this rank.
    pub const fn augen(self) -> u8 {
        match self {
            Rank::Ass => 11,
            Rank::Zehn => 10,
            Rank::Koenig => 4,
            Rank::Dame => 3,
            Rank::Bube => 2,
            Rank::Sieben | Rank::Acht | Rank::Neun => 0,
        }
    }
}

/// A card as index `0..32`: `suit * 8 + rank` (D-20).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "u8", into = "u8")
)]
pub struct Card(u8);

impl Card {
    /// Creates the card of the given suit and rank.
    pub const fn new(suit: Suit, rank: Rank) -> Card {
        Card(suit.index() * 8 + rank.index())
    }

    /// Card for an index `0..32`, `None` otherwise.
    pub const fn from_index(index: u8) -> Option<Card> {
        if index < 32 { Some(Card(index)) } else { None }
    }

    /// Index `0..32`.
    pub const fn index(self) -> u8 {
        self.0
    }

    /// Suit of the card (a Jack keeps its printed suit).
    pub const fn suit(self) -> Suit {
        Suit::ALL[(self.0 / 8) as usize]
    }

    /// Rank of the card.
    pub const fn rank(self) -> Rank {
        Rank::ALL[(self.0 % 8) as usize]
    }

    /// Card points (Augen).
    pub const fn augen(self) -> u8 {
        self.rank().augen()
    }

    /// The Jack of the given suit.
    pub const fn bube(suit: Suit) -> Card {
        Card::new(suit, Rank::Bube)
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}-{:?}", self.suit(), self.rank())
    }
}

impl From<Card> for u8 {
    fn from(card: Card) -> u8 {
        card.0
    }
}

/// Error for a card index outside `0..32`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidCardIndex(pub u8);

impl fmt::Display for InvalidCardIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid card index {}", self.0)
    }
}

impl TryFrom<u8> for Card {
    type Error = InvalidCardIndex;

    fn try_from(index: u8) -> Result<Card, InvalidCardIndex> {
        Card::from_index(index).ok_or(InvalidCardIndex(index))
    }
}

/// A set of cards as a `u32` bitset; bit `i` stands for the card with index `i`.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct CardSet(u32);

impl CardSet {
    /// The empty set.
    pub const EMPTY: CardSet = CardSet(0);
    /// All 32 cards.
    pub const ALL: CardSet = CardSet(u32::MAX);

    /// Set from raw bits.
    pub const fn from_bits(bits: u32) -> CardSet {
        CardSet(bits)
    }

    /// Raw bits.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Set containing exactly one card.
    pub const fn single(card: Card) -> CardSet {
        CardSet(1 << card.0)
    }

    /// Number of cards.
    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    /// `true` if the set has no cards.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// `true` if `card` is in the set.
    pub const fn contains(self, card: Card) -> bool {
        self.0 & (1 << card.0) != 0
    }

    /// Adds a card; returns `true` if it was not present.
    pub fn insert(&mut self, card: Card) -> bool {
        let absent = !self.contains(card);
        self.0 |= 1 << card.0;
        absent
    }

    /// Removes a card; returns `true` if it was present.
    pub fn remove(&mut self, card: Card) -> bool {
        let present = self.contains(card);
        self.0 &= !(1 << card.0);
        present
    }

    /// Union of both sets.
    pub const fn union(self, other: CardSet) -> CardSet {
        CardSet(self.0 | other.0)
    }

    /// Intersection of both sets.
    pub const fn intersection(self, other: CardSet) -> CardSet {
        CardSet(self.0 & other.0)
    }

    /// Cards in `self` but not in `other`.
    pub const fn difference(self, other: CardSet) -> CardSet {
        CardSet(self.0 & !other.0)
    }

    /// `true` if every card of `self` is in `other`.
    pub const fn is_subset(self, other: CardSet) -> bool {
        self.0 & !other.0 == 0
    }

    /// `true` if both sets share no card.
    pub const fn is_disjoint(self, other: CardSet) -> bool {
        self.0 & other.0 == 0
    }

    /// Sum of card points (Augen).
    pub fn augen(self) -> u8 {
        self.iter().map(Card::augen).sum()
    }

    /// Iterates the cards in ascending index order.
    pub fn iter(self) -> CardSetIter {
        CardSetIter(self.0)
    }
}

impl fmt::Debug for CardSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

/// Iterator over the cards of a [`CardSet`].
#[derive(Clone, Debug)]
pub struct CardSetIter(u32);

impl Iterator for CardSetIter {
    type Item = Card;

    fn next(&mut self) -> Option<Card> {
        if self.0 == 0 {
            return None;
        }
        let index = self.0.trailing_zeros() as u8;
        self.0 &= self.0 - 1;
        Some(Card(index))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.count_ones() as usize;
        (n, Some(n))
    }
}

impl ExactSizeIterator for CardSetIter {}

impl IntoIterator for CardSet {
    type Item = Card;
    type IntoIter = CardSetIter;

    fn into_iter(self) -> CardSetIter {
        self.iter()
    }
}

impl FromIterator<Card> for CardSet {
    fn from_iter<I: IntoIterator<Item = Card>>(iter: I) -> CardSet {
        let mut set = CardSet::EMPTY;
        for card in iter {
            set.insert(card);
        }
        set
    }
}

impl From<Card> for CardSet {
    fn from(card: Card) -> CardSet {
        CardSet::single(card)
    }
}

impl BitOr for CardSet {
    type Output = CardSet;
    fn bitor(self, rhs: CardSet) -> CardSet {
        self.union(rhs)
    }
}

impl BitOrAssign for CardSet {
    fn bitor_assign(&mut self, rhs: CardSet) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for CardSet {
    type Output = CardSet;
    fn bitand(self, rhs: CardSet) -> CardSet {
        self.intersection(rhs)
    }
}

impl BitAndAssign for CardSet {
    fn bitand_assign(&mut self, rhs: CardSet) {
        self.0 &= rhs.0;
    }
}

impl Sub for CardSet {
    type Output = CardSet;
    fn sub(self, rhs: CardSet) -> CardSet {
        self.difference(rhs)
    }
}

impl SubAssign for CardSet {
    fn sub_assign(&mut self, rhs: CardSet) {
        self.0 &= !rhs.0;
    }
}

impl Not for CardSet {
    type Output = CardSet;
    fn not(self) -> CardSet {
        CardSet(!self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_roundtrip_and_points() {
        let mut total = 0u32;
        for index in 0..32u8 {
            let card = Card::from_index(index).unwrap();
            assert_eq!(Card::new(card.suit(), card.rank()), card);
            total += u32::from(card.augen());
        }
        assert_eq!(total, 120);
        assert_eq!(Card::from_index(32), None);
    }

    #[test]
    fn set_operations() {
        let ass = Card::new(Suit::Herz, Rank::Ass);
        let zehn = Card::new(Suit::Herz, Rank::Zehn);
        let mut set = CardSet::EMPTY;
        assert!(set.insert(ass));
        assert!(!set.insert(ass));
        set.insert(zehn);
        assert_eq!(set.len(), 2);
        assert_eq!(set.augen(), 21);
        assert!(set.is_subset(Suit::Herz.cards()));
        assert!(set.is_disjoint(Suit::Karo.cards()));
        assert!(set.remove(ass));
        assert_eq!(set, CardSet::single(zehn));
        assert_eq!(CardSet::ALL.len(), 32);
        assert_eq!(CardSet::ALL.augen(), 120);
        assert_eq!((CardSet::ALL - Suit::Kreuz.cards()).len(), 24);
    }
}
