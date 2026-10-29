// SPDX-License-Identifier: EUPL-1.2

//! Deterministic Skat rule engine following the ISkO, with optional
//! Kontra/Re (D-03, D-26) and Seeger-Fabian scoring.
//!
//! Pure logic: no IO, no GUI, no texts, no strategy. The only randomness is
//! the seeded deal ([`Deal::from_seed`]). See `docs/architecture.md`.
//!
//! ```
//! use skat_core::{Game, Phase, RuleConfig};
//!
//! let mut game = Game::new(42, RuleConfig { kontra_re: true });
//! while let Some(&action) = game.legal_actions().last() {
//!     game.apply(action).unwrap();
//! }
//! assert!(matches!(game.phase(), Phase::Beendet | Phase::Eingepasst));
//! ```

mod bidding;
mod card;
mod deal;
mod game;
mod rules;
mod scoring;
mod seat;
mod view;

pub use bidding::{BidRecord, Call, is_reizwert, reizwerte};
pub use card::{Card, CardSet, CardSetIter, InvalidCardIndex, Rank, Suit};
pub use deal::{Deal, DealError};
pub use game::{Action, CompletedTrick, Event, Game, Phase, RuleConfig, RuleError, Trick};
pub use rules::{Announcement, Contract, Follow, GameType, legal_cards, trick_winner};
pub use scoring::{GameOutcome, GameResult, KontraLevel, Spitzen, settle, spitzen};
pub use seat::Seat;
pub use view::{OmniscientView, PlayerView};
