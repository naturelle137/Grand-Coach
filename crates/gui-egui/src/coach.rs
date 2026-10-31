// SPDX-License-Identifier: EUPL-1.2

//! Translated texts of the live coach (M5, D-49): turns the staged hints,
//! the note on the last move, the debriefing and the warning of
//! `skat-coach` (via `skat-app`) into lines. Before the move nothing ranks
//! or labels an option. Estimated figures (chances, expected points,
//! uncertainty, expected card counts) appear only with "show figures";
//! exact facts such as counted cards, Augen and bid values always do. A
//! debriefing moment shows a short summary without any estimate; the
//! comparison, figures, facts and assumptions are its details (D-52).

use skat_app::coach_model::{
    Amount, Assumption, Basis, CardClass, ComparedOption, Consideration, Decision, Estimate,
    Follow, Hints, Judgement, Moment, MomentKind, MoveReview, Note, Objective, Observation, Place,
    Reason, Role, Situation, Stage, Warning,
};
use skat_app::{Action, Contract, Debriefing, LastMove, PlayerId, Seat, TableView, Unavailable};

use crate::i18n::I18n;
use crate::text;

/// How a line is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// Section heading.
    Heading,
    /// Emphasised line.
    Strong,
    /// Ordinary line.
    Normal,
    /// Secondary line (uncertainty, assumptions).
    Weak,
}

/// One line of coach text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// The text.
    pub text: String,
    /// How to show it.
    pub style: Style,
}

fn line(text: String, style: Style) -> Line {
    Line { text, style }
}

/// Options listed in full; the rest only count.
const SHOWN_OPTIONS: usize = 12;

/// Player sitting on `seat` in the current deal.
fn seat_player(view: &TableView, seat: Seat) -> PlayerId {
    view.players
        .iter()
        .find(|p| p.seat == seat)
        .map_or(PlayerId::HUMAN, |p| p.player)
}

fn percent(x: f64) -> String {
    format!("{:.0} %", x * 100.0)
}

fn points(x: f64) -> String {
    format!("{x:+.1}")
}

/// A legal action of the current decision in words.
pub fn action(i18n: &I18n, view: &TableView, action: Action) -> String {
    action_at(i18n, view.bid, view.hand_game, action)
}

/// An action in words, at bid value `bid` and with `hand_game` as when it
/// was decided.
fn action_at(i18n: &I18n, bid: u16, hand_game: Option<bool>, action: Action) -> String {
    match action {
        Action::Bid(value) => i18n.tf("coach-action-bid", &[("value", value.into())]),
        Action::Hold => i18n.tf("coach-action-hold", &[("value", bid.into())]),
        Action::Pass => i18n.t("button-pass"),
        Action::Hand => i18n.t("button-hand"),
        Action::TakeSkat => i18n.t("button-take-skat"),
        Action::Discard(cards) => {
            let names: Vec<String> = cards.iter().map(|c| text::card(i18n, c)).collect();
            i18n.tf(
                "coach-action-discard",
                &[("cards", names.join(", ").into())],
            )
        }
        Action::Announce(a) => {
            let contract = Contract::new(a, hand_game == Some(true));
            i18n.tf(
                "coach-action-announce",
                &[("game", text::contract(i18n, contract).into())],
            )
        }
        Action::Play(card) => text::card(i18n, card),
        Action::Kontra => i18n.t("button-kontra"),
        Action::KeinKontra => i18n.t("button-kein-kontra"),
        Action::Re => i18n.t("button-re"),
        Action::KeinRe => i18n.t("button-kein-re"),
    }
}

fn decision(i18n: &I18n, decision: Decision) -> String {
    i18n.t(match decision {
        Decision::Bid => "coach-decision-bid",
        Decision::Answer => "coach-decision-answer",
        Decision::Pickup => "coach-decision-pickup",
        Decision::Discard => "coach-decision-discard",
        Decision::Announce => "coach-decision-announce",
        Decision::Card => "coach-decision-card",
        Decision::Kontra => "coach-decision-kontra",
        Decision::Re => "coach-decision-re",
    })
}

fn role(i18n: &I18n, role: Role) -> String {
    i18n.t(match role {
        Role::Bidder => "coach-role-bidder",
        Role::Declarer => "coach-role-declarer",
        Role::Defender => "coach-role-defender",
    })
}

fn objective(i18n: &I18n, role: Role, objective: Objective) -> String {
    let defender = role == Role::Defender;
    match objective {
        Objective::Undecided => i18n.t("coach-objective-undecided"),
        Objective::Augen { target } => i18n.tf(
            if defender {
                "coach-objective-augen-defender"
            } else {
                "coach-objective-augen"
            },
            &[("target", target.into())],
        ),
        Objective::AllTricks => i18n.t(if defender {
            "coach-objective-all-tricks-defender"
        } else {
            "coach-objective-all-tricks"
        }),
        Objective::NoTrick => i18n.t(if defender {
            "coach-objective-no-trick-defender"
        } else {
            "coach-objective-no-trick"
        }),
    }
}

fn holder(i18n: &I18n, view: &TableView, place: Place) -> String {
    match place {
        Place::Seat(seat) => text::player(i18n, seat_player(view, seat)),
        Place::Skat => i18n.t("coach-holder-skat"),
    }
}

fn class(i18n: &I18n, class: CardClass) -> String {
    i18n.t(match class {
        CardClass::Trumps => "coach-class-trumps",
        CardClass::Jacks => "coach-class-jacks",
    })
}

fn fact(i18n: &I18n, view: &TableView, o: Observation) -> String {
    let name = |seat: Seat| text::player(i18n, seat_player(view, seat));
    match o {
        Observation::Void { seat, follow } => {
            let what = match follow {
                Follow::Trumpf => i18n.t("coach-class-trumps"),
                Follow::Farbe(suit) => text::suit(i18n, suit),
            };
            i18n.tf(
                "coach-fact-void",
                &[("player", name(seat).into()), ("class", what.into())],
            )
        }
        Observation::TrumpsOut { count } => {
            i18n.tf("coach-fact-trumps-out", &[("count", count.into())])
        }
        Observation::AugenWon {
            declarer,
            defenders,
        } => i18n.tf(
            "coach-fact-augen-won",
            &[
                ("declarer", declarer.into()),
                ("defenders", defenders.into()),
            ],
        ),
        Observation::AugenOut { augen } => {
            i18n.tf("coach-fact-augen-out", &[("augen", augen.into())])
        }
        Observation::SkatKnown { cards } => {
            let names: Vec<String> = cards.iter().map(|c| text::card(i18n, c)).collect();
            i18n.tf(
                "coach-fact-skat-known",
                &[("cards", names.join(", ").into())],
            )
        }
        Observation::BidShown { seat, value } => i18n.tf(
            "coach-fact-bid-shown",
            &[("player", name(seat).into()), ("value", value.into())],
        ),
        Observation::KontraEligible { seat } => i18n.tf(
            "coach-fact-kontra-eligible",
            &[("player", name(seat).into())],
        ),
    }
}

fn inferred(
    i18n: &I18n,
    view: &TableView,
    i: &skat_app::coach_model::Inferred,
    figures: bool,
) -> String {
    let who = holder(i18n, view, i.inference.holder);
    let what = class(i18n, i.inference.class);
    if i.inference.certain {
        return i18n.tf(
            "coach-inferred-certain",
            &[
                ("holder", who.into()),
                ("count", i.inference.max.into()),
                ("class", what.into()),
            ],
        );
    }
    let id = match i.amount {
        Amount::Hardly => "coach-inferred-hardly",
        Amount::AboutOne => "coach-inferred-about-one",
        Amount::Several => "coach-inferred-several",
    };
    let mut out = i18n.tf(id, &[("holder", who.into()), ("class", what.into())]);
    if figures {
        out.push(' ');
        out.push_str(&i18n.tf(
            "coach-inferred-figures",
            &[
                ("expected", format!("{:.1}", i.inference.expected).into()),
                ("min", i.inference.min.into()),
                ("max", i.inference.max.into()),
            ],
        ));
    }
    out
}

fn judgement(i18n: &I18n, j: Judgement) -> String {
    i18n.t(match j {
        Judgement::Best => "coach-judgement-best",
        Judgement::Equivalent => "coach-judgement-equivalent",
        Judgement::Worse => "coach-judgement-worse",
        Judgement::Weak => "coach-judgement-weak",
        Judgement::Screened => "coach-judgement-screened",
        Judgement::NotEvaluated => "coach-judgement-not-evaluated",
    })
}

fn reason(i18n: &I18n, other: &str, r: Reason, figures: bool) -> String {
    let compare = |id_more: &str, id_less: &str, this: f64, that: f64, fmt: fn(f64) -> String| {
        let id = if this > that { id_more } else { id_less };
        let mut out = i18n.tf(id, &[("other", other.into())]);
        if figures {
            out.push_str(&format!(" ({} / {})", fmt(this), fmt(that)));
        }
        out
    };
    match r {
        Reason::WinChance { this, other: that } => compare(
            "coach-reason-win-more",
            "coach-reason-win-less",
            this,
            that,
            percent,
        ),
        Reason::Points { this, other: that } => compare(
            "coach-reason-points-more",
            "coach-reason-points-less",
            this,
            that,
            points,
        ),
        Reason::DeclarerPoints { this, other: that } => compare(
            "coach-reason-declarer-more",
            "coach-reason-declarer-less",
            this,
            that,
            points,
        ),
        Reason::ThrownIn { this, other: that } => compare(
            "coach-reason-thrown-in-more",
            "coach-reason-thrown-in-less",
            this,
            that,
            percent,
        ),
        Reason::SameValue => i18n.tf("coach-reason-same", &[("other", other.into())]),
        Reason::WithinUncertainty => i18n.t("coach-reason-uncertain"),
    }
}

/// Whose winning chance an estimate gives.
fn party(i18n: &I18n, role: Role) -> String {
    i18n.t(match role {
        Role::Bidder => "coach-party-own",
        Role::Declarer => "coach-party-declarer",
        Role::Defender => "coach-party-defenders",
    })
}

/// The figures of one estimate.
fn metrics(i18n: &I18n, role: Role, e: &Estimate) -> String {
    let party = party(i18n, role);
    let mut out = i18n.tf(
        "coach-metric-win",
        &[
            ("party", party.into()),
            ("chance", percent(e.win_chance).into()),
        ],
    );
    out.push_str(" · ");
    out.push_str(&i18n.tf(
        "coach-metric-score",
        &[
            ("score", points(e.expected_score).into()),
            ("se", format!("{:.1}", e.score_std_error).into()),
        ],
    ));
    if role != Role::Declarer {
        out.push_str(" · ");
        out.push_str(&i18n.tf(
            "coach-metric-game-points",
            &[("points", points(e.expected_game_points).into())],
        ));
    }
    out
}

/// What the seat knows and can infer, in two sections.
fn considerations(
    i18n: &I18n,
    view: &TableView,
    considerations: &[Consideration],
    figures: bool,
) -> Vec<Line> {
    let mut out = Vec::new();
    let facts: Vec<Line> = considerations
        .iter()
        .filter_map(|c| match c {
            Consideration::Known(o) => {
                Some(line(format!("• {}", fact(i18n, view, *o)), Style::Normal))
            }
            Consideration::Inferred(_) => None,
        })
        .collect();
    if !facts.is_empty() {
        out.push(line(i18n.t("coach-section-facts"), Style::Heading));
        out.extend(facts);
    }
    let guesses: Vec<Line> = considerations
        .iter()
        .filter_map(|c| match c {
            Consideration::Inferred(i) => Some(line(
                format!("• {}", inferred(i18n, view, i, figures)),
                Style::Normal,
            )),
            Consideration::Known(_) => None,
        })
        .collect();
    if !guesses.is_empty() {
        out.push(line(i18n.t("coach-section-inferred"), Style::Heading));
        out.extend(guesses);
    }
    out
}

/// The basis and the assumptions of the estimates.
fn basis(i18n: &I18n, basis: &Basis, assumptions: &[Assumption], figures: bool) -> Vec<Line> {
    let mut out = vec![line(i18n.t("coach-section-basis"), Style::Heading)];
    out.push(line(
        if basis.enumerated {
            i18n.t("coach-basis-exact")
        } else if figures {
            i18n.tf(
                "coach-basis-sampled-figures",
                &[("finished", basis.finished.into())],
            )
        } else {
            i18n.t("coach-basis-sampled")
        },
        Style::Weak,
    ));
    for a in assumptions {
        out.push(line(
            format!(
                "• {}",
                i18n.t(match a {
                    Assumption::UniformHiddenCards => "coach-assumption-uniform",
                    Assumption::BiddingEvidence => "coach-assumption-bidding",
                    Assumption::DoubleDummyPlay => "coach-assumption-double-dummy",
                    Assumption::ContinuationPolicy => "coach-assumption-policy",
                    Assumption::BidAsLimit => "coach-assumption-bid-limit",
                })
            ),
            Style::Weak,
        ));
    }
    out
}

/// The label of the help button that reveals `stage`.
pub fn stage_button(i18n: &I18n, stage: Stage) -> String {
    i18n.t(match stage {
        Stage::Considerations => "button-coach-considerations",
        Stage::Situation => "button-coach-situation",
        Stage::Figures => "button-coach-figures",
    })
}

/// The hints before the move, as lines: the revealed stages only, the
/// options in neutral order and never judged (D-49).
pub fn hints(i18n: &I18n, view: &TableView, h: &Hints, figures: bool) -> Vec<Line> {
    let complete = h.status == skat_app::coach_model::Status::Complete;
    let mut out = vec![
        line(
            format!("{} · {}", decision(i18n, h.decision), role(i18n, h.role)),
            Style::Heading,
        ),
        line(objective(i18n, h.role, h.objective), Style::Normal),
    ];
    if !complete {
        out.push(line(i18n.t("coach-provisional"), Style::Strong));
    }
    out.extend(considerations(i18n, view, &h.considerations, figures));
    if let Some(s) = &h.situation {
        out.push(line(i18n.t("coach-section-situation"), Style::Heading));
        out.push(line(
            match s.situation {
                Situation::Forced => i18n.t("coach-situation-forced"),
                Situation::OneStandsOut => i18n.t("coach-situation-one"),
                Situation::SeveralEqual { count } => {
                    i18n.tf("coach-situation-several", &[("count", count.into())])
                }
                Situation::Unclear => i18n.t("coach-situation-unclear"),
            },
            Style::Strong,
        ));
        if s.important && complete {
            out.push(line(i18n.t("coach-important"), Style::Strong));
        }
        match (&h.figures, figures) {
            (Some(all), true) => {
                out.push(line(i18n.t("coach-section-figures"), Style::Heading));
                for f in all.iter().take(SHOWN_OPTIONS) {
                    out.push(line(action(i18n, view, f.action), Style::Normal));
                    out.push(line(
                        match &f.estimate {
                            Some(e) => metrics(i18n, h.role, e),
                            None => i18n.t("coach-judgement-not-evaluated"),
                        },
                        Style::Weak,
                    ));
                }
            }
            _ => {
                out.push(line(i18n.t("coach-section-options"), Style::Heading));
                for &a in s.options.iter().take(SHOWN_OPTIONS) {
                    out.push(line(format!("• {}", action(i18n, view, a)), Style::Normal));
                }
            }
        }
        let rest = s.options.len().saturating_sub(SHOWN_OPTIONS);
        if rest > 0 {
            out.push(line(
                i18n.tf("coach-more-options", &[("count", rest.into())]),
                Style::Weak,
            ));
        }
    }
    out.extend(basis(i18n, &h.basis, &h.assumptions, figures));
    out
}

/// The note on the human's last move.
pub fn note(i18n: &I18n, last: &LastMove) -> Line {
    let text = i18n.t(match last.note {
        None => "coach-note-computing",
        Some(Note::Good) => "coach-note-good",
        Some(Note::AboutEqual) => "coach-note-about-equal",
        Some(Note::SomewhatWeaker) => "coach-note-somewhat-weaker",
        Some(Note::ClearlyWeaker) => "coach-note-clearly-weaker",
        Some(Note::Forced) => "coach-note-forced",
        Some(Note::Uncertain) => "coach-note-uncertain",
    });
    let played = action_at(i18n, last.bid, last.hand_game, last.action);
    line(
        i18n.tf(
            "coach-note",
            &[("action", played.into()), ("note", text.into())],
        ),
        match last.note {
            Some(Note::Good) => Style::Strong,
            _ => Style::Normal,
        },
    )
}

/// What a reason compares, with the party it is counted for.
fn metric_label(i18n: &I18n, role: Role, r: Reason) -> Option<String> {
    Some(match r {
        Reason::WinChance { .. } => i18n.tf(
            "coach-metric-label-win",
            &[("party", party(i18n, role).into())],
        ),
        Reason::Points { .. } => i18n.t(if role == Role::Defender {
            "coach-metric-label-points-defender"
        } else {
            "coach-metric-label-points"
        }),
        Reason::DeclarerPoints { .. } => i18n.t("coach-metric-label-declarer"),
        Reason::ThrownIn { .. } => i18n.t("coach-metric-label-thrown-in"),
        Reason::SameValue | Reason::WithinUncertainty => return None,
    })
}

/// The model comparison of `o` with the option it was compared with, each
/// reason under the name of its metric.
fn comparison(
    i18n: &I18n,
    role: Role,
    o: &ComparedOption,
    name: impl Fn(Action) -> String,
    figures: bool,
) -> Vec<Line> {
    let Some(other) = o.compared_with else {
        return Vec::new();
    };
    let other = name(other);
    let mut out = vec![line(
        i18n.tf(
            "coach-details-comparison",
            &[
                ("action", name(o.action).into()),
                ("other", other.clone().into()),
            ],
        ),
        Style::Strong,
    )];
    out.extend(o.reasons.iter().map(|&r| {
        let text = reason(i18n, &other, r, figures);
        let text = match metric_label(i18n, role, r) {
            Some(label) => format!("• {label}: {text}"),
            None => format!("• {text}"),
        };
        line(text, Style::Normal)
    }));
    out
}

/// The model's judgement of the chosen option, qualified as the model's.
fn chosen(i18n: &I18n, r: &MoveReview, name: &dyn Fn(Action) -> String) -> String {
    let action = name(r.chosen.action);
    let best = r.best.as_ref().map(|b| name(b.action));
    let id = match (r.chosen.judgement, &best) {
        (Judgement::Best, _) => "coach-moment-chosen-best",
        (Judgement::Equivalent, Some(_)) => "coach-moment-chosen-equivalent",
        (Judgement::Worse, Some(_)) => "coach-moment-chosen-worse",
        (Judgement::Weak, Some(_)) => "coach-moment-chosen-weak",
        _ => "coach-moment-chosen-unrated",
    };
    i18n.tf(
        id,
        &[
            ("action", action.into()),
            ("best", best.unwrap_or_default().into()),
        ],
    )
}

/// One moment of the debriefing: a short summary and its analysis details.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentText {
    /// What was decided and how the model rates it; never an estimate.
    pub summary: Vec<Line>,
    /// Comparison by metric, figures (if shown), what was known and the
    /// model assumptions; collapsed by default.
    pub details: Vec<Line>,
}

/// One moment of the debriefing. The summary names the decision, the
/// choice and the model's judgement and says honestly that no strategic
/// explanation exists yet: the score differences of the reasons are no
/// cause and stay in the details (D-52).
fn moment(i18n: &I18n, view: &TableView, m: &Moment, figures: bool) -> MomentText {
    let r = &m.review;
    let name = |a: Action| action_at(i18n, r.bid, r.hand_game, a);
    let mut what = decision(i18n, r.decision);
    if let Some(trick) = r.trick {
        what = format!(
            "{} · {what}",
            i18n.tf("coach-moment-trick", &[("trick", trick.into())])
        );
    }
    let mut summary = vec![
        line(what, Style::Heading),
        line(chosen(i18n, r, &name), Style::Strong),
    ];
    let trap = r.trap.as_ref().filter(|_| m.kind == MomentKind::Good);
    if let Some(trap) = trap {
        summary.push(line(
            i18n.tf("coach-moment-trap", &[("trap", name(trap.action).into())]),
            Style::Normal,
        ));
    }
    summary.push(line(i18n.t("coach-moment-no-reason"), Style::Normal));

    let mut details = Vec::new();
    let compared = match m.kind {
        MomentKind::Bad => Some(&r.chosen),
        MomentKind::Good => trap,
    };
    if let Some(o) = compared {
        details.extend(comparison(i18n, r.role, o, name, figures));
    }
    // The model's rating of each option named, with its figures if shown.
    let mut shown = Vec::new();
    for o in [Some(&r.chosen), r.best.as_ref(), trap]
        .into_iter()
        .flatten()
    {
        if shown.contains(&o.action) {
            continue;
        }
        shown.push(o.action);
        details.push(line(
            i18n.tf(
                "coach-details-rating",
                &[
                    ("action", name(o.action).into()),
                    ("judgement", judgement(i18n, o.judgement).into()),
                ],
            ),
            Style::Normal,
        ));
        if let Some(e) = o.estimate.filter(|_| figures) {
            details.push(line(metrics(i18n, r.role, &e), Style::Weak));
        }
    }
    if let Some(trap) = trap.filter(|_| figures) {
        details.push(line(
            i18n.tf(
                "coach-moment-trap-loss",
                &[("points", format!("{:.1}", trap.gap).into())],
            ),
            Style::Weak,
        ));
    }
    // One level below the moment: sections as emphasised lines.
    let below = |l: Line| match l.style {
        Style::Heading => line(l.text, Style::Strong),
        _ => l,
    };
    details.extend(
        considerations(i18n, view, &r.considerations, figures)
            .into_iter()
            .map(below),
    );
    details.extend(
        basis(i18n, &r.basis, &r.assumptions, figures)
            .into_iter()
            .map(below),
    );
    MomentText { summary, details }
}

/// The end-of-game debriefing (D-49): heading and status lines, then at
/// most three moments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebriefingText {
    /// Heading and status.
    pub lines: Vec<Line>,
    /// The moments in game order.
    pub moments: Vec<MomentText>,
}

/// The end-of-game debriefing (D-49).
pub fn debriefing(i18n: &I18n, view: &TableView, d: &Debriefing, figures: bool) -> DebriefingText {
    let mut lines = vec![line(i18n.t("coach-debriefing-title"), Style::Heading)];
    let mut moments = Vec::new();
    match d {
        Debriefing::Computing { remaining } => lines.push(line(
            i18n.tf(
                "coach-debriefing-computing",
                &[("count", (*remaining).into())],
            ),
            Style::Weak,
        )),
        Debriefing::Ready(all) if all.is_empty() => {
            lines.push(line(i18n.t("coach-debriefing-none"), Style::Normal));
        }
        Debriefing::Ready(all) => {
            moments = all.iter().map(|m| moment(i18n, view, m, figures)).collect();
        }
    }
    DebriefingText { lines, moments }
}

/// The warning before a weak action, as lines. It names no alternative
/// (D-49).
pub fn warning(i18n: &I18n, view: &TableView, w: &Warning, figures: bool) -> Vec<Line> {
    let chosen = action(i18n, view, w.chosen.action);
    let mut out = vec![line(
        i18n.tf("coach-warning-text", &[("action", chosen.into())]),
        Style::Strong,
    )];
    if figures {
        out.push(line(
            i18n.tf(
                "coach-warning-loss",
                &[("points", format!("{:.1}", w.chosen.gap).into())],
            ),
            Style::Weak,
        ));
    }
    out
}

/// Status of an unavailable analysis.
pub fn unavailable(i18n: &I18n, reason: Unavailable) -> String {
    i18n.t(match reason {
        Unavailable::Cancelled => "coach-unavailable-cancelled",
        Unavailable::Failed => "coach-unavailable-failed",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;
    use skat_app::coach_model::{self as model, Explanation};
    use skat_app::{
        Budget, Card, CoachLevel, CoachSettings, Command, Level, Opponents, Phase, Rank, Seat,
        SeriesLength, Session, SessionConfig, Suit,
    };

    /// Explanations of the human's card-play decisions in a seeded series.
    fn explanations() -> Vec<(TableView, Explanation)> {
        let mut out = Vec::new();
        let mut s = Session::new(SessionConfig {
            seed: 11,
            series: SeriesLength::Fixed(3),
            kontra_re: true,
            opponents: Opponents::Levels([Level::Easy; 2]),
        });
        s.set_coach_budget(Budget {
            samples: 6,
            nodes_per_task: 20_000_000,
        });
        s.execute(Command::SetCoach(CoachSettings {
            level: CoachLevel::EveryMove,
            warnings: false,
            show_figures: true,
        }))
        .unwrap();
        while !s.view().series_finished {
            s.run_opponents();
            let view = s.view();
            if view.series_finished {
                break;
            }
            if view.legal_actions.is_empty() {
                s.execute(Command::NextGame).unwrap();
                continue;
            }
            if view.phase == Phase::Spiel && view.tricks_played >= 3 {
                let result = s.coach_job().unwrap().run(None);
                if let Some(e) = result.explanation() {
                    out.push((view.clone(), e.clone()));
                }
            }
            let action = if view.legal_actions.contains(&Action::Pass) {
                Action::Pass
            } else {
                view.legal_actions[0]
            };
            s.execute(Command::Act(action)).unwrap();
        }
        out
    }

    fn joined(lines: &[Line]) -> String {
        lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The texts shown before the move at every stage, and the
    /// debriefing of every option as if chosen.
    fn rendered(i18n: &I18n, view: &TableView, e: &Explanation, figures: bool) -> Vec<Vec<Line>> {
        let mut out: Vec<Vec<Line>> = [Stage::Considerations, Stage::Situation, Stage::Figures]
            .into_iter()
            .map(|stage| hints(i18n, view, &model::hints(e, stage), figures))
            .collect();
        let reviews: Vec<_> = view
            .legal_actions
            .iter()
            .filter_map(|&a| model::review(e, a))
            .collect();
        let moments = model::moments(&reviews);
        out.push(flat(&debriefing(
            i18n,
            view,
            &Debriefing::Ready(moments),
            figures,
        )));
        out
    }

    /// Every line of a debriefing, details included.
    fn flat(d: &DebriefingText) -> Vec<Line> {
        let mut out = d.lines.clone();
        for m in &d.moments {
            out.extend(m.summary.iter().cloned());
            out.extend(m.details.iter().cloned());
        }
        out
    }

    #[test]
    fn hints_never_name_or_rank_the_best_option() {
        let all = explanations();
        for language in Language::ALL {
            let i18n = I18n::new(language);
            // "About as good" also reads the situation, so it is left out.
            let judged: Vec<String> = [Judgement::Best, Judgement::Worse, Judgement::Weak]
                .into_iter()
                .map(|j| judgement(&i18n, j))
                .collect();
            for (view, e) in &all {
                for stage in [Stage::Considerations, Stage::Situation, Stage::Figures] {
                    let text = joined(&hints(&i18n, view, &model::hints(e, stage), true));
                    for j in &judged {
                        assert!(!text.contains(j.as_str()), "{stage:?}: {text}");
                    }
                    if stage == Stage::Considerations {
                        // No option is named before the situation stage.
                        for &a in &view.legal_actions {
                            assert!(!text.contains(&format!("• {}", action(&i18n, view, a))));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn hidden_figures_keep_the_reasoning_without_estimates() {
        let all = explanations();
        assert!(all.len() >= 5, "{}", all.len());
        let mut moments = 0;
        for language in Language::ALL {
            let i18n = I18n::new(language);
            for (view, e) in &all {
                let with = rendered(&i18n, view, e, true);
                let without = rendered(&i18n, view, e, false);
                moments += with.last().map_or(0, Vec::len);
                for (with, without) in with.iter().zip(&without) {
                    // The verbal reasoning stays: same headings and reasons
                    // (the figures stage falls back to the plain options).
                    let headings =
                        |lines: &[Line]| lines.iter().filter(|l| l.style == Style::Heading).count();
                    assert_eq!(headings(with), headings(without));
                    let without = joined(without);
                    // No estimated figure leaks: no percentages, no ±, none
                    // of the formatted estimates.
                    assert!(
                        !without.contains('%') && !without.contains('±'),
                        "{without}"
                    );
                    for o in &e.options {
                        if let Some(est) = o.estimate {
                            assert!(!without.contains(&percent(est.win_chance)));
                            assert!(!without.contains(&points(est.expected_score)));
                        }
                    }
                    for c in &e.considerations {
                        if let Consideration::Inferred(i) = c
                            && !i.inference.certain
                        {
                            assert!(!without.contains(&format!("{:.1}", i.inference.expected)));
                        }
                    }
                }
                let figures_stage = joined(&with[2]);
                assert!(
                    figures_stage.contains('%') || e.options.iter().all(|o| o.estimate.is_none())
                );
            }
        }
        assert!(moments > 0, "some debriefing was rendered");
    }

    // ------------------------------------------------------------------
    // Debriefing fixtures (D-52). The decision: trick 5, the human defends;
    // Kreuz-Bube is best, Pik-Sieben a natural trap clearly lower, the
    // Herz-Sieben in between. The figures are invented for the fixture
    // only; what matters is where they may appear.

    const BEST: Card = Card::new(Suit::Kreuz, Rank::Bube);
    const MIDDLE: Card = Card::new(Suit::Herz, Rank::Sieben);
    const TRAP: Card = Card::new(Suit::Pik, Rank::Sieben);

    fn estimate(win_chance: f64, expected_score: f64) -> model::Estimate {
        model::Estimate {
            win_chance,
            expected_score,
            expected_game_points: -2.0 * expected_score,
            thrown_in: 0.0,
            score_std_error: 1.5,
            samples: 24,
            exact: false,
        }
    }

    fn option(card: Card, judgement: Judgement, gap: f64, reasons: Vec<Reason>) -> ComparedOption {
        let e = match card {
            BEST => estimate(0.62, 21.4),
            MIDDLE => estimate(0.55, 15.4),
            _ => estimate(0.31, 7.1),
        };
        ComparedOption {
            action: Action::Play(card),
            natural: true,
            judgement,
            estimate: Some(e),
            value: Some(e.expected_score),
            gap,
            margin: 3.0,
            compared_with: Some(Action::Play(if card == BEST { MIDDLE } else { BEST })),
            reasons,
        }
    }

    fn trap() -> ComparedOption {
        option(
            TRAP,
            Judgement::Weak,
            14.3,
            vec![
                Reason::WinChance {
                    this: 0.31,
                    other: 0.62,
                },
                Reason::Points {
                    this: 7.1,
                    other: 21.4,
                },
            ],
        )
    }

    /// The decision with `chosen` played.
    fn fixture(chosen: ComparedOption) -> model::MoveReview {
        let comparison = model::Comparison {
            chosen: chosen.action,
            best: Some(Action::Play(BEST)),
            reliability: model::Reliability::Reliable,
        };
        model::MoveReview {
            decision: Decision::Card,
            role: Role::Defender,
            status: model::Status::Complete,
            bid: 18,
            hand_game: Some(false),
            trick: Some(5),
            important: true,
            forced: false,
            considerations: vec![
                Consideration::Known(Observation::TrumpsOut { count: 4 }),
                Consideration::Inferred(model::Inferred {
                    inference: model::Inference {
                        holder: Place::Seat(Seat::Hinterhand),
                        class: CardClass::Trumps,
                        expected: 1.2,
                        min: 0,
                        max: 3,
                        certain: false,
                    },
                    amount: Amount::AboutOne,
                }),
            ],
            chosen,
            best: Some(option(BEST, Judgement::Best, 0.0, Vec::new())),
            trap: Some(trap()),
            basis: model::Basis {
                finished: 24,
                planned: 24,
                enumerated: false,
                effective_samples: 20.0,
            },
            assumptions: vec![Assumption::UniformHiddenCards, Assumption::DoubleDummyPlay],
            evidence: model::Evidence {
                version: 1,
                snapshot: model::Snapshot {
                    seat: Seat::Hinterhand,
                    phase: Phase::Spiel,
                    role: Role::Defender,
                    game: None,
                    bids: 3,
                    tricks: 4,
                    trick_cards: 1,
                    own: Default::default(),
                    own_skat: None,
                    revealed: None,
                },
                status: model::Status::Complete,
                items: Vec::new(),
            },
            comparison,
            teaching: model::Teaching::Fallback,
        }
    }

    fn good() -> model::MoveReview {
        fixture(option(BEST, Judgement::Best, 0.0, Vec::new()))
    }

    fn equivalent() -> model::MoveReview {
        let reasons = vec![Reason::WithinUncertainty];
        fixture(option(MIDDLE, Judgement::Equivalent, 0.5, reasons))
    }

    fn weak() -> model::MoveReview {
        fixture(trap())
    }

    fn worse() -> model::MoveReview {
        let reasons = vec![Reason::Points {
            this: 15.4,
            other: 21.4,
        }];
        fixture(option(MIDDLE, Judgement::Worse, 6.0, reasons))
    }

    fn uncertain() -> model::MoveReview {
        model::MoveReview {
            status: model::Status::BudgetExhausted,
            ..weak()
        }
    }

    fn forced() -> model::MoveReview {
        model::MoveReview {
            forced: true,
            ..good()
        }
    }

    fn table() -> TableView {
        Session::new(SessionConfig::default()).view()
    }

    fn ready(reviews: &[model::MoveReview]) -> Debriefing {
        Debriefing::Ready(model::moments(reviews))
    }

    /// Estimated figures of the fixture, as the frontend formats them.
    const ESTIMATES: [&str; 8] = [
        "62 %", "55 %", "31 %", "+21.4", "+15.4", "+7.1", "14.3", "1.2",
    ];

    fn assert_no_estimates(text: &str) {
        for figure in ESTIMATES {
            assert!(!text.contains(figure), "{figure} in {text}");
        }
        assert!(!text.contains('%') && !text.contains('±'), "{text}");
    }

    #[test]
    fn fixture_notes_follow_the_judgement() {
        let notes =
            [good(), equivalent(), worse(), weak(), uncertain(), forced()].map(|r| r.note());
        assert_eq!(
            notes,
            [
                Note::Good,
                Note::AboutEqual,
                Note::SomewhatWeaker,
                Note::ClearlyWeaker,
                Note::Uncertain,
                Note::Forced,
            ]
        );
        for language in Language::ALL {
            let i18n = I18n::new(language);
            let text = |note| {
                let last = LastMove {
                    action: Action::Play(MIDDLE),
                    bid: 18,
                    hand_game: Some(false),
                    note: Some(note),
                };
                super::note(&i18n, &last).text
            };
            assert!(text(Note::Forced).contains(&i18n.t("coach-note-forced")));
            assert!(text(Note::Uncertain).contains(&i18n.t("coach-note-uncertain")));
        }
    }

    #[test]
    fn no_moments_says_so() {
        let view = table();
        for language in Language::ALL {
            let i18n = I18n::new(language);
            for reviews in [vec![], vec![uncertain(), forced()]] {
                let d = debriefing(&i18n, &view, &ready(&reviews), true);
                assert!(d.moments.is_empty());
                assert_eq!(d.lines.len(), 2);
                assert_eq!(d.lines[1].text, i18n.t("coach-debriefing-none"));
            }
        }
    }

    #[test]
    fn one_weak_moment_reads_as_a_model_judgement_with_an_honest_fallback() {
        let view = table();
        let d = debriefing(&I18n::new(Language::De), &view, &ready(&[weak()]), false);
        assert_eq!(d.moments.len(), 1);
        let summary: Vec<&str> = d.moments[0]
            .summary
            .iter()
            .map(|l| l.text.as_str())
            .collect();
        assert_eq!(
            summary,
            [
                "Stich 5 · Karte spielen",
                "Du hast Pik-Sieben gewählt. Das Modell bewertet das deutlich niedriger als Kreuz-Bube.",
                "Eine strategische Begründung liegt noch nicht vor – nur der Modellvergleich (Analysedetails).",
            ]
        );
        let d = debriefing(&I18n::new(Language::En), &view, &ready(&[weak()]), false);
        let summary: Vec<&str> = d.moments[0]
            .summary
            .iter()
            .map(|l| l.text.as_str())
            .collect();
        assert_eq!(
            summary,
            [
                "Trick 5 · Play a card",
                "You chose Seven of Spades. The model rates it clearly lower than Jack of Clubs.",
                "No strategic explanation is available yet – only the model's comparison (analysis details).",
            ]
        );
    }

    #[test]
    fn good_and_equivalent_moments_name_the_trap_without_claiming_it_was_considered() {
        let view = table();
        for language in Language::ALL {
            let i18n = I18n::new(language);
            for (review, played, id) in [
                (good(), BEST, "coach-moment-chosen-best"),
                (equivalent(), MIDDLE, "coach-moment-chosen-equivalent"),
            ] {
                let d = debriefing(&i18n, &view, &ready(&[review]), false);
                assert_eq!(d.moments.len(), 1, "{id}");
                let m = &d.moments[0];
                let name = |c| text::card(&i18n, c);
                let chosen = i18n.tf(
                    id,
                    &[("action", name(played).into()), ("best", name(BEST).into())],
                );
                let trap = i18n.tf("coach-moment-trap", &[("trap", name(TRAP).into())]);
                let texts: Vec<&str> = m.summary.iter().map(|l| l.text.as_str()).collect();
                assert_eq!(texts[1..3], [chosen.as_str(), trap.as_str()]);
                assert_eq!(texts[3], i18n.t("coach-moment-no-reason"));
            }
        }
    }

    #[test]
    fn many_moments_keep_three_summaries_without_estimates() {
        let view = table();
        let all = [good(), equivalent(), worse(), weak(), uncertain(), forced()];
        for language in Language::ALL {
            let i18n = I18n::new(language);
            for figures in [false, true] {
                let d = debriefing(&i18n, &view, &ready(&all), figures);
                // The costliest bad (weak), the best good (good, first of
                // two equal traps), the next bad (worse); in game order.
                assert_eq!(d.moments.len(), 3);
                let chosen: Vec<String> = d
                    .moments
                    .iter()
                    .map(|m| m.summary[1].text.clone())
                    .collect();
                let name = |c| text::card(&i18n, c);
                assert!(chosen[0].contains(&name(BEST)), "{chosen:?}");
                assert!(chosen[1].contains(&name(MIDDLE)), "{chosen:?}");
                assert!(chosen[2].contains(&name(TRAP)), "{chosen:?}");
                for m in &d.moments {
                    // Never an estimate in the summary, figures or not.
                    assert_no_estimates(&joined(&m.summary));
                    let details = joined(&m.details);
                    // Party and metric labels, ratings and assumptions.
                    assert!(details.contains(&i18n.t("coach-assumption-uniform")));
                    assert!(details.contains(&i18n.t("coach-assumption-double-dummy")));
                    assert!(details.contains(&i18n.t("coach-section-facts")));
                    assert!(
                        details.contains(&i18n.tf("coach-fact-trumps-out", &[("count", 4.into())]))
                    );
                    assert!(details.contains(&i18n.tf(
                        "coach-details-rating",
                        &[
                            ("action", name(BEST).into()),
                            ("judgement", judgement(&i18n, Judgement::Best).into()),
                        ],
                    )));
                    assert!(details.contains(&i18n.t("coach-metric-label-points-defender")));
                    if figures {
                        assert!(details.contains("62 %") && details.contains("+21.4"));
                        assert!(details.contains(&i18n.t("coach-party-defenders")));
                    } else {
                        assert_no_estimates(&details);
                    }
                }
                let weak = joined(&d.moments[2].details);
                assert!(weak.contains(&i18n.tf(
                    "coach-metric-label-win",
                    &[("party", i18n.t("coach-party-defenders").into())],
                )));
            }
        }
    }
}
