// SPDX-License-Identifier: EUPL-1.2

//! What the coach hands the frontend for the two card-play mechanisms
//! (#44): only after the move, only with a reliable comparison, never
//! because the chosen card scored higher, and identical for the same view
//! with other concealed cards. Expected seats and sets are read off the
//! deals in the comments (derived in `skat-analysis/tests/mechanisms.rs`).

use skat_analysis::Budget;
use skat_coach::{
    Abstich, Certainty, Comparison, LessonContent, LessonId, Overtake, PARTNER_OVERTAKE,
    Reliability, Stage, TRUMPF_VULNERABILITY, Teaching, advise, hints, lesson_content, review,
    teach,
};
use skat_core::{
    Action, Announcement, Card, CardSet, Deal, Follow, Game, GameType, PlayerView, Rank,
    RuleConfig, Seat, Suit,
};

const SMALL: Budget = Budget {
    samples: 3,
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

fn deal(vh: &str, mh: &str, hh: &str, skat: &str) -> Deal {
    Deal::new([cards(vh), cards(mh), cards(hh)], cards(skat)).expect("valid test deal")
}

/// Kreuz Hand by VH at 18, then the given cards.
fn kreuz(deal: Deal, play: &[&str]) -> PlayerView {
    let mut g = Game::from_deal(deal, RuleConfig::default());
    let mut actions = vec![
        Action::Bid(18),
        Action::Hold,
        Action::Pass,
        Action::Pass,
        Action::Hand,
        Action::Announce(Announcement::plain(GameType::Farbe(Suit::Kreuz))),
    ];
    actions.extend(play.iter().map(|s| Action::Play(c(s))));
    for a in actions {
        g.apply(a).unwrap_or_else(|e| panic!("{a:?}: {e:?}"));
    }
    g.view(g.current_seat().expect("a decision"))
}

/// MH holds one Trumpf (C9) and no Pik, HH no Trumpf, C8 C7 in the Skat.
fn kreuz_low() -> Deal {
    deal(
        "CJ SJ HJ DJ CA CT CK CQ SA SK",
        "C9 HA HT HK HQ H9 DA DK DQ D7",
        "ST SQ S9 S8 S7 H8 H7 DT D9 D8",
        "C8 C7",
    )
}

/// The same view for VH: MH holds C8 C7, the Skat HQ H9.
fn kreuz_low_swapped() -> Deal {
    deal(
        "CJ SJ HJ DJ CA CT CK CQ SA SK",
        "C9 C8 C7 HA HT HK DA DK DQ D7",
        "ST SQ S9 S8 S7 H8 H7 DT D9 D8",
        "HQ H9",
    )
}

/// The same view for MH in trick 2: VH and HH swap the unseen SA and H8
/// (both still play the same cards legally).
fn kreuz_low_swapped_for_mh() -> Deal {
    deal(
        "CJ SJ HJ DJ CA CT CK CQ H8 SK",
        "C9 HA HT HK HQ H9 DA DK DQ D7",
        "ST SQ SA S8 S7 S9 H7 DT D9 D8",
        "C8 C7",
    )
}

/// Trick 1: CJ C9 D8 (HH no Trumpf); trick 2: SK D7 S7 (MH no Pik).
const PLAY: [&str; 6] = ["CJ", "C9", "D8", "SK", "D7", "S7"];

fn mechanism_key(t: &Teaching) -> Option<&str> {
    match t {
        Teaching::Lesson { id, .. }
            if id.key == TRUMPF_VULNERABILITY || id.key == PARTNER_OVERTAKE =>
        {
            Some(id.key.as_ref())
        }
        _ => None,
    }
}

#[test]
fn the_review_carries_the_abstich_with_its_condition() {
    // VH leads trick 3 (Hand, Skat unknown): MH has no Pik (proven), HH no
    // Trumpf; the two unseen Trumpf C8 C7 may be MH's or in the Skat.
    let view = kreuz(kreuz_low(), &PLAY);
    let e = advise(&view, SMALL, 1, None).expect("explained");
    let r = review(&e, Action::Play(c("SA"))).unwrap();
    assert_eq!(mechanism_key(&r.teaching), Some(TRUMPF_VULNERABILITY));
    assert_eq!(
        r.lesson,
        Some(LessonContent::TrumpfVulnerability {
            game: GameType::Farbe(Suit::Kreuz),
            threats: vec![Abstich {
                seat: Seat::Mittelhand,
                card: c("SA"),
                certainty: Certainty::Possible,
                suit_unseen: cards("ST SQ S9 S8"),
                trumpf_unseen: cards("C8 C7"),
            }],
        })
    );
    // A Trumpf lead has no mechanism: the fact lessons or the fallback.
    let r = review(&e, Action::Play(c("SJ"))).unwrap();
    assert_eq!(mechanism_key(&r.teaching), None);
}

#[test]
fn the_review_carries_the_partner_overtake() {
    // MH second after VH's SK: HH certainly holds SA or ST (counting).
    let view = kreuz(kreuz_low(), &PLAY[..4]);
    assert_eq!(view.seat, Seat::Mittelhand);
    let e = advise(&view, SMALL, 1, None).expect("explained");
    let r = review(&e, Action::Play(c("D7"))).unwrap();
    assert_eq!(mechanism_key(&r.teaching), Some(PARTNER_OVERTAKE));
    assert_eq!(
        r.lesson,
        Some(LessonContent::PartnerOvertake {
            game: GameType::Farbe(Suit::Kreuz),
            overtake: Overtake {
                partner: Seat::Hinterhand,
                declarer: Seat::Vorhand,
                best: c("SK"),
                led: Follow::Farbe(Suit::Pik),
                card: c("D7"),
                certainty: Certainty::Certain,
                higher: cards("SA ST"),
                trumping: false,
            },
        })
    );
}

#[test]
fn hints_never_carry_a_mechanism() {
    for view in [kreuz(kreuz_low(), &PLAY), kreuz(kreuz_low(), &PLAY[..4])] {
        let e = advise(&view, SMALL, 1, None).expect("explained");
        for stage in [Stage::Considerations, Stage::Situation, Stage::Figures] {
            let h = hints(&e, stage);
            assert_eq!(mechanism_key(&h.teaching), None);
            assert!(!matches!(
                h.lesson,
                Some(LessonContent::TrumpfVulnerability { .. })
                    | Some(LessonContent::PartnerOvertake { .. })
            ));
        }
    }
}

#[test]
fn the_content_does_not_depend_on_the_judgement() {
    // Every card of MH leaves SK winning: each review teaches the same
    // overtake, whether the model rates the card best or weaker.
    let view = kreuz(kreuz_low(), &PLAY[..4]);
    let e = advise(&view, SMALL, 1, None).expect("explained");
    for &action in &view.legal_actions {
        let r = review(&e, action).unwrap();
        let Some(LessonContent::PartnerOvertake { overtake, .. }) = r.lesson else {
            panic!("{action:?}: {:?}", r.teaching);
        };
        assert_eq!(Action::Play(overtake.card), action);
        assert_eq!(overtake.certainty, Certainty::Certain);
        assert_eq!(overtake.higher, cards("SA ST"));
    }
}

#[test]
fn a_higher_utility_alone_triggers_no_mechanism() {
    // The comparison of SA, made reliable, with SA as the best option but
    // the mechanisms removed: no mechanism lesson, so no causal text.
    let view = kreuz(kreuz_low(), &PLAY);
    let e = advise(&view, SMALL, 1, None).expect("explained");
    let sa = e
        .comparisons
        .iter()
        .find(|x| x.chosen == Action::Play(c("SA")))
        .unwrap()
        .clone();
    assert!(!sa.mechanisms.is_empty());
    let best_without = Comparison {
        best: Some(sa.chosen),
        reliability: Reliability::Reliable,
        mechanisms: Vec::new(),
        ..sa.clone()
    };
    let lessons = skat_analysis::lesson::catalogue();
    let t = teach(&lessons, &e.evidence, Some(&best_without));
    assert_eq!(mechanism_key(&t), None);
    // The other way round: the mechanism with an unreliable comparison is
    // not taught, and its content is withheld.
    let provisional = Comparison {
        reliability: Reliability::Incomplete,
        ..sa.clone()
    };
    assert_eq!(
        mechanism_key(&teach(&lessons, &e.evidence, Some(&provisional))),
        None
    );
    let forced = Teaching::Lesson {
        id: LessonId {
            key: TRUMPF_VULNERABILITY.into(),
            version: 1,
        },
        items: Vec::new(),
    };
    assert_eq!(
        lesson_content(&forced, &e.evidence, Some(&provisional)),
        None
    );
    assert_eq!(lesson_content(&forced, &e.evidence, None), None);
    assert!(lesson_content(&forced, &e.evidence, Some(&sa)).is_some());
}

#[test]
fn an_incomplete_analysis_teaches_no_mechanism() {
    let view = kreuz(kreuz_low(), &PLAY);
    let tiny = Budget {
        samples: 2,
        nodes_per_task: 10,
    };
    let e = advise(&view, tiny, 1, None).expect("explained");
    assert!(!e.complete());
    let r = review(&e, Action::Play(c("SA"))).unwrap();
    assert_eq!(mechanism_key(&r.teaching), None);
    assert!(!matches!(
        r.lesson,
        Some(LessonContent::TrumpfVulnerability { .. })
    ));
}

#[test]
fn same_view_other_concealed_cards_same_review() {
    for (a, b, play) in [
        (kreuz_low(), kreuz_low_swapped(), &PLAY[..]),
        (kreuz_low(), kreuz_low_swapped_for_mh(), &PLAY[..4]),
    ] {
        let va = kreuz(a, play);
        let vb = kreuz(b, play);
        assert_eq!(va, vb, "the swap must not change the view");
        let ea = advise(&va, SMALL, 7, None).unwrap();
        let eb = advise(&vb, SMALL, 7, None).unwrap();
        assert_eq!(ea, eb, "the whole explanation");
        for &action in &va.legal_actions {
            let ra = review(&ea, action).unwrap();
            let rb = review(&eb, action).unwrap();
            assert_eq!(ra.teaching, rb.teaching, "{action:?}");
            assert_eq!(ra.lesson, rb.lesson, "{action:?}");
            assert_eq!(ra.comparison, rb.comparison, "{action:?}");
        }
    }
}

#[test]
fn the_same_view_and_seed_give_the_same_mechanism_lesson() {
    let view = kreuz(kreuz_low(), &PLAY);
    let a = advise(&view, SMALL, 3, None).unwrap();
    let b = advise(&view, SMALL, 3, None).unwrap();
    let ra = review(&a, Action::Play(c("SA"))).unwrap();
    let rb = review(&b, Action::Play(c("SA"))).unwrap();
    assert_eq!(ra.teaching, rb.teaching);
    assert_eq!(ra.lesson, rb.lesson);
}
