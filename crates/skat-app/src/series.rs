// SPDX-License-Identifier: EUPL-1.2

//! Score sheet of a series with Seeger-Fabian totals (D-05, D-24, D-31).

use serde::{Deserialize, Serialize};
use skat_core::{Contract, GameResult};

use crate::session::PlayerId;

/// One line of the score sheet: a settled game or a thrown-in deal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameRecord {
    /// Game number in the series, from 1.
    pub number: u32,
    /// Dealer (Geber) of this deal.
    pub dealer: PlayerId,
    /// Declarer; `None` if the deal was thrown in (eingepasst).
    pub declarer: Option<PlayerId>,
    /// Played contract; `None` if thrown in.
    pub contract: Option<Contract>,
    /// Final bid; 0 if thrown in.
    pub bid: u16,
    /// Settled result as computed by `skat-core`; `None` if thrown in.
    pub result: Option<GameResult>,
    /// Seeger-Fabian points of this game per player (index: [`PlayerId`]).
    pub points: [i32; 3],
    /// Running totals after this game per player.
    pub totals: [i32; 3],
}

impl GameRecord {
    /// `true` if all passed and the deal was thrown in.
    pub fn eingepasst(&self) -> bool {
        self.result.is_none()
    }
}

/// All games of the series so far.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreSheet {
    /// Games in order.
    pub records: Vec<GameRecord>,
}

impl ScoreSheet {
    /// Seeger-Fabian totals per player.
    pub fn totals(&self) -> [i32; 3] {
        self.records.last().map_or([0; 3], |r| r.totals)
    }

    /// Games won as declarer per player.
    pub fn won(&self) -> [u32; 3] {
        self.count(true)
    }

    /// Games lost as declarer per player.
    pub fn lost(&self) -> [u32; 3] {
        self.count(false)
    }

    fn count(&self, won: bool) -> [u32; 3] {
        let mut counts = [0; 3];
        for record in &self.records {
            if let (Some(declarer), Some(result)) = (record.declarer, record.result)
                && result.won == won
            {
                counts[declarer.index()] += 1;
            }
        }
        counts
    }

    /// Appends a game and computes its running totals.
    pub(crate) fn push(&mut self, mut record: GameRecord) -> GameRecord {
        let before = self.totals();
        record.totals = core::array::from_fn(|i| before[i] + record.points[i]);
        self.records.push(record);
        record
    }

    /// Final result of the series as it stands.
    pub fn result(&self) -> SeriesResult {
        let totals = self.totals();
        let mut ranking = PlayerId::ALL.to_vec();
        ranking.sort_by_key(|p| core::cmp::Reverse(totals[p.index()]));
        SeriesResult {
            games: self.records.len() as u32,
            totals,
            won: self.won(),
            lost: self.lost(),
            ranking,
        }
    }
}

/// Final result of a series.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeriesResult {
    /// Games played, thrown-in deals included.
    pub games: u32,
    /// Seeger-Fabian totals per player.
    pub totals: [i32; 3],
    /// Games won as declarer per player.
    pub won: [u32; 3],
    /// Games lost as declarer per player.
    pub lost: [u32; 3],
    /// Players from best to worst total; ties keep player order.
    pub ranking: Vec<PlayerId>,
}
