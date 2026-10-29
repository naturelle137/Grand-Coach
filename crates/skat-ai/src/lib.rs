// SPDX-License-Identifier: EUPL-1.2

//! Opponent AI with three strength levels (M4, D-07, D-40): [`Level`]
//! selects a [`Profile`] of analysis effort and selection noise; an
//! [`Opponent`] picks one of the options evaluated by `skat-analysis`. The
//! weakest level plays the rules of thumb of [`BaselineOpponent`].
//!
//! An opponent only ever receives the [`PlayerView`](skat_core::PlayerView)
//! of its own seat (D-23) and picks one of the legal actions listed there.
//! [`arena`] plays AI-vs-AI games for calibration (`docs/ai.md`).

pub mod arena;
mod baseline;
mod opponent;

pub use baseline::BaselineOpponent;
pub use opponent::{Decision, Level, Opponent, Profile, utility};
pub use skat_analysis::{Budget, Cancel, Status};
