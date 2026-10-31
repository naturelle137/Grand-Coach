// SPDX-License-Identifier: EUPL-1.2

//! What the coach hands the frontend for the two exact lessons (#43):
//! `lesson_content` copies the cited facts and their trick evidence, and
//! falls back to no lesson when the evidence is missing. The expected
//! cards and tricks are read off the deals in the comments.

use skat_analysis::Budget;
use skat_coach::{
    Claim, LessonContent, LessonId, PROVEN_VOID, ProvenVoid, REMAINING_TRUMPF, Stage, Teaching,
    advise, hints, lesson_content, review,
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

/// Herz Hand by VH at 18; then the given cards.
fn herz(deal: Deal, play: &[&str]) -> PlayerView {
    let mut g = Game::from_deal(deal, RuleConfig::default());
    let mut actions = vec![
        Action::Bid(18),
        Action::Hold,
        Action::Pass,
        Action::Pass,
        Action::Hand,
        Action::Announce(Announcement::plain(GameType::Farbe(Suit::Herz))),
    ];
    actions.extend(play.iter().map(|s| Action::Play(c(s))));
    for a in actions {
        g.apply(a).unwrap_or_else(|e| panic!("{a:?}: {e:?}"));
    }
    g.view(g.current_seat().expect("a decision"))
}

fn herz_deal() -> Deal {
    deal(
        "CJ SJ HA HT HK HQ H9 SA ST CA",
        "DJ H8 H7 SK SQ S9 CK CQ C9 DA",
        "HJ S8 S7 CT C8 C7 DT DK DQ D9",
        "D8 D7",
    )
}

#[test]
fn hints_and_review_carry_the_void_with_its_trick() {
    // VH to lead trick 3. Trick 1: CJ H7 HJ; trick 2: HA H8 S7 – HH
    // played S7 on the Trumpf HA: void in Trumpf, proven in trick 2.
    let view = herz(herz_deal(), &["CJ", "H7", "HJ", "HA", "H8", "S7"]);
    let e = advise(&view, SMALL, 1, None).expect("explained");
    let expected = LessonContent::ProvenVoid {
        game: GameType::Farbe(Suit::Herz),
        voids: vec![ProvenVoid {
            seat: Seat::Hinterhand,
            follow: Follow::Trumpf,
            trick: 2,
            led: c("HA"),
            card: c("S7"),
        }],
    };
    let h = hints(&e, Stage::Considerations);
    assert_eq!(h.lesson.as_ref(), Some(&expected));
    for &action in &view.legal_actions {
        let r = review(&e, action).unwrap();
        assert_eq!(r.lesson.as_ref(), Some(&expected), "{action:?}");
    }
}

#[test]
fn remaining_trumpf_content_assigns_no_seat() {
    // HH to play after HA, H8 (trick 2). 11 Trumpf: own none, played CJ
    // H7 HJ HA H8, unseen SJ DJ HT HK HQ H9, Skat unknown. Nobody failed to
    // follow, so the Trumpf lesson is taught.
    let view = herz(herz_deal(), &["CJ", "H7", "HJ", "HA", "H8"]);
    assert_eq!(view.seat, Seat::Hinterhand);
    let e = advise(&view, SMALL, 1, None).expect("explained");
    let h = hints(&e, Stage::Considerations);
    let Some(LessonContent::RemainingTrumpf { game, sets }) = h.lesson else {
        panic!("{:?}", h.teaching);
    };
    assert_eq!(game, GameType::Farbe(Suit::Herz));
    assert_eq!(sets.played, cards("CJ H7 HJ HA H8"));
    assert_eq!(sets.unseen, cards("SJ DJ HT HK HQ H9"));
    assert!(sets.skat_unknown);
    assert!(matches!(&h.teaching, Teaching::Lesson { id, .. } if id.key == REMAINING_TRUMPF));
}

#[test]
fn missing_evidence_falls_back_to_no_lesson() {
    let view = herz(herz_deal(), &["CJ", "H7", "HJ", "HA", "H8", "S7"]);
    let e = advise(&view, SMALL, 1, None).expect("explained");
    assert_eq!(lesson_content(&Teaching::Fallback, &e.evidence), None);
    let lesson = |key: &'static str, items: Vec<usize>| Teaching::Lesson {
        id: LessonId {
            key: key.into(),
            version: 1,
        },
        items,
    };
    // Cited items that are absent or of another kind give no content.
    assert_eq!(
        lesson_content(&lesson(PROVEN_VOID, vec![]), &e.evidence),
        None
    );
    assert_eq!(
        lesson_content(&lesson(PROVEN_VOID, vec![999]), &e.evidence),
        None
    );
    let not_void = e
        .evidence
        .items
        .iter()
        .position(|i| !matches!(i.claim, Claim::Fact(skat_coach::Observation::Void { .. })))
        .unwrap();
    assert_eq!(
        lesson_content(&lesson(PROVEN_VOID, vec![not_void]), &e.evidence),
        None
    );
    assert_eq!(
        lesson_content(&lesson("unknown-lesson", vec![0]), &e.evidence),
        None
    );
    // Before the first card nothing is taught: hints and reviews fall back.
    let first = herz(herz_deal(), &[]);
    let e = advise(&first, SMALL, 1, None).expect("explained");
    assert_eq!(hints(&e, Stage::Situation).lesson, None);
    assert_eq!(hints(&e, Stage::Situation).teaching, Teaching::Fallback);
    let r = review(&e, first.legal_actions[0]).unwrap();
    assert_eq!((r.teaching, r.lesson), (Teaching::Fallback, None));
}

#[test]
fn hidden_cards_do_not_change_the_lesson() {
    // VH to lead trick 2 after CJ H7 HJ. B swaps MH's H8 with the Skat's
    // D8: MH still follows Trumpf with H7, VH sees the same.
    let b = deal(
        "CJ SJ HA HT HK HQ H9 SA ST CA",
        "DJ D8 H7 SK SQ S9 CK CQ C9 DA",
        "HJ S8 S7 CT C8 C7 DT DK DQ D9",
        "H8 D7",
    );
    let views = [herz_deal(), b].map(|d| herz(d, &["CJ", "H7", "HJ"]));
    assert_eq!(views[0], views[1]);
    let [a, b] = views
        .clone()
        .map(|v| advise(&v, SMALL, 5, None).expect("explained"));
    let [ha, hb] = [&a, &b].map(|e| hints(e, Stage::Considerations));
    assert_eq!(ha.teaching, hb.teaching);
    assert_eq!(ha.lesson, hb.lesson);
    let Some(LessonContent::RemainingTrumpf { sets, .. }) = ha.lesson else {
        panic!("{:?}", ha.teaching);
    };
    // Own SJ HA HT HK HQ H9; DJ and H8 unseen, with MH or in the Skat.
    assert_eq!(sets.unseen, cards("DJ H8"));
    // Same input, same lesson.
    let again = advise(&views[0], SMALL, 5, None).expect("explained");
    assert_eq!(hints(&again, Stage::Considerations), ha);
}
