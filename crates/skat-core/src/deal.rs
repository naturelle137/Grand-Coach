// SPDX-License-Identifier: EUPL-1.2

//! Dealing: the only source of randomness in the core (D-22).

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};

use crate::card::{Card, CardSet};
use crate::seat::Seat;

/// Hands of the three seats and the Skat before bidding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "DealData")
)]
pub struct Deal {
    hands: [CardSet; 3],
    skat: CardSet,
}

/// Why a fixed deal is invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DealError {
    /// A hand does not have exactly ten cards.
    HandSize(Seat),
    /// The Skat does not have exactly two cards.
    SkatSize,
    /// Hands and Skat overlap, so they do not form the full deck.
    Overlap,
}

impl core::fmt::Display for DealError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DealError::HandSize(seat) => write!(f, "hand of {seat:?} does not have ten cards"),
            DealError::SkatSize => write!(f, "the Skat does not have two cards"),
            DealError::Overlap => write!(f, "hands and Skat do not form the 32 cards"),
        }
    }
}

impl core::error::Error for DealError {}

/// Unvalidated wire form of [`Deal`]; deserialising goes through [`Deal::new`].
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct DealData {
    hands: [CardSet; 3],
    skat: CardSet,
}

#[cfg(feature = "serde")]
impl TryFrom<DealData> for Deal {
    type Error = DealError;

    fn try_from(data: DealData) -> Result<Deal, DealError> {
        Deal::new(data.hands, data.skat)
    }
}

impl Deal {
    /// Validates a fixed deal: ten cards per hand, two in the Skat, together
    /// exactly the 32 cards.
    pub fn new(hands: [CardSet; 3], skat: CardSet) -> Result<Deal, DealError> {
        for seat in Seat::ALL {
            if hands[seat.index()].len() != 10 {
                return Err(DealError::HandSize(seat));
            }
        }
        if skat.len() != 2 {
            return Err(DealError::SkatSize);
        }
        let all = hands[0] | hands[1] | hands[2] | skat;
        if all != CardSet::ALL {
            return Err(DealError::Overlap);
        }
        Ok(Deal { hands, skat })
    }

    /// Shuffles with `ChaCha8Rng` seeded from `seed` and deals in the ISkO
    /// pattern 3 – Skat 2 – 4 – 3, starting with Vorhand.
    ///
    /// The shuffle is a Fisher–Yates shuffle with rejection sampling written
    /// here, so the deal for a seed depends only on the ChaCha8 stream.
    pub fn from_seed(seed: u64) -> Deal {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut deck: [u8; 32] = core::array::from_fn(|i| i as u8);
        for i in (1..deck.len()).rev() {
            let j = uniform_below(&mut rng, i as u32 + 1) as usize;
            deck.swap(i, j);
        }
        let mut cards = deck.iter().map(|&index| Card::from_index(index).unwrap());
        let mut hands = [CardSet::EMPTY; 3];
        let mut skat = CardSet::EMPTY;
        for hand in &mut hands {
            hand.extend_from(&mut cards, 3);
        }
        skat.extend_from(&mut cards, 2);
        for packet in [4, 3] {
            for hand in &mut hands {
                hand.extend_from(&mut cards, packet);
            }
        }
        Deal { hands, skat }
    }

    /// Hand of a seat before bidding.
    pub fn hand(&self, seat: Seat) -> CardSet {
        self.hands[seat.index()]
    }

    /// The two Skat cards.
    pub fn skat(&self) -> CardSet {
        self.skat
    }
}

impl CardSet {
    fn extend_from(&mut self, cards: &mut impl Iterator<Item = Card>, n: usize) {
        for card in cards.take(n) {
            self.insert(card);
        }
    }
}

/// Uniform value in `0..n` without modulo bias.
fn uniform_below(rng: &mut ChaCha8Rng, n: u32) -> u32 {
    let zone = u32::MAX - u32::MAX % n;
    loop {
        let x = rng.next_u32();
        if x < zone {
            return x % n;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_deals_are_valid_and_reproducible() {
        for seed in 0..200 {
            let deal = Deal::from_seed(seed);
            assert_eq!(Deal::new(deal.hands, deal.skat), Ok(deal));
            assert_eq!(Deal::from_seed(seed), deal);
        }
        assert_ne!(Deal::from_seed(1), Deal::from_seed(2));
    }

    #[test]
    fn invalid_fixed_deals_are_rejected() {
        let d = Deal::from_seed(7);
        let mut hands = d.hands;
        let moved = hands[0].iter().next().unwrap();
        hands[0].remove(moved);
        assert_eq!(
            Deal::new(hands, d.skat),
            Err(DealError::HandSize(Seat::Vorhand))
        );
        let mut hands = d.hands;
        let other = hands[1].iter().next().unwrap();
        hands[0].remove(moved);
        hands[0].insert(other);
        assert_eq!(Deal::new(hands, d.skat), Err(DealError::Overlap));
        let skat = d.skat | CardSet::single(moved);
        assert_eq!(Deal::new(d.hands, skat), Err(DealError::SkatSize));
    }
}
