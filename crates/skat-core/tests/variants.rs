// SPDX-License-Identifier: EUPL-1.2

//! Every game variant played through on a fixed deal: bidding, announcement,
//! card play and settlement. The expected numbers were worked out by hand from
//! the ISkO multiplier (Spitzen + 1 + Hand + Schneider + Schneider angesagt +
//! Schwarz + Schwarz angesagt + Ouvert) times the base value; none of them is
//! read back from the engine. `docs/decisions.md` (D-28) maps variants to
//! tests.

mod common;

use common::*;
use skat_core::{Action, Announcement, Deal, Game, GameResult, GameType, Phase, Spitzen, Suit};

const KARO: GameType = GameType::Farbe(Suit::Karo);
const HERZ: GameType = GameType::Farbe(Suit::Herz);
const PIK: GameType = GameType::Farbe(Suit::Pik);
const KREUZ: GameType = GameType::Farbe(Suit::Kreuz);

fn plain(game: GameType) -> Announcement {
    Announcement::plain(game)
}

fn schneider(game: GameType) -> Announcement {
    Announcement {
        schneider: true,
        ..plain(game)
    }
}

fn schwarz(game: GameType) -> Announcement {
    Announcement {
        schneider: true,
        schwarz: true,
        ..plain(game)
    }
}

fn ouvert(game: GameType) -> Announcement {
    Announcement {
        ouvert: true,
        ..schwarz(game)
    }
}

fn null_ouvert() -> Announcement {
    Announcement {
        ouvert: true,
        ..plain(GameType::Null)
    }
}

/// Bidding, announcement and all cards; returns the finished game.
fn play_game(deal: Deal, bidding: &[Action], announce: Announcement, cards: &str) -> Game {
    let mut g = game(deal, false);
    run(&mut g, bidding);
    run(&mut g, &[Action::Announce(announce)]);
    play(&mut g, cards);
    assert_eq!(g.phase(), Phase::Beendet);
    g
}

/// Checks won, game value, list score and Seeger-Fabian points.
fn check(g: &Game, won: bool, game_value: u16, score: i32, sf: [i32; 3]) -> GameResult {
    let r = *g.result().expect("finished game");
    assert_eq!(r.won, won, "won");
    assert_eq!(r.game_value, game_value, "game value");
    assert_eq!(r.score, score, "score");
    assert_eq!(r.seeger_fabian(), sf, "Seeger-Fabian");
    r
}

const VH_PICKUP: [Action; 4] = [
    Action::Pass,
    Action::Pass,
    Action::Bid(18),
    Action::TakeSkat,
];
const VH_HAND: [Action; 4] = [Action::Pass, Action::Pass, Action::Bid(18), Action::Hand];

fn pickup(discard: &str) -> Vec<Action> {
    let mut v = VH_PICKUP.to_vec();
    v.push(Action::Discard(cards(discard)));
    v
}

// ---------------------------------------------------------------- Grand

/// Standard deal, Vorhand declarer with four Jacks; wins all ten tricks,
/// 120 Augen. Spitzen: mit 4.
const STD_ALL: &str = "CJ SA HQ  SJ ST H9  HJ SK H8  DJ SQ H7  CA S9 DA \
                       CT S8 DT  CK S7 DK  CQ HA DQ  C9 HT D9  C8 HK D8";

fn grand(bidding: &[Action], announce: Announcement) -> Game {
    let g = play_game(standard_deal(), bidding, announce, STD_ALL);
    let r = *g.result().unwrap();
    assert_eq!(
        r.outcome.spitzen,
        Some(Spitzen {
            mit: true,
            count: 4
        })
    );
    assert_eq!(r.outcome.declarer_augen, 120);
    assert_eq!(r.outcome.declarer_tricks, 10);
    g
}

#[test]
fn grand_with_pickup_wins_schwarz() {
    // 24 * (4 + 1 + Schneider + Schwarz) = 24 * 7
    let g = grand(&pickup("C7 D7"), plain(GameType::Grand));
    let r = check(&g, true, 168, 168, [218, 0, 0]);
    assert!(r.schneider && r.schwarz && !r.ueberreizt);
}

#[test]
fn grand_hand_with_schneider_announced() {
    // 24 * (4 + 1 + Hand + Schneider + angesagt + Schwarz) = 24 * 9
    let g = grand(&VH_HAND, schneider(GameType::Grand));
    check(&g, true, 216, 216, [266, 0, 0]);
}

#[test]
fn grand_hand_with_schwarz_announced() {
    // 24 * (4 + 1 + Hand + 2 * Schneider + 2 * Schwarz) = 24 * 10
    let g = grand(&VH_HAND, schwarz(GameType::Grand));
    check(&g, true, 240, 240, [290, 0, 0]);
}

#[test]
fn grand_ouvert() {
    // 24 * (4 + 1 + Hand + 2 * Schneider + 2 * Schwarz + Ouvert) = 24 * 11
    let g = grand(&VH_HAND, ouvert(GameType::Grand));
    check(&g, true, 264, 264, [314, 0, 0]);
    assert!(g.is_revealed());
}

// ---------------------------------------------------------------- Farben

/// Herz deal: Vorhand holds all Jacks and the top seven Herz (Skat: H7 D7),
/// the defenders hold no Trumpf. Wins all ten tricks, 120 Augen, mit 11.
fn herz_deal() -> Deal {
    deal(
        "CJ SJ HJ DJ HA HT HK HQ H9 H8",
        "CA CT CK CQ C9 C8 C7 SA ST SK",
        "SQ S9 S8 S7 DA DT DK DQ D9 D8",
        "H7 D7",
    )
}

const HERZ_ALL: &str = "CJ CA SQ  SJ CT S9  HJ CK S8  DJ CQ S7  HA C9 DA \
                        HT C8 DT  HK C7 DK  HQ SA DQ  H9 ST D9  H8 SK D8";

fn herz(bidding: &[Action], announce: Announcement) -> Game {
    let g = play_game(herz_deal(), bidding, announce, HERZ_ALL);
    let r = *g.result().unwrap();
    assert_eq!(
        r.outcome.spitzen,
        Some(Spitzen {
            mit: true,
            count: 11
        })
    );
    assert_eq!(r.outcome.declarer_augen, 120);
    g
}

#[test]
fn herz_with_pickup() {
    // 10 * (11 + 1 + Schneider + Schwarz) = 10 * 14; discarding the Skat
    // cards again keeps the ten cards of the deal.
    let g = herz(&pickup("H7 D7"), plain(HERZ));
    check(&g, true, 140, 140, [190, 0, 0]);
}

#[test]
fn herz_hand_with_schwarz_announced() {
    // 10 * (11 + 1 + Hand + 2 * Schneider + 2 * Schwarz) = 10 * 17
    let g = herz(&VH_HAND, schwarz(HERZ));
    check(&g, true, 170, 170, [220, 0, 0]);
}

#[test]
fn herz_ouvert() {
    // 10 * (11 + 1 + Hand + 2 * Schneider + 2 * Schwarz + Ouvert) = 10 * 18
    let g = herz(&VH_HAND, ouvert(HERZ));
    check(&g, true, 180, 180, [230, 0, 0]);
}

/// Pik deal: as the Herz deal with Pik as Trumpf. Skat: S7 D7.
#[test]
fn pik_hand_with_schneider_announced() {
    let d = deal(
        "CJ SJ HJ DJ SA ST SK SQ S9 S8",
        "CA CT CK CQ C9 C8 C7 HA HT HK",
        "HQ H9 H8 H7 DA DT DK DQ D9 D8",
        "S7 D7",
    );
    let g = play_game(
        d,
        &VH_HAND,
        schneider(PIK),
        "CJ CA HQ  SJ CT H9  HJ CK H8  DJ CQ H7  SA C9 DA \
         ST C8 DT  SK C7 DK  SQ HA DQ  S9 HT D9  S8 HK D8",
    );
    // 11 * (11 + 1 + Hand + 2 * Schneider + Schwarz) = 11 * 16
    let r = check(&g, true, 176, 176, [226, 0, 0]);
    assert_eq!(r.outcome.declarer_augen, 120);
}

#[test]
fn kreuz_ouvert() {
    // Kreuz with and without Hand is played in play.rs. Here, mit 11:
    // 12 * (11 + 1 + Hand + 2 * Schneider + 2 * Schwarz + Ouvert) = 12 * 18
    let g = play_game(standard_deal(), &VH_HAND, ouvert(KREUZ), STD_ALL);
    check(&g, true, 216, 216, [266, 0, 0]);
}

/// Vorhand loses the first trick to Mittelhand's Herz Dame, the only
/// Trumpf of the defence, and wins the other nine: 106 Augen, not Schwarz.
/// Spitzen: mit 7 (Herz Dame is missing).
fn herz_one_trick_lost(announce: Announcement) -> Game {
    let d = deal(
        "CJ SJ HJ DJ HA HT HK H9 H8 H7",
        "HQ CA CT CK CQ C9 C8 SA ST SK",
        "SQ S9 S8 S7 DA DT DK DQ D9 D8",
        "C7 D7",
    );
    let g = play_game(
        d,
        &VH_HAND,
        announce,
        "H7 HQ DA  CA SQ H8  SJ CT S9  HJ CK S8  DJ CQ S7 \
         HA C9 DT  HT C8 DK  HK SA DQ  H9 ST D9  CJ SK D8",
    );
    let r = *g.result().unwrap();
    assert_eq!(
        r.outcome.spitzen,
        Some(Spitzen {
            mit: true,
            count: 7
        })
    );
    assert_eq!(r.outcome.declarer_augen, 106);
    assert_eq!(r.outcome.declarer_tricks, 9);
    g
}

#[test]
fn schneider_announced_and_made_without_schwarz() {
    // 10 * (7 + 1 + Hand + 2 * Schneider) = 10 * 11
    let g = herz_one_trick_lost(schneider(HERZ));
    let r = check(&g, true, 110, 110, [160, 0, 0]);
    assert!(r.schneider && !r.schwarz);
}

#[test]
fn schwarz_announced_but_one_trick_lost() {
    // 10 * (7 + 1 + Hand + 2 * Schneider + 2 * Schwarz) = 10 * 13, lost
    let g = herz_one_trick_lost(schwarz(HERZ));
    let r = check(&g, false, 130, -260, [-310, 40, 40]);
    assert!(r.schneider && r.schwarz && !r.ueberreizt);
}

#[test]
fn ouvert_lost_with_one_trick_missing() {
    // 10 * (7 + 1 + Hand + 2 * Schneider + 2 * Schwarz + Ouvert) = 10 * 14
    let g = herz_one_trick_lost(ouvert(HERZ));
    check(&g, false, 140, -280, [-330, 40, 40]);
}

// ---------------------------------------------------------------- lost Karo

/// Hinterhand plays Karo ohne 4 and gets 47 Augen (see
/// `full_karo_game_lost_with_changing_leads` in play.rs).
const KARO_LOST: &str = "CJ S7 D8  SJ S8 D9  HJ S9 DQ  DJ SQ DK  CA SK DT \
                         H7 C8 HA  SA DA C9  HQ CQ HT  ST H8 CK  HK H9 CT";

fn karo_lost(bidding: &[Action], announce: Announcement) -> Game {
    let g = play_game(standard_deal(), bidding, announce, KARO_LOST);
    let r = *g.result().unwrap();
    assert_eq!(
        r.outcome.spitzen,
        Some(Spitzen {
            mit: false,
            count: 4
        })
    );
    assert_eq!(r.outcome.declarer_augen, 47);
    g
}

const HH_HAND: [Action; 4] = [Action::Pass, Action::Bid(18), Action::Pass, Action::Hand];

#[test]
fn karo_hand_lost() {
    // 9 * (4 + 1 + Hand) = 54
    let g = karo_lost(&HH_HAND, plain(KARO));
    let r = check(&g, false, 54, -108, [40, 40, -158]);
    assert!(!r.schneider && !r.schwarz);
}

#[test]
fn karo_hand_schneider_announced_lost() {
    // 9 * (4 + 1 + Hand + 2 * Schneider) = 9 * 8
    let g = karo_lost(&HH_HAND, schneider(KARO));
    check(&g, false, 72, -144, [40, 40, -194]);
}

#[test]
fn karo_hand_schwarz_announced_lost() {
    // 9 * (4 + 1 + Hand + 2 * Schneider + 2 * Schwarz) = 9 * 10
    let g = karo_lost(&HH_HAND, schwarz(KARO));
    check(&g, false, 90, -180, [40, 40, -230]);
}

#[test]
fn karo_ouvert_lost() {
    // 9 * (4 + 1 + Hand + 2 * Schneider + 2 * Schwarz + Ouvert) = 9 * 11
    let g = karo_lost(&HH_HAND, ouvert(KARO));
    check(&g, false, 99, -198, [40, 40, -248]);
}

#[test]
fn karo_overbid_is_lost_at_the_next_multiple() {
    // Hinterhand bids 50 and plays Karo with pickup: 9 * (4 + 1) = 45 < 50.
    // Scored as the smallest multiple of 9 that reaches 50, which is 54.
    let bidding = [
        Action::Pass,
        Action::Bid(50),
        Action::Pass,
        Action::TakeSkat,
        Action::Discard(cards("C7 D7")),
    ];
    let g = karo_lost(&bidding, plain(KARO));
    let r = check(&g, false, 45, -108, [40, 40, -158]);
    assert!(r.ueberreizt);
    assert_eq!(r.scored_value, 54);
}

// ---------------------------------------------------------------- Null

/// Mittelhand declares Null and wins all ten tricks without a single one
/// (see `null_won_after_ten_tricks` in play.rs).
fn null_deal() -> Deal {
    deal(
        "CA CK CQ CJ SA SK SQ SJ HA HK",
        "C7 C8 C9 S7 S8 S9 H7 H8 H9 D7",
        "CT ST HT HQ HJ DA DK DQ DJ DT",
        "D9 D8",
    )
}

const NULL_ALL: &str = "CA C7 CT  CK C8 HT  CQ C9 ST  CJ S7 HQ  SA S8 HJ \
                        SK S9 DA  SQ H7 DK  SJ H8 DQ  HA H9 DJ  HK D7 DT";

const MH_HAND: [Action; 4] = [Action::Bid(18), Action::Pass, Action::Pass, Action::Hand];

#[test]
fn null_hand_won() {
    let g = play_game(null_deal(), &MH_HAND, plain(GameType::Null), NULL_ALL);
    let r = check(&g, true, 35, 35, [0, 85, 0]);
    assert_eq!(r.outcome.tricks_played, 10);
}

#[test]
fn null_ouvert_with_pickup_won() {
    let bidding = [
        Action::Bid(18),
        Action::Pass,
        Action::Pass,
        Action::TakeSkat,
        Action::Discard(cards("D9 D8")),
    ];
    let g = play_game(null_deal(), &bidding, null_ouvert(), NULL_ALL);
    check(&g, true, 46, 46, [0, 96, 0]);
    assert!(g.is_revealed());
}

#[test]
fn null_ouvert_hand_won() {
    let g = play_game(null_deal(), &MH_HAND, null_ouvert(), NULL_ALL);
    check(&g, true, 59, 59, [0, 109, 0]);
}

#[test]
fn null_ouvert_hand_lost_ends_after_two_tricks() {
    let bidding = [Action::Bid(23), Action::Pass, Action::Pass, Action::Hand];
    let g = play_game(standard_deal(), &bidding, null_ouvert(), "CJ S7 HQ  HJ HA H9");
    let r = check(&g, false, 59, -118, [40, -168, 40]);
    assert_eq!(r.outcome.tricks_played, 2);
}
