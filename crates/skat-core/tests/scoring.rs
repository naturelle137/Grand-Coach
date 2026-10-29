// SPDX-License-Identifier: EUPL-1.2

//! Spitzen, game values, Überreizen, Kontra/Re and Seeger-Fabian.
//! Every expected number is worked out by hand in the comment next to it.

mod common;

use common::*;
use skat_core::{
    Contract, GameOutcome, GameType, KontraLevel, Seat, Spitzen, Suit, settle, spitzen,
};

fn contract(game: GameType, hand: bool, schneider: bool, schwarz: bool, ouvert: bool) -> Contract {
    Contract {
        game,
        hand,
        schneider,
        schwarz,
        ouvert,
    }
}

fn plain(game: GameType) -> Contract {
    contract(game, false, false, false, false)
}

fn mit(count: u8) -> Option<Spitzen> {
    Some(Spitzen { mit: true, count })
}

fn ohne(count: u8) -> Option<Spitzen> {
    Some(Spitzen { mit: false, count })
}

fn outcome(
    contract: Contract,
    bid: u16,
    spitzen: Option<Spitzen>,
    augen: u8,
    tricks: u8,
) -> GameOutcome {
    GameOutcome {
        declarer: Seat::Vorhand,
        contract,
        bid,
        spitzen,
        declarer_augen: augen,
        declarer_tricks: tricks,
        tricks_played: 10,
        kontra: KontraLevel::None,
    }
}

const KREUZ: GameType = GameType::Farbe(Suit::Kreuz);
const PIK: GameType = GameType::Farbe(Suit::Pik);
const HERZ: GameType = GameType::Farbe(Suit::Herz);
const KARO: GameType = GameType::Farbe(Suit::Karo);

#[test]
fn spitzen_from_cards() {
    let s = |game, hand: &str| spitzen(game, cards(hand));
    assert_eq!(s(KREUZ, "CJ SJ HJ CA S7"), mit(3));
    assert_eq!(s(KREUZ, "CJ SJ HJ DJ CA CT C7"), mit(6));
    assert_eq!(s(KREUZ, "HJ DJ CA"), ohne(2));
    assert_eq!(s(HERZ, "HA HT H7 SA"), ohne(4));
    assert_eq!(s(HERZ, "SA ST CA"), ohne(11));
    assert_eq!(s(HERZ, "CJ SJ HJ DJ HA HT HK HQ H9 H8 H7 SA"), mit(11));
    assert_eq!(s(GameType::Grand, "CJ SJ HJ DJ"), mit(4));
    assert_eq!(s(GameType::Grand, "CA CT"), ohne(4));
    assert_eq!(s(GameType::Grand, "SJ DJ"), ohne(1));
    assert_eq!(s(GameType::Null, "CJ SJ"), None);
}

#[test]
fn simple_wins_and_losses() {
    // Kreuz mit 2: 12 * (2 + 1) = 36.
    let r = settle(&outcome(plain(KREUZ), 18, mit(2), 70, 6));
    assert!(r.won && !r.schneider && !r.schwarz);
    assert_eq!((r.game_value, r.score), (36, 36));
    assert_eq!(r.seeger_fabian(), [86, 0, 0]);
    // Pik ohne 1, 60 Augen lose: 11 * 2 = 22, lost doubled -44.
    let r = settle(&outcome(plain(PIK), 22, ohne(1), 60, 5));
    assert!(!r.won);
    assert_eq!((r.game_value, r.score), (22, -44));
    assert_eq!(r.seeger_fabian(), [-94, 40, 40]);
    // 61 Augen win.
    assert!(settle(&outcome(plain(PIK), 22, ohne(1), 61, 5)).won);
    // Grand mit 1 Hand: 24 * (1 + 1 + 1) = 72.
    let r = settle(&outcome(
        contract(GameType::Grand, true, false, false, false),
        18,
        mit(1),
        80,
        7,
    ));
    assert_eq!(r.score, 72);
}

#[test]
fn schneider_and_schwarz_levels() {
    // Herz mit 1, 90 Augen: 10 * (1 + 1 + 1) = 30; 89 Augen: 20.
    assert_eq!(settle(&outcome(plain(HERZ), 18, mit(1), 90, 8)).score, 30);
    assert_eq!(settle(&outcome(plain(HERZ), 18, mit(1), 89, 8)).score, 20);
    // Declarer Schneider with 30 Augen: Karo mit 1 lost, 9 * 3 = 27, -54.
    let r = settle(&outcome(plain(KARO), 18, mit(1), 30, 2));
    assert!(!r.won && r.schneider);
    assert_eq!(r.score, -54);
    // 31 Augen: not Schneider, 9 * 2 = 18, -36.
    assert_eq!(settle(&outcome(plain(KARO), 18, mit(1), 31, 2)).score, -36);
    // Schwarz won: Grand mit 4 Hand, 24 * (4 + 1 + 1 + 1 + 1) = 192.
    let r = settle(&outcome(
        contract(GameType::Grand, true, false, false, false),
        18,
        mit(4),
        120,
        10,
    ));
    assert!(r.schwarz);
    assert_eq!(r.score, 192);
    // Declarer Schwarz: Grand ohne 2, 24 * (2 + 1 + 1 + 1) = 120, -240.
    let r = settle(&outcome(plain(GameType::Grand), 18, ohne(2), 0, 0));
    assert!(r.schwarz && r.schneider);
    assert_eq!(r.score, -240);
}

#[test]
fn announced_levels() {
    // Kreuz Hand Schneider angesagt mit 1, made with 90:
    // 12 * (1 + 1 + Hand + Schneider + angesagt) = 60.
    let ang = contract(KREUZ, true, true, false, false);
    assert_eq!(settle(&outcome(ang, 18, mit(1), 90, 8)).score, 60);
    // Missed with 85: lost with the same levels, -120.
    let r = settle(&outcome(ang, 18, mit(1), 85, 8));
    assert!(!r.won);
    assert_eq!(r.score, -120);
    // Declarer Schneider in it: no extra level (ISkO 5.2.5), still -120.
    assert_eq!(settle(&outcome(ang, 18, mit(1), 20, 1)).score, -120);
    // Exceeded to Schwarz: Karo mit 2, 9 * (2 + 1 + 1 + 1 + 1 + Schwarz) = 63.
    let r = settle(&outcome(
        contract(KARO, true, true, false, false),
        18,
        mit(2),
        120,
        10,
    ));
    assert_eq!(r.score, 63);
    // Schwarz angesagt, all tricks: Pik mit 3, 11 * (3 + 1 + 1 + 4) = 99.
    let schwarz = contract(PIK, true, true, true, false);
    assert_eq!(settle(&outcome(schwarz, 18, mit(3), 120, 10)).score, 99);
    // One trick lost with 115 Augen: lost, -198.
    let r = settle(&outcome(schwarz, 18, mit(3), 115, 9));
    assert!(!r.won);
    assert_eq!(r.score, -198);
}

#[test]
fn ouvert_values() {
    // Grand Ouvert mit 4: 24 * (4 + 1 + Hand + 4 + Ouvert) = 264.
    let go = contract(GameType::Grand, true, true, true, true);
    assert_eq!(settle(&outcome(go, 18, mit(4), 120, 10)).score, 264);
    // Grand Ouvert mit 1 losing one trick: 24 * 8 = 192, -384.
    assert_eq!(settle(&outcome(go, 18, mit(1), 110, 9)).score, -384);
    // Herz Ouvert ohne 2: 10 * (2 + 1 + 1 + 4 + 1) = 90.
    let ho = contract(HERZ, true, true, true, true);
    assert_eq!(settle(&outcome(ho, 18, ohne(2), 120, 10)).score, 90);
}

#[test]
fn null_values() {
    let null = |hand, ouvert| contract(GameType::Null, hand, false, false, ouvert);
    assert_eq!(
        settle(&outcome(null(false, false), 23, None, 40, 0)).score,
        23
    );
    assert_eq!(
        settle(&outcome(null(true, false), 23, None, 40, 0)).score,
        35
    );
    assert_eq!(
        settle(&outcome(null(false, true), 23, None, 40, 0)).score,
        46
    );
    assert_eq!(
        settle(&outcome(null(true, true), 23, None, 40, 0)).score,
        59
    );
    let r = settle(&outcome(null(true, false), 18, None, 0, 1));
    assert!(!r.won);
    assert_eq!(r.score, -70);
    assert_eq!(
        settle(&outcome(null(true, true), 18, None, 0, 1)).score,
        -118
    );
}

#[test]
fn ueberreizt() {
    // ISkO 5.4.2 example: bid 36 in a Herz game worth 30 -> lost with 40,
    // doubled -80, although the declarer made 70.
    let r = settle(&outcome(plain(HERZ), 36, mit(2), 70, 6));
    assert!(r.ueberreizt && !r.won);
    assert_eq!((r.game_value, r.scored_value, r.score), (30, 40, -80));
    // ISkO 5.4.1 example: bid 50 lost as -100 (Herz, 5 * 10).
    assert_eq!(settle(&outcome(plain(HERZ), 50, mit(1), 70, 6)).score, -100);
    // Pik mit 1 (22) at 46: smallest multiple of 11 reaching 46 is 55, -110.
    assert_eq!(settle(&outcome(plain(PIK), 46, mit(1), 70, 6)).score, -110);
    // Value equal to the bid is not overbid: Kreuz ohne 2 = 36 at 36.
    let r = settle(&outcome(plain(KREUZ), 36, ohne(2), 70, 6));
    assert!(r.won && !r.ueberreizt);
    assert_eq!(r.score, 36);
    // Schneider can rescue the bid: Kreuz mit 1 Schneider = 36 at 36.
    assert!(settle(&outcome(plain(KREUZ), 36, mit(1), 95, 8)).won);
    // Overbid and also lost on points: still the overbid value, -72.
    assert_eq!(
        settle(&outcome(plain(KREUZ), 36, ohne(1), 40, 3)).score,
        -72
    );
}

#[test]
fn kontra_and_re_multiply_game_points_only() {
    let mut o = outcome(plain(KREUZ), 18, mit(2), 70, 6); // 36
    o.kontra = KontraLevel::Kontra;
    let r = settle(&o);
    assert_eq!(r.score, 72);
    assert_eq!(r.seeger_fabian(), [122, 0, 0]);
    o.kontra = KontraLevel::Re;
    let r = settle(&o);
    assert_eq!(r.score, 144);
    assert_eq!(r.seeger_fabian(), [194, 0, 0]);
    // Lost with Kontra: 36 * -2 * 2 = -144, declarer -194, defenders 40.
    o.declarer_augen = 50;
    o.kontra = KontraLevel::Kontra;
    let r = settle(&o);
    assert_eq!(r.score, -144);
    assert_eq!(r.seeger_fabian(), [-194, 40, 40]);
    // Overbid check ignores Kontra: 36 at 40 is still overbid.
    let mut o = outcome(plain(KREUZ), 40, mit(2), 70, 6);
    o.kontra = KontraLevel::Re;
    let r = settle(&o);
    assert!(r.ueberreizt);
    // 48 * -2 * 4
    assert_eq!(r.score, -384);
    // Null Hand with Re, won: 35 * 4 = 140.
    let mut o = outcome(
        contract(GameType::Null, true, false, false, false),
        18,
        None,
        0,
        0,
    );
    o.kontra = KontraLevel::Re;
    assert_eq!(settle(&o).score, 140);
}

#[test]
fn seeger_fabian_for_each_declarer_seat() {
    let mut o = outcome(plain(KARO), 18, mit(1), 40, 3); // 18, lost -36
    o.declarer = Seat::Hinterhand;
    assert_eq!(settle(&o).seeger_fabian(), [40, 40, -86]);
    o.declarer = Seat::Mittelhand;
    o.declarer_augen = 80;
    assert_eq!(settle(&o).seeger_fabian(), [0, 68, 0]);
}
