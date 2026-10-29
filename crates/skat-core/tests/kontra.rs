// SPDX-License-Identifier: EUPL-1.2

//! Kontra/Re per D-03 and D-26: eligibility, timing, Re, Ouvert order and
//! scoring.

mod common;

use common::*;
use skat_core::{
    Action, Announcement, Event, Game, GameType, KontraLevel, Phase, RuleError, Seat, Suit,
};

const KREUZ: GameType = GameType::Farbe(Suit::Kreuz);

/// MH 18, VH holds, MH passes, HH passes: VH declarer at 18.
/// MH bid 18 (eligible); HH's first turn was at exactly 18 (not eligible).
const VH_AT_18: [Action; 4] = [Action::Bid(18), Action::Hold, Action::Pass, Action::Pass];

/// MH 18, VH holds, MH 20, VH passes, HH passes at 20: MH declarer at 20.
/// VH held (eligible); HH passed above 18 at its first turn (eligible).
const MH_AT_20: [Action; 5] = [
    Action::Bid(18),
    Action::Hold,
    Action::Bid(20),
    Action::Pass,
    Action::Pass,
];

fn has_kontra(g: &Game) -> bool {
    g.legal_actions().contains(&Action::Kontra)
}

/// Plays the first legal card until the game ends, asserting `check` before
/// each card.
fn play_out(g: &mut Game, mut check: impl FnMut(&Game)) {
    while g.phase() == Phase::Spiel {
        check(g);
        let card = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Play(_)))
            .unwrap();
        g.apply(card).unwrap();
    }
}

#[test]
fn disabled_kontra_is_never_offered() {
    let mut g = game(standard_deal(), false);
    run(&mut g, &VH_AT_18);
    run(
        &mut g,
        &[Action::Hand, Action::Announce(Announcement::plain(KREUZ))],
    );
    play(&mut g, "CJ");
    assert!(!has_kontra(&g));
    assert_rejected(&mut g, Action::Kontra, RuleError::KontraNotAllowed);
}

#[test]
fn kontra_re_on_first_card_quadruples() {
    let mut g = game(standard_deal(), true);
    run(&mut g, &VH_AT_18);
    run(&mut g, &[Action::TakeSkat, Action::Discard(cards("C7 D7"))]);
    run(&mut g, &[Action::Announce(Announcement::plain(KREUZ))]);
    // Declarer never has Kontra.
    assert!(!has_kontra(&g));
    play(&mut g, "CJ");
    assert_eq!(g.current_seat(), Some(Seat::Mittelhand));
    assert!(has_kontra(&g));
    let events = run(&mut g, &[Action::Kontra]);
    assert_eq!(
        events,
        vec![Event::Kontra {
            seat: Seat::Mittelhand
        }]
    );
    // Only Re or KeinRe, by the declarer.
    assert_eq!(g.current_seat(), Some(Seat::Vorhand));
    assert_eq!(g.legal_actions(), vec![Action::Re, Action::KeinRe]);
    assert_rejected(
        &mut g,
        Action::Play(c("SJ")),
        RuleError::UnexpectedAction {
            phase: Phase::Spiel,
        },
    );
    run(&mut g, &[Action::Re]);
    assert_eq!(g.kontra(), KontraLevel::Re);
    // Same defender continues and cannot say Kontra again.
    assert_eq!(g.current_seat(), Some(Seat::Mittelhand));
    assert!(!has_kontra(&g));
    assert_rejected(
        &mut g,
        Action::Re,
        RuleError::UnexpectedAction {
            phase: Phase::Spiel,
        },
    );
    play(&mut g, "SA");
    // Hinterhand passed at 18 at its first turn: not eligible.
    assert!(!has_kontra(&g));
    assert_rejected(&mut g, Action::Kontra, RuleError::KontraNotAllowed);
    play(
        &mut g,
        "HQ  SJ ST H9  HJ SK H8  DJ SQ H7  CA S9 DA \
         CT S8 DT  CK S7 DK  CQ HA DQ  C9 HT D9  C8 HK D8",
    );
    let r = *g.result().unwrap();
    // 168 * 4; Seeger-Fabian bonus 50 not multiplied.
    assert_eq!(r.score, 672);
    assert_eq!(r.seeger_fabian(), [722, 0, 0]);
}

#[test]
fn kontra_without_re_doubles_a_loss() {
    let mut g = game(standard_deal(), true);
    run(
        &mut g,
        &[Action::Pass, Action::Bid(18), Action::Hold, Action::Pass],
    );
    // VH holds 18 then HH passes: VH declarer, HH bid 18 (eligible).
    assert_eq!(g.declarer(), Some(Seat::Vorhand));
    run(
        &mut g,
        &[
            Action::Hand,
            Action::Announce(Announcement::plain(GameType::Null)),
        ],
    );
    play(&mut g, "CJ");
    // MH passed without a bid: not eligible.
    assert!(!has_kontra(&g));
    play(&mut g, "S7");
    assert!(has_kontra(&g));
    run(&mut g, &[Action::Kontra, Action::KeinRe]);
    assert_eq!(g.kontra(), KontraLevel::Kontra);
    assert_eq!(g.current_seat(), Some(Seat::Hinterhand));
    play(&mut g, "HQ"); // VH wins with CJ: Null Hand lost.
    let r = *g.result().unwrap();
    // 35 * -2 * 2
    assert_eq!(r.score, -140);
    assert_eq!(r.seeger_fabian(), [-190, 40, 40]);
}

#[test]
fn re_is_allowed_in_null() {
    let mut g = game(standard_deal(), true);
    run(&mut g, &VH_AT_18);
    run(
        &mut g,
        &[
            Action::Hand,
            Action::Announce(Announcement::plain(GameType::Null)),
        ],
    );
    play(&mut g, "C8");
    run(&mut g, &[Action::Kontra, Action::Re]);
    assert_eq!(g.kontra(), KontraLevel::Re);
}

#[test]
fn hinterhand_exception_and_one_kontra_per_game() {
    let mut g = game(standard_deal(), true);
    run(&mut g, &MH_AT_20);
    assert_eq!(g.declarer(), Some(Seat::Mittelhand));
    run(
        &mut g,
        &[
            Action::Hand,
            Action::Announce(Announcement::plain(GameType::Farbe(Suit::Pik))),
        ],
    );
    // VH leads and may say Kontra before its first card; it plays instead.
    assert!(has_kontra(&g));
    let mut hinterhand_offered = false;
    play_out(&mut g, |g| {
        let seat = g.current_seat().unwrap();
        match seat {
            // VH has played a card from now on (it led the first trick).
            Seat::Vorhand if !g.tricks().is_empty() || !g.current_trick().is_empty() => {
                assert!(!has_kontra(g))
            }
            Seat::Mittelhand => assert!(!has_kontra(g)),
            Seat::Hinterhand if g.tricks().is_empty() => {
                hinterhand_offered |= has_kontra(g);
            }
            Seat::Hinterhand => assert!(!has_kontra(g)),
            _ => {}
        }
    });
    assert!(hinterhand_offered);

    // After Kontra by VH, HH may not say Kontra.
    let mut g = game(standard_deal(), true);
    run(&mut g, &MH_AT_20);
    run(
        &mut g,
        &[
            Action::Hand,
            Action::Announce(Announcement::plain(GameType::Farbe(Suit::Pik))),
        ],
    );
    run(&mut g, &[Action::Kontra, Action::KeinRe]);
    assert_eq!(g.current_seat(), Some(Seat::Vorhand));
    play_out(&mut g, |g| assert!(!has_kontra(g)));
    assert_eq!(g.result().unwrap().outcome.kontra, KontraLevel::Kontra);
}

#[test]
fn no_defender_eligible_after_vorhand_alone() {
    let mut g = game(standard_deal(), true);
    run(
        &mut g,
        &[Action::Pass, Action::Pass, Action::Bid(18), Action::Hand],
    );
    run(&mut g, &[Action::Announce(Announcement::plain(KREUZ))]);
    play_out(&mut g, |g| assert!(!has_kontra(g)));
}

#[test]
fn ouvert_round_before_reveal() {
    let grand_ouvert = Announcement {
        schneider: true,
        schwarz: true,
        ouvert: true,
        ..Announcement::plain(GameType::Grand)
    };
    let mut g = game(standard_deal(), true);
    run(&mut g, &MH_AT_20);
    run(&mut g, &[Action::Hand]);
    let events = run(&mut g, &[Action::Announce(grand_ouvert)]);
    assert_eq!(events.len(), 1, "no reveal before the Kontra round");
    assert!(!g.is_revealed());
    assert_eq!(g.view(Seat::Vorhand).declarer_cards, None);
    // Seat order: VH first, then HH.
    assert_eq!(g.current_seat(), Some(Seat::Vorhand));
    assert_eq!(g.legal_actions(), vec![Action::Kontra, Action::KeinKontra]);
    assert_rejected(
        &mut g,
        Action::Play(c("CJ")),
        RuleError::UnexpectedAction {
            phase: Phase::Spiel,
        },
    );
    run(&mut g, &[Action::KeinKontra]);
    assert_eq!(g.current_seat(), Some(Seat::Hinterhand));
    run(&mut g, &[Action::Kontra]);
    assert_eq!(g.current_seat(), Some(Seat::Mittelhand));
    assert!(!g.is_revealed());
    let events = run(&mut g, &[Action::Re]);
    let mh_hand = cards("SA ST SK SQ S9 S8 S7 HA HT HK");
    assert_eq!(
        events,
        vec![
            Event::Re {
                seat: Seat::Mittelhand
            },
            Event::Revealed {
                declarer: Seat::Mittelhand,
                cards: mh_hand
            },
        ]
    );
    for seat in Seat::ALL {
        assert_eq!(g.view(seat).declarer_cards, Some(mh_hand));
    }
    // Play starts with VH; no Kontra during Ouvert play.
    assert_eq!(g.current_seat(), Some(Seat::Vorhand));
    assert!(!has_kontra(&g));
    play(&mut g, "CJ");
    assert_eq!(g.view(Seat::Hinterhand).declarer_cards, Some(mh_hand));
    play(&mut g, "SA");
    assert_eq!(
        g.view(Seat::Vorhand).declarer_cards,
        Some(mh_hand - cards("SA"))
    );
}

#[test]
fn ouvert_kontra_by_first_defender_ends_the_round() {
    let null_ouvert = Announcement {
        ouvert: true,
        ..Announcement::plain(GameType::Null)
    };
    let mut g = game(standard_deal(), true);
    run(&mut g, &MH_AT_20);
    run(
        &mut g,
        &[
            Action::TakeSkat,
            Action::Discard(cards("C7 D7")),
            Action::Announce(null_ouvert),
        ],
    );
    run(&mut g, &[Action::Kontra]);
    let events = run(&mut g, &[Action::KeinRe]);
    assert!(matches!(events[1], Event::Revealed { .. }));
    assert_eq!(g.current_seat(), Some(Seat::Vorhand));
    assert!(
        g.legal_actions()
            .iter()
            .all(|a| matches!(a, Action::Play(_)))
    );
}

#[test]
fn ouvert_reveals_at_once_without_kontra_round() {
    let null_ouvert = Announcement {
        ouvert: true,
        ..Announcement::plain(GameType::Null)
    };
    // Kontra/Re switched off.
    let mut g = game(standard_deal(), false);
    run(&mut g, &MH_AT_20);
    run(&mut g, &[Action::Hand]);
    let events = run(&mut g, &[Action::Announce(null_ouvert)]);
    assert!(matches!(
        events[1],
        Event::Revealed {
            declarer: Seat::Mittelhand,
            ..
        }
    ));
    // Switched on, but no eligible defender.
    let mut g = game(standard_deal(), true);
    run(
        &mut g,
        &[Action::Pass, Action::Pass, Action::Bid(18), Action::Hand],
    );
    run(&mut g, &[Action::Announce(null_ouvert)]);
    assert!(g.is_revealed());
    assert_eq!(g.current_seat(), Some(Seat::Vorhand));
}

#[test]
fn eligibility_from_public_history_matches_the_game() {
    let mut g = game(standard_deal(), true);
    run(&mut g, &MH_AT_20);
    let history = g.bidding_history();
    assert!(skat_core::kontra_eligible(history, Seat::Vorhand));
    assert!(skat_core::kontra_eligible(history, Seat::Hinterhand));
    let mut g = game(standard_deal(), true);
    run(&mut g, &VH_AT_18);
    let history = g.bidding_history();
    assert!(skat_core::kontra_eligible(history, Seat::Mittelhand));
    assert!(!skat_core::kontra_eligible(history, Seat::Hinterhand));
    // An invalid history gives no eligibility.
    let bogus = [skat_core::BidRecord {
        seat: Seat::Hinterhand,
        call: skat_core::Call::Bid(18),
    }];
    assert!(!skat_core::kontra_eligible(&bogus, Seat::Hinterhand));
}
