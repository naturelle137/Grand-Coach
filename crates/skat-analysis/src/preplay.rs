// SPDX-License-Identifier: EUPL-1.2

//! Exact facts before card play for the lessons on hand structure and game
//! value (#45, `docs/coach.md`): per candidate game the own Buben, Trumpf
//! and Spitzen, and the game values the rules allow with them.
//!
//! Everything here follows from the deciding seat's view and the rules
//! (ISkO 2.4 Spitzen, 5.1 game value, 5.4 Überreizen): the own cards, the
//! own discarded Skat once known, the Hand decision and the bid. An unknown
//! Skat counts for the Spitzen (also in Hand games) but is not seen; every
//! pair of unseen cards may lie in it, since nothing constrains the Skat
//! before card play. The ranges below are taken over all those pairs, so
//! they are bounds, not estimates. No search, no opponent model, and no
//! claim which game, bid or discard is better.

use skat_core::{
    Action, Announcement, CardSet, Contract, GameType, Phase, PlayerView, Spitzen, spitzen,
};

use crate::hand::TRUMP_GAMES;

/// What the deciding seat knows about the Skat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SkatState {
    /// Not seen (bidding, Hand or pickup still open, Hand game): it may
    /// hold any two unseen cards and counts for the declarer's Spitzen.
    Unknown,
    /// Picked up: among the twelve own cards, the discard not yet made.
    Held,
    /// Put away: the own discarded Skat, known and no longer playable.
    Discarded,
}

/// Lowest and highest value over every Skat the seat cannot exclude; equal
/// when the Skat is known or cannot matter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Range {
    /// The lowest value.
    pub low: u16,
    /// The highest value.
    pub high: u16,
}

impl Range {
    /// `true` if the value does not depend on the unknown Skat.
    pub fn fixed(self) -> bool {
        self.low == self.high
    }
}

/// The own Buben and Trumpf of one candidate game (Farbspiel or Grand).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Structure {
    /// The candidate game.
    pub game: GameType,
    /// What the seat knows about the Skat.
    pub skat: SkatState,
    /// Own Buben: in the hand and in the own discarded Skat.
    pub buben: CardSet,
    /// Trumpf of the game in the own hand (Buben included; with
    /// [`SkatState::Held`] all twelve cards).
    pub trumpf: CardSet,
    /// Trumpf in the own discarded Skat: they count for the Spitzen but
    /// cannot be played.
    pub skat_trumpf: CardSet,
    /// Spitzen of the own known cards (hand and own Skat).
    pub spitzen: Spitzen,
    /// Fewest and most Spitzen (counted "mit" or "ohne") over every Skat
    /// the seat cannot exclude.
    pub counts: Range,
    /// Most own Trumpf (hand and Skat) over every Skat the seat cannot
    /// exclude: the own ones plus at most two from an unknown Skat.
    pub trumpf_most: u8,
}

/// The game values of one game type with the own cards, before any level
/// made during card play (Schneider, Schwarz). `None` where the contract
/// is no longer possible (Hand decided) or does not exist (suit or Grand
/// Ouvert without Hand).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameValues {
    /// The game type (Farbspiel, Grand or Null).
    pub game: GameType,
    /// What the seat knows about the Skat.
    pub skat: SkatState,
    /// With pickup, nothing announced.
    pub plain: Option<Range>,
    /// Hand, nothing else announced.
    pub hand: Option<Range>,
    /// Null Ouvert with pickup (suit and Grand: none).
    pub ouvert: Option<Range>,
    /// Hand Ouvert: Null Ouvert Hand; suit and Grand with Schneider and
    /// Schwarz announced, as Ouvert requires.
    pub hand_ouvert: Option<Range>,
}

/// One contract with its value range and how it relates to the value the
/// game must reach.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContractValue {
    /// The contract.
    pub contract: Contract,
    /// Its value range.
    pub range: Range,
    /// How the range relates to the value to reach.
    pub reach: Reach,
}

/// How a contract's value relates to the value the seat must reach.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Reach {
    /// The lowest value reaches it: this contract cannot be überreizt.
    Reached,
    /// Only with some of the possible Skats.
    Skat,
    /// Not with the Spitzen and announced levels; only Schneider or
    /// Schwarz made during card play can still raise the value.
    Play,
    /// Never: the value is fixed (Null) or every level is announced.
    Never,
}

impl GameValues {
    /// Each available contract with its value range: plain, Hand, Ouvert,
    /// Hand Ouvert, in that order.
    pub fn contracts(&self) -> Vec<(Contract, Range)> {
        let [plain, ouvert, hand_ouvert] = announcements(self.game);
        [
            (plain, false, self.plain),
            (plain, true, self.hand),
            (ouvert, false, self.ouvert),
            (hand_ouvert, true, self.hand_ouvert),
        ]
        .into_iter()
        .filter_map(|(a, hand, range)| Some((Contract::new(a, hand), range?)))
        .collect()
    }

    /// Each available contract measured against `value`, the value the
    /// game must reach ([`commitment`]).
    pub fn against(&self, value: u16) -> Vec<ContractValue> {
        self.contracts()
            .into_iter()
            .map(|(contract, range)| ContractValue {
                contract,
                range,
                reach: GameValues::reach(contract, range, value),
            })
            .collect()
    }

    /// How `range` of `contract` relates to `value`.
    pub fn reach(contract: Contract, range: Range, value: u16) -> Reach {
        if range.low >= value {
            Reach::Reached
        } else if range.high >= value {
            Reach::Skat
        } else if contract.null_value().is_some() || contract.ouvert {
            Reach::Never
        } else {
            Reach::Play
        }
    }
}

/// The value the game must reach to avoid Überreizen: the bid reached
/// once bidding is over, or during bidding the lowest value an own Bid or
/// Hold commits the seat to. `None` without such a decision.
pub fn commitment(view: &PlayerView) -> Option<u16> {
    match view.phase {
        Phase::Reizen => view
            .legal_actions
            .iter()
            .filter_map(|a| match *a {
                Action::Bid(v) => Some(v),
                Action::Hold => Some(view.bid),
                _ => None,
            })
            .min(),
        Phase::SkatAufnahme | Phase::Druecken | Phase::Ansage
            if view.declarer == Some(view.seat) =>
        {
            Some(view.bid)
        }
        _ => None,
    }
}

/// Whether `view` is a decision before card play of the bidder or
/// declarer, the only ones these facts are about.
fn applies(view: &PlayerView) -> bool {
    match view.phase {
        Phase::Reizen => view.declarer.is_none(),
        Phase::SkatAufnahme | Phase::Druecken | Phase::Ansage => view.declarer == Some(view.seat),
        _ => false,
    }
}

/// The Skat state and the own known cards (hand and own Skat).
fn known(view: &PlayerView) -> (SkatState, CardSet) {
    match (view.own_skat, view.phase) {
        (Some(skat), _) => (SkatState::Discarded, view.hand | skat),
        (None, Phase::Druecken) => (SkatState::Held, view.hand),
        _ => (SkatState::Unknown, view.hand),
    }
}

/// Every Skat the seat cannot exclude: the empty addition if it is known,
/// otherwise every pair of unseen cards.
fn skats(state: SkatState, own: CardSet) -> Vec<CardSet> {
    if state != SkatState::Unknown {
        return vec![CardSet::EMPTY];
    }
    let unseen: Vec<_> = (CardSet::ALL - own).iter().collect();
    let mut out = Vec::with_capacity(unseen.len() * unseen.len() / 2);
    for (i, &a) in unseen.iter().enumerate() {
        for &b in &unseen[i + 1..] {
            out.push(CardSet::single(a) | CardSet::single(b));
        }
    }
    out
}

/// The hand structure of each Farbspiel and Grand, in the order Kreuz, Pik,
/// Herz, Karo, Grand; empty outside bidder and declarer decisions before
/// card play.
pub fn structures(view: &PlayerView) -> Vec<Structure> {
    if !applies(view) {
        return Vec::new();
    }
    let (state, own) = known(view);
    let skats = skats(state, own);
    let skat = view.own_skat.unwrap_or(CardSet::EMPTY);
    TRUMP_GAMES
        .into_iter()
        .map(|game| {
            let count = |cards: CardSet| spitzen(game, cards).map_or(0, |s| s.count);
            let counts = skats.iter().map(|&s| u16::from(count(own | s)));
            let trumps = game.trumps();
            Structure {
                game,
                skat: state,
                buben: own & GameType::BUBEN,
                trumpf: view.hand & trumps,
                skat_trumpf: skat & trumps,
                spitzen: spitzen(game, own).expect("a game with Trumpf"),
                counts: Range {
                    low: counts.clone().min().unwrap_or(0),
                    high: counts.max().unwrap_or(0),
                },
                trumpf_most: skats
                    .iter()
                    .map(|&s| ((own | s) & trumps).len() as u8)
                    .max()
                    .unwrap_or(0),
            }
        })
        .collect()
}

/// The game values of each game type, in the order Kreuz, Pik, Herz,
/// Karo, Grand, Null, for the contracts still possible; empty outside
/// bidder and declarer decisions before card play.
pub fn game_values(view: &PlayerView) -> Vec<GameValues> {
    if !applies(view) {
        return Vec::new();
    }
    let (state, own) = known(view);
    let skats = skats(state, own);
    let (pickup, hand) = match view.hand_game {
        None => (true, true),
        Some(h) => (!h, h),
    };
    let range = |contract: Contract| -> Range {
        if let Some(v) = contract.null_value() {
            return Range { low: v, high: v };
        }
        let values = skats.iter().map(|&s| value(contract, own | s));
        Range {
            low: values.clone().min().unwrap_or(0),
            high: values.max().unwrap_or(0),
        }
    };
    let mut games = TRUMP_GAMES.to_vec();
    games.push(GameType::Null);
    games
        .into_iter()
        .map(|game| {
            let [plain, ouvert, hand_ouvert] = announcements(game);
            GameValues {
                game,
                skat: state,
                plain: pickup.then(|| range(Contract::new(plain, false))),
                hand: hand.then(|| range(Contract::new(plain, true))),
                ouvert: (pickup && game == GameType::Null)
                    .then(|| range(Contract::new(ouvert, false))),
                hand_ouvert: hand.then(|| range(Contract::new(hand_ouvert, true))),
            }
        })
        .collect()
}

/// The plain, Ouvert and Hand Ouvert announcements of `game`; suit and
/// Grand Ouvert also announce Schneider and Schwarz (ISkO 5.2.6).
fn announcements(game: GameType) -> [Announcement; 3] {
    let plain = Announcement::plain(game);
    let ouvert = Announcement {
        ouvert: true,
        ..plain
    };
    let trump = game != GameType::Null;
    let hand_ouvert = Announcement {
        schneider: trump,
        schwarz: trump,
        ..ouvert
    };
    [plain, ouvert, hand_ouvert]
}

/// Game value of a suit or Grand `contract` with the declarer's `cards`
/// (hand and Skat), with the announced levels only (ISkO 5.1): Spitzen,
/// game, Hand, Schneider and Schwarz each with their announcement, Ouvert.
fn value(contract: Contract, cards: CardSet) -> u16 {
    let base = contract.game.base_value().expect("suit or Grand");
    let count = spitzen(contract.game, cards).map_or(0, |s| u16::from(s.count));
    let levels = [
        contract.hand,
        contract.schneider,
        contract.schneider,
        contract.schwarz,
        contract.schwarz,
        contract.ouvert,
    ];
    base * (count + 1 + levels.iter().map(|&l| u16::from(l)).sum::<u16>())
}
