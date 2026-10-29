// SPDX-License-Identifier: EUPL-1.2

//! Opponent model: heuristic, fallible evidence from the bidding.
//!
//! Sampling ([`crate::sampling`]) draws distributions uniformly within the
//! hard constraints. This module only weights each sample by how well the
//! other seats' bidding fits their hypothetical cards, assuming they bid
//! like the continuation policy ([`crate::policy`]):
//!
//! - a seat that bid or held a value its cards would not reach under the
//!   policy keeps weight [`BID_MISMATCH`],
//! - a seat that passed although the policy would have gone on keeps
//!   weight [`PASS_MISMATCH`] (passing with a good hand happens more often).
//!
//! Weights are never zero, so no distribution that satisfies the hard
//! constraints is excluded, and conflicting evidence only multiplies down.
//! The effective sample size reports how much the weights concentrate.

use skat_core::{BidRecord, Call, CardSet, PlayerView, Seat, reizwerte};

use crate::policy::choose_game;
use crate::sampling::World;

/// Weight factor when a seat's bid or hold exceeds its policy limit.
pub const BID_MISMATCH: f64 = 0.25;
/// Weight factor when a seat passed below its policy limit.
pub const PASS_MISMATCH: f64 = 0.5;

/// What the bidding showed about one seat.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BidSignal {
    /// Highest value the seat bid or held; 0 if none.
    pub shown: u16,
    /// Lowest value the seat refused by passing; `None` if it did not pass.
    pub refused: Option<u16>,
}

/// Bidding signals of all seats, read from a public bidding history.
pub fn signals(history: &[BidRecord]) -> [BidSignal; 3] {
    let mut out = [BidSignal::default(); 3];
    let mut standing = 0u16;
    for record in history {
        let signal = &mut out[record.seat.index()];
        match record.call {
            Call::Bid(value) | Call::Hold(value) => {
                signal.shown = signal.shown.max(value);
                standing = standing.max(value);
            }
            Call::Pass => {
                let next = reizwerte()
                    .into_iter()
                    .find(|&v| v > standing)
                    .unwrap_or(18);
                signal.refused = Some(next);
            }
        }
    }
    out
}

/// The cards a seat held when bidding, as far as `world` and the public
/// play tell: its current hand plus the cards it played. For a declarer
/// after pickup this is its twelve cards (hand, played cards and the
/// discard), an approximation of the ten it bid with.
pub fn bidding_cards(view: &PlayerView, world: &World, seat: Seat) -> CardSet {
    let mut cards = world.hands[seat.index()];
    for trick in &view.tricks {
        for i in 0..3 {
            if trick.seat_of(i) == seat {
                cards.insert(trick.cards[i]);
            }
        }
    }
    if let Some(leader) = view.current_trick.leader {
        for (i, card) in view.current_trick.cards.iter().enumerate() {
            if let Some(card) = card
                && Seat::from_index(leader.index() + i) == seat
            {
                cards.insert(*card);
            }
        }
    }
    if view.declarer == Some(seat) && view.hand_game == Some(false) {
        cards |= world.skat;
    }
    cards
}

/// Weight of `world` for the viewer of `view`: the product of the factors
/// of every other seat whose bidding does not fit its cards.
pub fn weight(view: &PlayerView, world: &World) -> f64 {
    let signals = signals(&view.bidding);
    let mut w = 1.0;
    for seat in Seat::ALL {
        if seat == view.seat {
            continue;
        }
        let (_, limit) = choose_game(bidding_cards(view, world, seat));
        let signal = signals[seat.index()];
        if signal.shown > limit {
            w *= BID_MISMATCH;
        }
        if signal.refused.is_some_and(|r| limit >= r) {
            w *= PASS_MISMATCH;
        }
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signals_from_history() {
        let history = [
            BidRecord {
                seat: Seat::Mittelhand,
                call: Call::Bid(18),
            },
            BidRecord {
                seat: Seat::Vorhand,
                call: Call::Hold(18),
            },
            BidRecord {
                seat: Seat::Mittelhand,
                call: Call::Pass,
            },
            BidRecord {
                seat: Seat::Hinterhand,
                call: Call::Pass,
            },
        ];
        let s = signals(&history);
        assert_eq!(s[Seat::Vorhand.index()].shown, 18);
        assert_eq!(s[Seat::Mittelhand.index()].shown, 18);
        assert_eq!(s[Seat::Mittelhand.index()].refused, Some(20));
        assert_eq!(s[Seat::Hinterhand.index()].refused, Some(20));
        assert_eq!(s[Seat::Vorhand.index()].refused, None);
    }
}
