// SPDX-License-Identifier: EUPL-1.2

//! Translated texts of the live coach (M5, D-49): turns the staged hints,
//! the note on the last move, the debriefing and the warning of
//! `skat-coach` (via `skat-app`) into lines. Before the move nothing ranks
//! or labels an option. Estimated figures (chances, expected points,
//! uncertainty, expected card counts) appear only with "show figures";
//! exact facts such as counted cards, Augen and bid values always do.

use skat_app::coach_model::{
    Amount, Assumption, Basis, CardClass, ComparedOption, Consideration, Decision, Estimate,
    Follow, Hints, Judgement, Moment, MomentKind, Note, Objective, Observation, Place, Reason,
    Role, Situation, Stage, Warning,
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

/// The figures of one estimate.
fn metrics(i18n: &I18n, role: Role, e: &Estimate) -> String {
    let party = i18n.t(match role {
        Role::Bidder => "coach-party-own",
        Role::Declarer => "coach-party-declarer",
        Role::Defender => "coach-party-defenders",
    });
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

/// Reasons of `o` against the option it was compared with.
fn reasons(
    i18n: &I18n,
    o: &ComparedOption,
    name: impl Fn(Action) -> String,
    figures: bool,
) -> Vec<Line> {
    let Some(other) = o.compared_with else {
        return Vec::new();
    };
    let other = name(other);
    o.reasons
        .iter()
        .map(|&r| {
            line(
                format!("• {}", reason(i18n, &other, r, figures)),
                Style::Normal,
            )
        })
        .collect()
}

/// One moment of the debriefing.
fn moment(i18n: &I18n, view: &TableView, m: &Moment, figures: bool) -> Vec<Line> {
    let r = &m.review;
    let name = |a: Action| action_at(i18n, r.bid, r.hand_game, a);
    let mut what = decision(i18n, r.decision);
    if let Some(trick) = r.trick {
        what = format!(
            "{what} · {}",
            i18n.tf("coach-moment-trick", &[("trick", trick.into())])
        );
    }
    let mut out = vec![line(
        i18n.tf(
            match m.kind {
                MomentKind::Good => "coach-moment-good",
                MomentKind::Bad => "coach-moment-bad",
            },
            &[("decision", what.into())],
        ),
        Style::Heading,
    )];
    out.push(line(
        i18n.tf(
            "coach-moment-chosen",
            &[
                ("action", name(r.chosen.action).into()),
                ("judgement", judgement(i18n, r.chosen.judgement).into()),
            ],
        ),
        Style::Strong,
    ));
    let figure_line = |o: &ComparedOption| {
        o.estimate.filter(|_| figures).map(|e| {
            line(
                format!("{}: {}", name(o.action), metrics(i18n, r.role, &e)),
                Style::Weak,
            )
        })
    };
    match m.kind {
        MomentKind::Bad => {
            if let Some(best) = &r.best {
                out.push(line(
                    i18n.tf("coach-moment-better", &[("best", name(best.action).into())]),
                    Style::Normal,
                ));
                out.extend(reasons(i18n, &r.chosen, name, figures));
                out.extend(figure_line(&r.chosen));
                out.extend(figure_line(best));
            }
        }
        MomentKind::Good => {
            if let Some(trap) = &r.trap {
                out.push(line(
                    i18n.tf("coach-moment-trap", &[("trap", name(trap.action).into())]),
                    Style::Normal,
                ));
                out.extend(reasons(i18n, trap, name, figures));
                if figures {
                    out.push(line(
                        i18n.tf(
                            "coach-moment-trap-loss",
                            &[("points", format!("{:.1}", trap.gap).into())],
                        ),
                        Style::Weak,
                    ));
                }
            }
        }
    }
    let knew = considerations(i18n, view, &r.considerations, figures);
    if !knew.is_empty() {
        out.push(line(i18n.t("coach-moment-knew"), Style::Strong));
        // One level below the moment: no section headings of their own.
        out.extend(knew.into_iter().filter(|l| l.style != Style::Heading));
    }
    out
}

/// The end-of-game debriefing, as lines (D-49).
pub fn debriefing(i18n: &I18n, view: &TableView, d: &Debriefing, figures: bool) -> Vec<Line> {
    let mut out = vec![line(i18n.t("coach-debriefing-title"), Style::Heading)];
    match d {
        Debriefing::Computing { remaining } => out.push(line(
            i18n.tf(
                "coach-debriefing-computing",
                &[("count", (*remaining).into())],
            ),
            Style::Weak,
        )),
        Debriefing::Ready(moments) if moments.is_empty() => {
            out.push(line(i18n.t("coach-debriefing-none"), Style::Normal));
        }
        Debriefing::Ready(moments) => {
            for m in moments {
                out.extend(moment(i18n, view, m, figures));
            }
        }
    }
    out
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
        Budget, CoachLevel, CoachSettings, Command, Level, Opponents, Phase, SeriesLength, Session,
        SessionConfig,
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
        out.push(debriefing(i18n, view, &Debriefing::Ready(moments), figures));
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
}
