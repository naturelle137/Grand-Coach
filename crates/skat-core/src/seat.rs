// SPDX-License-Identifier: EUPL-1.2

//! Seats at the three-player table.

/// Position relative to the dealer for one deal. With three players the
/// dealer (Geber) is Hinterhand.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Seat {
    /// Fore hand: left of the dealer, leads the first trick.
    Vorhand,
    /// Middle hand.
    Mittelhand,
    /// Rear hand, the dealer at a three-player table.
    Hinterhand,
}

impl Seat {
    /// All seats in playing order.
    pub const ALL: [Seat; 3] = [Seat::Vorhand, Seat::Mittelhand, Seat::Hinterhand];

    /// Index `0..3` (Vorhand = 0).
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Seat for an index modulo 3.
    pub const fn from_index(index: usize) -> Seat {
        Seat::ALL[index % 3]
    }

    /// Next seat in playing order (clockwise).
    pub const fn next(self) -> Seat {
        Seat::from_index(self.index() + 1)
    }
}
