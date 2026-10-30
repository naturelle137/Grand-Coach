// SPDX-License-Identifier: EUPL-1.2

//! Translated texts of the live coach (M5): turns the structured
//! explanation of `skat-coach` (via `skat-app`) into lines. Estimated
//! figures (chances, expected points, uncertainty, expected card counts)
//! appear only with "show figures"; exact facts such as counted cards,
//! Augen and bid values always do.

use skat_app::coach_model::{
    Amount, Assumption, CardClass, ComparedOption, Consideration, Decision, Explanation, Follow,
    Judgement, Objective, Observation, Place, Reason, Role, Verdict, Warning,
};
use skat_app::{Action, Contract, PlayerId, Seat, TableView, Unavailable};

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

/// Options shown in full; the rest only count.
const SHOWN_OPTIONS: usize = 5;

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

/// A legal action in words.
pub fn action(i18n: &I18n, view: &TableView, action: Action) -> String {
    match action {
        Action::Bid(value) => i18n.tf("coach-action-bid", &[("value", value.into())]),
        Action::Hold => i18n.tf("coach-action-hold", &[("value", view.bid.into())]),
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
            let contract = Contract::new(a, view.hand_game == Some(true));
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

fn option_lines(
    i18n: &I18n,
    view: &TableView,
    role: Role,
    o: &ComparedOption,
    figures: bool,
) -> Vec<Line> {
    let mut out = vec![line(
        format!(
            "{} – {}",
            action(i18n, view, o.action),
            judgement(i18n, o.judgement)
        ),
        if o.judgement == Judgement::Best {
            Style::Strong
        } else {
            Style::Normal
        },
    )];
    if figures && let Some(e) = o.estimate {
        let party = i18n.t(match role {
            Role::Bidder => "coach-party-own",
            Role::Declarer => "coach-party-declarer",
            Role::Defender => "coach-party-defenders",
        });
        let mut metrics = i18n.tf(
            "coach-metric-win",
            &[
                ("party", party.into()),
                ("chance", percent(e.win_chance).into()),
            ],
        );
        metrics.push_str(" · ");
        metrics.push_str(&i18n.tf(
            "coach-metric-score",
            &[
                ("score", points(e.expected_score).into()),
                ("se", format!("{:.1}", e.score_std_error).into()),
            ],
        ));
        if role != Role::Declarer {
            metrics.push_str(" · ");
            metrics.push_str(&i18n.tf(
                "coach-metric-game-points",
                &[("points", points(e.expected_game_points).into())],
            ));
        }
        out.push(line(metrics, Style::Weak));
    }
    if let Some(other) = o.compared_with {
        let other = action(i18n, view, other);
        for &r in &o.reasons {
            out.push(line(
                format!("• {}", reason(i18n, &other, r, figures)),
                Style::Normal,
            ));
        }
    }
    out
}

/// The explanation as lines.
pub fn explanation(i18n: &I18n, view: &TableView, e: &Explanation, figures: bool) -> Vec<Line> {
    let mut out = vec![
        line(
            format!("{} · {}", decision(i18n, e.decision), role(i18n, e.role)),
            Style::Heading,
        ),
        line(objective(i18n, e.role, e.objective), Style::Normal),
    ];
    if !e.complete() {
        out.push(line(i18n.t("coach-provisional"), Style::Strong));
    } else if e.important {
        out.push(line(i18n.t("coach-important"), Style::Strong));
    }
    let facts: Vec<Line> = e
        .considerations
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
    let guesses: Vec<Line> = e
        .considerations
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
    out.push(line(i18n.t("coach-section-options"), Style::Heading));
    let best = e.best().map(|b| action(i18n, view, b.action));
    out.push(line(
        match (e.verdict, &best) {
            (Verdict::Forced, _) => i18n.t("coach-verdict-forced"),
            (Verdict::Clear, Some(best)) if e.complete() => {
                i18n.tf("coach-verdict-clear", &[("best", best.clone().into())])
            }
            (Verdict::Close { equivalent }, Some(best)) if e.complete() => i18n.tf(
                "coach-verdict-close",
                &[("best", best.clone().into()), ("count", equivalent.into())],
            ),
            (Verdict::Unknown, _) => i18n.t("coach-verdict-unknown"),
            _ => i18n.t("coach-verdict-provisional"),
        },
        Style::Strong,
    ));
    for o in e.options.iter().take(SHOWN_OPTIONS) {
        out.extend(option_lines(i18n, view, e.role, o, figures));
    }
    let rest = e.options.len().saturating_sub(SHOWN_OPTIONS);
    if rest > 0 {
        out.push(line(
            i18n.tf("coach-more-options", &[("count", rest.into())]),
            Style::Weak,
        ));
    }
    out.push(line(i18n.t("coach-section-basis"), Style::Heading));
    out.push(line(
        if e.basis.enumerated {
            i18n.t("coach-basis-exact")
        } else if figures {
            i18n.tf(
                "coach-basis-sampled-figures",
                &[("finished", e.basis.finished.into())],
            )
        } else {
            i18n.t("coach-basis-sampled")
        },
        Style::Weak,
    ));
    for a in &e.assumptions {
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

/// The warning before a weak action, as lines.
pub fn warning(i18n: &I18n, view: &TableView, w: &Warning, figures: bool) -> Vec<Line> {
    let chosen = action(i18n, view, w.chosen.action);
    let best = action(i18n, view, w.best.action);
    let mut out = vec![line(
        i18n.tf(
            "coach-warning-text",
            &[("action", chosen.into()), ("best", best.clone().into())],
        ),
        Style::Strong,
    )];
    for &r in &w.chosen.reasons {
        out.push(line(
            format!("• {}", reason(i18n, &best, r, figures)),
            Style::Normal,
        ));
    }
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

    #[test]
    fn hidden_figures_keep_the_reasoning_without_estimates() {
        let all = explanations();
        assert!(all.len() >= 5, "{}", all.len());
        for language in Language::ALL {
            let i18n = I18n::new(language);
            for (view, e) in &all {
                let with = joined(&explanation(&i18n, view, e, true));
                let without = joined(&explanation(&i18n, view, e, false));
                // The verbal reasoning stays: same sections, same reasons.
                let headings = |figures| {
                    explanation(&i18n, view, e, figures)
                        .into_iter()
                        .filter(|l| l.style != Style::Weak)
                        .count()
                };
                assert_eq!(headings(true), headings(false));
                assert!(without.contains(&i18n.t("coach-section-options")));
                // No estimated figure leaks: no percentages, no ±, none of
                // the formatted estimates.
                assert!(
                    !without.contains('%') && !without.contains('±'),
                    "{without}"
                );
                assert!(with.contains('%'));
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
        }
    }
}
