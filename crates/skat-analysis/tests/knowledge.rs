// SPDX-License-Identifier: EUPL-1.2

//! Card counting, known voids, visibility, Skat knowledge and hidden-card
//! distributions on fixed positions. Expected counts are derived by hand.

mod common;

use common::*;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use skat_analysis::Knowledge;
use skat_analysis::sampling::{Diagnostic, HiddenCards, Place};
use skat_core::{
    Action, Announcement, CardSet, Follow, Game, GameType, PlayerView, RuleConfig, Seat, Suit,
};

/// VH: all Jacks and six Kreuz; MH: Pik and three Herz; HH: Herz and Karo.
fn standard() -> Game {
    Game::from_deal(
        deal(
            "CJ SJ HJ DJ CA CT CK CQ C9 C8",
            "SA ST SK SQ S9 S8 S7 HA HT HK",
            "HQ H9 H8 H7 DA DT DK DQ D9 D8",
            "C7 D7",
        ),
        RuleConfig::default(),
    )
}

/// MH 18, VH holds, MH and HH pass: VH declarer at 18.
const VH_AT_18: [Action; 4] = [Action::Bid(18), Action::Hold, Action::Pass, Action::Pass];

fn run(game: &mut Game, actions: &[Action]) {
    for &a in actions {
        game.apply(a).unwrap_or_else(|e| panic!("{a:?}: {e:?}"));
    }
}

fn play(game: &mut Game, notation: &str) {
    for s in notation.split_whitespace() {
        game.apply(Action::Play(c(s))).unwrap();
    }
}

/// Grand Hand by VH; trick 1 CJ S7 H7, trick 2 CA HK D8.
fn grand_after_two_tricks() -> Game {
    let mut g = standard();
    run(&mut g, &VH_AT_18);
    run(
        &mut g,
        &[
            Action::Hand,
            Action::Announce(Announcement::plain(GameType::Grand)),
        ],
    );
    play(&mut g, "CJ S7 H7 CA HK D8");
    g
}

#[test]
fn card_counting_after_two_tricks() {
    let g = grand_after_two_tricks();
    let k = Knowledge::from_view(&g.view(Seat::Mittelhand));
    assert_eq!(k.played, cards("CJ S7 H7 CA HK D8"));
    assert_eq!(k.own, cards("SA ST SK SQ S9 S8 HA HT"));
    assert_eq!(k.augen_won, [2 + 15, 0, 0]);
    assert_eq!(k.party_augen(), Some((17, 0)));
    assert_eq!(k.trumps_out(), cards("SJ HJ DJ"));
    // Out: 120 minus own (SA ST SK SQ HA HT) minus played.
    assert_eq!(k.own.augen(), 11 + 10 + 4 + 3 + 11 + 10);
    assert_eq!(k.augen_out(), 120 - 49 - 17);
}

#[test]
fn voids_are_proven_by_not_following() {
    let g = grand_after_two_tricks();
    let k = Knowledge::from_view(&g.view(Seat::Mittelhand));
    let kreuz = GameType::Grand.cards_of(Follow::Farbe(Suit::Kreuz));
    // HH discarded on a Jack lead and on a Kreuz lead.
    assert_eq!(k.voids[Seat::Hinterhand.index()], GameType::BUBEN | kreuz);
    // MH as well (seen from its own view, too).
    assert_eq!(k.voids[Seat::Mittelhand.index()], GameType::BUBEN | kreuz);
    assert_eq!(k.voids[Seat::Vorhand.index()], CardSet::EMPTY);
}

#[test]
fn hidden_distributions_match_a_hand_count() {
    // Seen by MH: 18 unseen cards for VH (8), HH (8) and the Skat (2). HH
    // may hold neither Jacks nor Kreuz, so the 9 unseen ones (SJ HJ DJ CT
    // CK CQ C9 C8 C7) go to VH or the Skat. HH takes 8 of the 9 other unseen
    // cards (9 ways), the last one joins the 9 in VH and Skat: C(10, 2) = 45
    // ways to pick the Skat. 9 * 45 = 405.
    let g = grand_after_two_tricks();
    let view = g.view(Seat::Mittelhand);
    let hidden = HiddenCards::from_view(&view).unwrap();
    assert_eq!(hidden.unseen().len(), 18);
    assert_eq!(hidden.count(), 405);
    let all = hidden.enumerate(1000).unwrap();
    assert_eq!(all.len(), 405);
    let mut distinct = all.clone();
    distinct.sort_by_key(|w| (w.hands.map(|h| h.bits()), w.skat.bits()));
    distinct.dedup();
    assert_eq!(distinct.len(), 405);
    // The real distribution is one of them.
    let omniscient = g.omniscient_view();
    assert!(
        all.iter()
            .any(|w| w.hands == omniscient.hands && w.skat == omniscient.skat)
    );
    assert!(hidden.enumerate(404).is_none());
}

#[test]
fn samples_conserve_cards_satisfy_constraints_and_are_uniform() {
    let g = grand_after_two_tricks();
    let view = g.view(Seat::Mittelhand);
    let hidden = HiddenCards::from_view(&view).unwrap();
    let all = hidden.enumerate(1000).unwrap();
    let mut counts = std::collections::HashMap::new();
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let draws = 405 * 100;
    let played = Knowledge::from_view(&view).played;
    for _ in 0..draws {
        let w = hidden.sample(&mut rng);
        assert!(hidden.admits(&w));
        // Every card in exactly one place.
        let parts = [w.hands[0], w.hands[1], w.hands[2], w.skat, played];
        assert_eq!(parts.iter().map(|p| p.len()).sum::<usize>(), 32);
        assert_eq!(
            parts.iter().fold(CardSet::EMPTY, |a, &p| a | p),
            CardSet::ALL
        );
        assert_eq!(w.hands.map(|h| h.len()), [8, 8, 8]);
        assert_eq!(w.hands[1], view.hand);
        assert!((w.hands[2] & (GameType::BUBEN | Suit::Kreuz.cards())).is_empty());
        *counts.entry(w).or_insert(0u32) += 1;
    }
    // Uniform: every one of the 405 distributions, each near 100 times
    // (Poisson sd 10; the bounds are beyond 4.5 sd).
    assert_eq!(counts.len(), 405);
    for w in &all {
        let n = counts[w];
        assert!((55..=145).contains(&n), "{n}");
    }
    // Same seed, same samples.
    let mut a = ChaCha8Rng::seed_from_u64(7);
    let mut b = ChaCha8Rng::seed_from_u64(7);
    for _ in 0..20 {
        assert_eq!(hidden.sample(&mut a), hidden.sample(&mut b));
    }
}

fn places(view: &PlayerView) -> Vec<Place> {
    HiddenCards::from_view(view)
        .unwrap()
        .slots()
        .iter()
        .map(|s| s.place)
        .collect()
}

#[test]
fn skat_knowledge_only_for_the_seat_that_saw_it() {
    // Pickup: the declarer knows its discard, the defenders do not.
    let mut g = standard();
    run(&mut g, &VH_AT_18);
    run(&mut g, &[Action::TakeSkat]);
    // While discarding, the declarer holds all twelve cards: no Skat place.
    assert_eq!(
        places(&g.view(Seat::Vorhand)),
        vec![Place::Seat(Seat::Mittelhand), Place::Seat(Seat::Hinterhand)]
    );
    // Defenders see a declarer with twelve unknown cards and no Skat.
    let mh = HiddenCards::from_view(&g.view(Seat::Mittelhand)).unwrap();
    assert_eq!(mh.slots()[0].size, 12);
    run(&mut g, &[Action::Discard(cards("C7 D7"))]);
    let vh = Knowledge::from_view(&g.view(Seat::Vorhand));
    assert_eq!(vh.skat, Some(cards("C7 D7")));
    assert_eq!(
        HiddenCards::from_view(&g.view(Seat::Vorhand))
            .unwrap()
            .known()
            .skat,
        cards("C7 D7")
    );
    for defender in [Seat::Mittelhand, Seat::Hinterhand] {
        let view = g.view(defender);
        assert_eq!(Knowledge::from_view(&view).skat, None);
        assert!(places(&view).contains(&Place::Skat));
        // The real Skat is not visible: it is still among the unseen cards.
        assert!(cards("C7 D7").is_subset(HiddenCards::from_view(&view).unwrap().unseen()));
    }

    // Hand: nobody knows the Skat, the declarer included.
    let mut g = standard();
    run(&mut g, &VH_AT_18);
    run(&mut g, &[Action::Hand]);
    for seat in Seat::ALL {
        assert!(places(&g.view(seat)).contains(&Place::Skat), "{seat:?}");
    }
}

#[test]
fn ouvert_cards_are_known_after_the_reveal_only() {
    let mut g = standard();
    run(&mut g, &VH_AT_18);
    let ouvert = Announcement {
        game: GameType::Grand,
        schneider: true,
        schwarz: true,
        ouvert: true,
    };
    run(&mut g, &[Action::Hand, Action::Announce(ouvert)]);
    // Kontra/Re is off: revealed at once.
    let view = g.view(Seat::Hinterhand);
    let hidden = HiddenCards::from_view(&view).unwrap();
    assert!(!places(&view).contains(&Place::Seat(Seat::Vorhand)));
    assert_eq!(
        hidden.known().hands[0],
        cards("CJ SJ HJ DJ CA CT CK CQ C9 C8")
    );
    // Two places left: MH (10) and the Skat (2) = 12 unseen cards.
    assert_eq!(hidden.count(), 66);

    // With Kontra/Re the cards stay hidden during the Kontra round.
    let mut g = Game::from_deal(
        deal(
            "CJ SJ HJ DJ CA CT CK CQ C9 C8",
            "SA ST SK SQ S9 S8 S7 HA HT HK",
            "HQ H9 H8 H7 DA DT DK DQ D9 D8",
            "C7 D7",
        ),
        RuleConfig { kontra_re: true },
    );
    run(&mut g, &VH_AT_18);
    run(&mut g, &[Action::Hand, Action::Announce(ouvert)]);
    let view = g.view(Seat::Mittelhand);
    assert_eq!(view.legal_actions, vec![Action::Kontra, Action::KeinKontra]);
    assert!(places(&view).contains(&Place::Seat(Seat::Vorhand)));
}

#[test]
fn inconsistent_views_give_diagnostics() {
    let g = grand_after_two_tricks();
    let view = g.view(Seat::Mittelhand);

    let mut twice = view.clone();
    twice.hand.insert(c("CJ"));
    twice.hand_sizes[1] += 1;
    let errors = HiddenCards::from_view(&twice).unwrap_err();
    assert!(
        errors.contains(&Diagnostic::CardTwice(c("CJ"))),
        "{errors:?}"
    );

    let mut sizes = view.clone();
    sizes.hand_sizes[2] = 9;
    let errors = HiddenCards::from_view(&sizes).unwrap_err();
    assert!(errors.contains(&Diagnostic::HandSize(Seat::Hinterhand)));
    assert!(errors.contains(&Diagnostic::CardCount {
        unseen: 18,
        places: 19
    }));

    let mut short = view.clone();
    short.hand.remove(c("SA"));
    let errors = HiddenCards::from_view(&short).unwrap_err();
    assert!(errors.contains(&Diagnostic::HandSize(Seat::Mittelhand)));
}
