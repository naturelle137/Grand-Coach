// SPDX-License-Identifier: EUPL-1.2

//! The live coach in the session (M5, D-48): settings, advice requests,
//! the pre-move warning and the lifecycle of coach analyses.
//!
//! The coach analyses only the human's [`PlayerView`] with
//! `skat-analysis` and explains it with `skat-coach`. Like opponent moves,
//! the analysis runs wherever the frontend wants ([`CoachJob::run`]); the
//! result is accepted only for the position it was computed for.

use serde::{Deserialize, Serialize};
use skat_ai::{Budget, Cancel};
use skat_coach::{Explanation, Status, Warning, advise, decision_seed};
use skat_core::{Action, PlayerView};

/// How much the coach says on its own (D-08).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoachLevel {
    /// No live explanations.
    Off,
    /// Explanations only when asked for ([`crate::Command::RequestAdvice`]).
    #[default]
    OnRequest,
    /// Explains important decisions on its own (D-48); asking works too.
    Important,
    /// Explains every decision on its own.
    EveryMove,
}

impl CoachLevel {
    /// All levels, from silent to talkative.
    pub const ALL: [CoachLevel; 4] = [
        CoachLevel::Off,
        CoachLevel::OnRequest,
        CoachLevel::Important,
        CoachLevel::EveryMove,
    ];
}

/// Coach settings. Changing them never changes the game or the opponents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoachSettings {
    /// Intervention level.
    pub level: CoachLevel,
    /// Warn before a weak action (D-09: off by default). Independent of the
    /// level; it also works with the level off.
    pub warnings: bool,
    /// Show numerical estimates; without them the frontend shows only the
    /// verbal reasoning.
    pub show_figures: bool,
}

impl Default for CoachSettings {
    fn default() -> CoachSettings {
        CoachSettings {
            level: CoachLevel::OnRequest,
            warnings: false,
            show_figures: true,
        }
    }
}

/// What the coach did with a human action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoachEvent {
    /// The action is held until the analysis can check it.
    Checking {
        /// The held action.
        action: Action,
    },
    /// The action is held because it is weak.
    Warned {
        /// The held action.
        action: Action,
    },
    /// The human withdrew the held action; the game is unchanged.
    Withdrawn {
        /// The withdrawn action.
        action: Action,
    },
}

/// Identifies one human decision: game number and actions applied so far.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct DecisionKey {
    pub(crate) number: u32,
    pub(crate) actions: usize,
}

/// A coach analysis to compute, possibly on another thread. It holds the
/// human's own view only (D-23).
#[derive(Clone, Debug)]
pub struct CoachJob {
    key: DecisionKey,
    view: Box<PlayerView>,
    budget: Budget,
    seed: u64,
}

impl CoachJob {
    /// Analyses and explains the decision. `cancel` stops the analysis; a
    /// cancelled result is never presented as advice.
    pub fn run(&self, cancel: Option<&Cancel>) -> CoachResult {
        let outcome = match advise(&self.view, self.budget, self.seed, cancel) {
            Some(e) if e.status == Status::Cancelled => Err(Unavailable::Cancelled),
            Some(e) => Ok(Box::new(e)),
            None => Err(Unavailable::Failed),
        };
        CoachResult {
            key: self.key,
            budget: self.budget,
            seed: self.seed,
            outcome,
        }
    }

    /// Sampling seed of this analysis (from the coach's own seed stream and
    /// the view, never from a deal seed).
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Work budget of this analysis.
    pub fn budget(&self) -> Budget {
        self.budget
    }
}

/// A finished coach analysis, to be passed to [`crate::Session::finish_coach`].
#[derive(Clone, Debug, PartialEq)]
pub struct CoachResult {
    key: DecisionKey,
    budget: Budget,
    seed: u64,
    outcome: Result<Box<Explanation>, Unavailable>,
}

impl CoachResult {
    /// `true` if the analysis was cancelled or failed.
    pub fn is_unavailable(&self) -> bool {
        self.outcome.is_err()
    }

    /// The explanation, if the analysis finished (completely or within
    /// its budget).
    pub fn explanation(&self) -> Option<&Explanation> {
        self.outcome.as_deref().ok()
    }
}

/// Why there is no advice for the current decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Unavailable {
    /// The analysis was cancelled before it finished; ask again.
    Cancelled,
    /// The analysis could not evaluate this view.
    Failed,
}

/// What the coach shows for the current decision.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Advice {
    /// Nothing to show: no decision of the human, the level is off, or it
    /// was neither requested nor due.
    None,
    /// Requested or due, still being computed.
    Computing,
    /// The finished explanation.
    Ready(Box<Explanation>),
    /// The analysis ran out of its work budget: estimates over the
    /// finished distributions only, no recommendation.
    Provisional(Box<Explanation>),
    /// No advice for this decision.
    Unavailable(Unavailable),
}

/// A human action held back by the warning.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Pending {
    /// Waiting for the analysis before the action is checked.
    Checking {
        /// The held action.
        action: Action,
    },
    /// The action is weak; proceed or reconsider.
    Warning {
        /// The held action.
        action: Action,
        /// Why it is weak.
        warning: Box<Warning>,
    },
}

impl Pending {
    /// The held action.
    pub fn action(&self) -> Action {
        match self {
            Pending::Checking { action } | Pending::Warning { action, .. } => *action,
        }
    }
}

/// The coach's part of the table view.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoachView {
    /// Current settings.
    pub settings: CoachSettings,
    /// The help action ([`crate::Command::RequestAdvice`]) is possible.
    pub help_available: bool,
    /// An analysis for the current decision is due and not finished.
    pub analysing: bool,
    /// What to show.
    pub advice: Advice,
    /// An action held back by the warning.
    pub pending: Option<Pending>,
}

/// Coach state of a session.
#[derive(Clone, Debug)]
pub(crate) struct CoachState {
    pub(crate) settings: CoachSettings,
    pub(crate) budget: Budget,
    /// The coach's own seed, from a separate stream of the series seed.
    pub(crate) seed: u64,
    /// The last result, for its decision only.
    pub(crate) cache: Option<CoachResult>,
    /// The decision the human asked about.
    pub(crate) requested: Option<DecisionKey>,
    /// An action held back.
    pub(crate) pending: Option<Pending>,
    /// Actions the human already confirmed after a warning at this
    /// decision; they are not warned about again.
    pub(crate) acknowledged: Vec<(DecisionKey, Action)>,
}

impl CoachState {
    pub(crate) fn new(seed: u64) -> CoachState {
        CoachState {
            settings: CoachSettings::default(),
            budget: Budget::INTERACTIVE,
            seed,
            cache: None,
            requested: None,
            pending: None,
            acknowledged: Vec::new(),
        }
    }

    /// Forgets everything tied to a decision (after any action).
    pub(crate) fn decision_changed(&mut self) {
        self.cache = None;
        self.requested = None;
        self.pending = None;
        self.acknowledged.clear();
    }

    /// The cached result of `key` with the current budget.
    pub(crate) fn result(&self, key: DecisionKey) -> Option<&CoachResult> {
        self.cache
            .as_ref()
            .filter(|r| r.key == key && r.budget == self.budget)
    }

    /// The finished explanation of `key`, if any.
    pub(crate) fn explanation(&self, key: DecisionKey) -> Option<&Explanation> {
        self.result(key).and_then(|r| r.outcome.as_deref().ok())
    }

    /// Whether an analysis of `key` is wanted.
    pub(crate) fn wanted(&self, key: DecisionKey) -> bool {
        self.settings.warnings
            || self.pending.is_some()
            || match self.settings.level {
                CoachLevel::Off => false,
                CoachLevel::OnRequest => self.requested == Some(key),
                CoachLevel::Important | CoachLevel::EveryMove => true,
            }
    }

    /// Whether the explanation of `key` is shown (when it exists).
    fn shown(&self, key: DecisionKey) -> bool {
        let requested = self.requested == Some(key);
        match self.settings.level {
            CoachLevel::Off => false,
            CoachLevel::OnRequest => requested,
            CoachLevel::Important => {
                requested || self.explanation(key).is_some_and(|e| e.important)
            }
            CoachLevel::EveryMove => true,
        }
    }

    pub(crate) fn job(&self, key: DecisionKey, view: PlayerView) -> CoachJob {
        let base = self.seed ^ u64::from(key.number).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        CoachJob {
            key,
            seed: decision_seed(base, &view),
            view: Box::new(view),
            budget: self.budget,
        }
    }

    /// Accepts `result` if it belongs to `key` and the current budget.
    pub(crate) fn accept(&mut self, key: DecisionKey, result: CoachResult) -> bool {
        if result.key != key || result.budget != self.budget {
            return false;
        }
        self.cache = Some(result);
        true
    }

    /// The coach part of the view for `key` (`None`: no human decision).
    pub(crate) fn view(&self, key: Option<DecisionKey>) -> CoachView {
        let Some(key) = key else {
            return CoachView {
                settings: self.settings,
                help_available: false,
                analysing: false,
                advice: Advice::None,
                pending: None,
            };
        };
        let result = self.result(key);
        let analysing = self.wanted(key) && result.is_none();
        // With level Important the explanation shows up only once the
        // finished analysis marks the decision important.
        let advice = if !self.shown(key) {
            Advice::None
        } else {
            match result.map(|r| &r.outcome) {
                None => Advice::Computing,
                Some(Ok(e)) if e.complete() => Advice::Ready(e.clone()),
                Some(Ok(e)) => Advice::Provisional(e.clone()),
                Some(Err(reason)) => Advice::Unavailable(*reason),
            }
        };
        CoachView {
            settings: self.settings,
            help_available: self.settings.level != CoachLevel::Off,
            analysing,
            advice,
            pending: self.pending.clone(),
        }
    }

    /// The warning before `action` at `key`, if the finished analysis
    /// judges it weak and the human has not confirmed it already.
    pub(crate) fn warning(&self, key: DecisionKey, action: Action) -> Option<Warning> {
        if self.acknowledged.contains(&(key, action)) {
            return None;
        }
        self.explanation(key)?.warning(action)
    }
}
