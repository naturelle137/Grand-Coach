// SPDX-License-Identifier: EUPL-1.2

//! View model of the table for the human player.

use serde::{Deserialize, Serialize};
use skat_core::{
    Action, Call, Card, CompletedTrick, Contract, GameResult, KontraLevel, Phase, PlayerView, Seat,
};

use crate::coach::CoachView;
use crate::series::{ScoreSheet, SeriesResult};
use crate::session::{PlayerId, SessionConfig};
use crate::sort::sort_hand;

/// Everything a frontend shows. Built only from the human's
/// [`PlayerView`], so it never contains hidden cards of the opponents or an
/// unrevealed Skat (D-23).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableView {
    /// Settings of the session.
    pub config: SessionConfig,
    /// Number of the current game, from 1.
    pub game_number: u32,
    /// Dealer of the current game.
    pub dealer: PlayerId,
    /// The three players (index: [`PlayerId`]).
    pub players: [SeatInfo; 3],
    /// Current phase of the game.
    pub phase: Phase,
    /// Player whose turn it is; `None` when the game is over.
    pub acting: Option<PlayerId>,
    /// `true` if an opponent has to act ([`crate::Command::AdvanceOpponent`]).
    pub opponent_to_move: bool,
    /// Current or final bid.
    pub bid: u16,
    /// Bidding history.
    pub bidding: Vec<(PlayerId, Call)>,
    /// Declarer, once bidding is over.
    pub declarer: Option<PlayerId>,
    /// `Some(true)` for Hand, `Some(false)` after Skat pickup.
    pub hand_game: Option<bool>,
    /// Announced contract.
    pub contract: Option<Contract>,
    /// Kontra/Re level.
    pub kontra: KontraLevel,
    /// The human's cards in display order.
    pub hand: Vec<Card>,
    /// The two cards the human discarded as declarer.
    pub own_skat: Option<Vec<Card>>,
    /// Open cards of an Ouvert declarer, in display order.
    pub declarer_cards: Option<Vec<Card>>,
    /// Trick in progress.
    pub current_trick: Vec<PlayedCard>,
    /// Last completed trick.
    pub last_trick: Option<TrickView>,
    /// Number of completed tricks.
    pub tricks_played: u8,
    /// Legal actions of the human; empty when it is not the human's turn.
    pub legal_actions: Vec<Action>,
    /// Result once the game is finished.
    pub result: Option<ResultView>,
    /// Score sheet of the series.
    pub sheet: ScoreSheet,
    /// `true` once the series is over.
    pub series_finished: bool,
    /// Final result once the series is over.
    pub series_result: Option<SeriesResult>,
    /// [`crate::Command::NextGame`] is possible.
    pub can_next_game: bool,
    /// [`crate::Command::EndSeries`] is possible.
    pub can_end_series: bool,
    /// The live coach.
    pub coach: CoachView,
}

/// One player at the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatInfo {
    /// The player.
    pub player: PlayerId,
    /// Seat in the current deal.
    pub seat: Seat,
    /// Number of cards held.
    pub cards: u8,
    /// Last call in the bidding.
    pub last_call: Option<Call>,
    /// Tricks won in this game.
    pub tricks: u8,
    /// Seeger-Fabian total of the series.
    pub total: i32,
}

/// A card played by a player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayedCard {
    /// Player who played it.
    pub player: PlayerId,
    /// The card.
    pub card: Card,
}

/// A completed trick.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrickView {
    /// Cards in play order.
    pub cards: Vec<PlayedCard>,
    /// Player who won it.
    pub winner: PlayerId,
    /// Card points in the trick.
    pub augen: u8,
}

/// A finished game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultView {
    /// Declarer.
    pub declarer: PlayerId,
    /// Settled result from `skat-core`.
    pub result: GameResult,
    /// Seeger-Fabian points of this game per player.
    pub points: [i32; 3],
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_view(
    config: &SessionConfig,
    game_number: u32,
    dealer: PlayerId,
    sheet: &ScoreSheet,
    series_finished: bool,
    opponent_to_move: bool,
    view: &PlayerView,
    coach: CoachView,
) -> TableView {
    let player = |seat: Seat| PlayerId::on_seat(seat, dealer);
    let game = view.contract.map(|c| c.game);
    let totals = sheet.totals();
    let players = PlayerId::ALL.map(|p| {
        let seat = p.seat(dealer);
        SeatInfo {
            player: p,
            seat,
            cards: view.hand_sizes[seat.index()],
            last_call: view
                .bidding
                .iter()
                .rev()
                .find(|r| r.seat == seat)
                .map(|r| r.call),
            tricks: view.tricks.iter().filter(|t| t.winner == seat).count() as u8,
            total: totals[p.index()],
        }
    });
    let trick_view = |t: &CompletedTrick| TrickView {
        cards: (0..3)
            .map(|i| PlayedCard {
                player: player(t.seat_of(i)),
                card: t.cards[i],
            })
            .collect(),
        winner: player(t.winner),
        augen: t.augen(),
    };
    let current_trick = view
        .current_trick
        .cards
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let leader = view.current_trick.leader?;
            c.map(|card| PlayedCard {
                player: player(Seat::from_index(leader.index() + i)),
                card,
            })
        })
        .collect();
    let game_over = view.current_seat.is_none();
    let result = view.result.map(|result| {
        let by_seat = result.seeger_fabian();
        ResultView {
            declarer: player(result.outcome.declarer),
            result,
            points: PlayerId::ALL.map(|p| by_seat[p.seat(dealer).index()]),
        }
    });
    TableView {
        config: *config,
        game_number,
        dealer,
        players,
        phase: view.phase,
        acting: view.current_seat.map(player),
        opponent_to_move,
        bid: view.bid,
        bidding: view
            .bidding
            .iter()
            .map(|r| (player(r.seat), r.call))
            .collect(),
        declarer: view.declarer.map(player),
        hand_game: view.hand_game,
        contract: view.contract,
        kontra: view.kontra,
        hand: sort_hand(view.hand, game),
        own_skat: view.own_skat.map(|s| sort_hand(s, game)),
        declarer_cards: view.declarer_cards.map(|s| sort_hand(s, game)),
        current_trick,
        last_trick: view.tricks.last().map(trick_view),
        tricks_played: view.tricks.len() as u8,
        legal_actions: if series_finished {
            Vec::new()
        } else {
            view.legal_actions.clone()
        },
        result,
        sheet: sheet.clone(),
        series_finished,
        series_result: series_finished.then(|| sheet.result()),
        can_next_game: game_over && !series_finished,
        can_end_series: game_over
            && !series_finished
            && config.series == crate::session::SeriesLength::Free,
        coach,
    }
}
