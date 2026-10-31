// SPDX-License-Identifier: EUPL-1.2

//! What the coach hands the frontend for the two pre-play lessons (#45):
//! `lesson_content` copies the cited structure and value facts, names no
//! bid, game or discard, and does not depend on the move chosen. The
//! expected values are read off the deals in the comments (rules and
//! derivations: `skat-analysis/tests/preplay.rs`).

use skat_analysis::{Budget, Cancel, analyze};
use skat_coach::{
    Consideration, ContractValue, GAME_VALUE, HAND_STRUCTURE, LessonContent, Observation, Range,
    Reach, SkatState, Stage, Status, Teaching, advise, explain, hints, lesson_content, review,
};
use skat_core::{
    Action, Card, CardSet, Contract, Deal, Game, GameType, PlayerView, Rank, RuleConfig, Seat, Suit,
};

/// Small budget for content checks; exact facts do not depend on it.
const TINY: Budget = Budget {
    samples: 2,
    nodes_per_task: 20_000,
};

/// Enough to finish, so the model has a preferred option.
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

/// MH: CJ HJ and seven Herz Trumpf; Skat DJ C7.
fn deal_d() -> Deal {
    deal(
        "SJ CA CT CK CQ C9 C8 ST SK SQ",
        "CJ HJ HA HT HK H9 H7 SA S7 D7",
        "S9 S8 HQ H8 DA DT DK DQ D9 D8",
        "DJ C7",
    )
}

/// MH: all four Buben and the top side cards.
fn strong() -> Deal {
    deal(
        "CK CQ C9 C8 C7 SK SQ S9 S8 S7",
        "CJ SJ HJ DJ CA CT SA ST HA HT",
        "HK HQ H9 H8 H7 DA DT DK DQ D9",
        "D8 D7",
    )
}

/// MH: no Bube, no Ass, no Zehn, only Damen high.
fn weak() -> Deal {
    deal(
        "CJ SJ HJ DJ CA CT SA ST HA HT",
        "SQ S9 S8 HK H9 H8 DQ D9 D8 CQ",
        "CK C9 C8 C7 SK S7 H7 DA DT DK",
        "HQ D7",
    )
}

fn view(deal: Deal, actions: &[Action]) -> PlayerView {
    let mut g = Game::from_deal(deal, RuleConfig::default());
    for &a in actions {
        g.apply(a).unwrap_or_else(|e| panic!("{a:?}: {e:?}"));
    }
    g.view(g.current_seat().expect("a decision"))
}

fn key(t: &Teaching) -> Option<&str> {
    match t {
        Teaching::Lesson { id, .. } => Some(id.key.as_ref()),
        Teaching::Fallback => None,
    }
}

#[test]
fn opening_call_hands_over_the_structure_of_every_candidate_game() {
    let v = view(deal_d(), &[]);
    let e = advise(&v, TINY, 1, None).expect("a decision");
    let h = hints(&e, Stage::Considerations);
    assert_eq!(key(&h.teaching), Some(HAND_STRUCTURE));
    let Some(LessonContent::HandStructure { games }) = &h.lesson else {
        panic!("{:?}", h.lesson);
    };
    // Kreuz, Pik, Herz, Karo, Grand: all candidates, none singled out.
    let order: Vec<GameType> = games.iter().map(|s| s.game).collect();
    assert_eq!(
        order,
        [
            GameType::Farbe(Suit::Kreuz),
            GameType::Farbe(Suit::Pik),
            GameType::Farbe(Suit::Herz),
            GameType::Farbe(Suit::Karo),
            GameType::Grand
        ]
    );
    // Herz: CJ HJ + five Herz, mit 1, 1..7 with the unknown Skat.
    assert_eq!(games[2].trumpf, cards("CJ HJ HA HT HK H9 H7"));
    assert_eq!(games[2].counts, Range { low: 1, high: 7 });
    assert!(games.iter().all(|s| s.skat == SkatState::Unknown));
    // The structure facts are not repeated as considerations.
    assert!(h.considerations.iter().all(|c| !matches!(
        c,
        Consideration::Known(
            Observation::Structure(_) | Observation::GameValues(_) | Observation::Commitment { .. }
        )
    )));
}

#[test]
fn a_later_call_hands_over_the_values_against_the_next_bid() {
    // MH 18, VH holds: MH to bid again, at least 20.
    let v = view(deal_d(), &[Action::Bid(18), Action::Hold]);
    let e = advise(&v, TINY, 1, None).expect("a decision");
    let h = hints(&e, Stage::Considerations);
    assert_eq!(key(&h.teaching), Some(GAME_VALUE));
    let Some(LessonContent::GameValue {
        value,
        bidding,
        kontra_re,
        skat,
        games,
    }) = &h.lesson
    else {
        panic!("{:?}", h.lesson);
    };
    assert_eq!((*value, *bidding, *kontra_re), (20, true, false));
    assert_eq!(*skat, SkatState::Unknown);
    assert_eq!(games.len(), 6);
    // Karo with pickup 9 × 2..5 = 18..45: only with a suitable Skat.
    assert_eq!(games[3].0, GameType::Farbe(Suit::Karo));
    assert_eq!(
        games[3].1[0],
        ContractValue {
            contract: Contract::new(
                skat_core::Announcement::plain(GameType::Farbe(Suit::Karo)),
                false
            ),
            range: Range { low: 18, high: 45 },
            reach: Reach::Skat,
        }
    );
    // Null: 23, 35, 46, 59, all reach 20.
    assert_eq!(games[5].0, GameType::Null);
    let null: Vec<(u16, Reach)> = games[5].1.iter().map(|c| (c.range.low, c.reach)).collect();
    assert_eq!(
        null,
        [
            (23, Reach::Reached),
            (35, Reach::Reached),
            (46, Reach::Reached),
            (59, Reach::Reached)
        ]
    );
}

#[test]
fn the_lesson_after_the_move_does_not_depend_on_the_move() {
    // Every bid and the pass get the same lesson content: it names no
    // option, and the note on the move stays a separate model judgement.
    for actions in [vec![], vec![Action::Bid(18), Action::Hold]] {
        let v = view(deal_d(), &actions);
        let e = advise(&v, TINY, 1, None).expect("a decision");
        let before = hints(&e, Stage::Figures);
        let reviews: Vec<_> = e
            .actions
            .iter()
            .map(|&a| review(&e, a).expect("legal"))
            .collect();
        for r in &reviews {
            assert_eq!(r.teaching, before.teaching);
            assert_eq!(r.lesson, before.lesson);
            assert!(r.lesson.is_some());
        }
    }
}

#[test]
fn discard_and_announcement_hand_over_their_lessons() {
    let at_48 = [Action::Bid(48), Action::Pass, Action::Pass];
    // Pickup decision: value lesson (Spitzen still open).
    let e = advise(&view(deal_d(), &at_48), TINY, 1, None).expect("a decision");
    let h = hints(&e, Stage::Considerations);
    assert_eq!(key(&h.teaching), Some(GAME_VALUE));
    assert!(matches!(
        h.lesson,
        Some(LessonContent::GameValue {
            value: 48,
            bidding: false,
            ..
        })
    ));
    // Drücken: twelve cards, structure lesson with the Skat held.
    let mut actions = at_48.to_vec();
    actions.push(Action::TakeSkat);
    let e = advise(&view(deal_d(), &actions), TINY, 1, None).expect("a decision");
    let h = hints(&e, Stage::Considerations);
    assert_eq!(key(&h.teaching), Some(HAND_STRUCTURE));
    let Some(LessonContent::HandStructure { games }) = &h.lesson else {
        panic!("{:?}", h.lesson);
    };
    assert!(games.iter().all(|s| s.skat == SkatState::Held));
    // Announcement after the discard: values with the known Skat.
    actions.push(Action::Discard(cards("DJ H7")));
    let e = advise(&view(deal_d(), &actions), TINY, 1, None).expect("a decision");
    let h = hints(&e, Stage::Considerations);
    let Some(LessonContent::GameValue { skat, games, .. }) = &h.lesson else {
        panic!("{:?}", h.lesson);
    };
    assert_eq!(*skat, SkatState::Discarded);
    // Grand 24 × 2 = 48 reaches 48 exactly; Herz 20 only with Schneider
    // or Schwarz made in play.
    assert_eq!(games[4].1[0].reach, Reach::Reached);
    assert_eq!(games[2].1[0].reach, Reach::Play);
}

#[test]
fn incomplete_analysis_keeps_the_exact_lesson() {
    let v = view(deal_d(), &[Action::Bid(18), Action::Hold]);
    let cancel = Cancel::new();
    cancel.cancel();
    let a = analyze(&v, TINY, 1, Some(&cancel)).expect("consistent");
    let e = explain(&v, &a).expect("a decision");
    assert_ne!(e.status, Status::Complete);
    let full = advise(&v, TINY, 1, None).expect("a decision");
    let h = hints(&e, Stage::Situation);
    assert_eq!(key(&h.teaching), Some(GAME_VALUE));
    assert_eq!(h.lesson, hints(&full, Stage::Situation).lesson);
    let r = review(&e, Action::Pass).expect("legal");
    assert_eq!(r.note(), skat_coach::Note::Uncertain);
    assert_eq!(r.lesson, h.lesson);
}

#[test]
fn same_view_and_seed_give_the_same_lesson() {
    let v = view(deal_d(), &[Action::Bid(18), Action::Hold]);
    let a = advise(&v, TINY, 7, None).expect("a decision");
    let b = advise(&v, TINY, 7, None).expect("a decision");
    assert_eq!(a, b);
    assert_eq!(
        hints(&a, Stage::Figures).lesson,
        hints(&b, Stage::Figures).lesson
    );
}

#[test]
fn the_lesson_shown_before_the_move_does_not_reveal_the_preferred_call() {
    // D-49, implicit leaks: two opening calls whose preferred option
    // differs get the same lesson key and kind of content before the move.
    let [s, w] = [strong(), weak()].map(|d| {
        let v = view(d, &[]);
        assert_eq!(v.seat, Seat::Mittelhand);
        advise(&v, SMALL, 3, None).expect("a decision")
    });
    assert_eq!(s.status, Status::Complete);
    assert_eq!(w.status, Status::Complete);
    let best = |e: &skat_coach::Explanation| e.best().map(|o| o.action);
    assert_ne!(
        best(&s),
        best(&w),
        "precondition: different preferred calls"
    );
    for stage in [Stage::Considerations, Stage::Situation, Stage::Figures] {
        let (hs, hw) = (hints(&s, stage), hints(&w, stage));
        assert_eq!(hs.teaching, hw.teaching);
        assert!(matches!(
            hs.lesson,
            Some(LessonContent::HandStructure { .. })
        ));
        assert!(matches!(
            hw.lesson,
            Some(LessonContent::HandStructure { .. })
        ));
    }
}

#[test]
fn missing_evidence_gives_no_lesson() {
    // A teaching that cites no structure or value item shows nothing.
    let v = view(deal_d(), &[Action::Bid(18), Action::Hold]);
    let e = advise(&v, TINY, 1, None).expect("a decision");
    for key in [GAME_VALUE, HAND_STRUCTURE] {
        let teaching = Teaching::Lesson {
            id: skat_coach::LessonId {
                key: key.into(),
                version: 1,
            },
            items: Vec::new(),
        };
        assert_eq!(lesson_content(&teaching, &e.evidence, None), None);
    }
    assert_eq!(lesson_content(&Teaching::Fallback, &e.evidence, None), None);
}
