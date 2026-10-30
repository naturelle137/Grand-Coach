// SPDX-License-Identifier: EUPL-1.2

//! Shared analysis for opponents and coach (architecture: "coach and AI do
//! not compute separately").
//!
//! - card counting: what one seat knows from its
//!   [`PlayerView`](skat_core::PlayerView) ([`Knowledge`]) and the trick in
//!   progress ([`TrickState`]),
//! - opponent model: hidden-card distributions under hard constraints
//!   ([`sampling`]) weighted by bidding evidence ([`evidence`]),
//! - option evaluation for every decision ([`analyze`]): Monte Carlo over
//!   those distributions, double-dummy card play ([`double_dummy`]) and a
//!   shared continuation policy ([`policy`]), within a deterministic
//!   [`Budget`],
//! - simple hand measures ([`hand`]).
//!
//! Semantics, budgets and measured latency: `docs/analysis.md`.

mod analysis;
mod budget;
pub mod double_dummy;
pub mod evidence;
pub mod hand;
mod knowledge;
pub mod policy;
pub mod sampling;
mod trick;

pub use analysis::{
    Analysis, Assumption, CardClass, Effort, Estimate, Inference, Observation, OptionValue, Status,
    analyze, analyze_forgetful, analyze_known, utility,
};
pub use budget::{Budget, Cancel, Interrupted, Meter};
pub use knowledge::Knowledge;
pub use trick::TrickState;
