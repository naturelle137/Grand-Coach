// SPDX-License-Identifier: EUPL-1.2

//! The two card-play mechanisms (#44, `docs/coach.md`): an opponent who
//! can trump a full side card (Abstich) and the partner who can beat the
//! declarer's card (overtaking).
//!
//! Every expected seat, certainty and card set is worked out by hand from
//! the deal and the play in the comments, not taken from the code under
//! test. `mechanisms_match_an_exhaustive_reference` compares the predicates
//! with plain code over every hidden distribution consistent with the view
//! of many random positions: "possible" needs a witness distribution,
//! "certain" all of them.

mod common;

use common::*;
use skat_analysis::lesson::{
    Check, Comparison, Condition, EvidenceClass, GAME_VALUE, GameKind, HAND_STRUCTURE, LessonKind,
    PARTNER_OVERTAKE, PROVEN_VOID, REMAINING_TRUMPF, Reliability, Role, Support,
    TRUMPF_VULNERABILITY, Unmet, catalogue, compare, gather,
};
use skat_analysis::mechanism::{
    Abstich, Certainty, Mechanism, MechanismKind, Overtake, mechanisms,
};
use skat_analysis::sampling::HiddenCards;
use skat_analysis::{Budget, analyze};
use skat_core::{
    Action, Announcement, Card, CardSet, Deal, Follow, Game, GameType, Phase, PlayerView, Rank,
    RuleConfig, Seat, Suit, legal_cards,
};

/// MH 18, VH holds, MH and HH pass: VH declarer at 18.
const VH_AT_18: [Action; 4] = [Action::Bid(18), Action::Hold, Action::Pass, Action::Pass];

const SMALL: Budget = Budget {
    samples: 3,
    nodes_per_task: 50_000_000,
};

fn announce(game: GameType) -> Action {
    Action::Announce(Announcement::plain(game))
}

const KREUZ: GameType = GameType::Farbe(Suit::Kreuz);
const HERZ: GameType = GameType::Farbe(Suit::Herz);

/// Plays the deal: VH declarer at 18, then `setup` (Hand or pickup,
/// discard, announcement), then the cards. Kontra/Re questions are
/// declined on the way.
fn play(deal: Deal, setup: &[Action], cards: &[&str]) -> Game {
    let mut g = Game::from_deal(deal, RuleConfig::default());
    let moves = setup
        .iter()
        .copied()
        .chain(cards.iter().map(|s| Action::Play(c(s))));
    for a in VH_AT_18.into_iter().chain(moves) {
        decline_kontra(&mut g);
        g.apply(a).unwrap_or_else(|e| panic!("{a:?}: {e:?}"));
    }
    decline_kontra(&mut g);
    g
}

fn decline_kontra(g: &mut Game) {
    loop {
        let legal = g.legal_actions();
        let answer = [Action::KeinKontra, Action::KeinRe]
            .into_iter()
            .find(|a| legal.contains(a));
        match answer {
            Some(a) => {
                g.apply(a).unwrap();
            }
            None => return,
        }
    }
}

/// The view of the seat to move.
fn to_move(g: &Game) -> PlayerView {
    g.view(g.current_seat().expect("a decision"))
}

fn of(view: &PlayerView, card: &str) -> Vec<Mechanism> {
    mechanisms(view, Action::Play(c(card)))
}

/// Herz Hand by VH (the deal of `exact_lessons.rs`).
fn herz_deal() -> Deal {
    deal(
        "CJ SJ HA HT HK HQ H9 SA ST CA",
        "DJ H8 H7 SK SQ S9 CK CQ C9 DA",
        "HJ S8 S7 CT C8 C7 DT DK DQ D9",
        "D8 D7",
    )
}

/// Kreuz by VH; MH holds one Trumpf (C9) and no Pik, HH no Trumpf, the
/// Trumpf C8 C7 lie in the Skat.
fn kreuz_low() -> Deal {
    deal(
        "CJ SJ HJ DJ CA CT CK CQ SA SK",
        "C9 HA HT HK HQ H9 DA DK DQ D7",
        "ST SQ S9 S8 S7 H8 H7 DT D9 D8",
        "C8 C7",
    )
}

/// The same view for VH with other concealed cards: MH holds the Skat's
/// Trumpf C8 C7, the Skat holds HQ H9. MH's played cards C9 and D7 stay
/// legal (C9 follows Trumpf, D7 is played without Pik).
fn kreuz_low_swapped() -> Deal {
    deal(
        "CJ SJ HJ DJ CA CT CK CQ SA SK",
        "C9 C8 C7 HA HT HK DA DK DQ D7",
        "ST SQ S9 S8 S7 H8 H7 DT D9 D8",
        "HQ H9",
    )
}

/// Kreuz by VH; MH holds four Trumpf and no Pik, HH no Trumpf.
fn kreuz_high() -> Deal {
    deal(
        "CJ SJ CA CT CK SA SK HA HT DA",
        "HJ DJ CQ C9 HK HQ H9 DK DQ D7",
        "ST SQ S9 S8 S7 H8 H7 DT D9 D8",
        "C8 C7",
    )
}

/// Trick 1: VH leads CJ, MH follows with C9, HH has no Trumpf (D8).
/// Trick 2: VH leads SK, MH has no Pik (D7), HH follows with S7.
const KREUZ_PLAY: [&str; 6] = ["CJ", "C9", "D8", "SK", "D7", "S7"];

/// Grand Hand by VH, who holds no Herz.
fn grand_deal() -> Deal {
    deal(
        "CJ SJ HJ DJ CA CT CK S7 DA DT",
        "SA ST HA HT HK D7 D8 C7 C8 C9",
        "SK SQ S9 S8 HQ H9 H8 H7 DK DQ",
        "CQ D9",
    )
}

/// Grand by VH; HH holds no Pik.
fn grand_pik() -> Deal {
    deal(
        "CJ SJ CA CT CK SA SK S9 DA DT",
        "HJ ST SQ S8 HA HT HK D7 D8 C7",
        "DJ HQ H9 H8 H7 DK DQ D9 C8 C9",
        "S7 CQ",
    )
}

/// Grand Ouvert by VH; MH holds no Pik, HH six of them.
fn grand_ouvert() -> Deal {
    deal(
        "CJ SJ HJ DJ CA CT CK S7 DA DT",
        "HA HT HK D7 D8 C7 C8 C9 DK DQ",
        "SA ST SK SQ S9 S8 HQ H9 H8 H7",
        "CQ D9",
    )
}

fn abstich(view: &PlayerView, card: &str) -> Vec<Abstich> {
    of(view, card)
        .into_iter()
        .map(|m| match m {
            Mechanism::Abstich(a) => a,
            other => panic!("{other:?}"),
        })
        .collect()
}

fn overtake(view: &PlayerView, card: &str) -> Option<Overtake> {
    let found = of(view, card);
    assert!(found.len() <= 1, "{found:?}");
    found
        .into_iter()
        .map(|m| match m {
            Mechanism::Overtake(o) => o,
            other => panic!("{other:?}"),
        })
        .next()
}

// --- Abstich -------------------------------------------------------------

#[test]
fn abstich_possible_for_the_declarer_but_not_by_a_seat_without_trumpf() {
    // VH leads trick 3 after CJ H7 HJ | HA H8 S7. HH played S7 on the
    // Trumpf HA: no Trumpf. Trumpf (11): own SJ HT HK HQ H9, played CJ HJ
    // HA H8 H7 -> unseen DJ only. Kreuz side cards: own CA -> unseen CT CK
    // CQ C9 C8 C7. MH (8 cards) may hold DJ and no Kreuz (the six Kreuz fit
    // into HH's 8 cards and the Skat), and may hold a Kreuz: possible.
    // HH cannot trump: a Trumpf count alone proves nothing for a seat.
    let g = play(
        herz_deal(),
        &[Action::Hand, announce(HERZ)],
        &["CJ", "H7", "HJ", "HA", "H8", "S7"],
    );
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Vorhand);
    assert_eq!(
        abstich(&view, "CA"),
        [Abstich {
            seat: Seat::Mittelhand,
            card: c("CA"),
            certainty: Certainty::Possible,
            suit_unseen: cards("CT CK CQ C9 C8 C7"),
            trumpf_unseen: cards("DJ"),
        }]
    );
    // Pik: own SA ST, played S7 -> unseen SK SQ S9 S8. The Zehn also wins
    // an empty trick.
    for card in ["SA", "ST"] {
        let found = abstich(&view, card);
        assert_eq!(found.len(), 1, "{card}");
        assert_eq!(found[0].seat, Seat::Mittelhand);
        assert_eq!(found[0].suit_unseen, cards("SK SQ S9 S8"));
    }
    // Trumpf leads are no side cards.
    for card in ["SJ", "HT", "H9"] {
        assert!(of(&view, card).is_empty(), "{card}");
    }
}

#[test]
fn abstich_certain_when_the_void_is_proven_and_trumpf_must_be_there() {
    // Kreuz, VH picks up C8 C7 and discards HT DA (known Skat, no
    // Trumpf). KREUZ_PLAY: HH no Trumpf (trick 1), MH no Pik (trick 2).
    // VH leads trick 3. Trumpf: own SJ CA CT CK C8 C7, played CJ C9, own
    // Skat none -> unseen HJ DJ CQ. HH cannot hold them and the Skat is
    // known, so MH holds all three: MH has no Pik and Trumpf in every
    // distribution. Pik: own SA, played SK S7 -> unseen ST SQ S9 S8.
    let setup = [
        Action::TakeSkat,
        Action::Discard(cards("HT DA")),
        announce(KREUZ),
    ];
    let g = play(kreuz_high(), &setup, &KREUZ_PLAY);
    let view = to_move(&g);
    assert_eq!(view.own_skat, Some(cards("HT DA")));
    assert_eq!(
        abstich(&view, "SA"),
        [Abstich {
            seat: Seat::Mittelhand,
            card: c("SA"),
            certainty: Certainty::Certain,
            suit_unseen: cards("ST SQ S9 S8"),
            trumpf_unseen: cards("HJ DJ CQ"),
        }]
    );
    // Herz Ass: none. VH saw 16 cards (own 8, Skat 2, played 6); MH and HH
    // hold the other 16, 8 each. MH has no Pik and HH no Trumpf, so the 4
    // unseen Pik (ST SQ S9 S8) are HH's and the 3 Trumpf MH's. MH's other 5
    // cards come from the unseen Herz (HK HQ H9 H8 H7) and Karo (DK DQ DT
    // D9): with only 4 Karo, MH holds a Herz in every distribution and
    // cannot trump the Herz Ass.
    assert!(abstich(&view, "HA").is_empty());
}

#[test]
fn abstich_certain_by_counting_even_with_an_unknown_skat() {
    // The same deal as a Hand game: VH holds CJ SJ CA CT CK SA SK HA HT DA.
    // Unseen Trumpf after KREUZ_PLAY: 11 - own (SJ CA CT CK) - played (CJ
    // C9) = HJ DJ CQ C8 C7. HH holds none, the Skat at most two, so MH
    // holds at least three: certain.
    let g = play(kreuz_high(), &[Action::Hand, announce(KREUZ)], &KREUZ_PLAY);
    let view = to_move(&g);
    let found = abstich(&view, "SA");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].certainty, Certainty::Certain);
    assert_eq!(found[0].trumpf_unseen, cards("HJ DJ CQ C8 C7"));
}

#[test]
fn abstich_only_possible_when_the_skat_can_hold_every_unseen_trumpf() {
    // Boundary: Kreuz Hand by VH with KREUZ_PLAY. Unseen Trumpf: 11 - own
    // (SJ HJ DJ CA CT CK CQ) - played (CJ C9) = C8 C7. MH has no Pik
    // (proven), HH no Trumpf, but both Trumpf can lie in the unknown Skat:
    // possible, not certain (in the deal they are in the Skat).
    let g = play(kreuz_low(), &[Action::Hand, announce(KREUZ)], &KREUZ_PLAY);
    let view = to_move(&g);
    assert_eq!(
        abstich(&view, "SA"),
        [Abstich {
            seat: Seat::Mittelhand,
            card: c("SA"),
            certainty: Certainty::Possible,
            suit_unseen: cards("ST SQ S9 S8"),
            trumpf_unseen: cards("C8 C7"),
        }]
    );
}

#[test]
fn no_abstich_when_no_trumpf_is_unseen() {
    // Boundary: the same deal with pickup; VH discards the Trumpf C8 C7.
    // Unseen Trumpf: 11 - own 7 - played 2 - own Skat 2 = 0. MH's proven
    // Pik void alone gives no Abstich.
    let setup = [
        Action::TakeSkat,
        Action::Discard(cards("C8 C7")),
        announce(KREUZ),
    ];
    let g = play(kreuz_low(), &setup, &KREUZ_PLAY);
    let view = to_move(&g);
    assert!(of(&view, "SA").is_empty());
}

#[test]
fn abstich_for_a_defender_names_only_the_declarer() {
    // Grand Hand by VH. Trick 1: VH S7, MH SA, HH S8: MH wins and leads
    // trick 2. Seats after MH: HH (partner, never an Abstich risk) and VH.
    // Herz: own HA HT HK -> unseen HQ H9 H8 H7; Trumpf: the four Buben,
    // all unseen. VH may hold no Herz and a Bube, or Herz: possible.
    let g = play(
        grand_deal(),
        &[Action::Hand, announce(GameType::Grand)],
        &["S7", "SA", "S8"],
    );
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Mittelhand);
    let expected = |card: &str| Abstich {
        seat: Seat::Vorhand,
        card: c(card),
        certainty: Certainty::Possible,
        suit_unseen: cards("HQ H9 H8 H7"),
        trumpf_unseen: cards("CJ SJ HJ DJ"),
    };
    assert_eq!(abstich(&view, "HA"), [expected("HA")]);
    assert_eq!(abstich(&view, "HT"), [expected("HT")]);
    // Not an Ass or Zehn.
    assert!(of(&view, "HK").is_empty());
    assert!(of(&view, "C7").is_empty());
}

#[test]
fn no_abstich_by_seat_order_or_following() {
    // Defender second, only the partner after: MH's SA on VH's S7.
    let g = play(
        grand_deal(),
        &[Action::Hand, announce(GameType::Grand)],
        &["S7"],
    );
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Mittelhand);
    assert!(of(&view, "SA").is_empty());
    assert!(of(&view, "ST").is_empty());

    // Kreuz Hand, trick 2 after SK D7: HH plays last; its ST wins but
    // nobody plays after it.
    let g = play(
        kreuz_low(),
        &[Action::Hand, announce(KREUZ)],
        &KREUZ_PLAY[..5],
    );
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Hinterhand);
    assert!(of(&view, "ST").is_empty());

    // MH second after SK has no Pik: its HA does not follow the led suit
    // (only the overtaking mechanism applies there).
    let g = play(
        kreuz_low(),
        &[Action::Hand, announce(KREUZ)],
        &KREUZ_PLAY[..4],
    );
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Mittelhand);
    assert!(
        of(&view, "HA")
            .iter()
            .all(|m| m.kind() == MechanismKind::Overtake)
    );
}

#[test]
fn abstich_for_the_declarer_playing_second() {
    // Kreuz Hand. Trick 1 as KREUZ_PLAY; trick 2: VH SK, MH D7, HH ST: HH
    // wins and leads SQ in trick 3. VH must follow with SA, which wins so
    // far; MH plays after it. MH has no Pik (trick 2); unseen Trumpf: C8 C7
    // (11 - own SJ HJ DJ CA CT CK CQ - played CJ C9), which may lie with MH
    // or in the Skat: possible. Pik: own SA, played SK ST SQ -> unseen S9
    // S8 S7.
    let g = play(
        kreuz_low(),
        &[Action::Hand, announce(KREUZ)],
        &["CJ", "C9", "D8", "SK", "D7", "ST", "SQ"],
    );
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Vorhand);
    assert_eq!(
        abstich(&view, "SA"),
        [Abstich {
            seat: Seat::Mittelhand,
            card: c("SA"),
            certainty: Certainty::Possible,
            suit_unseen: cards("S9 S8 S7"),
            trumpf_unseen: cards("C8 C7"),
        }]
    );
}

#[test]
fn no_abstich_for_a_card_that_does_not_win_the_trick() {
    // Grand Hand by MH (MH bids 18, VH and HH pass). Trick 1: VH DK, MH
    // DQ, HH DA: HH wins and leads HA in trick 2. VH (defender) must
    // follow with HT, its only Herz (HJ is Trumpf in Grand). The declarer
    // MH plays after VH, but HT does not win the trick: no Abstich.
    let deal = deal(
        "C7 CT CK S8 SQ HT HJ D8 DT DK",
        "CJ CQ CA S9 ST SJ H7 HQ HK DQ",
        "C8 C9 S7 SA H8 H9 HA D7 D9 DA",
        "SK DJ",
    );
    let mut g = Game::from_deal(deal, RuleConfig::default());
    let moves = [
        Action::Bid(18),
        Action::Pass,
        Action::Pass,
        Action::Hand,
        announce(GameType::Grand),
    ];
    for a in moves
        .into_iter()
        .chain(["DK", "DQ", "DA", "HA"].map(|s| Action::Play(c(s))))
    {
        decline_kontra(&mut g);
        g.apply(a).unwrap_or_else(|e| panic!("{a:?}: {e:?}"));
    }
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Vorhand);
    assert_eq!(view.declarer, Some(Seat::Mittelhand));
    assert_eq!(view.legal_actions, [Action::Play(c("HT"))]);
    assert!(of(&view, "HT").is_empty());
}

// --- Overtaking ----------------------------------------------------------

#[test]
fn overtake_certain_by_counting_with_a_higher_card() {
    // Kreuz Hand, trick 2: VH leads SK, MH (no Pik) decides. MH holds HA
    // HT HK HQ H9 DA DK DQ D7 and saw CJ C9 D8 SK: 19 cards unseen, for VH
    // (8), HH (9) and the Skat (2). HH has no Trumpf (trick 1), so the 9
    // unseen Trumpf (SJ HJ DJ CA CT CK CQ C8 C7) fill 9 of the 10 places of
    // VH and the Skat. Of the 10 unseen non-Trumpf (SA ST SQ S9 S8 S7 H8 H7
    // DT D9) HH holds 9, so at most one is elsewhere: HH holds SA or ST,
    // both above SK. Certain, although neither card was seen. Trumping is
    // impossible for HH.
    let g = play(
        kreuz_low(),
        &[Action::Hand, announce(KREUZ)],
        &KREUZ_PLAY[..4],
    );
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Mittelhand);
    let expected = Overtake {
        partner: Seat::Hinterhand,
        declarer: Seat::Vorhand,
        best: c("SK"),
        led: Follow::Farbe(Suit::Pik),
        card: c("D7"),
        certainty: Certainty::Certain,
        higher: cards("SA ST"),
        trumping: false,
    };
    assert_eq!(overtake(&view, "D7"), Some(expected));
    // Every card of MH leaves SK winning (MH holds no Trumpf any more).
    for &a in &view.legal_actions {
        let Action::Play(card) = a else { panic!() };
        assert_eq!(
            mechanisms(&view, a),
            [Mechanism::Overtake(Overtake { card, ..expected })]
        );
    }
}

#[test]
fn overtake_by_trumping_only_after_a_proven_void() {
    // Grand Hand. Trick 1: VH SA, MH S8, HH C8: HH has no Pik. Trick 2: VH
    // leads SK, MH decides. Pik above SK: SA (played), ST (own) -> none
    // unseen. HH can only trump: Buben unseen to MH: CJ SJ DJ (MH holds
    // HJ). VH (8 cards) and the Skat can hold all three, so HH may have
    // none: possible, not certain – a void alone does not prove it.
    let g = play(
        grand_pik(),
        &[Action::Hand, announce(GameType::Grand)],
        &["SA", "S8", "C8", "SK"],
    );
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Mittelhand);
    assert_eq!(
        overtake(&view, "SQ"),
        Some(Overtake {
            partner: Seat::Hinterhand,
            declarer: Seat::Vorhand,
            best: c("SK"),
            led: Follow::Farbe(Suit::Pik),
            card: c("SQ"),
            certainty: Certainty::Possible,
            higher: CardSet::EMPTY,
            trumping: true,
        })
    );
    // MH's ST takes the trick itself: nothing for the partner to do.
    assert_eq!(overtake(&view, "ST"), None);
}

#[test]
fn overtake_certain_against_an_open_declarer() {
    // Grand Ouvert by VH: VH's cards are open. Trick 1: VH leads S7, MH
    // has no Pik and plays D7. Every other Pik is above S7: SA ST SK SQ
    // S9 S8, all unseen to MH; the only hidden places are HH and the Skat
    // (2 cards), so HH holds at least four: certain. No Trumpf is unseen
    // (all four Buben are open in VH's hand), so no trumping.
    let ouvert = Action::Announce(Announcement {
        game: GameType::Grand,
        schneider: true,
        schwarz: true,
        ouvert: true,
    });
    let g = play(grand_ouvert(), &[Action::Hand, ouvert], &["S7"]);
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Mittelhand);
    assert!(view.declarer_cards.is_some());
    assert_eq!(
        overtake(&view, "D7"),
        Some(Overtake {
            partner: Seat::Hinterhand,
            declarer: Seat::Vorhand,
            best: c("S7"),
            led: Follow::Farbe(Suit::Pik),
            card: c("D7"),
            certainty: Certainty::Certain,
            higher: cards("SA ST SK SQ S9 S8"),
            trumping: false,
        })
    );
}

#[test]
fn overtake_on_a_trumpf_lead() {
    // Herz Hand. Trick 1: VH leads H9, MH decides (DJ H8 H7 Trumpf).
    // Trumpf above H9: CJ SJ HJ DJ HA HT HK HQ; MH holds DJ -> unseen CJ SJ
    // HJ HA HT HK HQ (7). VH (9 cards) and the Skat can hold all seven:
    // possible. With DJ MH takes the trick itself.
    let g = play(herz_deal(), &[Action::Hand, announce(HERZ)], &["H9"]);
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Mittelhand);
    let expected = Overtake {
        partner: Seat::Hinterhand,
        declarer: Seat::Vorhand,
        best: c("H9"),
        led: Follow::Trumpf,
        card: c("H7"),
        certainty: Certainty::Possible,
        higher: cards("CJ SJ HJ HA HT HK HQ"),
        trumping: false,
    };
    assert_eq!(overtake(&view, "H7"), Some(expected));
    assert_eq!(
        overtake(&view, "H8"),
        Some(Overtake {
            card: c("H8"),
            ..expected
        })
    );
    assert_eq!(overtake(&view, "DJ"), None);
}

#[test]
fn no_overtake_when_the_partner_cannot_beat_the_card() {
    // Kreuz Hand, trick 3: VH leads SA (highest Pik), MH (no Pik) decides.
    // No Pik is above SA, and HH has no Trumpf (trick 1): HH cannot beat
    // it in any distribution.
    let g = play(
        kreuz_low(),
        &[Action::Hand, announce(KREUZ)],
        &[&KREUZ_PLAY[..], &["SA"]].concat(),
    );
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Mittelhand);
    for &a in &view.legal_actions {
        assert!(mechanisms(&view, a).is_empty(), "{a:?}");
    }
    // Herz Hand, trick 1: VH leads CJ, the highest Trumpf.
    let g = play(herz_deal(), &[Action::Hand, announce(HERZ)], &["CJ"]);
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Mittelhand);
    assert_eq!(overtake(&view, "H7"), None);
}

#[test]
fn no_overtake_by_seat_order_or_role() {
    // Grand Hand, trick 2: MH (defender) leads HA; HH is second, but the
    // partner led, not the declarer.
    let g = play(
        grand_deal(),
        &[Action::Hand, announce(GameType::Grand)],
        &["S7", "SA", "S8", "HA"],
    );
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Hinterhand);
    for &a in &view.legal_actions {
        assert!(mechanisms(&view, a).is_empty(), "{a:?}");
    }
    // The declarer has no partner: VH third in the same trick.
    let g = play(
        grand_deal(),
        &[Action::Hand, announce(GameType::Grand)],
        &["S7", "SA", "S8", "HA", "H7"],
    );
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Vorhand);
    for &a in &view.legal_actions {
        assert!(mechanisms(&view, a).is_empty(), "{a:?}");
    }
    // Defender third: the partner played before. Grand, trick 1 after VH
    // SA and MH S8: HH (no Pik) decides.
    let g = play(
        grand_pik(),
        &[Action::Hand, announce(GameType::Grand)],
        &["SA", "S8"],
    );
    let view = to_move(&g);
    assert_eq!(view.seat, Seat::Hinterhand);
    for &a in &view.legal_actions {
        assert!(mechanisms(&view, a).is_empty(), "{a:?}");
    }
}

#[test]
fn no_mechanism_in_null_or_outside_card_play() {
    // Null Hand by VH: no Trumpf, no mechanisms (out of scope).
    let g = play(
        herz_deal(),
        &[Action::Hand, announce(GameType::Null)],
        &["SA"],
    );
    let view = to_move(&g);
    for &a in &view.legal_actions {
        assert!(mechanisms(&view, a).is_empty(), "{a:?}");
    }
    // Before card play.
    let g = Game::from_deal(herz_deal(), RuleConfig::default());
    let view = to_move(&g);
    assert!(mechanisms(&view, view.legal_actions[0]).is_empty());
}

// --- Information boundary, determinism and the lesson predicate ---------

#[test]
fn same_view_with_other_concealed_cards_gives_the_same_mechanisms() {
    let setup = [Action::Hand, announce(KREUZ)];
    let a = to_move(&play(kreuz_low(), &setup, &KREUZ_PLAY));
    let b = to_move(&play(kreuz_low_swapped(), &setup, &KREUZ_PLAY));
    assert_eq!(a, b, "the swap must not change VH's view");
    for &action in &a.legal_actions {
        assert_eq!(mechanisms(&a, action), mechanisms(&b, action), "{action:?}");
    }
    let ea = analyze(&a, SMALL, 5, None).unwrap();
    let eb = analyze(&b, SMALL, 5, None).unwrap();
    for &action in &a.legal_actions {
        assert_eq!(compare(&a, &ea, action), compare(&b, &eb, action));
    }
    // And for MH's view in trick 2 (overtaking): VH's and HH's concealed
    // cards are unchanged by this swap, so check determinism there.
    let m = to_move(&play(kreuz_low(), &setup, &KREUZ_PLAY[..4]));
    assert_eq!(of(&m, "D7"), of(&m.clone(), "D7"));
}

fn lesson(key: &str) -> skat_analysis::lesson::Lesson {
    catalogue()
        .into_iter()
        .find(|l| l.id.key == key)
        .expect("in the catalogue")
}

#[test]
fn the_catalogue_puts_the_two_mechanisms_first() {
    let keys: Vec<String> = catalogue().iter().map(|l| l.id.key.to_string()).collect();
    assert_eq!(
        keys,
        [
            TRUMPF_VULNERABILITY,
            PARTNER_OVERTAKE,
            PROVEN_VOID,
            REMAINING_TRUMPF,
            GAME_VALUE,
            HAND_STRUCTURE
        ]
    );
    for (key, roles, kind) in [
        (
            TRUMPF_VULNERABILITY,
            vec![Role::Declarer, Role::Defender],
            MechanismKind::Abstich,
        ),
        (
            PARTNER_OVERTAKE,
            vec![Role::Defender],
            MechanismKind::Overtake,
        ),
    ] {
        let l = lesson(key);
        assert_eq!(l.id.version, 1);
        assert_eq!(l.kind, LessonKind::Mechanism);
        assert_eq!(l.support, Support::Supported);
        assert_eq!(l.applies.phases, [Phase::Spiel]);
        assert_eq!(l.applies.roles, roles);
        assert_eq!(l.applies.games, [GameKind::Suit, GameKind::Grand]);
        assert_eq!(
            l.permitted,
            [
                EvidenceClass::Exact,
                EvidenceClass::Comparison,
                EvidenceClass::Mechanism
            ]
        );
        assert_eq!(l.requires, [Condition::Mechanism(kind)]);
        assert!(l.excludes.is_empty());
    }
}

#[test]
fn a_mechanism_lesson_needs_the_move_a_reliable_comparison_and_the_mechanism() {
    // VH's view of `abstich_possible_for_the_declarer_...`: CA has an
    // Abstich, SJ (Trumpf) none.
    let g = play(
        herz_deal(),
        &[Action::Hand, announce(HERZ)],
        &["CJ", "H7", "HJ", "HA", "H8", "S7"],
    );
    let view = to_move(&g);
    let analysis = analyze(&view, SMALL, 1, None).unwrap();
    let evidence = gather(&view, &analysis);
    let vulnerable = lesson(TRUMPF_VULNERABILITY);
    let ca = compare(&view, &analysis, Action::Play(c("CA"))).unwrap();
    assert_eq!(ca.reliability, Reliability::Reliable);
    assert_eq!(
        vulnerable.check(&evidence, None),
        Check::Fails(Unmet::BeforeMove)
    );
    assert_eq!(
        vulnerable.check(&evidence, Some(&ca)),
        Check::Applies { items: Vec::new() }
    );
    // Not reliable: provisional, even with the mechanism present.
    for reliability in [
        Reliability::Incomplete,
        Reliability::Screened,
        Reliability::NotEvaluated,
    ] {
        let c = Comparison {
            reliability,
            ..ca.clone()
        };
        assert_eq!(
            vulnerable.check(&evidence, Some(&c)),
            Check::Fails(Unmet::Provisional(reliability))
        );
    }
    // A reliable comparison in which the chosen card is the best one, but
    // without a mechanism, triggers nothing: a higher utility alone is no
    // cause.
    let sj = compare(&view, &analysis, Action::Play(c("SJ"))).unwrap();
    let best = Comparison {
        best: Some(sj.chosen),
        ..sj.clone()
    };
    assert!(best.mechanisms.is_empty());
    assert_eq!(
        vulnerable.check(&evidence, Some(&best)),
        Check::Fails(Unmet::Missing(Condition::Mechanism(MechanismKind::Abstich)))
    );
    // The overtake lesson is for defenders only.
    assert_eq!(
        lesson(PARTNER_OVERTAKE).check(&evidence, Some(&ca)),
        Check::Fails(Unmet::Role)
    );
}

#[test]
fn an_incomplete_analysis_keeps_the_mechanism_but_no_lesson() {
    let g = play(
        herz_deal(),
        &[Action::Hand, announce(HERZ)],
        &["CJ", "H7", "HJ", "HA", "H8", "S7"],
    );
    let view = to_move(&g);
    let tiny = Budget {
        samples: 2,
        nodes_per_task: 10,
    };
    let analysis = analyze(&view, tiny, 1, None).unwrap();
    assert_ne!(analysis.status, skat_analysis::Status::Complete);
    let ca = compare(&view, &analysis, Action::Play(c("CA"))).unwrap();
    assert_eq!(ca.reliability, Reliability::Incomplete);
    // The exact mechanism does not depend on the search ...
    assert_eq!(ca.mechanisms, of(&view, "CA"));
    // ... but a mechanism lesson needs a reliable comparison.
    assert_eq!(
        lesson(TRUMPF_VULNERABILITY).check(&gather(&view, &analysis), Some(&ca)),
        Check::Fails(Unmet::Provisional(Reliability::Incomplete))
    );
}

// --- Exhaustive reference ------------------------------------------------

/// Plays a seeded game to card play: MH bids 18, VH passes, HH passes; MH
/// announces the game of `seed` as a Hand game. Then plays `cards` cards
/// chosen by a fixed rule.
fn random_position(seed: u64, cards: usize) -> Option<Game> {
    let game_type = match seed % 3 {
        0 => GameType::Grand,
        1 => GameType::Farbe(Suit::ALL[(seed / 3 % 4) as usize]),
        _ => GameType::Farbe(Suit::Herz),
    };
    let mut g = Game::new(seed, RuleConfig::default());
    let mut played = 0;
    let mut step = seed;
    while played < cards {
        decline_kontra(&mut g);
        let action = match g.phase() {
            Phase::Reizen if g.bid() == 0 && g.legal_actions().contains(&Action::Bid(18)) => {
                Action::Bid(18)
            }
            Phase::Reizen => Action::Pass,
            Phase::SkatAufnahme => Action::Hand,
            Phase::Ansage => announce(game_type),
            Phase::Spiel => {
                let legal = g.legal_actions();
                step = step.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                played += 1;
                legal[(step >> 33) as usize % legal.len()]
            }
            _ => return None,
        };
        g.apply(action).ok()?;
    }
    decline_kontra(&mut g);
    (g.phase() == Phase::Spiel).then_some(g)
}

/// Possible/certain of `holds` over every distribution of `hidden`, or
/// `None` if none holds.
fn over_all(
    hidden: &[skat_analysis::sampling::World],
    holds: impl Fn(&skat_analysis::sampling::World) -> bool,
) -> Option<Certainty> {
    let n = hidden.iter().filter(|w| holds(w)).count();
    match n {
        0 => None,
        n if n == hidden.len() => Some(Certainty::Certain),
        _ => Some(Certainty::Possible),
    }
}

/// The reference: the specification of both mechanisms in plain code over
/// the enumerated distributions; it does not call the predicates.
fn reference(
    view: &PlayerView,
    card: Card,
    worlds: &[skat_analysis::sampling::World],
) -> Vec<Mechanism> {
    let game = view.contract.unwrap().game;
    let declarer = view.declarer.unwrap();
    let trick = view.current_trick;
    let before: Vec<Card> = trick.cards.iter().flatten().copied().collect();
    let leader = trick.leader.unwrap_or(view.seat);
    let led = before.first().copied().unwrap_or(card);
    let mut best = led;
    for &b in &before {
        if game.beats(b, best, led) {
            best = b;
        }
    }
    let wins = before.is_empty() || game.beats(card, best, led);
    let after: Vec<Seat> = (before.len() + 1..3)
        .map(|i| Seat::from_index(leader.index() + i))
        .collect();
    let unseen = HiddenCards::from_view(view).unwrap().unseen();
    let trumps = game.trumps();
    let mut out = Vec::new();
    // Abstich.
    let side = !game.is_trump(card);
    let suit = game.cards_of(game.follow(card));
    if side
        && matches!(card.rank(), Rank::Ass | Rank::Zehn)
        && game.follow(led) == game.follow(card)
        && wins
    {
        for &x in &after {
            let opponent = view.seat == declarer || x == declarer;
            if !opponent {
                continue;
            }
            let hand = |w: &skat_analysis::sampling::World| w.hands[x.index()];
            if let Some(certainty) = over_all(worlds, |w| {
                (hand(w) & suit).is_empty() && !(hand(w) & trumps).is_empty()
            }) {
                out.push(Mechanism::Abstich(Abstich {
                    seat: x,
                    card,
                    certainty,
                    suit_unseen: suit & unseen,
                    trumpf_unseen: trumps & unseen,
                }));
            }
        }
    }
    // Overtaking.
    if before.len() == 1
        && leader == declarer
        && view.seat != declarer
        && !game.beats(card, best, led)
    {
        let partner = after[0];
        let class = game.cards_of(game.follow(best));
        let beats = |c: Card| game.beats(c, best, best);
        let can_beat = |w: &skat_analysis::sampling::World| {
            legal_cards(game, w.hands[partner.index()], Some(best))
                .iter()
                .any(beats)
        };
        if let Some(certainty) = over_all(worlds, can_beat) {
            let higher: CardSet = (class & unseen)
                .iter()
                .filter(|&h| {
                    beats(h) && worlds.iter().any(|w| w.hands[partner.index()].contains(h))
                })
                .collect();
            let trumping = !game.is_trump(best)
                && worlds.iter().any(|w| {
                    let h = w.hands[partner.index()];
                    (h & class).is_empty() && !(h & trumps).is_empty()
                });
            out.push(Mechanism::Overtake(Overtake {
                partner,
                declarer,
                best,
                led: game.follow(best),
                card,
                certainty,
                higher,
                trumping,
            }));
        }
    }
    out
}

/// Compares the predicates with [`reference`] on every legal card of the
/// positions of `seeds`; returns the number of comparisons and the
/// expected mechanisms found per kind and certainty.
fn compare_with_reference(seeds: std::ops::Range<u64>) -> (usize, [usize; 4]) {
    let mut compared = 0;
    let mut found = [0usize; 4];
    for seed in seeds {
        for cards in [13, 16, 19, 22] {
            let Some(g) = random_position(seed, cards) else {
                continue;
            };
            let view = to_move(&g);
            let Ok(hidden) = HiddenCards::from_view(&view) else {
                panic!("seed {seed}: inconsistent view");
            };
            let Some(worlds) = hidden.enumerate(20_000) else {
                continue;
            };
            for &action in &view.legal_actions {
                let Action::Play(card) = action else { continue };
                let expected = reference(&view, card, &worlds);
                assert_eq!(
                    mechanisms(&view, action),
                    expected,
                    "seed {seed}, {cards} cards, {card:?}"
                );
                compared += 1;
                for m in &expected {
                    let i = match m {
                        Mechanism::Abstich(a) => a.certainty as usize,
                        Mechanism::Overtake(o) => 2 + o.certainty as usize,
                    };
                    found[i] += 1;
                }
            }
        }
    }
    (compared, found)
}

#[test]
fn mechanisms_match_an_exhaustive_reference() {
    let (compared, found) = compare_with_reference(0..25);
    assert!(compared > 50, "{compared}");
    assert!(found.iter().sum::<usize>() > 0, "{found:?}");
}

#[test]
#[ignore = "slow: CI only (docs/testing.md)"]
fn mechanisms_match_an_exhaustive_reference_on_many_positions() {
    let (compared, found) = compare_with_reference(0..400);
    // The reference must have seen every case, so the comparison means
    // something: possible and certain for both mechanisms.
    assert!(compared > 1000, "{compared}");
    assert!(found.iter().all(|&n| n > 0), "{found:?}");
}
