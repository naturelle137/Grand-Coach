// SPDX-License-Identifier: EUPL-1.2

//! Evidence contract (D-53): fixtures for exact facts, sample summaries,
//! model comparisons, mechanisms and unsupported lessons, provenance, the
//! information boundary and serde round trips.
//!
//! The expected facts and their sources are worked out from the cards and
//! the rules in the comments, independently of the analysis.

mod common;

use common::*;
use skat_analysis::lesson::{
    Applicability, CONTRACT_VERSION, Check, Claim, Comparison, Condition, Evidence, EvidenceClass,
    FactKind, GameKind, Item, Lesson, LessonId, LessonKind, Reliability, Role, Snapshot, Source,
    Support, Unmet, compare, gather,
};
use skat_analysis::sampling::Place;
use skat_analysis::{
    Budget, Cancel, CardClass, Inference, Observation, Status, TrumpfSets, analyze,
};
use skat_core::{
    Action, Announcement, CardSet, Deal, Follow, Game, GameType, Phase, PlayerView, RuleConfig,
    Seat, Suit,
};

/// Small and node-limited: the search may stop early, which must not change
/// exact evidence.
const TINY: Budget = Budget {
    samples: 2,
    nodes_per_task: 20_000,
};

/// Enough to finish, so sample summaries are compared too.
const SWAP: Budget = Budget {
    samples: 3,
    nodes_per_task: 50_000_000,
};

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

fn play(deal: Deal, kontra_re: bool, actions: &[Action]) -> Game {
    let mut g = Game::from_deal(deal, RuleConfig { kontra_re });
    for &a in VH_AT_18.iter().chain(actions) {
        g.apply(a).unwrap_or_else(|e| panic!("{a:?}: {e:?}"));
    }
    g
}

fn grand_hand() -> [Action; 2] {
    [
        Action::Hand,
        Action::Announce(Announcement::plain(GameType::Grand)),
    ]
}

/// Grand Hand by VH; trick 1: VH CJ, MH SA, HH HQ (neither defender holds
/// a Jack, so both fail to follow Trumpf).
fn after_trick_one() -> Vec<Action> {
    let mut actions = grand_hand().to_vec();
    actions.extend(["CJ", "SA", "HQ"].map(|s| Action::Play(c(s))));
    actions
}

fn evidence_of(view: &PlayerView, budget: Budget, seed: u64) -> Evidence {
    gather(
        view,
        &analyze(view, budget, seed, None).expect("consistent view"),
    )
}

fn fact(observation: Observation, sources: &[Source]) -> Item {
    Item {
        claim: Claim::Fact(observation),
        sources: sources.to_vec(),
    }
}

fn facts(e: &Evidence) -> Vec<&Item> {
    e.items
        .iter()
        .filter(|i| i.class() == EvidenceClass::Exact)
        .collect()
}

fn lesson(kind: LessonKind, permitted: &[EvidenceClass], requires: &[Condition]) -> Lesson {
    Lesson {
        id: LessonId {
            key: "test.lesson".into(),
            version: 1,
        },
        kind,
        support: Support::Supported,
        applies: Applicability::default(),
        permitted: permitted.to_vec(),
        requires: requires.to_vec(),
        excludes: Vec::new(),
    }
}

/// A fact lesson on proven voids during card play in Trumpf games.
fn void_lesson() -> Lesson {
    Lesson {
        applies: Applicability {
            phases: vec![Phase::Spiel],
            roles: Vec::new(),
            games: vec![GameKind::Suit, GameKind::Grand],
        },
        ..lesson(
            LessonKind::Fact,
            &[EvidenceClass::Exact],
            &[Condition::Fact(FactKind::Void)],
        )
    }
}

/// A mechanism resting on a proven void and a reliable comparison.
fn mechanism() -> Lesson {
    lesson(
        LessonKind::Mechanism,
        &[
            EvidenceClass::Exact,
            EvidenceClass::Comparison,
            EvidenceClass::Mechanism,
        ],
        &[Condition::Fact(FactKind::Void)],
    )
}

/// Hand-made evidence for a defender during card play.
fn evidence(items: Vec<Item>, status: Status) -> Evidence {
    Evidence {
        version: 1,
        snapshot: Snapshot {
            seat: Seat::Mittelhand,
            phase: Phase::Spiel,
            role: Role::Defender,
            game: Some(GameType::Farbe(Suit::Herz)),
            bids: 4,
            tricks: 3,
            trick_cards: 1,
            own: cards("SA ST SK"),
            own_skat: None,
            revealed: None,
        },
        status,
        items,
    }
}

fn void_item() -> Item {
    fact(
        Observation::Void {
            seat: Seat::Hinterhand,
            follow: Follow::Trumpf,
        },
        &[Source::Trick {
            index: 1,
            seat: Seat::Hinterhand,
            led: c("HA"),
            card: c("SA"),
        }],
    )
}

/// A sampled summary saying "no Trumpf at HH" in every evaluated
/// distribution, without proof.
fn sampled_none() -> Item {
    Item {
        claim: Claim::Sample(Inference {
            holder: Place::Seat(Seat::Hinterhand),
            class: CardClass::Trumps,
            expected: 0.0,
            min: 0,
            max: 0,
            certain: false,
        }),
        sources: vec![Source::Model],
    }
}

fn comparison(reliability: Reliability) -> Comparison {
    Comparison {
        chosen: Action::Play(c("SA")),
        best: Some(Action::Play(c("ST"))),
        reliability,
    }
}

#[test]
fn exact_facts_carry_their_provenance() {
    // VH (declarer) to lead trick 2 after trick 1 (CJ, SA, HQ):
    // - MH and HH did not follow Trumpf in trick 1: both proven void,
    // - Trumpf in Grand are the four Jacks; VH holds SJ HJ DJ and CJ is
    //   played: no trumps out,
    // - VH won 2 + 11 + 3 = 16 Augen, the defenders none,
    // - VH still holds SJ HJ DJ CA CT CK CQ C9 C8 = 34 Augen; 120 - 34 - 16
    //   = 70 Augen are out,
    // - MH bid 18 (call 0), VH held 18 (call 1); Kontra/Re is off.
    let g = play(standard_deal(), false, &after_trick_one());
    let view = g.view(Seat::Vorhand);
    let e = evidence_of(&view, TINY, 1);
    let expected = [
        fact(
            Observation::Void {
                seat: Seat::Mittelhand,
                follow: Follow::Trumpf,
            },
            &[Source::Trick {
                index: 0,
                seat: Seat::Mittelhand,
                led: c("CJ"),
                card: c("SA"),
            }],
        ),
        fact(
            Observation::Void {
                seat: Seat::Hinterhand,
                follow: Follow::Trumpf,
            },
            &[Source::Trick {
                index: 0,
                seat: Seat::Hinterhand,
                led: c("CJ"),
                card: c("HQ"),
            }],
        ),
        fact(
            Observation::TrumpsOut { count: 0 },
            &[Source::OwnHand, Source::Played],
        ),
        // The four Jacks: three own, CJ played (trick 0, VH), none unseen;
        // Hand game, so the Skat is unknown.
        fact(
            Observation::TrumpfSets(TrumpfSets {
                total: cards("CJ SJ HJ DJ"),
                own: cards("SJ HJ DJ"),
                played: cards("CJ"),
                own_skat: CardSet::EMPTY,
                revealed: CardSet::EMPTY,
                unseen: CardSet::EMPTY,
                skat_unknown: true,
            }),
            &[
                Source::OwnHand,
                Source::Played,
                Source::Trick {
                    index: 0,
                    seat: Seat::Vorhand,
                    led: c("CJ"),
                    card: c("CJ"),
                },
            ],
        ),
        fact(
            Observation::AugenWon {
                declarer: 16,
                defenders: 0,
            },
            &[Source::Played],
        ),
        fact(
            Observation::AugenOut { augen: 70 },
            &[Source::OwnHand, Source::Played],
        ),
        fact(
            Observation::BidShown {
                seat: Seat::Mittelhand,
                value: 18,
            },
            &[Source::Bid { index: 0 }],
        ),
        fact(
            Observation::BidShown {
                seat: Seat::Vorhand,
                value: 18,
            },
            &[Source::Bid { index: 1 }],
        ),
    ];
    let got = facts(&e);
    assert_eq!(got.len(), expected.len(), "{got:#?}");
    for item in &expected {
        assert!(got.contains(&item), "missing {item:?}");
    }
    assert_eq!(e.version, CONTRACT_VERSION);
    assert_eq!(
        e.snapshot,
        Snapshot {
            seat: Seat::Vorhand,
            phase: Phase::Spiel,
            role: Role::Declarer,
            game: Some(GameType::Grand),
            bids: 4,
            tricks: 1,
            trick_cards: 0,
            own: cards("SJ HJ DJ CA CT CK CQ C9 C8"),
            own_skat: None,
            revealed: None,
        }
    );
    // Sample summaries come from the model, after the facts.
    let first_sample = e
        .items
        .iter()
        .position(|i| i.class() == EvidenceClass::Sample)
        .expect("inferences in a Grand");
    assert!(
        e.items[first_sample..]
            .iter()
            .all(|i| { i.class() == EvidenceClass::Sample && i.sources == [Source::Model] })
    );
}

#[test]
fn exact_lesson_holds_with_an_incomplete_search() {
    let g = play(standard_deal(), false, &after_trick_one());
    let view = g.view(Seat::Vorhand);
    let tiny = Budget {
        samples: 4,
        nodes_per_task: 10,
    };
    let e = evidence_of(&view, tiny, 1);
    assert_eq!(e.status, Status::BudgetExhausted);
    let voids: Vec<usize> = (0..2).collect();
    assert_eq!(
        void_lesson().check(&e, None),
        Check::Applies { items: voids }
    );
    // Cancelled: the same facts.
    let cancel = Cancel::new();
    cancel.cancel();
    let cancelled = gather(&view, &analyze(&view, tiny, 1, Some(&cancel)).unwrap());
    assert_eq!(cancelled.status, Status::Cancelled);
    assert_eq!(facts(&cancelled), facts(&e));
}

#[test]
fn absent_or_inapplicable_evidence_fails() {
    // Counterexample: before the first card nobody has failed to follow.
    let g = play(standard_deal(), false, &grand_hand());
    let e = evidence_of(&g.view(Seat::Vorhand), TINY, 1);
    assert_eq!(
        void_lesson().check(&e, None),
        Check::Fails(Unmet::Missing(Condition::Fact(FactKind::Void)))
    );
    // Boundary: the same voids, but exact evidence not permitted.
    let g = play(standard_deal(), false, &after_trick_one());
    let e = evidence_of(&g.view(Seat::Vorhand), TINY, 1);
    let blind = Lesson {
        permitted: vec![EvidenceClass::Sample],
        ..void_lesson()
    };
    assert_eq!(
        blind.check(&e, None),
        Check::Fails(Unmet::Missing(Condition::Fact(FactKind::Void)))
    );
    // Applicability: role, phase, game, exclusions, support.
    let defenders_only = Lesson {
        applies: Applicability {
            roles: vec![Role::Defender],
            ..void_lesson().applies
        },
        ..void_lesson()
    };
    assert_eq!(defenders_only.check(&e, None), Check::Fails(Unmet::Role));
    let bidding = Lesson {
        applies: Applicability {
            phases: vec![Phase::Reizen],
            ..Applicability::default()
        },
        ..void_lesson()
    };
    assert_eq!(bidding.check(&e, None), Check::Fails(Unmet::Phase));
    let null = Lesson {
        applies: Applicability {
            games: vec![GameKind::Null],
            ..Applicability::default()
        },
        ..void_lesson()
    };
    assert_eq!(null.check(&e, None), Check::Fails(Unmet::Game));
    let excluded = Lesson {
        excludes: vec![Condition::Fact(FactKind::AugenWon)],
        ..void_lesson()
    };
    assert_eq!(
        excluded.check(&e, None),
        Check::Fails(Unmet::Excluded(Condition::Fact(FactKind::AugenWon)))
    );
    let unsupported = Lesson {
        support: Support::Unsupported,
        ..void_lesson()
    };
    assert_eq!(
        unsupported.check(&e, None),
        Check::Fails(Unmet::Unsupported)
    );
}

#[test]
fn sample_summaries_prove_nothing() {
    // "0 trumps at HH in every sampled distribution" is not a proven void.
    let e = evidence(vec![sampled_none()], Status::Complete);
    assert_eq!(e.items[0].class(), EvidenceClass::Sample);
    let both = lesson(
        LessonKind::Fact,
        &[EvidenceClass::Exact, EvidenceClass::Sample],
        &[Condition::Fact(FactKind::Void)],
    );
    assert_eq!(
        both.check(&e, None),
        Check::Fails(Unmet::Missing(Condition::Fact(FactKind::Void)))
    );
    // It may be cited as a sample, only where samples are permitted.
    let context = lesson(
        LessonKind::Fact,
        &[EvidenceClass::Sample],
        &[Condition::Sample(CardClass::Trumps)],
    );
    assert_eq!(context.check(&e, None), Check::Applies { items: vec![0] });
    let exact_only = Lesson {
        permitted: vec![EvidenceClass::Exact],
        ..context
    };
    assert_eq!(
        exact_only.check(&e, None),
        Check::Fails(Unmet::Missing(Condition::Sample(CardClass::Trumps)))
    );
}

#[test]
fn mechanisms_need_a_reliable_comparison_after_the_move() {
    let e = evidence(vec![void_item(), sampled_none()], Status::Complete);
    let m = mechanism();
    assert_eq!(m.check(&e, None), Check::Fails(Unmet::BeforeMove));
    for r in [
        Reliability::Incomplete,
        Reliability::Screened,
        Reliability::NotEvaluated,
    ] {
        assert_eq!(
            m.check(&e, Some(&comparison(r))),
            Check::Fails(Unmet::Provisional(r))
        );
    }
    assert_eq!(
        m.check(&e, Some(&comparison(Reliability::Reliable))),
        Check::Applies { items: vec![0] }
    );
    // Without the void: no mechanism, whatever the comparison says.
    let bare = evidence(vec![sampled_none()], Status::Complete);
    assert_eq!(
        m.check(&bare, Some(&comparison(Reliability::Reliable))),
        Check::Fails(Unmet::Missing(Condition::Fact(FactKind::Void)))
    );
    // A mechanism must permit option-quality evidence; a fact lesson must
    // not.
    let silent = Lesson {
        permitted: vec![EvidenceClass::Exact],
        ..mechanism()
    };
    assert_eq!(
        silent.check(&e, Some(&comparison(Reliability::Reliable))),
        Check::Fails(Unmet::Invalid)
    );
    let loud = Lesson {
        kind: LessonKind::Fact,
        ..mechanism()
    };
    assert_eq!(loud.check(&e, None), Check::Fails(Unmet::Invalid));
}

#[test]
fn comparisons_are_reliable_only_when_complete_and_unscreened() {
    let g = play(standard_deal(), false, &after_trick_one());
    let view = g.view(Seat::Vorhand);
    let complete = analyze(
        &view,
        Budget {
            samples: 2,
            ..Budget::INTERACTIVE
        },
        1,
        None,
    )
    .unwrap();
    assert_eq!(complete.status, Status::Complete);
    for o in &complete.options {
        let c = compare(&view, &complete, o.action).unwrap();
        assert_eq!(c.reliability, Reliability::Reliable);
        assert!(c.best.is_some());
    }
    let tiny = Budget {
        samples: 4,
        nodes_per_task: 10,
    };
    let incomplete = analyze(&view, tiny, 1, None).unwrap();
    for o in &incomplete.options {
        let c = compare(&view, &incomplete, o.action).unwrap();
        assert_eq!(c.reliability, Reliability::Incomplete);
    }
    assert!(compare(&view, &complete, Action::Pass).is_none());
}

#[test]
fn hidden_cards_do_not_change_the_evidence() {
    // MH to play in trick 2 after VH led CA. B moves C9 from VH to HH and
    // swaps HH's D9 and D8 with the Skat's C7 (the Skat and HH get Kreuz);
    // HH still holds no Jack, so the public play stays legal.
    let b = deal(
        "CJ SJ HJ DJ CA CT CK CQ D9 C8",
        "SA ST SK SQ S9 S8 S7 HA HT HK",
        "HQ H9 H8 H7 DA DT DK DQ C9 C7",
        "D8 D7",
    );
    let mut actions = after_trick_one();
    actions.push(Action::Play(c("CA")));
    assert_same_evidence([standard_deal(), b], false, &actions, Seat::Mittelhand);
}

#[test]
fn own_skat_is_permitted_evidence_for_the_declarer() {
    // VH discarded C7 D7 and knows them; MH and HH exchange HK and DK.
    let b = deal(
        "CJ SJ HJ DJ CA CT CK CQ C9 C8",
        "SA ST SK SQ S9 S8 S7 HA HT DK",
        "HQ H9 H8 H7 DA DT HK DQ D9 D8",
        "C7 D7",
    );
    let actions = [
        Action::TakeSkat,
        Action::Discard(cards("C7 D7")),
        Action::Announce(Announcement::plain(GameType::Grand)),
    ];
    let e = assert_same_evidence([standard_deal(), b], false, &actions, Seat::Vorhand);
    assert_eq!(e.snapshot.own_skat, Some(cards("C7 D7")));
    assert!(e.items.contains(&fact(
        Observation::SkatKnown {
            cards: cards("C7 D7")
        },
        &[Source::OwnSkat]
    )));
    // A defender does not know it.
    let g = play(standard_deal(), false, &actions);
    let defender = gather(
        &g.view(Seat::Mittelhand),
        &analyze(&g.view(Seat::Mittelhand), TINY, 1, None).unwrap(),
    );
    assert_eq!(defender.snapshot.own_skat, None);
    assert!(
        defender
            .items
            .iter()
            .all(|i| i.sources != [Source::OwnSkat])
    );
}

#[test]
fn ouvert_cards_are_permitted_evidence_for_the_defenders() {
    // Grand Ouvert Hand by VH; MH to play after VH led CJ. HH and the Skat
    // exchange H7 and C7: hidden from MH. VH's revealed cards are not.
    let ouvert = Announcement {
        game: GameType::Grand,
        schneider: true,
        schwarz: true,
        ouvert: true,
    };
    let actions = [
        Action::Hand,
        Action::Announce(ouvert),
        Action::Play(c("CJ")),
    ];
    let b = deal(
        "CJ SJ HJ DJ CA CT CK CQ C9 C8",
        "SA ST SK SQ S9 S8 S7 HA HT HK",
        "HQ H9 H8 C7 DA DT DK DQ D9 D8",
        "H7 D7",
    );
    let e = assert_same_evidence([standard_deal(), b], false, &actions, Seat::Mittelhand);
    assert_eq!(
        e.snapshot.revealed,
        Some(cards("SJ HJ DJ CA CT CK CQ C9 C8"))
    );
    // Moving a card between the declarer and HH changes what MH sees.
    let c_ = deal(
        "CJ SJ HJ DJ CA CT CK CQ C9 D8",
        "SA ST SK SQ S9 S8 S7 HA HT HK",
        "HQ H9 H8 H7 DA DT DK DQ D9 C8",
        "C7 D7",
    );
    let seen = [standard_deal(), c_].map(|d| play(d, false, &actions).view(Seat::Mittelhand));
    assert_ne!(seen[0], seen[1]);
    // The declarer's own cards are not "revealed" to itself.
    let g = play(standard_deal(), false, &actions[..2]);
    assert_eq!(Snapshot::of(&g.view(Seat::Vorhand)).revealed, None);
}

/// The views of `seat` in both deals are equal, and so is the evidence of
/// equal analyses (same seed and budget). Returns it.
fn assert_same_evidence(
    deals: [Deal; 2],
    kontra_re: bool,
    actions: &[Action],
    seat: Seat,
) -> Evidence {
    let views = deals.map(|d| play(d, kontra_re, actions).view(seat));
    assert_eq!(views[0], views[1], "precondition: the same information set");
    assert!(!views[0].legal_actions.is_empty(), "a decision of {seat:?}");
    let [a, b] = views.map(|v| evidence_of(&v, SWAP, 7));
    assert_eq!(a.status, Status::Complete);
    assert!(a.items.iter().any(|i| i.class() == EvidenceClass::Sample));
    assert_eq!(a, b);
    a
}

#[cfg(feature = "serde")]
#[test]
fn contract_round_trips_through_serde() {
    fn round_trip<T>(value: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let json = serde_json::to_string(value).unwrap();
        assert_eq!(&serde_json::from_str::<T>(&json).unwrap(), value, "{json}");
    }
    let g = play(standard_deal(), false, &after_trick_one());
    round_trip(&evidence_of(&g.view(Seat::Vorhand), TINY, 1));
    round_trip(&evidence(
        vec![void_item(), sampled_none()],
        Status::Cancelled,
    ));
    round_trip(&comparison(Reliability::Screened));
    round_trip(&mechanism());
    round_trip(&Lesson {
        support: Support::Unsupported,
        excludes: vec![Condition::Revealed, Condition::OwnSkat, Condition::Complete],
        ..void_lesson()
    });
    round_trip(&Check::Applies { items: vec![0, 2] });
    round_trip(&Check::Fails(Unmet::Provisional(Reliability::Incomplete)));
}
