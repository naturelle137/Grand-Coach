// SPDX-License-Identifier: EUPL-1.2

//! Shared analysis for opponents and coach (architecture: "coach and AI do
//! not compute separately").
//!
//! M2 only needs the baseline: what one seat knows from its
//! [`PlayerView`](skat_core::PlayerView) ([`Knowledge`]), the state of the
//! trick in progress ([`TrickState`]) and simple hand measures ([`hand`]).
//! Strategic evaluation (Monte Carlo, double-dummy) follows in M3.

pub mod hand;
mod knowledge;
mod trick;

pub use knowledge::Knowledge;
pub use trick::TrickState;
