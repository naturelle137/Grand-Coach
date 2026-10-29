// SPDX-License-Identifier: EUPL-1.2

//! Reizen: bid values and the bidding procedure (ISkO 3.3).

use crate::seat::Seat;

/// Highest multiplier of a suit game: 11 Spitzen, game, Hand, Schneider,
/// Schneider announced, Schwarz, Schwarz announced, Ouvert.
const MAX_SUIT_MULTIPLIER: u16 = 18;
/// Highest multiplier of a Grand: 4 Spitzen plus the same seven levels.
const MAX_GRAND_MULTIPLIER: u16 = 11;
/// Fixed Null values (ISkO 5.1.2).
const NULL_VALUES: [u16; 4] = [23, 35, 46, 59];

/// All valid bid values (Reizwerte) in ascending order: every value a game
/// can have (ISkO 5.6.1).
pub fn reizwerte() -> Vec<u16> {
    let mut values: Vec<u16> = NULL_VALUES.to_vec();
    for base in [9, 10, 11, 12] {
        values.extend((2..=MAX_SUIT_MULTIPLIER).map(|m| base * m));
    }
    values.extend((2..=MAX_GRAND_MULTIPLIER).map(|m| 24 * m));
    values.sort_unstable();
    values.dedup();
    values
}

/// `true` if `value` is a valid bid value.
pub fn is_reizwert(value: u16) -> bool {
    NULL_VALUES.contains(&value)
        || [9, 10, 11, 12].iter().any(|&base| {
            value.is_multiple_of(base) && (2..=MAX_SUIT_MULTIPLIER).contains(&(value / base))
        })
        || (value.is_multiple_of(24) && (2..=MAX_GRAND_MULTIPLIER).contains(&(value / 24)))
}

/// One call in the bidding history.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Call {
    /// The seat said (bid) this value.
    Bid(u16),
    /// The seat held (said "yes" to) this value.
    Hold(u16),
    /// The seat passed.
    Pass,
}

/// A call together with the seat that made it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BidRecord {
    /// Who called.
    pub seat: Seat,
    /// What was called.
    pub call: Call,
}

/// Stage of the bidding (ISkO 3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum Stage {
    /// Mittelhand bids to Vorhand.
    First,
    /// Hinterhand bids to the survivor of the first stage.
    Second,
    /// Mittelhand and Hinterhand passed without a bid; Vorhand plays at 18
    /// or passes (ISkO 3.3.6, 3.3.7).
    VorhandAlone,
}

/// Internal bidding state.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct Bidding {
    pub(crate) stage: Stage,
    /// Seat that says values in the current stage.
    pub(crate) asker: Seat,
    /// Seat that answers in the current stage.
    pub(crate) holder: Seat,
    /// `true` if it is the holder's turn to answer.
    pub(crate) awaiting_holder: bool,
    /// Current bid, 0 if nothing has been bid yet.
    pub(crate) bid: u16,
    /// Seats that bid or held a value (D-26 Kontra eligibility).
    pub(crate) bid_or_held: [bool; 3],
    /// Hinterhand passed at its first turn while the bid was above 18 (D-26).
    pub(crate) hinterhand_passed_above_18: bool,
    pub(crate) history: Vec<BidRecord>,
}

/// Result of a bidding step.
pub(crate) enum BidOutcome {
    Continue,
    Declarer(Seat, u16),
    Eingepasst,
}

impl Bidding {
    pub(crate) fn new() -> Bidding {
        Bidding {
            stage: Stage::First,
            asker: Seat::Mittelhand,
            holder: Seat::Vorhand,
            awaiting_holder: false,
            bid: 0,
            bid_or_held: [false; 3],
            hinterhand_passed_above_18: false,
            history: Vec::new(),
        }
    }

    pub(crate) fn current_seat(&self) -> Seat {
        match self.stage {
            Stage::VorhandAlone => Seat::Vorhand,
            _ if self.awaiting_holder => self.holder,
            _ => self.asker,
        }
    }

    fn record(&mut self, seat: Seat, call: Call) {
        self.history.push(BidRecord { seat, call });
    }

    /// Caller has validated `value`.
    pub(crate) fn bid(&mut self, value: u16) -> BidOutcome {
        let seat = self.current_seat();
        self.record(seat, Call::Bid(value));
        self.bid = value;
        self.bid_or_held[seat.index()] = true;
        if self.stage == Stage::VorhandAlone {
            return BidOutcome::Declarer(Seat::Vorhand, value);
        }
        self.awaiting_holder = true;
        BidOutcome::Continue
    }

    /// Caller has validated that the holder is to answer.
    pub(crate) fn hold(&mut self) -> BidOutcome {
        let seat = self.holder;
        self.record(seat, Call::Hold(self.bid));
        self.bid_or_held[seat.index()] = true;
        self.awaiting_holder = false;
        BidOutcome::Continue
    }

    pub(crate) fn pass(&mut self) -> BidOutcome {
        let seat = self.current_seat();
        if seat == Seat::Hinterhand
            && self.stage == Stage::Second
            && !self.history.iter().any(|r| r.seat == Seat::Hinterhand)
            && self.bid > 18
        {
            self.hinterhand_passed_above_18 = true;
        }
        self.record(seat, Call::Pass);
        // The other seat of the current stage survives.
        let survivor = if seat == self.asker {
            self.holder
        } else {
            self.asker
        };
        match self.stage {
            Stage::First => {
                self.stage = Stage::Second;
                self.asker = Seat::Hinterhand;
                self.holder = survivor;
                self.awaiting_holder = false;
                BidOutcome::Continue
            }
            Stage::Second if self.bid == 0 => {
                self.stage = Stage::VorhandAlone;
                BidOutcome::Continue
            }
            Stage::Second => BidOutcome::Declarer(survivor, self.bid),
            Stage::VorhandAlone => BidOutcome::Eingepasst,
        }
    }

    /// Rebuilds the bidding from a call history, checking every call against
    /// the bidding rules. `None` if a call is out of turn or not allowed.
    pub(crate) fn replay(history: &[BidRecord]) -> Option<(Bidding, BidOutcome)> {
        let mut bidding = Bidding::new();
        let mut outcome = BidOutcome::Continue;
        for record in history {
            if !matches!(outcome, BidOutcome::Continue) || record.seat != bidding.current_seat() {
                return None;
            }
            outcome = match record.call {
                Call::Bid(value) => {
                    let alone = bidding.stage == Stage::VorhandAlone;
                    let allowed = is_reizwert(value)
                        && value > bidding.bid
                        && !bidding.awaiting_holder
                        && (!alone || value == 18);
                    if !allowed {
                        return None;
                    }
                    bidding.bid(value)
                }
                Call::Hold(value) => {
                    let allowed = bidding.awaiting_holder
                        && bidding.stage != Stage::VorhandAlone
                        && value == bidding.bid;
                    if !allowed {
                        return None;
                    }
                    bidding.hold()
                }
                Call::Pass => bidding.pass(),
            };
        }
        Some((bidding, outcome))
    }

    /// Kontra eligibility per D-26.1.
    pub(crate) fn kontra_eligible(&self, seat: Seat) -> bool {
        self.bid_or_held[seat.index()]
            || (seat == Seat::Hinterhand && self.hinterhand_passed_above_18)
    }
}

/// Kontra eligibility per D-26.1, derived from a public bidding history:
/// `true` if `seat` may say Kontra as a defender. `false` for an invalid
/// history. Lets analysis predict Kontra without the game state.
pub fn kontra_eligible(history: &[BidRecord], seat: Seat) -> bool {
    Bidding::replay(history).is_some_and(|(bidding, _)| bidding.kontra_eligible(seat))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reizwert_check_matches_list() {
        let list = reizwerte();
        for value in 0..300 {
            assert_eq!(is_reizwert(value), list.contains(&value), "{value}");
        }
    }
}
