// SPDX-License-Identifier: EUPL-1.2

//! Spitzen, game value, Überreizen, Kontra/Re and Seeger-Fabian (D-24).
//!
//! All functions here are pure: they work on a [`GameOutcome`] and can be
//! tested without playing a game.

use crate::card::CardSet;
use crate::rules::{Contract, GameType};
use crate::seat::Seat;

/// Matadors: played "with" or "without" `count` Spitzen (ISkO 2.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Spitzen {
    /// `true` for "mit" (declarer holds the Kreuz Bube).
    pub mit: bool,
    /// Length of the unbroken run held or missing.
    pub count: u8,
}

/// Spitzen of the declarer's cards (ten cards plus Skat, also in Hand
/// games). `None` for Null.
pub fn spitzen(game: GameType, declarer_cards: CardSet) -> Option<Spitzen> {
    let order = game.trump_order();
    let first = *order.first()?;
    let mit = declarer_cards.contains(first);
    let count = order
        .iter()
        .take_while(|&&card| declarer_cards.contains(card) == mit)
        .count() as u8;
    Some(Spitzen { mit, count })
}

/// Kontra/Re level of a game (D-03).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KontraLevel {
    /// No Kontra.
    #[default]
    None,
    /// Kontra: game points doubled.
    Kontra,
    /// Kontra and Re: game points quadrupled.
    Re,
}

impl KontraLevel {
    /// Factor applied to the game points.
    pub const fn factor(self) -> i32 {
        match self {
            KontraLevel::None => 1,
            KontraLevel::Kontra => 2,
            KontraLevel::Re => 4,
        }
    }
}

/// Everything needed to settle a finished game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameOutcome {
    /// Declarer.
    pub declarer: Seat,
    /// The game played.
    pub contract: Contract,
    /// Final bid.
    pub bid: u16,
    /// Spitzen of the declarer; ignored for Null.
    pub spitzen: Option<Spitzen>,
    /// Card points of the declarer including the Skat.
    pub declarer_augen: u8,
    /// Tricks won by the declarer.
    pub declarer_tricks: u8,
    /// Tricks played (10, or fewer if a Null game ended early).
    pub tricks_played: u8,
    /// Kontra/Re level.
    pub kontra: KontraLevel,
}

/// Settled result of a game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameResult {
    /// The input this result was computed from.
    pub outcome: GameOutcome,
    /// `true` if the declarer won.
    pub won: bool,
    /// The game value did not reach the bid (ISkO 5.4).
    pub ueberreizt: bool,
    /// Schneider level counted (suit and Grand).
    pub schneider: bool,
    /// Schwarz level counted (suit and Grand).
    pub schwarz: bool,
    /// Game value from Spitzen and levels, or the fixed Null value.
    pub game_value: u16,
    /// Value the game is scored with: `game_value`, or for an overbid game
    /// the smallest multiple of the base value that reaches the bid.
    pub scored_value: u16,
    /// Points for the declarer's list: `+scored_value` if won,
    /// `-2 * scored_value` if lost, times the Kontra/Re factor.
    pub score: i32,
}

impl GameResult {
    /// Seeger-Fabian contribution of each seat (index by [`Seat::index`]):
    /// the declarer gets `score + 50` if won, `score - 50` if lost; each
    /// defender gets 40 if the declarer lost. The 50 and 40 are not affected
    /// by Kontra/Re (D-03.4).
    pub fn seeger_fabian(&self) -> [i32; 3] {
        let mut points = [0; 3];
        let declarer = self.outcome.declarer;
        points[declarer.index()] = self.score + if self.won { 50 } else { -50 };
        if !self.won {
            for seat in Seat::ALL {
                if seat != declarer {
                    points[seat.index()] = 40;
                }
            }
        }
        points
    }
}

/// Settles a finished game (ISkO 5.2 to 5.4, Kontra/Re per D-03).
pub fn settle(outcome: &GameOutcome) -> GameResult {
    let contract = outcome.contract;
    let (won, ueberreizt, schneider, schwarz, game_value, scored_value) =
        if let Some(value) = contract.null_value() {
            let won = outcome.declarer_tricks == 0;
            (won, false, false, false, value, value)
        } else {
            settle_trump_game(outcome)
        };
    let base = i32::from(scored_value);
    let score = if won { base } else { -2 * base } * outcome.kontra.factor();
    GameResult {
        outcome: *outcome,
        won,
        ueberreizt,
        schneider,
        schwarz,
        game_value,
        scored_value,
        score,
    }
}

fn settle_trump_game(o: &GameOutcome) -> (bool, bool, bool, bool, u16, u16) {
    let c = o.contract;
    let base = c.game.base_value().expect("suit or Grand");
    let augen = o.declarer_augen;
    let all_tricks = o.declarer_tricks == o.tricks_played && o.tricks_played == 10;
    let no_tricks = o.declarer_tricks == 0;
    let schneider = c.schneider || augen >= 90 || augen <= 30;
    let schwarz = c.schwarz || all_tricks || no_tricks;
    let spitzen = u16::from(o.spitzen.map_or(0, |s| s.count));
    let multiplier = spitzen
        + 1
        + u16::from(c.hand)
        + u16::from(schneider)
        + u16::from(c.schneider)
        + u16::from(schwarz)
        + u16::from(c.schwarz)
        + u16::from(c.ouvert);
    let game_value = base * multiplier;
    let ueberreizt = game_value < o.bid;
    let fulfilled = augen >= 61 && (!c.schneider || augen >= 90) && (!c.schwarz || all_tricks);
    let won = fulfilled && !ueberreizt;
    let scored_value = if ueberreizt {
        o.bid.div_ceil(base) * base
    } else {
        game_value
    };
    (
        won,
        ueberreizt,
        schneider,
        schwarz,
        game_value,
        scored_value,
    )
}
