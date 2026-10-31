// SPDX-License-Identifier: EUPL-1.2

//! The two exact-information lessons of the catalogue (#43,
//! `docs/coach.md`): remaining Trumpf and proven voids.
//!
//! Every expected set, count and lesson result is worked out by hand from
//! the deal and the play in the comments, not taken from the code under
//! test. `knowledge_matches_the_hidden_truth` checks the underlying facts
//! against the full deal of random games, an oracle independent of the
//! lesson predicates.

mod common;

use common::*;
use skat_analysis::lesson::{
    CONTRACT_VERSION, Check, Claim, Condition, Evidence, EvidenceClass, FactKind, GameKind,
    LessonKind, PROVEN_VOID, REMAINING_TRUMPF, Source, Support, Unmet, catalogue, gather,
};
use skat_analysis::{Budget, Knowledge, Observation, Status, TrumpfSets, analyze};
use skat_core::{
    Action, Announcement, CardSet, Deal, Follow, Game, GameType, Phase, PlayerView, RuleConfig,
    Seat, Suit,
};

/// Small and node-limited: exact evidence must not depend on the search.
const TINY: Budget = Budget {
    samples: 2,
    nodes_per_task: 20_000,
};

/// Enough to finish, so the whole evidence is compared.
const SWAP: Budget = Budget {
    samples: 3,
    nodes_per_task: 50_000_000,
};

/// MH 18, VH holds, MH and HH pass: VH declarer at 18.
const VH_AT_18: [Action; 4] = [Action::Bid(18), Action::Hold, Action::Pass, Action::Pass];

fn standard_deal() -> Deal {
    deal(
        "CJ SJ HJ DJ CA CT CK CQ C9 C8",
        "SA ST SK SQ S9 S8 S7 HA HT HK",
        "HQ H9 H8 H7 DA DT DK DQ D9 D8",
        "C7 D7",
    )
}

/// A Herz game with Trumpf on both defenders' sides.
fn herz_deal() -> Deal {
    deal(
        "CJ SJ HA HT HK HQ H9 SA ST CA",
        "DJ H8 H7 SK SQ S9 CK CQ C9 DA",
        "HJ S8 S7 CT C8 C7 DT DK DQ D9",
        "D8 D7",
    )
}

/// Herz Hand by VH. Trick 1: CJ, H7, HJ (both follow Trumpf; VH wins).
/// Trick 2: HA, H8, S7 (HH fails Trumpf). Trick 3: SA, SK, S8 (all
/// follow). The first `cards` cards of that play.
fn herz_play(cards: usize) -> Vec<Action> {
    let mut actions = vec![
        Action::Hand,
        Action::Announce(Announcement::plain(GameType::Farbe(Suit::Herz))),
    ];
    let play = ["CJ", "H7", "HJ", "HA", "H8", "S7", "SA", "SK", "S8"];
    actions.extend(play[..cards].iter().map(|s| Action::Play(c(s))));
    actions
}

fn play(deal: Deal, actions: &[Action]) -> Game {
    let mut g = Game::from_deal(deal, RuleConfig::default());
    for &a in VH_AT_18.iter().chain(actions) {
        g.apply(a).unwrap_or_else(|e| panic!("{a:?}: {e:?}"));
    }
    g
}

/// The view of the seat to move.
fn to_move(g: &Game) -> PlayerView {
    let seat = Seat::ALL
        .into_iter()
        .find(|&s| !g.view(s).legal_actions.is_empty())
        .expect("a decision");
    g.view(seat)
}

fn evidence(view: &PlayerView, budget: Budget, seed: u64) -> Evidence {
    gather(
        view,
        &analyze(view, budget, seed, None).expect("consistent"),
    )
}

fn lesson(key: &str) -> skat_analysis::lesson::Lesson {
    catalogue()
        .into_iter()
        .find(|l| l.id.key == key)
        .expect("in the catalogue")
}

/// The index and value of the only exact item of `kind`.
fn only(e: &Evidence, kind: FactKind) -> (usize, Observation) {
    let found: Vec<(usize, Observation)> = e
        .items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| match item.claim {
            Claim::Fact(o) if FactKind::of(&o) == kind => Some((i, o)),
            _ => None,
        })
        .collect();
    assert_eq!(found.len(), 1, "{kind:?} in {:#?}", e.items);
    found[0]
}

fn trumpf_sets(e: &Evidence) -> (usize, TrumpfSets) {
    match only(e, FactKind::TrumpfSets) {
        (i, Observation::TrumpfSets(sets)) => (i, sets),
        other => panic!("{other:?}"),
    }
}

fn trick(index: u8, seat: Seat, led: &str, card: &str) -> Source {
    Source::Trick {
        index,
        seat,
        led: c(led),
        card: c(card),
    }
}

fn applies(e: &Evidence, key: &str) -> Check {
    lesson(key).check(e, None)
}

#[test]
fn the_catalogue_holds_two_exact_card_play_lessons() {
    let all = catalogue();
    let keys: Vec<&str> = all.iter().map(|l| l.id.key.as_ref()).collect();
    assert_eq!(keys, [PROVEN_VOID, REMAINING_TRUMPF]);
    for l in &all {
        assert_eq!(l.id.version, 1);
        assert_eq!(l.kind, LessonKind::Fact);
        assert_eq!(l.support, Support::Supported);
        assert_eq!(l.applies.phases, [Phase::Spiel]);
        assert!(l.applies.roles.is_empty(), "every role");
        assert_eq!(l.permitted, [EvidenceClass::Exact]);
        assert!(l.excludes.is_empty());
    }
    assert_eq!(
        lesson(PROVEN_VOID).applies.games,
        [GameKind::Suit, GameKind::Grand, GameKind::Null]
    );
    assert_eq!(
        lesson(REMAINING_TRUMPF).applies.games,
        [GameKind::Suit, GameKind::Grand]
    );
    assert_eq!(CONTRACT_VERSION, 2);
}

#[test]
fn remaining_trumpf_in_a_partially_played_trick() {
    // HH to play in trick 2 after HA, H8. Herz game: 11 Trumpf (4 Buben,
    // 7 Herz). HH held only HJ (played in trick 1): own none. Played: CJ
    // H7 HJ (trick 1), HA H8 (trick 2) = 5. 11 - 0 - 5 = 6 unseen: SJ DJ
    // HT HK HQ H9. HH is a defender: the Skat is unknown.
    let g = play(herz_deal(), &herz_play(5));
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Hinterhand);
    let e = evidence(&view, TINY, 1);
    let (i, sets) = trumpf_sets(&e);
    assert_eq!(
        sets,
        TrumpfSets {
            total: cards("CJ SJ HJ DJ HA HT HK HQ H9 H8 H7"),
            own: CardSet::EMPTY,
            played: cards("CJ H7 HJ HA H8"),
            own_skat: CardSet::EMPTY,
            revealed: CardSet::EMPTY,
            unseen: cards("SJ DJ HT HK HQ H9"),
            skat_unknown: true,
        }
    );
    // Provenance: own hand, the play, and each played Trumpf with its trick.
    assert_eq!(
        e.items[i].sources,
        [
            Source::OwnHand,
            Source::Played,
            trick(0, Seat::Vorhand, "CJ", "CJ"),
            trick(0, Seat::Mittelhand, "CJ", "H7"),
            trick(0, Seat::Hinterhand, "CJ", "HJ"),
            trick(1, Seat::Vorhand, "HA", "HA"),
            trick(1, Seat::Mittelhand, "HA", "H8"),
        ]
    );
    // The existing count keeps its meaning: 6 here as well.
    assert_eq!(
        only(&e, FactKind::TrumpsOut).1,
        Observation::TrumpsOut { count: 6 }
    );
    assert_eq!(
        applies(&e, REMAINING_TRUMPF),
        Check::Applies { items: vec![i] }
    );
    // Counterexample: everyone followed so far, so no void is proven.
    assert_eq!(
        applies(&e, PROVEN_VOID),
        Check::Fails(Unmet::Missing(Condition::Recent(FactKind::Void)))
    );
}

#[test]
fn hand_declarer_does_not_know_the_skat() {
    // VH (Herz Hand) to lead trick 2. Own Trumpf: SJ HA HT HK HQ H9 (6);
    // played CJ H7 HJ; 11 - 6 - 3 = 2 unseen: DJ H8, with MH or in the
    // unknown Skat. B moves MH's H8 into the Skat for D8: MH still follows
    // Trumpf with H7 in trick 1, and VH sees the same.
    let b = deal(
        "CJ SJ HA HT HK HQ H9 SA ST CA",
        "DJ D8 H7 SK SQ S9 CK CQ C9 DA",
        "HJ S8 S7 CT C8 C7 DT DK DQ D9",
        "H8 D7",
    );
    let views = [herz_deal(), b].map(|d| to_move(&play(d, &herz_play(3))));
    assert_eq!(views[0].seat, Seat::Vorhand);
    assert_eq!(views[0], views[1], "precondition: one information set");
    let [a, b] = views.map(|v| evidence(&v, SWAP, 7));
    assert_eq!(a.status, Status::Complete);
    assert_eq!(a, b, "hidden cards do not change the evidence");
    let catalogue = catalogue();
    for l in &catalogue {
        assert_eq!(l.check(&a, None), l.check(&b, None));
    }
    let (i, sets) = trumpf_sets(&a);
    assert_eq!(sets.own, cards("SJ HA HT HK HQ H9"));
    assert_eq!(sets.played, cards("CJ H7 HJ"));
    assert_eq!(sets.own_skat, CardSet::EMPTY);
    assert_eq!(sets.unseen, cards("DJ H8"));
    assert!(sets.skat_unknown);
    assert!(!a.items[i].sources.contains(&Source::OwnSkat));
    assert_eq!(
        applies(&a, REMAINING_TRUMPF),
        Check::Applies { items: vec![i] }
    );
}

#[test]
fn recency_boundary_of_both_lessons() {
    // VH to lead trick 3, after trick 2 (HA, H8, S7): HH played S7 on the
    // Trumpf HA, so HH holds no Trumpf. Proven in trick index 1, the last
    // completed trick: both lessons apply, the void first.
    let g = play(herz_deal(), &herz_play(6));
    let e = evidence(&to_move(&g), TINY, 1);
    let (v, void) = only(&e, FactKind::Void);
    assert_eq!(
        void,
        Observation::Void {
            seat: Seat::Hinterhand,
            follow: Follow::Trumpf,
        }
    );
    assert_eq!(e.items[v].sources, [trick(1, Seat::Hinterhand, "HA", "S7")]);
    assert_eq!(applies(&e, PROVEN_VOID), Check::Applies { items: vec![v] });
    // Own SJ HT HK HQ H9 (5), played CJ H7 HJ HA H8 (5): DJ unseen.
    let (t, sets) = trumpf_sets(&e);
    assert_eq!(sets.unseen, cards("DJ"));
    assert_eq!(
        applies(&e, REMAINING_TRUMPF),
        Check::Applies { items: vec![t] }
    );

    // One trick later (SA, SK, S8: no Trumpf, everyone follows): the void
    // is still a fact but no longer new, and no Trumpf was played in the
    // last trick. Neither lesson repeats.
    let g = play(herz_deal(), &herz_play(9));
    let e = evidence(&to_move(&g), TINY, 1);
    assert_eq!(e.snapshot.tricks, 3);
    assert_eq!(only(&e, FactKind::Void).1, void);
    assert_eq!(
        applies(&e, PROVEN_VOID),
        Check::Fails(Unmet::Missing(Condition::Recent(FactKind::Void)))
    );
    assert_eq!(
        applies(&e, REMAINING_TRUMPF),
        Check::Fails(Unmet::Missing(Condition::Recent(FactKind::TrumpfSets)))
    );
}

#[test]
fn grand_counts_only_the_buben() {
    // Grand Hand by VH, standard deal. Trick 1: CJ, SA, HQ. MH to play in
    // trick 2 after VH led CA. Trumpf are the 4 Buben only, so CA is no
    // Trumpf. MH holds no Bube; played: CJ; unseen SJ HJ DJ.
    let mut actions = vec![
        Action::Hand,
        Action::Announce(Announcement::plain(GameType::Grand)),
    ];
    actions.extend(["CJ", "SA", "HQ", "CA"].map(|s| Action::Play(c(s))));
    let g = play(standard_deal(), &actions);
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Mittelhand);
    let e = evidence(&view, TINY, 1);
    let (t, sets) = trumpf_sets(&e);
    assert_eq!(
        sets,
        TrumpfSets {
            total: cards("CJ SJ HJ DJ"),
            own: CardSet::EMPTY,
            played: cards("CJ"),
            own_skat: CardSet::EMPTY,
            revealed: CardSet::EMPTY,
            unseen: cards("SJ HJ DJ"),
            skat_unknown: true,
        }
    );
    assert_eq!(
        applies(&e, REMAINING_TRUMPF),
        Check::Applies { items: vec![t] }
    );
    // HH failed Trumpf in trick 1 (HQ on CJ): cited. MH's own failure in
    // trick 1 is no observation: a seat knows its own cards.
    let (v, void) = only(&e, FactKind::Void);
    assert_eq!(
        void,
        Observation::Void {
            seat: Seat::Hinterhand,
            follow: Follow::Trumpf,
        }
    );
    assert_eq!(e.items[v].sources, [trick(0, Seat::Hinterhand, "CJ", "HQ")]);
    assert_eq!(applies(&e, PROVEN_VOID), Check::Applies { items: vec![v] });
}

#[test]
fn a_discarded_trumpf_is_known_and_not_unseen() {
    // VH picks up C7 D7, discards them and plays Kreuz. Trick 1: CJ, SA,
    // HQ. 11 Trumpf: own SJ HJ DJ CA CT CK CQ C9 C8 (9), played CJ, own
    // Skat C7: nothing unseen, all Trumpf are accounted for (exhausted).
    // `TrumpsOut` keeps its meaning and still counts C7.
    let actions = [
        Action::TakeSkat,
        Action::Discard(cards("C7 D7")),
        Action::Announce(Announcement::plain(GameType::Farbe(Suit::Kreuz))),
        Action::Play(c("CJ")),
        Action::Play(c("SA")),
        Action::Play(c("HQ")),
    ];
    let g = play(standard_deal(), &actions);
    let declarer = evidence(&g.view(Seat::Vorhand), TINY, 1);
    let (t, sets) = trumpf_sets(&declarer);
    assert_eq!(
        sets,
        TrumpfSets {
            total: cards("CJ SJ HJ DJ CA CT CK CQ C9 C8 C7"),
            own: cards("SJ HJ DJ CA CT CK CQ C9 C8"),
            played: cards("CJ"),
            own_skat: cards("C7"),
            revealed: CardSet::EMPTY,
            unseen: CardSet::EMPTY,
            skat_unknown: false,
        }
    );
    assert!(declarer.items[t].sources.contains(&Source::OwnSkat));
    assert_eq!(
        only(&declarer, FactKind::TrumpsOut).1,
        Observation::TrumpsOut { count: 1 }
    );
    assert_eq!(
        applies(&declarer, REMAINING_TRUMPF),
        Check::Applies { items: vec![t] }
    );
    // The same card is unseen for a defender: MH before its card in
    // trick 1, CJ led. Own no Trumpf; 11 - 1 = 10 unseen, C7 among them.
    let g = play(standard_deal(), &actions[..4]);
    let defender = evidence(&g.view(Seat::Mittelhand), TINY, 1);
    let (_, sets) = trumpf_sets(&defender);
    assert_eq!(sets.own_skat, CardSet::EMPTY);
    assert_eq!(sets.unseen, cards("SJ HJ DJ CA CT CK CQ C9 C8 C7"));
    assert!(sets.skat_unknown);
}

#[test]
fn ouvert_cards_are_seen_trumpf() {
    // Grand Ouvert Hand by VH, MH to play after CJ. VH's revealed cards
    // hold SJ HJ DJ: with CJ played, no Bube is concealed from MH.
    let ouvert = Announcement {
        game: GameType::Grand,
        schneider: true,
        schwarz: true,
        ouvert: true,
    };
    let g = play(
        standard_deal(),
        &[
            Action::Hand,
            Action::Announce(ouvert),
            Action::Play(c("CJ")),
        ],
    );
    let e = evidence(&g.view(Seat::Mittelhand), TINY, 1);
    let (t, sets) = trumpf_sets(&e);
    assert_eq!(sets.revealed, cards("SJ HJ DJ"));
    assert_eq!(sets.played, cards("CJ"));
    assert_eq!(sets.own, CardSet::EMPTY);
    assert_eq!(sets.unseen, CardSet::EMPTY);
    assert!(e.items[t].sources.contains(&Source::Revealed));
    assert_eq!(
        applies(&e, REMAINING_TRUMPF),
        Check::Applies { items: vec![t] }
    );
    // The declarer itself sees no "revealed" cards.
    let own = evidence(&g.view(Seat::Vorhand), TINY, 1);
    assert_eq!(trumpf_sets(&own).1.revealed, CardSet::EMPTY);
}

#[test]
fn null_has_no_trumpf_lesson_but_a_void_lesson() {
    // Null Hand by VH. VH leads CA, MH plays SA: MH holds no Kreuz (in
    // Null CJ is a Kreuz card too). HH to play.
    let actions = [
        Action::Hand,
        Action::Announce(Announcement::plain(GameType::Null)),
        Action::Play(c("CA")),
        Action::Play(c("SA")),
    ];
    let g = play(standard_deal(), &actions);
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Hinterhand);
    let e = evidence(&view, TINY, 1);
    assert!(e.items.iter().all(|i| !matches!(
        i.claim,
        Claim::Fact(Observation::TrumpfSets(_) | Observation::TrumpsOut { .. })
    )));
    assert_eq!(applies(&e, REMAINING_TRUMPF), Check::Fails(Unmet::Game));
    let (v, void) = only(&e, FactKind::Void);
    assert_eq!(
        void,
        Observation::Void {
            seat: Seat::Mittelhand,
            follow: Follow::Farbe(Suit::Kreuz),
        }
    );
    assert_eq!(e.items[v].sources, [trick(0, Seat::Mittelhand, "CA", "SA")]);
    assert_eq!(applies(&e, PROVEN_VOID), Check::Applies { items: vec![v] });
}

#[test]
fn no_lesson_before_card_play_or_before_the_first_card() {
    // Bidding: no game, no card play.
    let g = Game::from_deal(standard_deal(), RuleConfig::default());
    let e = evidence(&to_move(&g), TINY, 1);
    assert_eq!(applies(&e, PROVEN_VOID), Check::Fails(Unmet::Phase));
    assert_eq!(applies(&e, REMAINING_TRUMPF), Check::Fails(Unmet::Phase));
    // First card of a Grand: nothing played, nothing proven.
    let g = play(
        standard_deal(),
        &[
            Action::Hand,
            Action::Announce(Announcement::plain(GameType::Grand)),
        ],
    );
    let e = evidence(&to_move(&g), TINY, 1);
    assert_eq!(
        applies(&e, PROVEN_VOID),
        Check::Fails(Unmet::Missing(Condition::Recent(FactKind::Void)))
    );
    assert_eq!(
        applies(&e, REMAINING_TRUMPF),
        Check::Fails(Unmet::Missing(Condition::Recent(FactKind::TrumpfSets)))
    );
}

#[test]
fn exact_lessons_are_deterministic_and_seed_independent() {
    let g = play(herz_deal(), &herz_play(6));
    let view = to_move(&g);
    let exact = |e: &Evidence| -> Vec<_> {
        e.items
            .iter()
            .filter(|i| i.class() == EvidenceClass::Exact)
            .cloned()
            .collect()
    };
    let a = evidence(&view, TINY, 1);
    assert_eq!(a, evidence(&view, TINY, 1), "same input, same evidence");
    let other = evidence(&view, SWAP, 99);
    assert_eq!(exact(&a), exact(&other));
    for l in catalogue() {
        assert_eq!(l.check(&a, None), l.check(&other, None), "{:?}", l.id);
    }
}

/// A small deterministic generator for the random games below.
fn next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state >> 33
}

#[test]
fn knowledge_matches_the_hidden_truth() {
    // Oracle: the full deal. For every seat at every card-play position of
    // random games, the Trumpf sets partition all Trumpf, and each place is
    // right: unseen Trumpf lie in another hand or the Skat, a known Skat
    // holds the own-Skat Trumpf, revealed Trumpf are in the declarer's
    // hand, and a seat proven void holds no card of that class.
    let mut positions = 0;
    for seed in 0..300u64 {
        let mut g = Game::new(seed, RuleConfig::default());
        let mut state = seed;
        while g.phase() != Phase::Beendet && g.phase() != Phase::Eingepasst {
            let seat = Seat::ALL
                .into_iter()
                .find(|&s| !g.view(s).legal_actions.is_empty())
                .expect("someone to move");
            if g.phase() == Phase::Spiel {
                check_truth(&g);
                positions += 1;
            }
            let legal = g.view(seat).legal_actions;
            let action = legal[next(&mut state) as usize % legal.len()];
            g.apply(action).expect("legal");
        }
    }
    assert!(positions > 1000, "{positions}");
}

fn check_truth(g: &Game) {
    let hands = Seat::ALL.map(|s| g.view(s).hand);
    let any = g.view(Seat::Vorhand);
    let mut played: CardSet = any.tricks.iter().flat_map(|t| t.cards).collect();
    played |= any.current_trick.cards.iter().flatten().copied().collect();
    let skat = CardSet::ALL - hands[0] - hands[1] - hands[2] - played;
    for seat in Seat::ALL {
        let view = g.view(seat);
        let k = Knowledge::from_view(&view);
        let game = k.game.expect("announced");
        let others = Seat::ALL
            .into_iter()
            .filter(|&o| o != seat)
            .fold(CardSet::EMPTY, |acc, o| acc | hands[o.index()]);
        match k.trumpf_sets() {
            None => assert_eq!(game, GameType::Null),
            Some(t) => {
                assert_eq!(t.total, game.trumps());
                let parts = [t.own, t.played, t.own_skat, t.revealed, t.unseen];
                let sum: usize = parts.iter().map(|p| p.len()).sum();
                let union = parts.iter().fold(CardSet::EMPTY, |a, &p| a | p);
                assert_eq!(union, t.total);
                assert_eq!(sum, t.total.len(), "disjoint");
                assert_eq!(t.own, t.total & hands[seat.index()]);
                assert!(t.own_skat.is_subset(skat));
                let declarer = hands[view.declarer.unwrap().index()];
                assert!(t.revealed.is_subset(declarer));
                assert!(t.unseen.is_subset(others | skat));
                assert_eq!(t.skat_unknown, view.own_skat.is_none());
                if !t.skat_unknown {
                    assert!(t.unseen.is_disjoint(skat));
                }
            }
        }
        for o in Seat::ALL {
            assert!(k.voids[o.index()].is_disjoint(hands[o.index()]), "{o:?}");
        }
    }
}
