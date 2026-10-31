// SPDX-License-Identifier: EUPL-1.2

//! Coach explanations for fixed positions. The expected facts and the
//! qualitative reading of each position are stated from the cards
//! (comments), independently of the analysis; [`assert_traceable`] checks
//! that every claim of every explanation comes from the analysis.
//!
//! Notation: suit `C` Kreuz, `S` Pik, `H` Herz, `D` Karo; rank
//! `7 8 9 T J Q K A`.

use skat_analysis::lesson::{
    Applicability, Condition, EvidenceClass, FactKind, Lesson, LessonId, LessonKind, Support,
    catalogue, gather,
};
use skat_analysis::sampling::Place;
use skat_analysis::{Analysis, Budget, Cancel, CardClass, Observation, Status, analyze, utility};
use skat_coach::{
    Amount, BAD_MOMENTS, Consideration, Decision, Explanation, GOOD_MOMENTS, Judgement,
    MOMENT_POINTS, Moment, MomentKind, Note, Objective, PRIMARY_MOMENTS, Reason, Reliability, Role,
    Situation, Stage, Teaching, Verdict, WEAK_POINTS, decision_seed, explain, hints,
    lesson_content, moments, review, teach,
};
use skat_core::{
    Action, Announcement, Card, CardSet, Deal, Follow, Game, GameType, PlayerView, Rank,
    RuleConfig, Seat, Suit,
};

const SMALL: Budget = Budget {
    samples: 4,
    nodes_per_task: 50_000_000,
};

fn c(s: &str) -> Card {
    let mut chars = s.chars();
    let suit = match chars.next() {
        Some('C') => Suit::Kreuz,
        Some('S') => Suit::Pik,
        Some('H') => Suit::Herz,
        Some('D') => Suit::Karo,
        _ => panic!("bad suit in {s}"),
    };
    let rank = match chars.next() {
        Some('7') => Rank::Sieben,
        Some('8') => Rank::Acht,
        Some('9') => Rank::Neun,
        Some('T') => Rank::Zehn,
        Some('J') => Rank::Bube,
        Some('Q') => Rank::Dame,
        Some('K') => Rank::Koenig,
        Some('A') => Rank::Ass,
        _ => panic!("bad rank in {s}"),
    };
    Card::new(suit, rank)
}

fn cards(s: &str) -> CardSet {
    s.split_whitespace().map(c).collect()
}

fn deal(vh: &str, mh: &str, hh: &str, skat: &str) -> Deal {
    Deal::new([cards(vh), cards(mh), cards(hh)], cards(skat)).expect("valid test deal")
}

/// VH: all Jacks and six Kreuz. MH: Pik and Herz. HH: Herz and Karo.
fn standard_deal() -> Deal {
    deal(
        "CJ SJ HJ DJ CA CT CK CQ C9 C8",
        "SA ST SK SQ S9 S8 S7 HA HT HK",
        "HQ H9 H8 H7 DA DT DK DQ D9 D8",
        "C7 D7",
    )
}

/// MH 18, VH holds, MH and HH pass: VH declarer at 18.
const VH_AT_18: [Action; 4] = [Action::Bid(18), Action::Hold, Action::Pass, Action::Pass];

fn game(deal: Deal, kontra_re: bool) -> Game {
    Game::from_deal(deal, RuleConfig { kontra_re })
}

fn run(g: &mut Game, actions: &[Action]) {
    for &a in actions {
        g.apply(a).unwrap_or_else(|e| panic!("{a:?}: {e:?}"));
    }
}

fn hand_game(g: &mut Game, announcement: Announcement) {
    run(g, &VH_AT_18);
    run(g, &[Action::Hand, Action::Announce(announcement)]);
}

/// Plays the first legal action until `seat` has to play a card after at
/// least `tricks` completed tricks.
fn play_until(g: &mut Game, seat: Seat, tricks: usize) {
    loop {
        let current = g.current_seat().expect("game still running");
        if current == seat && g.tricks().len() >= tricks {
            return;
        }
        let first = g.legal_actions()[0];
        g.apply(first).unwrap();
    }
}

fn current_view(g: &Game) -> PlayerView {
    g.view(g.current_seat().expect("a decision is pending"))
}

fn coach(view: &PlayerView, budget: Budget, seed: u64) -> (Analysis, Explanation) {
    let analysis = analyze(view, budget, seed, None).expect("consistent view");
    let explanation = explain(view, &analysis).expect("a decision to explain");
    assert_traceable(view, &analysis, &explanation);
    (analysis, explanation)
}

fn known(e: &Explanation) -> Vec<Observation> {
    e.considerations
        .iter()
        .filter_map(|c| match c {
            Consideration::Known(o) => Some(*o),
            Consideration::Inferred(_) => None,
        })
        .collect()
}

fn inferred(e: &Explanation, holder: Place) -> skat_coach::Inferred {
    e.considerations
        .iter()
        .find_map(|c| match c {
            Consideration::Inferred(i) if i.inference.holder == holder => Some(*i),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no inference for {holder:?}"))
}

/// Every claim traces back to the analysis: facts are its observations,
/// inferences its inferences, estimates its estimates; judgements, reasons
/// and the important mark follow the documented thresholds; estimates are
/// never marked certain unless the analysis is exact.
fn assert_traceable(view: &PlayerView, a: &Analysis, e: &Explanation) {
    assert_eq!(e.seat, view.seat);
    assert_eq!(e.status, a.status);
    assert_eq!(e.assumptions, a.assumptions);
    for c in &e.considerations {
        match c {
            Consideration::Known(o) => assert!(a.observations.contains(o), "{o:?}"),
            Consideration::Inferred(i) => {
                assert!(a.inferences.contains(&i.inference), "{i:?}");
                assert_eq!(i.amount, Amount::of(i.inference.expected));
                if i.inference.certain {
                    assert_eq!(i.inference.min, i.inference.max);
                }
            }
        }
    }
    let mut actions: Vec<Action> = e.options.iter().map(|o| o.action).collect();
    actions.sort_by_key(|a| view.legal_actions.iter().position(|b| b == a));
    assert_eq!(actions, view.legal_actions, "every legal option once");
    let mut last = f64::INFINITY;
    for (rank, o) in e.options.iter().enumerate() {
        let source = a.options.iter().find(|x| x.action == o.action).unwrap();
        assert_eq!(o.estimate, source.estimate);
        if let Some(est) = o.estimate {
            assert_eq!(o.value, Some(utility(view, &est)));
            if !est.exact {
                assert!(est.score_std_error >= 0.0);
            }
        }
        match o.judgement {
            Judgement::Best => assert_eq!(rank, 0),
            Judgement::Weak => assert!(o.gap >= WEAK_POINTS && o.gap > o.margin, "{o:?}"),
            Judgement::Worse => assert!(o.gap > o.margin && o.gap < WEAK_POINTS, "{o:?}"),
            Judgement::Equivalent => {
                assert!(o.gap <= o.margin || o.gap <= skat_coach::TIE_POINTS)
            }
            Judgement::Screened | Judgement::NotEvaluated => {}
        }
        if matches!(
            o.judgement,
            Judgement::Best | Judgement::Equivalent | Judgement::Worse | Judgement::Weak
        ) {
            let value = o.value.unwrap();
            assert!(value <= last, "ranked by value");
            last = value;
        }
        let other = o
            .compared_with
            .and_then(|w| a.options.iter().find(|x| x.action == w))
            .and_then(|x| x.estimate);
        for r in &o.reasons {
            let (this, other) = (o.estimate.unwrap(), other.expect("reason has a reference"));
            match *r {
                Reason::WinChance { this: t, other: x } => {
                    assert_eq!((t, x), (this.win_chance, other.win_chance))
                }
                Reason::ThrownIn { this: t, other: x } => {
                    assert_eq!((t, x), (this.thrown_in, other.thrown_in))
                }
                Reason::DeclarerPoints { this: t, other: x } => assert_eq!(
                    (t, x),
                    (this.expected_game_points, other.expected_game_points)
                ),
                Reason::Points { this: t, other: x } => {
                    assert_eq!((t, x), (utility(view, &this), utility(view, &other)))
                }
                Reason::SameValue | Reason::WithinUncertainty => {}
            }
        }
    }
    let equivalent = e
        .options
        .iter()
        .filter(|o| o.judgement == Judgement::Equivalent)
        .count();
    match e.verdict {
        Verdict::Forced => assert_eq!(e.options.len(), 1),
        Verdict::Clear => assert_eq!(equivalent, 0),
        Verdict::Close { equivalent: n } => assert_eq!(usize::from(n), equivalent),
        Verdict::Unknown => assert!(e.options.iter().all(|o| o.estimate.is_none())),
    }
    if e.status != Status::Complete {
        assert!(!e.important);
        assert!(view.legal_actions.iter().all(|&x| e.warning(x).is_none()));
    }
    for &action in &view.legal_actions {
        if let Some(w) = e.warning(action) {
            assert_eq!(w.chosen.judgement, Judgement::Weak);
        }
    }
    assert_eq!(e.evidence, gather(view, a), "evidence from the analysis");
    let compared: Vec<Action> = e.comparisons.iter().map(|c| c.chosen).collect();
    assert_eq!(compared, view.legal_actions);
    for c in &e.comparisons {
        assert_eq!(c.best, e.best().map(|o| o.action), "the coach's best");
        let judgement = e.option(c.chosen).unwrap().judgement;
        let expected = match judgement {
            _ if e.status != Status::Complete => Reliability::Incomplete,
            Judgement::Screened => Reliability::Screened,
            Judgement::NotEvaluated => Reliability::NotEvaluated,
            _ => Reliability::Reliable,
        };
        assert_eq!(c.reliability, expected, "{c:?}");
    }
    assert_no_telling(view, e);
}

/// The hints before the move reveal one stage at a time and never rank or
/// judge the options (D-49); the review after the move judges exactly the
/// chosen option.
fn assert_no_telling(view: &PlayerView, e: &Explanation) {
    let first = hints(e, Stage::Considerations);
    assert_eq!(first.considerations, e.considerations);
    assert!(first.situation.is_none() && first.figures.is_none());
    let second = hints(e, Stage::Situation);
    let situation = second.situation.expect("stage 2 has the situation");
    assert!(second.figures.is_none());
    assert_eq!(situation.options, view.legal_actions, "neutral order");
    assert_eq!(situation.important, e.important);
    match situation.situation {
        Situation::Forced => assert_eq!(e.verdict, Verdict::Forced),
        Situation::Unclear => {
            assert!(e.status != Status::Complete || e.verdict == Verdict::Unknown)
        }
        Situation::OneStandsOut => assert_eq!(e.verdict, Verdict::Clear),
        Situation::SeveralEqual { count } => {
            assert_eq!(
                e.verdict,
                Verdict::Close {
                    equivalent: count - 1
                }
            )
        }
    }
    let figures = hints(e, Stage::Figures)
        .figures
        .expect("stage 3 has figures");
    assert!(
        figures
            .iter()
            .map(|f| f.action)
            .eq(view.legal_actions.iter().copied())
    );
    for f in &figures {
        assert_eq!(f.estimate, e.option(f.action).unwrap().estimate);
    }
    for &action in &view.legal_actions {
        let r = review(e, action).expect("every legal action can be reviewed");
        assert_eq!(r.chosen.action, action);
        let expected = match r.chosen.judgement {
            _ if e.verdict == Verdict::Forced => Note::Forced,
            _ if e.status != Status::Complete => Note::Uncertain,
            Judgement::Best => Note::Good,
            Judgement::Equivalent => Note::AboutEqual,
            Judgement::Worse => Note::SomewhatWeaker,
            Judgement::Weak => Note::ClearlyWeaker,
            Judgement::Screened | Judgement::NotEvaluated => Note::Uncertain,
        };
        assert_eq!(r.note(), expected);
        assert_eq!(r.evidence, e.evidence);
        assert_eq!(
            Some(&r.comparison),
            e.comparisons.iter().find(|c| c.chosen == action)
        );
        // Catalogue lessons are fact lessons on the evidence alone: the
        // same whichever action is chosen, and the same as before the move
        // (D-49: the lesson shown before the move implies no action).
        assert_eq!(r.teaching, teach(&catalogue(), &e.evidence, None));
        assert_eq!(
            r.lesson,
            lesson_content(&r.teaching, &r.evidence, Some(&r.comparison))
        );
    }
    for stage in [Stage::Considerations, Stage::Situation, Stage::Figures] {
        let h = hints(e, stage);
        assert_eq!(h.evidence, e.evidence);
        assert_eq!(h.teaching, teach(&catalogue(), &e.evidence, None));
        assert_eq!(h.lesson, lesson_content(&h.teaching, &h.evidence, None));
    }
    // The Trumpf sets are shown by their lesson only, not as a fact line.
    assert!(
        !known(e)
            .iter()
            .any(|o| matches!(o, Observation::TrumpfSets(_)))
    );
}

#[test]
fn bidding_early_uncertainty() {
    // MH holds no Jack and only Pik and Herz: the Jacks are somewhere among
    // VH, HH and the Skat, which the bidding can only estimate. A bid of
    // 264 is far beyond any game MH could play.
    let g = game(standard_deal(), false);
    let view = current_view(&g);
    let budget = Budget {
        samples: 2,
        ..SMALL
    };
    let (_, e) = coach(&view, budget, 1);
    assert_eq!(
        (e.decision, e.role, e.objective),
        (Decision::Bid, Role::Bidder, Objective::Undecided)
    );
    assert!(known(&e).is_empty(), "no card has been played or bid");
    let mut total = 0.0;
    for holder in [
        Place::Seat(Seat::Vorhand),
        Place::Seat(Seat::Hinterhand),
        Place::Skat,
    ] {
        let i = inferred(&e, holder);
        assert_eq!(i.inference.class, CardClass::Jacks);
        assert!(!i.inference.certain);
        total += i.inference.expected;
    }
    assert!((total - 4.0).abs() < 1e-9);
    assert!(!e.basis.enumerated);
    assert!(e.options.iter().all(|o| !o.estimate.unwrap().exact));
    assert_eq!(
        e.option(Action::Bid(264)).unwrap().judgement,
        Judgement::Weak
    );
    // A jump bid alone does not make the decision important.
    assert!(!skat_coach::natural(&view, Action::Bid(264)));
    assert!(skat_coach::natural(&view, Action::Bid(18)));
}

#[test]
fn answering_holds_or_passes() {
    // MH said 18; VH (all Jacks, six Kreuz) answers with hold or pass.
    let mut g = game(standard_deal(), false);
    run(&mut g, &[Action::Bid(18)]);
    let view = current_view(&g);
    assert_eq!(view.legal_actions, vec![Action::Hold, Action::Pass]);
    let budget = Budget {
        samples: 1,
        ..SMALL
    };
    let (_, e) = coach(&view, budget, 12);
    assert_eq!((e.decision, e.role), (Decision::Answer, Role::Bidder));
    assert!(known(&e).contains(&Observation::BidShown {
        seat: Seat::Mittelhand,
        value: 18
    }));
    // With a sure game, holding is best.
    assert_eq!(e.best().unwrap().action, Action::Hold);
}

#[test]
fn hand_or_pickup_differs_in_points_only() {
    // VH holds all Jacks and six Kreuz: every game VH can sensibly
    // announce is won with or without the Skat, so the options differ in
    // points, not in the win chance.
    let mut g = game(standard_deal(), false);
    run(&mut g, &VH_AT_18);
    let (_, e) = coach(&current_view(&g), SMALL, 2);
    assert_eq!((e.decision, e.role), (Decision::Pickup, Role::Declarer));
    assert_ne!(e.verdict, Verdict::Unknown);
    for o in &e.options {
        assert_eq!(o.estimate.unwrap().win_chance, 1.0);
        assert!(
            o.reasons
                .iter()
                .all(|r| !matches!(r, Reason::WinChance { .. })),
            "{o:?}"
        );
    }
    // VH bid 18 and is declarer; that is VH's own bid, so not listed.
    assert!(known(&e).iter().all(|o| !matches!(
        o,
        Observation::BidShown {
            seat: Seat::Vorhand,
            ..
        }
    )));
}

#[test]
fn every_discard_is_compared_or_marked_screened() {
    // VH holds only low cards: Drücken screens all 66 discards and
    // compares the best six.
    let low = deal(
        "C7 C8 C9 S7 S8 S9 H7 H8 H9 D7",
        "CJ SJ HJ DJ CA CT CK CQ SA ST",
        "SK SQ HA HT HK HQ DA DT DK DQ",
        "D8 D9",
    );
    let mut g = game(low, false);
    run(&mut g, &VH_AT_18);
    run(&mut g, &[Action::TakeSkat]);
    let budget = Budget {
        samples: 5,
        ..SMALL
    };
    let (_, e) = coach(&current_view(&g), budget, 3);
    assert_eq!(e.decision, Decision::Discard);
    let screened = e
        .options
        .iter()
        .filter(|o| o.judgement == Judgement::Screened)
        .count();
    assert_eq!(screened, 60);
    assert_eq!(e.options[0].judgement, Judgement::Best);
}

#[test]
fn overbid_announcement_is_weak_and_important() {
    // VH declarer at 99 playing Hand. VH has no Karo, so a Karo game is
    // worth at most 9 * 10 = 90 < 99: always overbid and lost. Kreuz Hand
    // with ten trumps wins every trick, and so does Grand Hand.
    let mut g = game(standard_deal(), false);
    let mut bids: Vec<Action> = Vec::new();
    for v in skat_core::reizwerte().into_iter().filter(|&v| v <= 99) {
        bids.push(Action::Bid(v));
        bids.push(Action::Hold);
    }
    run(&mut g, &bids);
    run(&mut g, &[Action::Pass, Action::Pass, Action::Hand]);
    let view = current_view(&g);
    let (_, e) = coach(&view, SMALL, 4);
    assert_eq!((e.decision, e.role), (Decision::Announce, Role::Declarer));
    let karo = Action::Announce(Announcement::plain(GameType::Farbe(Suit::Karo)));
    let option = e.option(karo).unwrap();
    assert_eq!(option.judgement, Judgement::Weak);
    assert!(option.reasons.contains(&Reason::WinChance {
        this: 0.0,
        other: 1.0
    }));
    let warning = e.warning(karo).expect("weak plain announcement warns");
    assert_eq!(warning.chosen.action, karo);
    assert!(warning.chosen.gap >= WEAK_POINTS);
    assert!(e.important, "a natural option is weak");

    // The debriefing: choosing the overbid game is a bad moment, avoiding
    // it a good one; the caps hold however many moments there are.
    let bad = review(&e, karo).unwrap();
    assert_eq!(bad.note(), Note::ClearlyWeaker);
    let best = e.best().unwrap().action;
    let good = review(&e, best).unwrap();
    assert_eq!(good.note(), Note::Good);
    assert!(
        good.trap
            .as_ref()
            .is_some_and(|t| t.natural && t.gap >= WEAK_POINTS)
    );
    let found = moments(&[good.clone(), bad.clone()]);
    assert_eq!(
        found.iter().map(|m| m.kind).collect::<Vec<_>>(),
        [MomentKind::Good, MomentKind::Bad],
        "game order"
    );
    // At most three in total: the costliest bad, the best good, then the
    // next bad one; all in game order.
    let many: Vec<_> = (0..5).flat_map(|_| [good.clone(), bad.clone()]).collect();
    let found = moments(&many);
    let count = |found: &[Moment], k| found.iter().filter(|m| m.kind == k).count();
    assert_eq!(found.len(), PRIMARY_MOMENTS);
    assert_eq!(count(&found, MomentKind::Bad), 2);
    assert_eq!(count(&found, MomentKind::Good), 1);
    assert_eq!(
        found.iter().map(|m| m.kind).collect::<Vec<_>>(),
        [MomentKind::Good, MomentKind::Bad, MomentKind::Bad],
        "ties in game order: reviews 0, 1 and 3"
    );
    // Only one kind: up to its own cap.
    let only_bad = moments(&vec![bad.clone(); 5]);
    assert_eq!(only_bad.len(), PRIMARY_MOMENTS.min(BAD_MOMENTS));
    let only_good = moments(&vec![good.clone(); 5]);
    assert_eq!(only_good.len(), GOOD_MOMENTS);
    // A costlier bad moment later in the game wins its slot.
    let mut costly = bad.clone();
    costly.chosen.gap += 1.0;
    let mut reviews = vec![bad.clone(); 3];
    reviews.push(costly.clone());
    let found = moments(&reviews);
    assert_eq!(found.len(), 3);
    assert_eq!(found[2].review, costly, "kept and shown in game order");
    // A small loss gets the note only, no moment.
    let mut small = bad.clone();
    small.chosen.judgement = Judgement::Worse;
    small.chosen.gap = MOMENT_POINTS - 0.5;
    assert_eq!(small.note(), Note::SomewhatWeaker);
    assert!(moments(&[small.clone()]).is_empty());
    small.chosen.gap = MOMENT_POINTS;
    assert_eq!(moments(&[small]).len(), 1);
}

#[test]
fn grand_declarer_late_deductions() {
    // VH plays Grand Hand with all four Jacks: no trump is out from the
    // start. In the last tricks every distribution is enumerated, so the
    // estimates are exact and the defenders' trump count is certain.
    let mut g = game(standard_deal(), false);
    hand_game(&mut g, Announcement::plain(GameType::Grand));
    play_until(&mut g, Seat::Vorhand, 7);
    let budget = Budget {
        samples: 1000,
        ..SMALL
    };
    let (_, e) = coach(&current_view(&g), budget, 5);
    assert_eq!(
        (e.decision, e.role, e.objective),
        (
            Decision::Card,
            Role::Declarer,
            Objective::Augen { target: 61 }
        )
    );
    assert!(known(&e).contains(&Observation::TrumpsOut { count: 0 }));
    assert!(
        known(&e)
            .iter()
            .any(|o| matches!(o, Observation::AugenWon { .. }))
    );
    for seat in [Seat::Mittelhand, Seat::Hinterhand] {
        let i = inferred(&e, Place::Seat(seat));
        assert_eq!(
            (i.inference.class, i.amount),
            (CardClass::Trumps, Amount::Hardly)
        );
        assert!(i.inference.certain && i.inference.max == 0);
    }
    assert!(e.basis.enumerated);
    for o in &e.options {
        assert!(o.estimate.unwrap().exact);
        assert!(!o.reasons.contains(&Reason::WithinUncertainty));
    }
}

#[test]
fn grand_defender_early_estimates() {
    // MH defends against Grand Hand after VH led the Kreuz Jack: three
    // trumps are out and MH holds none; where they are is only estimated.
    let mut g = game(standard_deal(), false);
    hand_game(&mut g, Announcement::plain(GameType::Grand));
    run(&mut g, &[Action::Play(c("CJ"))]);
    let view = current_view(&g);
    assert_eq!(view.seat, Seat::Mittelhand);
    let (_, e) = coach(&view, SMALL, 6);
    assert_eq!(
        (e.decision, e.role, e.objective),
        (
            Decision::Card,
            Role::Defender,
            Objective::Augen { target: 61 }
        )
    );
    assert!(known(&e).contains(&Observation::TrumpsOut { count: 3 }));
    assert!(known(&e).contains(&Observation::BidShown {
        seat: Seat::Vorhand,
        value: 18
    }));
    let total: f64 = [
        Place::Seat(Seat::Vorhand),
        Place::Seat(Seat::Hinterhand),
        Place::Skat,
    ]
    .iter()
    .map(|&h| inferred(&e, h).inference.expected)
    .sum();
    assert!((total - 3.0).abs() < 1e-9);
    assert!(!inferred(&e, Place::Seat(Seat::Vorhand)).inference.certain);
    assert!(!e.basis.enumerated);
}

#[test]
fn suit_game_defender_sees_proven_void() {
    // Kreuz Hand by VH. VH leads the Kreuz Jack; neither MH nor HH holds a
    // trump, so HH's discard proves HH void in trumps: from MH's view a
    // known fact, and HH's trump count is certain (zero).
    let mut g = game(standard_deal(), false);
    hand_game(&mut g, Announcement::plain(GameType::Farbe(Suit::Kreuz)));
    play_until(&mut g, Seat::Mittelhand, 1);
    let (_, e) = coach(&current_view(&g), SMALL, 7);
    assert_eq!(
        (e.role, e.objective),
        (Role::Defender, Objective::Augen { target: 61 })
    );
    assert!(known(&e).contains(&Observation::Void {
        seat: Seat::Hinterhand,
        follow: Follow::Trumpf
    }));
    let hh = inferred(&e, Place::Seat(Seat::Hinterhand));
    assert!(hh.inference.certain && hh.inference.max == 0);
    assert!(!inferred(&e, Place::Seat(Seat::Vorhand)).inference.certain);
}

#[test]
fn null_objective_without_trumps() {
    // VH plays Null Hand: no trumps to count; the goal is no trick.
    let mut g = game(standard_deal(), false);
    hand_game(&mut g, Announcement::plain(GameType::Null));
    let (_, e) = coach(&current_view(&g), SMALL, 8);
    assert_eq!(
        (e.decision, e.role, e.objective),
        (Decision::Card, Role::Declarer, Objective::NoTrick)
    );
    assert!(
        e.considerations
            .iter()
            .all(|c| !matches!(c, Consideration::Inferred(_)))
    );
    assert!(
        known(&e)
            .iter()
            .all(|o| !matches!(o, Observation::TrumpsOut { .. }))
    );
}

#[test]
fn kontra_changes_only_the_declarer_points() {
    // Grand Ouvert Hand by VH. Kontra multiplies only the declarer's
    // points, so the two options differ in declarer points and never in
    // the win chance.
    let mut g = game(standard_deal(), true);
    run(&mut g, &VH_AT_18);
    let ouvert = Announcement {
        game: GameType::Grand,
        schneider: true,
        schwarz: true,
        ouvert: true,
    };
    run(&mut g, &[Action::Hand, Action::Announce(ouvert)]);
    let view = current_view(&g);
    let (_, e) = coach(&view, SMALL, 9);
    assert_eq!(
        (e.decision, e.role, e.objective),
        (Decision::Kontra, Role::Defender, Objective::AllTricks)
    );
    for o in &e.options {
        assert!(
            o.reasons
                .iter()
                .any(|r| matches!(r, Reason::DeclarerPoints { .. })),
            "{o:?}"
        );
        assert!(
            o.reasons
                .iter()
                .all(|r| !matches!(r, Reason::WinChance { .. })),
            "{o:?}"
        );
    }
    assert!(
        known(&e)
            .iter()
            .any(|o| matches!(o, Observation::KontraEligible { .. }))
    );
}

#[test]
fn re_on_a_sure_game_is_best() {
    // Grand Hand by VH with all Jacks and six Kreuz wins every trick, so Re
    // doubles a sure win: KeinRe gives away points.
    let mut g = game(standard_deal(), true);
    hand_game(&mut g, Announcement::plain(GameType::Grand));
    run(&mut g, &[Action::Play(c("CJ")), Action::Kontra]);
    let (_, e) = coach(&current_view(&g), SMALL, 10);
    assert_eq!((e.decision, e.role), (Decision::Re, Role::Declarer));
    assert_eq!(e.best().unwrap().action, Action::Re);
    assert_eq!(e.verdict, Verdict::Clear);
    assert_eq!(e.option(Action::KeinRe).unwrap().judgement, Judgement::Weak);
}

#[test]
fn incomplete_analysis_backs_no_warning() {
    let mut g = game(standard_deal(), true);
    run(&mut g, &VH_AT_18);
    let ouvert = Announcement {
        game: GameType::Grand,
        schneider: true,
        schwarz: true,
        ouvert: true,
    };
    run(&mut g, &[Action::Hand, Action::Announce(ouvert)]);
    let view = current_view(&g);
    let cancel = Cancel::new();
    cancel.cancel();
    let a = analyze(&view, SMALL, 9, Some(&cancel)).unwrap();
    assert_eq!(a.status, Status::Cancelled);
    let e = explain(&view, &a).unwrap();
    assert_traceable(&view, &a, &e);
    assert!(!e.complete() && !e.important);
    assert!(e.warning(Action::Kontra).is_none());

    let tiny = Budget {
        samples: 4,
        nodes_per_task: 10,
    };
    let a = analyze(&view, tiny, 9, None).unwrap();
    assert_eq!(a.status, Status::BudgetExhausted);
    let e = explain(&view, &a).unwrap();
    assert_traceable(&view, &a, &e);
    assert!(e.warning(Action::Kontra).is_none());
}

#[test]
fn explanation_needs_the_matching_analysis() {
    let g = game(standard_deal(), false);
    let view = current_view(&g);
    let a = analyze(&view, SMALL, 1, None).unwrap();
    let other = g.view(Seat::Vorhand);
    assert!(explain(&other, &a).is_none(), "not VH's turn");
    let mut moved = g.clone();
    moved.apply(Action::Bid(18)).unwrap();
    assert!(
        explain(&current_view(&moved), &a).is_none(),
        "another decision"
    );
}

/// Explanation of `seat` at the end of `actions` in each deal, with the
/// same seed and budget.
fn explanations(
    deals: [Deal; 2],
    kontra_re: bool,
    actions: &[Action],
    seat: Seat,
) -> [Explanation; 2] {
    deals.map(|d| {
        let mut g = game(d, kontra_re);
        run(&mut g, actions);
        let view = g.view(seat);
        let seed = decision_seed(77, &view);
        let budget = Budget {
            samples: 2,
            ..SMALL
        };
        coach(&view, budget, seed).1
    })
}

#[test]
fn concealed_cards_do_not_change_the_advice() {
    // MH's cards and the public history are the same; VH, HH and the Skat
    // differ.
    let other = deal(
        "CJ SJ HJ DJ CA CT CK CQ DA DT",
        "SA ST SK SQ S9 S8 S7 HA HT HK",
        "HQ H9 H8 H7 C9 C8 DK DQ D9 D8",
        "C7 D7",
    );
    let mut actions = VH_AT_18.to_vec();
    actions.extend([
        Action::Hand,
        Action::Announce(Announcement::plain(GameType::Grand)),
        Action::Play(c("CJ")),
    ]);
    let [a, b] = explanations([standard_deal(), other], false, &actions, Seat::Mittelhand);
    assert_eq!(a, b);
}

#[test]
fn known_skat_and_ouvert_cards_follow_the_visibility_rules() {
    // Declarer after Drücken knows the Skat; the defenders' hands differ.
    let a = standard_deal();
    let b = deal(
        "CJ SJ HJ DJ CA CT CK CQ C9 C8",
        "SA ST SK SQ S9 S8 S7 HA HT DK",
        "HQ H9 H8 H7 DA DT HK DQ D9 D8",
        "C7 D7",
    );
    let mut actions = VH_AT_18.to_vec();
    actions.extend([
        Action::TakeSkat,
        Action::Discard(cards("C7 D7")),
        Action::Announce(Announcement::plain(GameType::Grand)),
    ]);
    let [x, y] = explanations([a, b], false, &actions, Seat::Vorhand);
    assert_eq!(x, y);
    assert!(
        known(&x)
            .iter()
            .any(|o| matches!(o, Observation::SkatKnown { .. }))
    );

    // A defender against Ouvert sees the declarer's cards; the other
    // defender's hand and the Skat differ.
    let a = standard_deal();
    let b = deal(
        "CJ SJ HJ DJ CA CT CK CQ C9 C8",
        "SA ST SK SQ S9 S8 S7 HA HT HK",
        "HQ H9 H8 C7 DA DT DK DQ D9 D8",
        "H7 D7",
    );
    let mut actions = VH_AT_18.to_vec();
    actions.extend([
        Action::Hand,
        Action::Announce(Announcement {
            game: GameType::Grand,
            schneider: true,
            schwarz: true,
            ouvert: true,
        }),
    ]);
    let [x, y] = explanations([a, b], true, &actions, Seat::Mittelhand);
    assert_eq!(x, y);
}

#[test]
fn same_inputs_same_explanation() {
    let mut g = game(standard_deal(), false);
    hand_game(&mut g, Announcement::plain(GameType::Farbe(Suit::Kreuz)));
    play_until(&mut g, Seat::Mittelhand, 1);
    let view = current_view(&g);
    let seed = decision_seed(5, &view);
    assert_eq!(seed, decision_seed(5, &view));
    assert_ne!(seed, decision_seed(6, &view));
    assert_eq!(coach(&view, SMALL, seed).1, coach(&view, SMALL, seed).1);
}

fn test_lesson(kind: LessonKind, permitted: &[EvidenceClass]) -> Lesson {
    Lesson {
        id: LessonId {
            key: "test.void".into(),
            version: 1,
        },
        kind,
        support: Support::Supported,
        applies: Applicability::default(),
        permitted: permitted.to_vec(),
        requires: vec![Condition::Fact(FactKind::Void)],
        excludes: Vec::new(),
    }
}

/// A fact lesson on proven voids, and a mechanism resting on them.
fn test_lessons() -> [Lesson; 2] {
    [
        test_lesson(LessonKind::Fact, &[EvidenceClass::Exact]),
        test_lesson(
            LessonKind::Mechanism,
            &[
                EvidenceClass::Exact,
                EvidenceClass::Comparison,
                EvidenceClass::Mechanism,
            ],
        ),
    ]
}

/// VH (Grand Hand) to lead trick 2 after CJ, SA, HQ: both defenders are
/// proven void in Trumpf.
fn after_voids() -> PlayerView {
    let mut g = game(standard_deal(), false);
    hand_game(&mut g, Announcement::plain(GameType::Grand));
    run(&mut g, &["CJ", "SA", "HQ"].map(|s| Action::Play(c(s))));
    current_view(&g)
}

#[test]
fn unreliable_comparisons_back_no_mechanism() {
    let [fact, mechanism] = test_lessons();
    let mechanism_only = [mechanism.clone()];
    let view = after_voids();
    // Complete: the mechanism may back the review of every option.
    let (_, e) = coach(&view, SMALL, 3);
    assert!(e.complete());
    let r = review(&e, view.legal_actions[0]).unwrap();
    assert_eq!(r.comparison.reliability, Reliability::Reliable);
    assert!(matches!(
        teach(&mechanism_only, &r.evidence, Some(&r.comparison)),
        Teaching::Lesson { id, .. } if id == mechanism.id
    ));
    // Cancelled and out of budget: the exact fact lesson still holds, the
    // mechanism falls back.
    let cancel = Cancel::new();
    cancel.cancel();
    let tiny = Budget {
        samples: 4,
        nodes_per_task: 10,
    };
    let analyses = [
        analyze(&view, SMALL, 3, Some(&cancel)).unwrap(),
        analyze(&view, tiny, 3, None).unwrap(),
    ];
    for a in &analyses {
        assert_ne!(a.status, Status::Complete);
        let e = explain(&view, a).unwrap();
        assert_traceable(&view, a, &e);
        for &action in &view.legal_actions {
            let r = review(&e, action).unwrap();
            assert_eq!(r.comparison.reliability, Reliability::Incomplete);
            assert_eq!(
                teach(&mechanism_only, &r.evidence, Some(&r.comparison)),
                Teaching::Fallback
            );
            assert!(matches!(
                teach(std::slice::from_ref(&fact), &r.evidence, Some(&r.comparison)),
                Teaching::Lesson { id, .. } if id == fact.id
            ));
        }
    }
    // Absent evidence: before the first card nobody is proven void.
    let mut g = game(standard_deal(), false);
    hand_game(&mut g, Announcement::plain(GameType::Grand));
    let view = current_view(&g);
    let (_, e) = coach(&view, SMALL, 3);
    let r = review(&e, view.legal_actions[0]).unwrap();
    assert_eq!(
        teach(&test_lessons(), &r.evidence, Some(&r.comparison)),
        Teaching::Fallback
    );
}

#[test]
fn screened_discards_back_no_mechanism() {
    let low = deal(
        "C7 C8 C9 S7 S8 S9 H7 H8 H9 D7",
        "CJ SJ HJ DJ CA CT CK CQ SA ST",
        "SK SQ HA HT HK HQ DA DT DK DQ",
        "D8 D9",
    );
    let mut g = game(low, false);
    run(&mut g, &VH_AT_18);
    run(&mut g, &[Action::TakeSkat]);
    let view = current_view(&g);
    let budget = Budget {
        samples: 5,
        ..SMALL
    };
    let (_, e) = coach(&view, budget, 3);
    let screened = e
        .options
        .iter()
        .find(|o| o.judgement == Judgement::Screened)
        .unwrap();
    let r = review(&e, screened.action).unwrap();
    assert_eq!(r.comparison.reliability, Reliability::Screened);
    // Even a mechanism without conditions does not apply.
    let unconditional = Lesson {
        requires: Vec::new(),
        ..test_lessons()[1].clone()
    };
    assert_eq!(
        teach(
            std::slice::from_ref(&unconditional),
            &r.evidence,
            Some(&r.comparison)
        ),
        Teaching::Fallback
    );
    let best = review(&e, e.best().unwrap().action).unwrap();
    assert!(matches!(
        teach(&[unconditional], &best.evidence, Some(&best.comparison)),
        Teaching::Lesson { .. }
    ));
}

#[test]
fn hints_name_no_preferred_action_or_plan() {
    // The same decision with opposite option rankings: everything shown
    // before the move except the figures is the same; no mechanism lesson
    // is selected before the move.
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
    let (a, reversed) = (ranked(true), ranked(false));
    let x = explain(&view, &a).unwrap();
    let y = explain(&view, &reversed).unwrap();
    assert_ne!(
        x.best().map(|o| o.action),
        y.best().map(|o| o.action),
        "precondition: another best option"
    );
    let [fact, mechanism] = test_lessons();
    for stage in [Stage::Considerations, Stage::Situation] {
        let (hx, hy) = (hints(&x, stage), hints(&y, stage));
        assert_eq!(hx.evidence, hy.evidence);
        assert_eq!(hx.teaching, hy.teaching);
        assert_eq!(hx.considerations, hy.considerations);
        assert_eq!(
            hx.situation.as_ref().map(|s| &s.options),
            hy.situation.as_ref().map(|s| &s.options)
        );
    }
    for e in [&x, &y] {
        assert_eq!(
            teach(std::slice::from_ref(&mechanism), &e.evidence, None),
            Teaching::Fallback
        );
        assert_eq!(
            teach(&[mechanism.clone(), fact.clone()], &e.evidence, None),
            teach(std::slice::from_ref(&fact), &e.evidence, None),
        );
    }
}
