// SPDX-License-Identifier: EUPL-1.2

//! The live coach in the session (M5, D-48, D-49): settings, staged hints,
//! the pre-move warning, the note after each own move, the end-of-game
//! debriefing and the lifecycle of coach analyses.
//!
//! The coach analyses only the human's [`PlayerView`] with
//! `skat-analysis` and explains it with `skat-coach`. Like opponent moves,
//! the analysis runs wherever the frontend wants ([`CoachJob::run`]); the
//! result is accepted only for the position it was computed for. Before
//! the move the view carries only [`Hints`], never the ranking (D-49).

use serde::{Deserialize, Serialize};
use skat_ai::{Budget, Cancel};
use skat_coach::{
    Explanation, Hints, Moment, MoveReview, Note, Stage, Status, Warning, advise, decision_seed,
    hints, moments, review,
};
use skat_core::{Action, PlayerView};

/// How much the coach says on its own (D-08).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoachLevel {
    /// No live explanations, no notes, no debriefing.
    Off,
    /// Hints only when asked for ([`crate::Command::RequestAdvice`]).
    #[default]
    OnRequest,
    /// Shows the situation of important decisions on its own (D-48);
    /// asking works too.
    Important,
    /// Shows what to think about at every decision on its own.
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
    /// verbal reasoning and exact facts. Off by default (D-52).
    pub show_figures: bool,
}

impl Default for CoachSettings {
    fn default() -> CoachSettings {
        CoachSettings {
            level: CoachLevel::OnRequest,
            warnings: false,
            show_figures: false,
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
    review: bool,
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

    /// `true` if the decision was already played: the analysis only backs
    /// the note and the debriefing. A job for the current decision should
    /// go first.
    pub fn is_review(&self) -> bool {
        self.review
    }

    /// `true` if both jobs analyse the same decision.
    pub fn same_decision(&self, other: &CoachJob) -> bool {
        self.key == other.key
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
    /// The hints of the revealed stages.
    Ready(Box<Hints>),
    /// The analysis ran out of its work budget: estimates over the
    /// finished distributions only, no situation reading.
    Provisional(Box<Hints>),
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

/// The human's last own move and the coach's note on it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LastMove {
    /// The action played.
    pub action: Action,
    /// The bid value reached when it was played (to name "hold").
    pub bid: u16,
    /// Hand game, once decided (to name announcements).
    pub hand_game: Option<bool>,
    /// The note; `None` while the analysis is still running.
    pub note: Option<Note>,
}

/// The end-of-game debriefing (D-49).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Debriefing {
    /// This many own decisions are still being analysed.
    Computing {
        /// Decisions left.
        remaining: u32,
    },
    /// At most three particularly good and bad moments, in game order;
    /// may be empty.
    Ready(Vec<Moment>),
}

/// The coach's part of the table view.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoachView {
    /// Current settings.
    pub settings: CoachSettings,
    /// The help action ([`crate::Command::RequestAdvice`]) is possible.
    pub help_available: bool,
    /// The stage the next [`crate::Command::RequestAdvice`] reveals; `None`
    /// if everything is shown or there is no decision.
    pub next_stage: Option<Stage>,
    /// An analysis for the shown advice is due and not finished.
    pub analysing: bool,
    /// What to show.
    pub advice: Advice,
    /// An action held back by the warning.
    pub pending: Option<Pending>,
    /// The note on the human's last move in this game; once the game is
    /// over, not if that move was forced (it adds nothing to the
    /// debriefing).
    pub last_move: Option<LastMove>,
    /// The debriefing, once the game is over.
    pub debriefing: Option<Debriefing>,
}

/// An own decision played before its analysis finished.
#[derive(Clone, Debug)]
pub(crate) struct Queued {
    key: DecisionKey,
    view: Box<PlayerView>,
    action: Action,
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
    /// The decision the human asked about, and the stage asked for.
    pub(crate) requested: Option<(DecisionKey, Stage)>,
    /// An action held back.
    pub(crate) pending: Option<Pending>,
    /// Actions the human already confirmed after a warning at this
    /// decision; they are not warned about again.
    pub(crate) acknowledged: Vec<(DecisionKey, Action)>,
    /// Own decisions of this game still to analyse, oldest first.
    pub(crate) queue: Vec<Queued>,
    /// Reviews of this game's own decisions.
    pub(crate) reviews: Vec<(DecisionKey, MoveReview)>,
    /// The last own move of this game.
    pub(crate) last_move: Option<(DecisionKey, LastMove)>,
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
            queue: Vec::new(),
            reviews: Vec::new(),
            last_move: None,
        }
    }

    /// Forgets the notes and reviews of the finished game.
    pub(crate) fn game_changed(&mut self) {
        self.decision_changed();
        self.queue.clear();
        self.reviews.clear();
        self.last_move = None;
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

    /// Whether the current decision needs an analysis: for the warning,
    /// and with the coach on always, for the note after the move.
    pub(crate) fn wanted(&self, key: DecisionKey) -> bool {
        self.visible(key) || self.settings.level != CoachLevel::Off
    }

    /// Whether the player waits for the analysis: asked for, due by the
    /// level, or needed by the warning.
    fn visible(&self, key: DecisionKey) -> bool {
        self.settings.warnings
            || self.pending.is_some()
            || match self.settings.level {
                CoachLevel::Off => false,
                CoachLevel::OnRequest => self.requested.is_some_and(|(k, _)| k == key),
                CoachLevel::Important | CoachLevel::EveryMove => true,
            }
    }

    /// The highest stage the player may see: figures only if shown.
    fn top_stage(&self) -> Stage {
        if self.settings.show_figures {
            Stage::Figures
        } else {
            Stage::Situation
        }
    }

    /// The stage shown for `key`, if any (D-49). With level Important the
    /// situation shows up on its own once the finished analysis marks the
    /// decision important.
    fn shown(&self, key: DecisionKey) -> Option<Stage> {
        let requested = self.requested.filter(|(k, _)| *k == key).map(|(_, s)| s);
        let automatic = match self.settings.level {
            CoachLevel::Off => return None,
            CoachLevel::OnRequest => None,
            CoachLevel::Important => self
                .explanation(key)
                .is_some_and(|e| e.important)
                .then_some(Stage::Situation),
            CoachLevel::EveryMove => Some(Stage::Considerations),
        };
        requested.max(automatic).map(|s| s.min(self.top_stage()))
    }

    /// The stage a request at `key` reveals next.
    pub(crate) fn next_stage(&self, key: DecisionKey) -> Option<Stage> {
        match self.shown(key) {
            None => Some(Stage::Considerations),
            Some(s) => s.next().filter(|n| *n <= self.top_stage()),
        }
    }

    /// Records the human's own action at `key` from `view` (before it is
    /// applied): reviewed now if the analysis is there, queued otherwise.
    pub(crate) fn record(&mut self, key: DecisionKey, view: &PlayerView, action: Action) {
        if self.settings.level == CoachLevel::Off {
            return;
        }
        let mut last = LastMove {
            action,
            bid: view.bid,
            hand_game: view.hand_game,
            note: None,
        };
        match self.result(key).map(|r| r.outcome.clone()) {
            Some(Ok(e)) => last.note = self.store_review(key, &e, action),
            Some(Err(Unavailable::Failed)) => last.note = Some(Note::Uncertain),
            // Cancelled or still running: analysed again after the move.
            Some(Err(Unavailable::Cancelled)) | None => self.queue.push(Queued {
                key,
                view: Box::new(view.clone()),
                action,
            }),
        }
        self.last_move = Some((key, last));
    }

    /// Stores the review of `action`; its note.
    fn store_review(&mut self, key: DecisionKey, e: &Explanation, action: Action) -> Option<Note> {
        let r = review(e, action)?;
        let note = r.note();
        self.reviews.push((key, r));
        Some(note)
    }

    /// The oldest queued review job.
    pub(crate) fn review_job(&self) -> Option<CoachJob> {
        let q = self.queue.first()?;
        let mut job = self.job(q.key, (*q.view).clone());
        job.review = true;
        Some(job)
    }

    /// Accepts a result for a queued decision; `false` if it belongs to
    /// none of them. A cancelled one stays queued.
    pub(crate) fn accept_review(&mut self, result: &CoachResult) -> bool {
        if result.budget != self.budget {
            return false;
        }
        let Some(i) = self.queue.iter().position(|q| q.key == result.key) else {
            return false;
        };
        let note = match &result.outcome {
            Err(Unavailable::Cancelled) => return true,
            Err(Unavailable::Failed) => {
                self.queue.remove(i);
                Note::Uncertain
            }
            Ok(e) => {
                let q = self.queue.remove(i);
                self.store_review(q.key, e, q.action)
                    .unwrap_or(Note::Uncertain)
            }
        };
        if let Some((k, last)) = &mut self.last_move
            && *k == result.key
        {
            last.note = Some(note);
        }
        true
    }

    /// The debriefing of a finished game.
    fn debriefing(&self) -> Debriefing {
        if !self.queue.is_empty() {
            return Debriefing::Computing {
                remaining: self.queue.len() as u32,
            };
        }
        let mut reviews = self.reviews.clone();
        reviews.sort_by_key(|(k, _)| k.actions);
        let reviews: Vec<MoveReview> = reviews.into_iter().map(|(_, r)| r).collect();
        Debriefing::Ready(moments(&reviews))
    }

    pub(crate) fn job(&self, key: DecisionKey, view: PlayerView) -> CoachJob {
        let base = self.seed ^ u64::from(key.number).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        CoachJob {
            key,
            seed: decision_seed(base, &view),
            view: Box::new(view),
            budget: self.budget,
            review: false,
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

    /// The coach part of the view: `key` is the human's current decision
    /// (`None`: none); `game_over` adds the debriefing.
    pub(crate) fn view(&self, key: Option<DecisionKey>, game_over: bool) -> CoachView {
        let on = self.settings.level != CoachLevel::Off;
        let mut view = CoachView {
            settings: self.settings,
            help_available: false,
            next_stage: None,
            analysing: false,
            advice: Advice::None,
            pending: None,
            last_move: self
                .last_move
                .as_ref()
                .filter(|(_, m)| on && !(game_over && m.note == Some(Note::Forced)))
                .map(|(_, m)| m.clone()),
            debriefing: (on && game_over).then(|| self.debriefing()),
        };
        let Some(key) = key else {
            return view;
        };
        let result = self.result(key);
        view.analysing = self.visible(key) && result.is_none();
        view.advice = match self.shown(key) {
            None => Advice::None,
            Some(stage) => match result.map(|r| &r.outcome) {
                None => Advice::Computing,
                Some(Ok(e)) if e.complete() => Advice::Ready(Box::new(hints(e, stage))),
                Some(Ok(e)) => Advice::Provisional(Box::new(hints(e, stage))),
                Some(Err(reason)) => Advice::Unavailable(*reason),
            },
        };
        view.help_available = on;
        view.next_stage = self.next_stage(key).filter(|_| on);
        view.pending = self.pending.clone();
        view
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
