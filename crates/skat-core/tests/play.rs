// SPDX-License-Identifier: EUPL-1.2

//! Card order, Trumpf, Bedienen, Stich winners, Skat handling, announcements
//! and complete games on fixed deals.

mod common;

use common::*;
use skat_core::{
    Action, Announcement, Event, GameType, KontraLevel, Phase, RuleError, Seat, Spitzen, Suit,
    legal_cards, trick_winner,
};

const KREUZ: GameType = GameType::Farbe(Suit::Kreuz);
const HERZ: GameType = GameType::Farbe(Suit::Herz);
const KARO: GameType = GameType::Farbe(Suit::Karo);

fn winner(game: GameType, trick: &str) -> usize {
    let v: Vec<_> = trick.split_whitespace().map(c).collect();
    trick_winner(game, [v[0], v[1], v[2]])
}

#[test]
fn jacks_order_and_beat_the_trump_suit() {
    assert_eq!(winner(HERZ, "DJ HJ SJ"), 2);
    assert_eq!(winner(HERZ, "SJ CJ HJ"), 1);
    assert_eq!(winner(HERZ, "HA DJ HT"), 1);
    assert_eq!(winner(GameType::Grand, "DJ CJ HA"), 1);
    assert_eq!(winner(GameType::Grand, "HA H7 DJ"), 2);
}

#[test]
fn suit_order_ace_ten_king() {
    assert_eq!(winner(HERZ, "HT HA HK"), 1);
    assert_eq!(winner(KREUZ, "SK ST S9"), 1);
    assert_eq!(winner(KREUZ, "SQ SK SA"), 2);
    assert_eq!(winner(KREUZ, "S9 S8 S7"), 0);
    assert_eq!(winner(GameType::Grand, "D7 DQ D8"), 1);
}

#[test]
fn trump_beats_side_suit_and_off_suit_never_wins() {
    assert_eq!(winner(HERZ, "SA ST H7"), 2);
    assert_eq!(winner(HERZ, "S7 CA DA"), 0);
    assert_eq!(winner(KARO, "CA D7 HA"), 1);
    assert_eq!(winner(GameType::Grand, "S7 SA HJ"), 2);
}

#[test]
fn null_order_has_no_trumps_and_jack_between_queen_and_ten() {
    assert_eq!(winner(GameType::Null, "HJ HQ HT"), 1);
    assert_eq!(winner(GameType::Null, "HT HJ H9"), 1);
    assert_eq!(winner(GameType::Null, "H7 HA SA"), 1);
    assert_eq!(winner(GameType::Null, "D8 CJ D7"), 0);
    assert_eq!(winner(GameType::Null, "DK DA DQ"), 1);
}

#[test]
fn bedienen_counts_jacks_as_trumps() {
    let hand = cards("DJ DA C7");
    // Karo led in a Herz game: the Karo Bube is Trumpf, not Karo.
    assert_eq!(legal_cards(HERZ, hand, Some(c("D7"))), cards("DA"));
    assert_eq!(
        legal_cards(HERZ, cards("DJ C7"), Some(c("D7"))),
        cards("DJ C7")
    );
    // Trumpf led: Jacks and trump suit follow.
    assert_eq!(
        legal_cards(HERZ, cards("DJ HA C7"), Some(c("HJ"))),
        cards("DJ HA")
    );
    assert_eq!(
        legal_cards(HERZ, cards("DJ HA C7"), Some(c("H7"))),
        cards("DJ HA")
    );
    // Grand: a Jack led demands a Jack only.
    assert_eq!(
        legal_cards(GameType::Grand, cards("DJ HA C7"), Some(c("CJ"))),
        cards("DJ")
    );
    assert_eq!(
        legal_cards(GameType::Grand, cards("DJ HA C7"), Some(c("H7"))),
        cards("HA")
    );
    // Null: Jacks belong to their suit.
    assert_eq!(
        legal_cards(GameType::Null, cards("DJ HA C7"), Some(c("D7"))),
        cards("DJ")
    );
    assert_eq!(
        legal_cards(GameType::Null, cards("DJ HA C7"), Some(c("CJ"))),
        cards("C7")
    );
    // Leading: anything.
    assert_eq!(legal_cards(KARO, hand, None), hand);
}

#[test]
fn full_kreuz_game_schwarz_with_eleven() {
    let mut g = game(standard_deal(), false);
    run(
        &mut g,
        &[
            Action::Pass,
            Action::Pass,
            Action::Bid(18),
            Action::TakeSkat,
        ],
    );
    assert_eq!(g.phase(), Phase::Druecken);
    assert_eq!(g.view(Seat::Vorhand).hand.len(), 12);
    run(&mut g, &[Action::Discard(cards("C7 D7"))]);
    run(&mut g, &[Action::Announce(Announcement::plain(KREUZ))]);
    assert_eq!(g.current_seat(), Some(Seat::Vorhand));
    let events = play(
        &mut g,
        "CJ SA HQ  SJ ST H9  HJ SK H8  DJ SQ H7  CA S9 DA \
         CT S8 DT  CK S7 DK  CQ HA DQ  C9 HT D9  C8 HK D8",
    );
    assert_eq!(g.phase(), Phase::Beendet);
    assert!(
        g.tricks()
            .iter()
            .all(|t| t.winner == Seat::Vorhand && t.leader == Seat::Vorhand)
    );
    let r = *g.result().unwrap();
    assert_eq!(events.last(), Some(&Event::GameFinished { result: r }));
    assert_eq!(
        r.outcome.spitzen,
        Some(Spitzen {
            mit: true,
            count: 11
        })
    );
    assert_eq!(r.outcome.declarer_augen, 120);
    assert_eq!(r.outcome.declarer_tricks, 10);
    assert!(r.won && r.schneider && r.schwarz && !r.ueberreizt);
    // 12 * (11 + game + Schneider + Schwarz)
    assert_eq!(r.game_value, 168);
    assert_eq!(r.score, 168);
    assert_eq!(r.seeger_fabian(), [218, 0, 0]);
}

/// Hinterhand plays Karo ohne 4 and loses with 47.
#[test]
fn full_karo_game_lost_with_changing_leads() {
    let mut g = game(standard_deal(), false);
    run(&mut g, &[Action::Pass, Action::Bid(18), Action::Pass]);
    assert_eq!(g.declarer(), Some(Seat::Hinterhand));
    run(&mut g, &[Action::TakeSkat, Action::Discard(cards("C7 D7"))]);
    run(&mut g, &[Action::Announce(Announcement::plain(KARO))]);
    // Vorhand leads the first trick even though Hinterhand is declarer.
    assert_eq!(g.current_seat(), Some(Seat::Vorhand));
    play(&mut g, "CJ");
    assert_rejected(
        &mut g,
        Action::Play(c("CA")),
        RuleError::CardNotInHand { card: c("CA") },
    );
    play(&mut g, "S7");
    // Hinterhand holds Karo, which is Trumpf, so it must follow.
    assert_rejected(
        &mut g,
        Action::Play(c("HQ")),
        RuleError::MustFollowSuit { card: c("HQ") },
    );
    play(
        &mut g,
        "D8  SJ S8 D9  HJ S9 DQ  DJ SQ DK  CA SK DT \
         H7 C8 HA  SA DA C9  HQ CQ HT  ST H8 CK  HK H9 CT",
    );
    let expected = [
        (Seat::Vorhand, Seat::Vorhand),
        (Seat::Vorhand, Seat::Vorhand),
        (Seat::Vorhand, Seat::Vorhand),
        (Seat::Vorhand, Seat::Vorhand),
        (Seat::Vorhand, Seat::Hinterhand),
        (Seat::Hinterhand, Seat::Mittelhand),
        (Seat::Mittelhand, Seat::Hinterhand),
        (Seat::Hinterhand, Seat::Mittelhand),
        (Seat::Mittelhand, Seat::Mittelhand),
        (Seat::Mittelhand, Seat::Mittelhand),
    ];
    let actual: Vec<_> = g.tricks().iter().map(|t| (t.leader, t.winner)).collect();
    assert_eq!(actual, expected);
    let r = *g.result().unwrap();
    assert_eq!(
        r.outcome.spitzen,
        Some(Spitzen {
            mit: false,
            count: 4
        })
    );
    assert_eq!(r.outcome.declarer_augen, 47);
    assert_eq!(r.outcome.declarer_tricks, 2);
    assert!(!r.won && !r.schneider && !r.ueberreizt);
    assert_eq!(r.game_value, 45);
    assert_eq!(r.score, -90);
    assert_eq!(r.seeger_fabian(), [40, 40, -140]);
}

#[test]
fn null_ends_when_declarer_takes_a_trick() {
    let mut g = game(standard_deal(), false);
    run(
        &mut g,
        &[Action::Bid(23), Action::Pass, Action::Pass, Action::Hand],
    );
    assert_eq!(g.declarer(), Some(Seat::Mittelhand));
    run(
        &mut g,
        &[Action::Announce(Announcement::plain(GameType::Null))],
    );
    play(&mut g, "CJ S7 HQ  HJ HA H9");
    assert_eq!(g.phase(), Phase::Beendet);
    assert_eq!(g.tricks().len(), 2);
    assert_eq!(g.view(Seat::Vorhand).hand.len(), 8);
    let r = *g.result().unwrap();
    assert_eq!((r.outcome.tricks_played, r.outcome.declarer_tricks), (2, 1));
    assert!(!r.won);
    assert_eq!(r.game_value, 35);
    assert_eq!(r.score, -70);
    assert_eq!(r.seeger_fabian(), [40, -120, 40]);
    assert_rejected(&mut g, Action::Play(c("CA")), RuleError::GameOver);
}

#[test]
fn null_won_after_ten_tricks() {
    // Mittelhand holds only low cards and is void nowhere it matters.
    let d = deal(
        "CA CK CQ CJ SA SK SQ SJ HA HK",
        "C7 C8 C9 S7 S8 S9 H7 H8 H9 D7",
        "CT ST HT HQ HJ DA DK DQ DJ DT",
        "D9 D8",
    );
    let mut g = game(d, false);
    run(
        &mut g,
        &[
            Action::Bid(18),
            Action::Pass,
            Action::Pass,
            Action::TakeSkat,
        ],
    );
    run(&mut g, &[Action::Discard(cards("D9 D8"))]);
    run(
        &mut g,
        &[Action::Announce(Announcement::plain(GameType::Null))],
    );
    play(
        &mut g,
        "CA C7 CT  CK C8 HT  CQ C9 ST  CJ S7 HQ  SA S8 HJ \
         SK S9 DA  SQ H7 DK  SJ H8 DQ  HA H9 DJ  HK D7 DT",
    );
    let r = *g.result().unwrap();
    assert_eq!(r.outcome.tricks_played, 10);
    assert!(r.won);
    assert_eq!(r.score, 23);
    assert_eq!(r.seeger_fabian(), [0, 73, 0]);
}

#[test]
fn hand_game_keeps_the_skat_for_the_declarer() {
    let mut g = game(standard_deal(), false);
    run(
        &mut g,
        &[Action::Pass, Action::Pass, Action::Bid(18), Action::Hand],
    );
    assert_eq!(g.phase(), Phase::Ansage);
    run(&mut g, &[Action::Announce(Announcement::plain(KREUZ))]);
    play(
        &mut g,
        "CJ SA HQ  SJ ST H9  HJ SK H8  DJ SQ H7  CA S9 DA \
         CT S8 DT  CK S7 DK  CQ HA DQ  C9 HT D9  C8 HK D8",
    );
    let r = *g.result().unwrap();
    assert_eq!(r.outcome.declarer_augen, 120);
    // 12 * (11 + game + Hand + Schneider + Schwarz)
    assert_eq!(r.score, 180);
}

#[test]
fn discard_must_be_two_own_cards() {
    let mut g = game(standard_deal(), false);
    run(
        &mut g,
        &[
            Action::Pass,
            Action::Pass,
            Action::Bid(18),
            Action::TakeSkat,
        ],
    );
    assert_eq!(g.legal_actions().len(), 66);
    for bad in ["C7", "C7 D7 C8", "C7 SA", ""] {
        assert_rejected(
            &mut g,
            Action::Discard(cards(bad)),
            RuleError::InvalidDiscard,
        );
    }
    assert_rejected(
        &mut g,
        Action::Hand,
        RuleError::UnexpectedAction {
            phase: Phase::Druecken,
        },
    );
    // Any two cards may be pushed, including Jacks.
    run(&mut g, &[Action::Discard(cards("CJ CA"))]);
    assert_eq!(g.view(Seat::Vorhand).own_skat, Some(cards("CJ CA")));
}

#[test]
fn announcement_rules() {
    let null = Announcement::plain(GameType::Null);
    let schneider = Announcement {
        schneider: true,
        ..Announcement::plain(KREUZ)
    };
    let schwarz_only = Announcement {
        schwarz: true,
        ..Announcement::plain(KREUZ)
    };
    let ouvert_plain = Announcement {
        ouvert: true,
        ..Announcement::plain(GameType::Grand)
    };
    let null_ouvert = Announcement {
        ouvert: true,
        ..null
    };

    // Skat picked up, bid 24.
    let mut g = game(standard_deal(), false);
    run(
        &mut g,
        &[
            Action::Bid(24),
            Action::Pass,
            Action::Pass,
            Action::TakeSkat,
        ],
    );
    run(&mut g, &[Action::Discard(cards("SA ST"))]);
    assert_rejected(
        &mut g,
        Action::Announce(schneider),
        RuleError::InvalidAnnouncement,
    );
    assert_rejected(
        &mut g,
        Action::Announce(ouvert_plain),
        RuleError::InvalidAnnouncement,
    );
    assert_rejected(
        &mut g,
        Action::Announce(null),
        RuleError::NullValueBelowBid { value: 23, bid: 24 },
    );
    // Null Ouvert is allowed after pickup (46).
    assert!(g.legal_actions().contains(&Action::Announce(null_ouvert)));
    assert_eq!(g.legal_actions().len(), 6);

    // Hand, bid 18.
    let mut g = game(standard_deal(), false);
    run(
        &mut g,
        &[Action::Bid(18), Action::Pass, Action::Pass, Action::Hand],
    );
    assert_rejected(
        &mut g,
        Action::Announce(schwarz_only),
        RuleError::InvalidAnnouncement,
    );
    assert_rejected(
        &mut g,
        Action::Announce(ouvert_plain),
        RuleError::InvalidAnnouncement,
    );
    let schneider_null = Announcement {
        schneider: true,
        ..null
    };
    assert_rejected(
        &mut g,
        Action::Announce(schneider_null),
        RuleError::InvalidAnnouncement,
    );
    assert_eq!(g.legal_actions().len(), 22);
    let events = run(&mut g, &[Action::Announce(schneider)]);
    let contract = g.contract().unwrap();
    assert!(contract.hand && contract.schneider && !contract.schwarz);
    assert_eq!(
        events,
        vec![Event::Announced {
            declarer: Seat::Mittelhand,
            contract
        }]
    );
    assert_eq!(g.kontra(), KontraLevel::None);
}

#[test]
fn null_hand_values_against_high_bids() {
    let mut g = game(standard_deal(), false);
    run(
        &mut g,
        &[Action::Bid(48), Action::Pass, Action::Pass, Action::Hand],
    );
    let null_ouvert_hand = Announcement {
        ouvert: true,
        ..Announcement::plain(GameType::Null)
    };
    assert!(
        g.legal_actions()
            .contains(&Action::Announce(null_ouvert_hand))
    );
    assert_rejected(
        &mut g,
        Action::Announce(Announcement::plain(GameType::Null)),
        RuleError::NullValueBelowBid { value: 35, bid: 48 },
    );
}
