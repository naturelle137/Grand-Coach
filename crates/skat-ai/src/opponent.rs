// SPDX-License-Identifier: EUPL-1.2

//! Opponents with strength levels (M4, D-07, D-40).
//!
//! A level is a [`Profile`]: how much analysis effort the opponent spends
//! and how much it deviates from the best option. Every strategic number
//! comes from [`skat_analysis::analyze`]; this module only picks one of the
//! analysed options. The weakest level plays by the shared rules of thumb
//! ([`BaselineOpponent`]) without search.
//!
//! Information boundary: [`Opponent::decide`] takes one seat's
//! [`PlayerView`] and nothing else (D-23). Its randomness comes from the
//! opponent's own seed and the view, never from the deal seed, so the same
//! view, seed and profile give the same decision.

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};
use skat_analysis::{Analysis, Budget, Cancel, Estimate, Status, analyze};
use skat_core::{Action, KontraLevel, Phase, PlayerView};

use crate::BaselineOpponent;

/// Strength level of an opponent, from weakest to strongest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Level {
    /// Rules of thumb without search.
    Easy,
    /// Analysed card play and Kontra/Re with a small budget.
    #[default]
    Medium,
    /// Analysed card play and Kontra/Re with the interactive budget.
    Strong,
}

impl Level {
    /// All levels from weakest to strongest.
    pub const ALL: [Level; 3] = [Level::Easy, Level::Medium, Level::Strong];

    /// The documented profile of this level (`docs/ai.md`).
    pub const fn profile(self) -> Profile {
        match self {
            Level::Easy => Profile {
                budget: None,
                temperature: 0.0,
                search_bidding: false,
                search_declaring: false,
            },
            Level::Medium => Profile {
                budget: Some(Budget {
                    samples: 6,
                    nodes_per_task: Budget::INTERACTIVE.nodes_per_task,
                }),
                temperature: 0.0,
                search_bidding: false,
                search_declaring: false,
            },
            Level::Strong => Profile {
                budget: Some(Budget::INTERACTIVE),
                temperature: 0.0,
                search_bidding: false,
                search_declaring: false,
            },
        }
    }
}

/// What makes a level strong or weak.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profile {
    /// Analysis budget; `None` plays the rules of thumb of
    /// [`BaselineOpponent`] without search.
    pub budget: Option<Budget>,
    /// Selection temperature in Seeger-Fabian points: an option is chosen
    /// with probability proportional to `exp(utility / temperature)`, so
    /// options a few points worse than the best are chosen now and then.
    /// 0 always chooses the best option.
    pub temperature: f64,
    /// Whether the analysis decides the bidding; otherwise the rules of
    /// thumb do.
    pub search_bidding: bool,
    /// Whether the analysis decides Hand or pickup, Drücken and the
    /// announcement; otherwise the rules of thumb do. Card play and
    /// Kontra/Re are always analysed when there is a budget.
    pub search_declaring: bool,
}

/// An automated player with a profile and its own seed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opponent {
    /// How the opponent decides.
    pub profile: Profile,
    /// Seed of the opponent's randomness (sampling and selection). It must
    /// not be derived from the deal seed.
    pub seed: u64,
}

/// A decision and how it was reached.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decision {
    /// The chosen legal action.
    pub action: Action,
    /// Status of the analysis; `None` if none ran (rules of thumb, or the
    /// view was rejected).
    pub status: Option<Status>,
    /// `true` if the rules of thumb decided because the analysis gave no
    /// estimate (cancelled before any distribution finished, or an
    /// inconsistent view). Always a legal action.
    pub fallback: bool,
}

impl Opponent {
    /// An opponent of `level` with `seed`.
    pub const fn new(level: Level, seed: u64) -> Opponent {
        Opponent {
            profile: level.profile(),
            seed,
        }
    }

    /// Chooses one of `view.legal_actions`; `None` if the list is empty,
    /// i.e. it is not this seat's turn. `cancel` stops the analysis early
    /// (M3 semantics): the choice then uses the distributions finished so
    /// far, or falls back to the rules of thumb if there are none.
    pub fn decide(&self, view: &PlayerView, cancel: Option<&Cancel>) -> Option<Decision> {
        let rules = BaselineOpponent.decide(view)?;
        let Some((budget, seed)) = self.analysis_input(view) else {
            return Some(Decision {
                action: rules,
                status: None,
                fallback: false,
            });
        };
        let Ok(analysis) = analyze(view, budget, seed, cancel) else {
            return Some(Decision {
                action: rules,
                status: None,
                fallback: true,
            });
        };
        let mut rng = ChaCha8Rng::seed_from_u64(mix(seed, 2));
        Some(
            match select(view, &analysis, rules, self.profile.temperature, &mut rng) {
                Some(action) => Decision {
                    action,
                    status: Some(analysis.status),
                    fallback: false,
                },
                None => Decision {
                    action: rules,
                    status: Some(analysis.status),
                    fallback: true,
                },
            },
        )
    }
}

impl Opponent {
    /// Budget and sampling seed the opponent passes to
    /// [`analyze`] together with `view`, or `None` if it decides by the rules
    /// of thumb. Both depend only on the profile, the opponent's seed and
    /// the view.
    pub fn analysis_input(&self, view: &PlayerView) -> Option<(Budget, u64)> {
        if view.legal_actions.is_empty() {
            return None;
        }
        let searched = match view.phase {
            Phase::Reizen => self.profile.search_bidding,
            Phase::SkatAufnahme | Phase::Druecken | Phase::Ansage => self.profile.search_declaring,
            _ => true,
        };
        let budget = self.profile.budget.filter(|_| searched)?;
        Some((budget, mix(decision_seed(self.seed, view), 1)))
    }
}

/// What an option is worth to the deciding seat: its own expected
/// Seeger-Fabian points; a defender also gains half of every point the
/// declarer loses, since both defenders gain relative to the declarer
/// (this makes Kontra worth saying).
pub fn utility(view: &PlayerView, estimate: &Estimate) -> f64 {
    let defender = view.declarer.is_some_and(|d| d != view.seat);
    if defender {
        estimate.expected_score - estimate.expected_game_points / 2.0
    } else {
        estimate.expected_score
    }
}

/// Bonus in points for the rules-of-thumb action. Double-dummy values only
/// resolve scoring classes, so many options tie exactly; the bonus breaks
/// such ties towards the sensible card instead of the first listed one.
const RULES_BONUS: f64 = 0.5;

/// Picks an option of `analysis`. Only options evaluated on the most
/// distributions compete (after Drücken screening, the kept ones), so a
/// lucky estimate from few distributions cannot win. `None` if no option
/// has an estimate.
fn select(
    view: &PlayerView,
    analysis: &Analysis,
    rules: Action,
    temperature: f64,
    rng: &mut ChaCha8Rng,
) -> Option<Action> {
    let most = analysis
        .options
        .iter()
        .filter_map(|o| o.estimate.map(|e| e.samples))
        .max()?;
    let candidates: Vec<(Action, f64)> = analysis
        .options
        .iter()
        .filter_map(|o| match o.estimate {
            Some(e) if e.samples == most => {
                let bonus = if o.action == rules { RULES_BONUS } else { 0.0 };
                Some((o.action, utility(view, &e) + bonus))
            }
            _ => None,
        })
        .collect();
    let best = candidates
        .iter()
        .map(|&(_, u)| u)
        .fold(f64::NEG_INFINITY, f64::max);
    if temperature <= 0.0 {
        return candidates
            .iter()
            .find(|&&(_, u)| u == best)
            .map(|&(a, _)| a);
    }
    let weights: Vec<f64> = candidates
        .iter()
        .map(|&(_, u)| ((u - best) / temperature).exp())
        .collect();
    let total: f64 = weights.iter().sum();
    // 53 random bits give a uniform number in [0, 1).
    let mut x = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64 * total;
    for (&(action, _), w) in candidates.iter().zip(&weights) {
        if x < *w {
            return Some(action);
        }
        x -= w;
    }
    candidates.last().map(|&(a, _)| a)
}

/// Seed of one decision: the opponent's seed mixed with public facts of the
/// view that change from decision to decision. It depends only on the view.
fn decision_seed(seed: u64, view: &PlayerView) -> u64 {
    let phase = match view.phase {
        Phase::Reizen => 0,
        Phase::SkatAufnahme => 1,
        Phase::Druecken => 2,
        Phase::Ansage => 3,
        Phase::Spiel => 4,
        Phase::Beendet => 5,
        Phase::Eingepasst => 6,
    };
    let kontra = match view.kontra {
        KontraLevel::None => 0,
        KontraLevel::Kontra => 1,
        KontraLevel::Re => 2,
    };
    let facts = [
        view.seat.index() as u64,
        phase,
        u64::from(view.hand.bits()),
        view.bidding.len() as u64,
        view.tricks.len() as u64,
        view.current_trick.len() as u64,
        kontra,
        view.legal_actions.len() as u64,
    ];
    facts.iter().fold(seed, |h, &f| mix(h, f))
}

/// Mixes `value` into `hash` (SplitMix64 finaliser).
fn mix(hash: u64, value: u64) -> u64 {
    let mut z = hash ^ value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
