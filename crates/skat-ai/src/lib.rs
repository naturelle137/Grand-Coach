// SPDX-License-Identifier: EUPL-1.2

//! Opponent AI. M2 has one baseline opponent: legal, fair and reproducible,
//! deliberately weak. Strength levels follow in M4 (D-07).
//!
//! An opponent only ever receives the [`PlayerView`] of its own seat (D-23)
//! and picks one of the legal actions listed there.

mod baseline;

pub use baseline::BaselineOpponent;

use skat_core::{Action, PlayerView};

/// An automated player.
pub trait Opponent {
    /// Chooses one of `view.legal_actions`; `None` if the list is empty,
    /// i.e. it is not this seat's turn.
    fn decide(&self, view: &PlayerView) -> Option<Action>;
}
