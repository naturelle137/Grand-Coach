// SPDX-License-Identifier: EUPL-1.2

//! Live coach (M5): turns one [`Analysis`] of `skat-analysis` into a
//! structured [`Explanation`] (D-13): what to think about, which options
//! there are and how they compare, with the reasons the estimates give.
//!
//! The coach does not compute strategy of its own ("coach and AI do not
//! compute separately"): every fact is an analysis observation, every
//! inferred distribution an analysis inference, every comparison an
//! analysis estimate. The coach only ranks, groups and labels them by the
//! documented thresholds below (`docs/coach.md`, D-48). It returns no
//! texts; frontends translate the structure.
//!
//! Information boundary: [`explain`] takes the deciding seat's
//! [`PlayerView`] and the analysis of exactly that view (D-23).

use skat_analysis::{Budget, Cancel, analyze, utility};
use skat_core::{Action, Contract, GameType, KontraLevel, Phase, PlayerView, Seat};

pub use skat_analysis::sampling::Place;
pub use skat_analysis::{
    Analysis, Assumption, CardClass, Estimate, Inference, Observation, Status,
};
pub use skat_core::Follow;

/// Differences up to this many Seeger-Fabian points count as equal
/// (double-dummy values resolve scoring classes, so options often tie).
pub const TIE_POINTS: f64 = 1.0;

/// An option is weak, worth a warning, when it is expected to cost at
/// least this many Seeger-Fabian points against the best option, and more
/// than its noise margin.
pub const WEAK_POINTS: f64 = 10.0;

/// The noise margin of a comparison is this many combined standard errors.
pub const MARGIN_SE: f64 = 2.0;

/// Win chances or throw-in probabilities that differ by less than this are
/// not mentioned as a reason.
pub const CHANCE_STEP: f64 = 0.05;

/// The kind of decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Decision {
    /// Say a bid value or pass (asking seat).
    Bid,
    /// Hold or pass (answering seat).
    Answer,
    /// Play Hand or pick up the Skat.
    Pickup,
    /// Put two cards into the Skat (Drücken).
    Discard,
    /// Announce the game.
    Announce,
    /// Play a card; Kontra may be one of the options.
    Card,
    /// Kontra or not, in the Ouvert decision round.
    Kontra,
    /// Answer Kontra with Re or not.
    Re,
}

/// The deciding seat's role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Role {
    /// Before the declarer is known.
    Bidder,
    /// Alleinspieler.
    Declarer,
    /// Gegenspieler.
    Defender,
}

/// What the declarer has to achieve in this game (rule facts of the
/// contract); the defenders try to prevent it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Objective {
    /// No game announced yet: the analysis values each option by how the
    /// game that follows is likely to end.
    Undecided,
    /// At least this many Augen (61, or 90 with Schneider announced).
    Augen {
        /// Augen the declarer needs.
        target: u8,
    },
    /// Every trick (Schwarz announced).
    AllTricks,
    /// No trick at all (Null).
    NoTrick,
}

/// Something worth thinking about, with its kind of evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Consideration {
    /// A fact the seat knows exactly (card counting).
    Known(Observation),
    /// An estimate about hidden cards from the opponent model.
    Inferred(Inferred),
}

/// An inference with its qualitative reading.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Inferred {
    /// The analysis inference.
    pub inference: Inference,
    /// The expected count in words.
    pub amount: Amount,
}

/// Qualitative reading of an expected number of cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Amount {
    /// Expected fewer than 0.5.
    Hardly,
    /// Expected 0.5 to below 1.5.
    AboutOne,
    /// Expected 1.5 or more.
    Several,
}

impl Amount {
    /// Reading of an expected count.
    pub fn of(expected: f64) -> Amount {
        if expected < 0.5 {
            Amount::Hardly
        } else if expected < 1.5 {
            Amount::AboutOne
        } else {
            Amount::Several
        }
    }
}

/// How an option compares with the best one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Judgement {
    /// The highest value.
    Best,
    /// Equal within [`TIE_POINTS`] or within the noise margin.
    Equivalent,
    /// Worse beyond the noise margin, by less than [`WEAK_POINTS`].
    Worse,
    /// Worse beyond the noise margin and by at least [`WEAK_POINTS`].
    Weak,
    /// Evaluated on fewer distributions than the best options (Drücken
    /// screening); not compared.
    Screened,
    /// No distribution finished for it.
    NotEvaluated,
}

/// Why one option differs from another, read off the two estimates.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Reason {
    /// The deciding seat's party wins with this probability (this option)
    /// against `other` (the compared option).
    WinChance {
        /// This option.
        this: f64,
        /// The compared option.
        other: f64,
    },
    /// Expected value for the seat in Seeger-Fabian points (for a defender
    /// including half of the declarer's lost game points, see
    /// `skat_analysis::utility`).
    Points {
        /// This option.
        this: f64,
        /// The compared option.
        other: f64,
    },
    /// Expected game points of the declarer's list entry (with Kontra/Re);
    /// the defenders want them low.
    DeclarerPoints {
        /// This option.
        this: f64,
        /// The compared option.
        other: f64,
    },
    /// Probability that all pass and the deal is thrown in.
    ThrownIn {
        /// This option.
        this: f64,
        /// The compared option.
        other: f64,
    },
    /// Both options lead to the same value (within [`TIE_POINTS`]).
    SameValue,
    /// The difference is within the noise margin of the estimates.
    WithinUncertainty,
}

/// One legal option, compared.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComparedOption {
    /// The action.
    pub action: Action,
    /// Its standing.
    pub judgement: Judgement,
    /// The analysis estimate, unchanged.
    pub estimate: Option<Estimate>,
    /// Value used for ranking (`skat_analysis::utility`).
    pub value: Option<f64>,
    /// How much lower its value is than the best option's (0 for the best).
    pub gap: f64,
    /// Noise margin of that gap: [`MARGIN_SE`] combined standard errors.
    pub margin: f64,
    /// The option the reasons compare with: the best option, or for the
    /// best option the runner-up.
    pub compared_with: Option<Action>,
    /// Differences to `compared_with`.
    pub reasons: Vec<Reason>,
}

/// Overall reading of the comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Verdict {
    /// Only one legal option.
    Forced,
    /// The best option is better than every other beyond the noise margin.
    Clear,
    /// Several options are equivalent to the best.
    Close {
        /// Number of options equivalent to the best (without it).
        equivalent: u8,
    },
    /// No option has an estimate.
    Unknown,
}

/// What the estimates rest on.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Basis {
    /// Distributions evaluated completely.
    pub finished: u32,
    /// Distributions planned.
    pub planned: u32,
    /// `true` if every consistent distribution was enumerated.
    pub enumerated: bool,
    /// Effective sample size of the bidding-evidence weights.
    pub effective_samples: f64,
}

/// The coach's structured explanation of one decision.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Explanation {
    /// The deciding seat.
    pub seat: Seat,
    /// The kind of decision.
    pub decision: Decision,
    /// The seat's role.
    pub role: Role,
    /// What the declarer has to achieve.
    pub objective: Objective,
    /// Status of the analysis. Only [`Status::Complete`] backs a
    /// recommendation, an important-decision mark or a warning.
    pub status: Status,
    /// Facts and inferences worth thinking about, facts first.
    pub considerations: Vec<Consideration>,
    /// Every legal option, best first.
    pub options: Vec<ComparedOption>,
    /// Overall reading.
    pub verdict: Verdict,
    /// Whether this is an important decision (D-48): complete, and a
    /// natural option is weak.
    pub important: bool,
    /// What the estimates rest on.
    pub basis: Basis,
    /// Model assumptions behind the estimates.
    pub assumptions: Vec<Assumption>,
}

/// Why the coach warns before an action.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Warning {
    /// The chosen, weak option.
    pub chosen: ComparedOption,
    /// The best option.
    pub best: ComparedOption,
}

impl Explanation {
    /// `true` if the analysis finished; otherwise the explanation is
    /// provisional at best.
    pub fn complete(&self) -> bool {
        self.status == Status::Complete
    }

    /// The best option, if any was evaluated.
    pub fn best(&self) -> Option<&ComparedOption> {
        self.options
            .first()
            .filter(|o| o.judgement == Judgement::Best)
    }

    /// The comparison of `action`.
    pub fn option(&self, action: Action) -> Option<&ComparedOption> {
        self.options.iter().find(|o| o.action == action)
    }

    /// The warning before `action`: only for a complete analysis in which
    /// `action` is [`Judgement::Weak`].
    pub fn warning(&self, action: Action) -> Option<Warning> {
        if !self.complete() {
            return None;
        }
        let chosen = self.option(action)?;
        (chosen.judgement == Judgement::Weak).then(|| Warning {
            chosen: chosen.clone(),
            best: self
                .best()
                .expect("a weak option implies a best one")
                .clone(),
        })
    }
}

/// Explains the decision of `view.seat` from `analysis`, which must be the
/// analysis of `view`. `None` if it is not this seat's turn or the analysis
/// belongs to another decision.
pub fn explain(view: &PlayerView, analysis: &Analysis) -> Option<Explanation> {
    if view.legal_actions.is_empty()
        || analysis.status == Status::NoDecision
        || analysis.seat != view.seat
        || analysis.phase != view.phase
        || analysis
            .options
            .iter()
            .map(|o| o.action)
            .ne(view.legal_actions.iter().copied())
    {
        return None;
    }
    let decision = decision(view);
    let role = match view.declarer {
        None => Role::Bidder,
        Some(d) if d == view.seat => Role::Declarer,
        Some(_) => Role::Defender,
    };
    let options = compare(view, analysis, role);
    let verdict = verdict(&options);
    let important = analysis.status == Status::Complete
        && options
            .iter()
            .filter(|o| {
                matches!(
                    o.judgement,
                    Judgement::Best | Judgement::Equivalent | Judgement::Worse | Judgement::Weak
                )
            })
            .count()
            >= 2
        && options
            .iter()
            .any(|o| o.judgement == Judgement::Weak && natural(view, o.action));
    Some(Explanation {
        seat: view.seat,
        decision,
        role,
        objective: view.contract.map_or(Objective::Undecided, objective),
        status: analysis.status,
        considerations: considerations(view, analysis, decision, role),
        options,
        verdict,
        important,
        basis: Basis {
            finished: analysis.effort.finished,
            planned: analysis.effort.planned,
            enumerated: analysis.effort.enumerated,
            effective_samples: analysis.effort.effective_samples,
        },
        assumptions: analysis.assumptions.clone(),
    })
}

/// Analyses `view` with `skat-analysis` and explains it: the one call a
/// live coach needs. `None` if it is not this seat's turn or the view is
/// inconsistent.
pub fn advise(
    view: &PlayerView,
    budget: Budget,
    seed: u64,
    cancel: Option<&Cancel>,
) -> Option<Explanation> {
    let analysis = analyze(view, budget, seed, cancel).ok()?;
    explain(view, &analysis)
}

fn decision(view: &PlayerView) -> Decision {
    let has = |a: Action| view.legal_actions.contains(&a);
    match view.phase {
        Phase::Reizen if has(Action::Hold) => Decision::Answer,
        Phase::Reizen => Decision::Bid,
        Phase::SkatAufnahme => Decision::Pickup,
        Phase::Druecken => Decision::Discard,
        Phase::Ansage => Decision::Announce,
        _ if has(Action::Re) => Decision::Re,
        _ if has(Action::KeinKontra) => Decision::Kontra,
        _ => Decision::Card,
    }
}

fn objective(contract: Contract) -> Objective {
    match contract.game {
        GameType::Null => Objective::NoTrick,
        _ if contract.schwarz => Objective::AllTricks,
        _ if contract.schneider => Objective::Augen { target: 90 },
        _ => Objective::Augen { target: 61 },
    }
}

/// Options a player would ordinarily weigh: every option except jump bids
/// above the next bid value and Schneider or Schwarz announcements. Only
/// these make a decision important (D-48); warnings cover every option.
pub fn natural(view: &PlayerView, action: Action) -> bool {
    match action {
        Action::Bid(value) => {
            view.legal_actions
                .iter()
                .filter_map(|a| match a {
                    Action::Bid(v) => Some(*v),
                    _ => None,
                })
                .min()
                == Some(value)
        }
        Action::Announce(a) => !a.schneider && !a.schwarz,
        _ => true,
    }
}

fn considerations(
    view: &PlayerView,
    analysis: &Analysis,
    decision: Decision,
    role: Role,
) -> Vec<Consideration> {
    let playing = view.phase == Phase::Spiel;
    let mut out: Vec<Consideration> = analysis
        .observations
        .iter()
        .filter(|o| match o {
            Observation::Void { .. } | Observation::TrumpsOut { .. } => playing,
            Observation::AugenWon {
                declarer,
                defenders,
            } => playing && declarer + defenders > 0,
            Observation::AugenOut { .. } => playing,
            Observation::SkatKnown { .. } => true,
            Observation::BidShown { seat, .. } => *seat != view.seat,
            Observation::KontraEligible { .. } => {
                role == Role::Declarer
                    || decision == Decision::Kontra
                    || view.legal_actions.contains(&Action::Kontra)
            }
        })
        .copied()
        .map(Consideration::Known)
        .collect();
    out.extend(analysis.inferences.iter().map(|&inference| {
        Consideration::Inferred(Inferred {
            inference,
            amount: Amount::of(inference.expected),
        })
    }));
    out
}

/// Ranks the options: fully evaluated ones by value, then screened ones,
/// then unevaluated ones; ties keep the order of the legal actions.
fn compare(view: &PlayerView, analysis: &Analysis, role: Role) -> Vec<ComparedOption> {
    let most = analysis
        .options
        .iter()
        .filter_map(|o| o.estimate.map(|e| e.samples))
        .max()
        .unwrap_or(0);
    let tier = |e: &Option<Estimate>| match e {
        Some(e) if e.samples == most => 0,
        Some(_) => 1,
        None => 2,
    };
    let mut ranked: Vec<(usize, u8, f64)> = analysis
        .options
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let value = o.estimate.map_or(f64::NEG_INFINITY, |e| utility(view, &e));
            (i, tier(&o.estimate), value)
        })
        .collect();
    ranked.sort_by(|a, b| a.1.cmp(&b.1).then(b.2.total_cmp(&a.2)).then(a.0.cmp(&b.0)));
    let best = ranked
        .first()
        .filter(|r| r.1 == 0)
        .map(|r| analysis.options[r.0]);
    let runner_up = ranked
        .get(1)
        .filter(|r| r.1 == 0)
        .map(|r| analysis.options[r.0]);
    ranked
        .iter()
        .enumerate()
        .map(|(rank, &(i, tier, value))| {
            let option = analysis.options[i];
            let Some(estimate) = option.estimate else {
                return ComparedOption {
                    action: option.action,
                    judgement: Judgement::NotEvaluated,
                    estimate: None,
                    value: None,
                    gap: 0.0,
                    margin: 0.0,
                    compared_with: None,
                    reasons: Vec::new(),
                };
            };
            let reference = if rank == 0 { runner_up } else { best };
            let reference = reference.and_then(|r| r.estimate.map(|e| (r.action, e)));
            let best_value = best
                .and_then(|b| b.estimate)
                .map_or(value, |e| utility(view, &e));
            let gap = (best_value - value).max(0.0);
            let margin = best.and_then(|b| b.estimate).map_or(0.0, |b| {
                MARGIN_SE * (b.score_std_error.powi(2) + estimate.score_std_error.powi(2)).sqrt()
            });
            let judgement = if tier == 1 {
                Judgement::Screened
            } else if rank == 0 {
                Judgement::Best
            } else if gap <= TIE_POINTS || gap <= margin {
                Judgement::Equivalent
            } else if gap >= WEAK_POINTS {
                Judgement::Weak
            } else {
                Judgement::Worse
            };
            let reasons = match reference {
                Some((_, other)) if tier == 0 => reasons(view, role, &estimate, &other),
                _ => Vec::new(),
            };
            ComparedOption {
                action: option.action,
                judgement,
                estimate: Some(estimate),
                value: Some(value),
                gap,
                margin,
                compared_with: reference.filter(|_| tier == 0).map(|(a, _)| a),
                reasons,
            }
        })
        .collect()
}

/// Differences between two estimates worth naming.
fn reasons(view: &PlayerView, role: Role, this: &Estimate, other: &Estimate) -> Vec<Reason> {
    let mut out = Vec::new();
    if (this.win_chance - other.win_chance).abs() >= CHANCE_STEP {
        out.push(Reason::WinChance {
            this: this.win_chance,
            other: other.win_chance,
        });
    }
    if (this.thrown_in - other.thrown_in).abs() >= CHANCE_STEP {
        out.push(Reason::ThrownIn {
            this: this.thrown_in,
            other: other.thrown_in,
        });
    }
    if role == Role::Defender
        && (this.expected_game_points - other.expected_game_points).abs() > TIE_POINTS
    {
        out.push(Reason::DeclarerPoints {
            this: this.expected_game_points,
            other: other.expected_game_points,
        });
    }
    let (a, b) = (utility(view, this), utility(view, other));
    let margin = MARGIN_SE * (this.score_std_error.powi(2) + other.score_std_error.powi(2)).sqrt();
    if (a - b).abs() <= TIE_POINTS {
        out.push(Reason::SameValue);
    } else {
        out.push(Reason::Points { this: a, other: b });
        if (a - b).abs() <= margin {
            out.push(Reason::WithinUncertainty);
        }
    }
    out
}

fn verdict(options: &[ComparedOption]) -> Verdict {
    let compared: Vec<&ComparedOption> = options
        .iter()
        .filter(|o| !matches!(o.judgement, Judgement::Screened | Judgement::NotEvaluated))
        .collect();
    if options.len() == 1 {
        return Verdict::Forced;
    }
    if compared.is_empty() {
        return Verdict::Unknown;
    }
    let equivalent = compared
        .iter()
        .filter(|o| o.judgement == Judgement::Equivalent)
        .count();
    if equivalent == 0 {
        Verdict::Clear
    } else {
        Verdict::Close {
            equivalent: equivalent.min(usize::from(u8::MAX)) as u8,
        }
    }
}

/// Sampling seed of the coach's analysis of `view`: `base` (the coach's own
/// seed, never a deal seed) mixed with public facts of the view, so the
/// same view and base give the same advice.
pub fn decision_seed(base: u64, view: &PlayerView) -> u64 {
    let kontra = match view.kontra {
        KontraLevel::None => 0,
        KontraLevel::Kontra => 1,
        KontraLevel::Re => 2,
    };
    let facts = [
        view.seat.index() as u64,
        view.phase as u64,
        u64::from(view.hand.bits()),
        view.bidding.len() as u64,
        view.tricks.len() as u64,
        view.current_trick.len() as u64,
        kontra,
        view.legal_actions.len() as u64,
    ];
    facts.iter().fold(base, |h, &f| mix(h, f))
}

/// SplitMix64 finaliser.
fn mix(hash: u64, value: u64) -> u64 {
    let mut z = hash ^ value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
