// SPDX-License-Identifier: EUPL-1.2

//! Reizen and Einpassen (ISkO 3.3) on fixed deals.

mod common;

use common::*;
use skat_core::{Action, BidRecord, Call, Event, Phase, RuleError, Seat, is_reizwert, reizwerte};

#[test]
fn reizwerte_are_the_isko_table() {
    // ISkO 5.6.1, written out by hand.
    let expected: Vec<u16> = vec![
        18, 20, 22, 23, 24, 27, 30, 33, 35, 36, 40, 44, 45, 46, 48, 50, 54, 55, 59, 60, 63, 66, 70,
        72, 77, 80, 81, 84, 88, 90, 96, 99, 100, 108, 110, 117, 120, 121, 126, 130, 132, 135, 140,
        143, 144, 150, 153, 154, 156, 160, 162, 165, 168, 170, 176, 180, 187, 192, 198, 204, 216,
        240, 264,
    ];
    assert_eq!(reizwerte(), expected);
    for value in [0, 17, 19, 21, 25, 26, 28, 191, 228, 252, 265, 288] {
        assert!(!is_reizwert(value), "{value}");
    }
}

#[test]
fn all_pass_is_eingepasst() {
    let mut g = game(standard_deal(), false);
    assert_eq!(g.current_seat(), Some(Seat::Mittelhand));
    run(&mut g, &[Action::Pass]);
    assert_eq!(g.current_seat(), Some(Seat::Hinterhand));
    run(&mut g, &[Action::Pass]);
    assert_eq!(g.current_seat(), Some(Seat::Vorhand));
    assert_eq!(g.legal_actions(), vec![Action::Bid(18), Action::Pass]);
    let events = run(&mut g, &[Action::Pass]);
    assert_eq!(
        events,
        vec![
            Event::Passed {
                seat: Seat::Vorhand
            },
            Event::Eingepasst
        ]
    );
    assert_eq!(g.phase(), Phase::Eingepasst);
    assert_eq!(g.current_seat(), None);
    assert!(g.legal_actions().is_empty());
    assert!(g.result().is_none());
    assert_rejected(&mut g, Action::Bid(18), RuleError::GameOver);
}

#[test]
fn vorhand_plays_18_after_two_passes() {
    let mut g = game(standard_deal(), false);
    run(&mut g, &[Action::Pass, Action::Pass]);
    assert_rejected(
        &mut g,
        Action::Bid(20),
        RuleError::InvalidBidValue { value: 20 },
    );
    assert_rejected(
        &mut g,
        Action::Hold,
        RuleError::UnexpectedAction {
            phase: Phase::Reizen,
        },
    );
    let events = run(&mut g, &[Action::Bid(18)]);
    assert_eq!(
        events[1],
        Event::BiddingWon {
            declarer: Seat::Vorhand,
            bid: 18
        }
    );
    assert_eq!(g.phase(), Phase::SkatAufnahme);
    assert_eq!(g.current_seat(), Some(Seat::Vorhand));
}

#[test]
fn mittelhand_against_vorhand_then_hinterhand() {
    let mut g = game(standard_deal(), false);
    // MH 18, VH yes, MH 20, VH passes; HH passes: MH plays at 20.
    run(&mut g, &[Action::Bid(18)]);
    assert_eq!(g.current_seat(), Some(Seat::Vorhand));
    assert_eq!(g.legal_actions(), vec![Action::Hold, Action::Pass]);
    run(&mut g, &[Action::Hold, Action::Bid(20), Action::Pass]);
    assert_eq!(g.current_seat(), Some(Seat::Hinterhand));
    // Hinterhand continues above the last value (ISkO 3.3.4).
    assert_eq!(g.legal_actions().first(), Some(&Action::Bid(22)));
    let events = run(&mut g, &[Action::Pass]);
    assert_eq!(
        events.last(),
        Some(&Event::BiddingWon {
            declarer: Seat::Mittelhand,
            bid: 20
        })
    );
    assert_eq!(
        g.bidding_history(),
        &[
            BidRecord {
                seat: Seat::Mittelhand,
                call: Call::Bid(18)
            },
            BidRecord {
                seat: Seat::Vorhand,
                call: Call::Hold(18)
            },
            BidRecord {
                seat: Seat::Mittelhand,
                call: Call::Bid(20)
            },
            BidRecord {
                seat: Seat::Vorhand,
                call: Call::Pass
            },
            BidRecord {
                seat: Seat::Hinterhand,
                call: Call::Pass
            },
        ]
    );
}

#[test]
fn hinterhand_outbids_the_holder() {
    let mut g = game(standard_deal(), false);
    // MH passes at once; HH bids to VH, VH holds 18 and 23, passes at 30.
    run(
        &mut g,
        &[
            Action::Pass,
            Action::Bid(18),
            Action::Hold,
            Action::Bid(23),
            Action::Hold,
        ],
    );
    assert_eq!(g.bid(), 23);
    let events = run(&mut g, &[Action::Bid(30), Action::Pass]);
    assert_eq!(
        events.last(),
        Some(&Event::BiddingWon {
            declarer: Seat::Hinterhand,
            bid: 30
        })
    );
}

#[test]
fn holder_wins_when_asker_passes() {
    let mut g = game(standard_deal(), false);
    run(&mut g, &[Action::Bid(18), Action::Pass]); // VH passes, MH survives at 18
    run(
        &mut g,
        &[Action::Bid(20), Action::Hold, Action::Bid(24), Action::Hold],
    );
    let events = run(&mut g, &[Action::Pass]);
    assert_eq!(
        events.last(),
        Some(&Event::BiddingWon {
            declarer: Seat::Mittelhand,
            bid: 24
        })
    );
}

#[test]
fn jump_bids_are_allowed() {
    let mut g = game(standard_deal(), false);
    run(&mut g, &[Action::Bid(48), Action::Pass, Action::Pass]);
    assert_eq!(g.declarer(), Some(Seat::Mittelhand));
    assert_eq!(g.bid(), 48);
}

#[test]
fn invalid_bids_are_rejected() {
    let mut g = game(standard_deal(), false);
    assert_rejected(
        &mut g,
        Action::Bid(19),
        RuleError::InvalidBidValue { value: 19 },
    );
    assert_rejected(
        &mut g,
        Action::Bid(300),
        RuleError::InvalidBidValue { value: 300 },
    );
    assert_rejected(
        &mut g,
        Action::Hold,
        RuleError::UnexpectedAction {
            phase: Phase::Reizen,
        },
    );
    assert_rejected(
        &mut g,
        Action::TakeSkat,
        RuleError::UnexpectedAction {
            phase: Phase::Reizen,
        },
    );
    run(&mut g, &[Action::Bid(20)]);
    assert_rejected(
        &mut g,
        Action::Bid(22),
        RuleError::UnexpectedAction {
            phase: Phase::Reizen,
        },
    );
    run(&mut g, &[Action::Hold]);
    assert_rejected(
        &mut g,
        Action::Bid(20),
        RuleError::BidTooLow {
            value: 20,
            current: 20,
        },
    );
    assert_rejected(
        &mut g,
        Action::Bid(18),
        RuleError::BidTooLow {
            value: 18,
            current: 20,
        },
    );
    assert!(!g.legal_actions().contains(&Action::Bid(20)));
    assert!(g.legal_actions().contains(&Action::Bid(22)));
}
