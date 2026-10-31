// SPDX-License-Identifier: EUPL-1.2

//! Focused live hints and teachable-moment selection (#46, D-58): the
//! optional focus, pedagogical eligibility apart from the engine's
//! important mark, no preferred action before the move with any focus,
//! and the deterministic choice of zero to three primary moments, one per
//! concept. The expected selections are stated from the policy in
//! `docs/coach.md` and the hand-made review variants below, not from the
//! implementation's output.

use skat_analysis::lesson::catalogue;
use skat_analysis::{Analysis, Budget, Status, analyze};
use skat_coach::{
    Concept, Explanation, Judgement, MOMENT_POINTS, MomentKind, MoveReview, PRIMARY_MOMENTS, Stage,
    Taught, Teaching, explain, hints, hints_for, moments, review,
};
use skat_core::{
    Action, Announcement, Card, CardSet, Deal, Game, GameType, PlayerView, Rank, RuleConfig, Suit,
};

const SMALL: Budget = Budget {
    samples: 4,
    nodes_per_task: 50_000_000,
};

fn c(s: &str) -> Card {
    let suit = match &s[..1] {
        "C" => Suit::Kreuz,
        "S" => Suit::Pik,
        "H" => Suit::Herz,
        "D" => Suit::Karo,
        _ => panic!("bad suit in {s}"),
    };
    let rank = match &s[1..] {
        "7" => Rank::Sieben,
        "8" => Rank::Acht,
        "9" => Rank::Neun,
        "T" => Rank::Zehn,
        "J" => Rank::Bube,
        "Q" => Rank::Dame,
        "K" => Rank::Koenig,
        "A" => Rank::Ass,
        _ => panic!("bad rank in {s}"),
    };
    Card::new(suit, rank)
}

fn cards(s: &str) -> CardSet {
    s.split_whitespace().map(c).collect()
}

/// VH plays Grand Hand at 18 and leads CJ; MH plays SA, HH HQ. VH leads
/// trick 2: both defenders are proven void in Trumpf (they did not follow
/// the Bube), and a Trumpf was played, so `proven-void` and
/// `remaining-trumpf` both apply; no pre-play lesson and, before the move,
/// no mechanism.
fn after_voids() -> PlayerView {
    let deal = Deal::new(
        [
            cards("CJ SJ HJ DJ CA CT CK CQ C9 C8"),
            cards("SA ST SK SQ S9 S8 S7 HA HT HK"),
            cards("HQ H9 H8 H7 DA DT DK DQ D9 D8"),
        ],
        cards("C7 D7"),
    )
    .unwrap();
    let mut g = Game::from_deal(deal, RuleConfig::default());
    for a in [
        Action::Bid(18),
        Action::Hold,
        Action::Pass,
        Action::Pass,
        Action::Hand,
        Action::Announce(Announcement::plain(GameType::Grand)),
        Action::Play(c("CJ")),
        Action::Play(c("SA")),
        Action::Play(c("HQ")),
    ] {
        g.apply(a).unwrap();
    }
    g.view(g.current_seat().unwrap())
}

fn explained(view: &PlayerView) -> (Analysis, Explanation) {
    let analysis = analyze(view, SMALL, 3, None).unwrap();
    let e = explain(view, &analysis).unwrap();
    (analysis, e)
}

#[test]
fn concepts_are_the_catalogue_lessons() {
    let keys: Vec<String> = catalogue().iter().map(|l| l.id.key.to_string()).collect();
    for concept in Concept::ALL {
        assert!(keys.contains(&concept.key().to_owned()), "{concept:?}");
    }
    for lesson in catalogue() {
        let concept = Concept::of(&lesson.id).expect("every lesson has a concept");
        assert_eq!(concept.key(), lesson.id.key);
        assert_eq!(
            concept.mechanism(),
            lesson.kind == skat_coach::LessonKind::Mechanism
        );
    }
    assert_eq!(Concept::ALL.len(), catalogue().len());
}

#[test]
fn focus_picks_the_lesson_when_several_apply() {
    let view = after_voids();
    let (_, e) = explained(&view);
    let concept = |h: &skat_coach::Hints| h.concept;
    // Without a focus the catalogue order: the void comes first.
    let plain = hints(&e, Stage::Considerations);
    assert_eq!(concept(&plain), Some(Concept::ProvenVoid));
    assert!(plain.teachable());
    for (focus, expected) in [
        (Concept::ProvenVoid, Concept::ProvenVoid),
        (Concept::RemainingTrumpf, Concept::RemainingTrumpf),
        // Not applicable here: the first lesson that applies.
        (Concept::HandStructure, Concept::ProvenVoid),
        (Concept::TrumpfVulnerability, Concept::ProvenVoid),
    ] {
        let h = hints_for(&e, Stage::Considerations, Some(focus));
        assert_eq!(concept(&h), Some(expected), "{focus:?}");
        assert!(matches!(h.teaching, Teaching::Lesson { .. }));
    }
    // Pedagogical eligibility before the move, apart from `important`.
    assert!(e.teachable(None));
    assert!(e.teachable(Some(Concept::ProvenVoid)));
    assert!(e.teachable(Some(Concept::RemainingTrumpf)));
    assert!(!e.teachable(Some(Concept::HandStructure)));
    // Mechanisms need the move: never eligible before it.
    assert!(!e.teachable(Some(Concept::TrumpfVulnerability)));
    assert!(!e.teachable(Some(Concept::PartnerOvertake)));
    // After the move every applicable lesson is kept; the focus picks.
    let r = review(&e, view.legal_actions[0]).unwrap();
    let concepts: Vec<Concept> = r.taught.iter().map(|t| t.concept).collect();
    assert!(concepts.contains(&Concept::ProvenVoid));
    assert!(concepts.contains(&Concept::RemainingTrumpf));
    assert_eq!(
        r.lesson_for(Some(Concept::RemainingTrumpf))
            .map(|t| t.concept),
        Some(Concept::RemainingTrumpf)
    );
    assert_eq!(r.lesson_for(None).map(|t| &t.content), r.lesson.as_ref());
}

#[test]
fn no_focus_or_stage_reveals_the_preferred_action() {
    // The same decision with opposite option rankings: with every focus,
    // all hints before the figures are identical, the comparison question
    // included, so neither wording, lesson, concept, option list nor plan
    // ID can imply the preferred action. Figures (opt-in) keep the
    // neutral order and carry no judgement.
    let view = after_voids();
    let analysis = analyze(&view, SMALL, 3, None).unwrap();
    let ranked = |rising: bool| {
        let mut a = analysis.clone();
        let n = a.options.len();
        for (i, o) in a.options.iter_mut().enumerate() {
            let e = o.estimate.as_mut().unwrap();
            e.expected_score = 20.0 * if rising { i } else { n - 1 - i } as f64;
        }
        a
    };
    let x = explain(&view, &ranked(true)).unwrap();
    let y = explain(&view, &ranked(false)).unwrap();
    assert_ne!(x.best().map(|o| o.action), y.best().map(|o| o.action));
    let focuses = std::iter::once(None).chain(Concept::ALL.into_iter().map(Some));
    for focus in focuses {
        // Repeated requests: every stage up to the situation.
        for stage in [Stage::Considerations, Stage::Situation] {
            let (hx, hy) = (hints_for(&x, stage, focus), hints_for(&y, stage, focus));
            assert_eq!(hx, hy, "{focus:?} {stage:?}");
            assert_eq!(hx.compare_question, stage == Stage::Situation);
            assert!(hx.figures.is_none());
            if let Some(s) = &hx.situation {
                assert_eq!(s.options, view.legal_actions, "neutral order");
            }
            assert!(!hx.concept.is_some_and(Concept::mechanism));
        }
        let figures = hints_for(&x, Stage::Figures, focus).figures.unwrap();
        assert!(
            figures
                .iter()
                .map(|f| f.action)
                .eq(view.legal_actions.iter().copied())
        );
    }
}

/// A review of the fixture with `concept` as its only lesson, the chosen
/// option judged `judgement` with loss `gap`, and the analysis `status`.
/// The lesson content is the fixture's; the selection reads the concept
/// only.
fn variant(
    base: &MoveReview,
    concept: Option<Concept>,
    judgement: Judgement,
    gap: f64,
    status: Status,
) -> MoveReview {
    let mut r = base.clone();
    let first = base.taught[0].clone();
    r.taught = concept
        .map(|concept| Taught { concept, ..first })
        .into_iter()
        .collect();
    r.teaching = r
        .taught
        .first()
        .map_or(Teaching::Fallback, |t| t.teaching.clone());
    r.lesson = r.taught.first().map(|t| t.content.clone());
    r.chosen.judgement = judgement;
    r.chosen.gap = gap;
    r.status = status;
    r.important = false;
    r.trap = None;
    r.forced = false;
    r
}

fn base() -> MoveReview {
    let view = after_voids();
    let (_, e) = explained(&view);
    review(&e, view.legal_actions[0]).unwrap()
}

/// The index of each moment's review in `reviews` (the fixture variants
/// differ in their chosen gap), with its kind and concept.
fn picked(reviews: &[MoveReview], focus: Option<Concept>) -> Vec<(usize, MomentKind, Concept)> {
    moments(reviews, focus)
        .iter()
        .map(|m| {
            let i = reviews.iter().position(|r| *r == m.review).unwrap();
            (i, m.kind, m.concept)
        })
        .collect()
}

#[test]
fn zero_eligible_moments() {
    let b = base();
    let done = Status::Complete;
    // A big loss without a lesson, a forced move with one, nothing at all.
    let costly = variant(&b, None, Judgement::Weak, 40.0, done);
    let mut forced = variant(&b, Some(Concept::ProvenVoid), Judgement::Best, 0.0, done);
    forced.forced = true;
    assert!(moments(&[], None).is_empty());
    assert!(moments(&[costly.clone(), forced.clone()], None).is_empty());
    // An important decision with an avoided trap but no lesson: none.
    let mut trap = costly.clone();
    trap.chosen.judgement = Judgement::Best;
    trap.important = true;
    trap.trap = Some(costly.chosen.clone());
    assert!(moments(&[trap], Some(Concept::ProvenVoid)).is_empty());
}

#[test]
fn sparse_games_and_equivalent_choices() {
    let b = base();
    let done = Status::Complete;
    // A single eligible decision, about as good as the best: one neutral
    // moment; no praise or criticism quota fills the rest.
    let equal = variant(
        &b,
        Some(Concept::ProvenVoid),
        Judgement::Equivalent,
        0.5,
        done,
    );
    assert_eq!(
        picked(std::slice::from_ref(&equal), None),
        [(0, MomentKind::Neutral, Concept::ProvenVoid)]
    );
    // A small loss below MOMENT_POINTS is neutral too, and a loss of
    // exactly MOMENT_POINTS is bad.
    let small = variant(
        &b,
        Some(Concept::RemainingTrumpf),
        Judgement::Worse,
        MOMENT_POINTS - 0.5,
        done,
    );
    let bad = variant(
        &b,
        Some(Concept::GameValue),
        Judgement::Worse,
        MOMENT_POINTS,
        done,
    );
    assert_eq!(
        picked(&[equal.clone(), small.clone(), bad.clone()], None),
        [
            (0, MomentKind::Neutral, Concept::ProvenVoid),
            (1, MomentKind::Neutral, Concept::RemainingTrumpf),
            (2, MomentKind::Bad, Concept::GameValue),
        ]
    );
    // Only good moments: no bad one is required.
    let mut good = variant(&b, Some(Concept::HandStructure), Judgement::Best, 0.0, done);
    good.important = true;
    good.trap = Some(bad.chosen.clone());
    assert_eq!(
        picked(std::slice::from_ref(&good), None),
        [(0, MomentKind::Good, Concept::HandStructure)]
    );
}

#[test]
fn one_moment_per_concept_chosen_deterministically() {
    let b = base();
    let done = Status::Complete;
    let v = |concept, judgement, gap| variant(&b, Some(concept), judgement, gap, done);
    // Five void moments, increasing loss: only the costliest is kept.
    let voids: Vec<MoveReview> = (0..5)
        .map(|i| v(Concept::ProvenVoid, Judgement::Weak, 10.0 + f64::from(i)))
        .collect();
    assert_eq!(
        picked(&voids, None),
        [(4, MomentKind::Bad, Concept::ProvenVoid)]
    );
    // Equal weights: the earlier one.
    let same = vec![v(Concept::ProvenVoid, Judgement::Weak, 12.0); 3];
    assert_eq!(moments(&same, None)[0].review, same[0]);
    // Four concepts: at most three, ranked by rating, then mechanism, then
    // loss; shown in game order.
    let mixed = vec![
        v(Concept::ProvenVoid, Judgement::Equivalent, 0.5), // neutral fact
        v(Concept::RemainingTrumpf, Judgement::Weak, 8.0),  // bad, 8
        v(Concept::TrumpfVulnerability, Judgement::Equivalent, 0.4), // neutral mechanism
        v(Concept::GameValue, Judgement::Weak, 20.0),       // bad, 20
    ];
    let expected = [
        (1, MomentKind::Bad, Concept::RemainingTrumpf),
        (2, MomentKind::Neutral, Concept::TrumpfVulnerability),
        (3, MomentKind::Bad, Concept::GameValue),
    ];
    assert_eq!(picked(&mixed, None), expected);
    assert_eq!(expected.len(), PRIMARY_MOMENTS);
    // Same input, same output.
    assert_eq!(moments(&mixed, None), moments(&mixed.clone(), None));
    // The focus comes first, even neutral, and is marked.
    let focused = moments(&mixed, Some(Concept::ProvenVoid));
    assert_eq!(
        picked(&mixed, Some(Concept::ProvenVoid)),
        [
            (0, MomentKind::Neutral, Concept::ProvenVoid),
            (1, MomentKind::Bad, Concept::RemainingTrumpf),
            (3, MomentKind::Bad, Concept::GameValue),
        ]
    );
    assert_eq!(
        focused.iter().map(|m| m.focused).collect::<Vec<_>>(),
        [true, false, false]
    );
    // A focus nobody has changes nothing.
    assert_eq!(
        picked(&mixed, Some(Concept::PartnerOvertake)),
        picked(&mixed, None)
    );
}

#[test]
fn provisional_estimates_are_never_rated() {
    let b = base();
    // Out of budget, judged weak on the finished samples: exact facts stay
    // teachable, but the moment is neutral and ranks below rated ones.
    let provisional = variant(
        &b,
        Some(Concept::ProvenVoid),
        Judgement::Weak,
        30.0,
        Status::BudgetExhausted,
    );
    let rated = |concept| variant(&b, Some(concept), Judgement::Weak, 6.0, Status::Complete);
    assert_eq!(
        picked(std::slice::from_ref(&provisional), None),
        [(0, MomentKind::Neutral, Concept::ProvenVoid)]
    );
    let all = [
        provisional.clone(),
        rated(Concept::RemainingTrumpf),
        rated(Concept::GameValue),
        rated(Concept::HandStructure),
    ];
    let found = picked(&all, None);
    assert_eq!(found.len(), PRIMARY_MOMENTS);
    assert!(
        found
            .iter()
            .all(|&(i, kind, _)| i > 0 && kind == MomentKind::Bad)
    );
    // Cancelled without a lesson: nothing.
    let cancelled = variant(&b, None, Judgement::Weak, 30.0, Status::Cancelled);
    assert!(moments(&[cancelled], None).is_empty());
}
