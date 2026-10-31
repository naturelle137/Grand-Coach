// SPDX-License-Identifier: EUPL-1.2

//! The two pre-play lessons of the catalogue (#45, `docs/coach.md`): own
//! hand structure and the legal game values.
//!
//! Every expected Trumpf set, Spitzen count, value range and lesson result
//! below is worked out by hand from the deal in the comments, not taken
//! from the code under test. Rules: Spitzen count the declarer's cards
//! including the Skat, also in Hand games (ISkO 2.4); game value = base
//! value × (Spitzen + 1 + levels): Hand 1, announced Schneider 2, announced
//! Schwarz 2, Ouvert 1 (ISkO 5.1); base values Karo 9, Herz 10, Pik 11,
//! Kreuz 12, Grand 24; Null 23 / Hand 35 / Ouvert 46 / Hand Ouvert 59; a
//! game worth less than the bid is lost as überreizt (ISkO 5.4).
//! `ranges_match_an_independent_enumeration` checks the ranges against
//! plain code over every possible Skat of random deals, and against the
//! real Skat.

mod common;

use common::*;
use skat_analysis::lesson::{
    Check, Claim, Condition, Evidence, EvidenceClass, FactKind, GAME_VALUE, HAND_STRUCTURE,
    LessonKind, Role, Source, Support, Unmet, catalogue, gather,
};
use skat_analysis::preplay::{GameValues, Range, Reach, SkatState, Structure};
use skat_analysis::{Budget, Cancel, Observation, Status, analyze};
use skat_core::{
    Action, Card, CardSet, Deal, Game, GameType, Phase, PlayerView, Rank, RuleConfig, Seat,
    Spitzen, Suit,
};

/// Small and node-limited: exact evidence must not depend on the search.
const TINY: Budget = Budget {
    samples: 2,
    nodes_per_task: 20_000,
};

/// MH holds the Kreuz and Herz Bube and seven Herz Trumpf; HH holds no
/// Bube; the Skat holds DJ and C7.
fn deal_d() -> Deal {
    deal(
        "SJ CA CT CK CQ C9 C8 ST SK SQ",
        "CJ HJ HA HT HK H9 H7 SA S7 D7",
        "S9 S8 HQ H8 DA DT DK DQ D9 D8",
        "DJ C7",
    )
}

/// MH holds all four Buben.
fn deal_e() -> Deal {
    deal(
        "CK CQ C9 C8 C7 SK SQ S9 S8 S7",
        "CJ SJ HJ DJ CA CT SA ST HA HT",
        "HK HQ H9 H8 H7 DA DT DK DQ D9",
        "D8 D7",
    )
}

/// MH bids 48, VH and HH pass: MH declarer at 48.
const MH_AT_48: [Action; 3] = [Action::Bid(48), Action::Pass, Action::Pass];

fn play(deal: Deal, config: RuleConfig, actions: &[Action]) -> Game {
    let mut g = Game::from_deal(deal, config);
    for &a in actions {
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

fn view(deal: Deal, actions: &[Action]) -> PlayerView {
    to_move(&play(deal, RuleConfig::default(), actions))
}

fn evidence(view: &PlayerView) -> Evidence {
    gather(view, &analyze(view, TINY, 1, None).expect("consistent"))
}

fn lesson(key: &str) -> skat_analysis::lesson::Lesson {
    catalogue()
        .into_iter()
        .find(|l| l.id.key == key)
        .expect("in the catalogue")
}

fn of_kind(e: &Evidence, kind: FactKind) -> Vec<usize> {
    e.items
        .iter()
        .enumerate()
        .filter(|(_, i)| matches!(i.claim, Claim::Fact(o) if FactKind::of(&o) == kind))
        .map(|(i, _)| i)
        .collect()
}

fn structures(e: &Evidence) -> Vec<Structure> {
    e.items
        .iter()
        .filter_map(|i| match i.claim {
            Claim::Fact(Observation::Structure(s)) => Some(s),
            _ => None,
        })
        .collect()
}

fn values(e: &Evidence) -> Vec<GameValues> {
    e.items
        .iter()
        .filter_map(|i| match i.claim {
            Claim::Fact(Observation::GameValues(v)) => Some(v),
            _ => None,
        })
        .collect()
}

fn commitment(e: &Evidence) -> Option<(u16, bool)> {
    e.items.iter().find_map(|i| match i.claim {
        Claim::Fact(Observation::Commitment { value, kontra_re }) => Some((value, kontra_re)),
        _ => None,
    })
}

fn r(low: u16, high: u16) -> Option<Range> {
    Some(Range { low, high })
}

fn fixed(v: u16) -> Option<Range> {
    r(v, v)
}

fn mit(count: u8) -> Spitzen {
    Spitzen { mit: true, count }
}

fn ohne(count: u8) -> Spitzen {
    Spitzen { mit: false, count }
}

const KREUZ: GameType = GameType::Farbe(Suit::Kreuz);
const PIK: GameType = GameType::Farbe(Suit::Pik);
const HERZ: GameType = GameType::Farbe(Suit::Herz);
const KARO: GameType = GameType::Farbe(Suit::Karo);

/// The reach of each contract of `v` against `value`, in contract order.
fn reaches(v: &GameValues, value: u16) -> Vec<Reach> {
    v.against(value).iter().map(|c| c.reach).collect()
}

#[test]
fn the_catalogue_ends_with_the_two_pre_play_lessons() {
    let keys: Vec<String> = catalogue().iter().map(|l| l.id.key.to_string()).collect();
    assert_eq!(keys[keys.len() - 2..], [GAME_VALUE, HAND_STRUCTURE]);
    for key in [GAME_VALUE, HAND_STRUCTURE] {
        let l = lesson(key);
        assert_eq!(l.id.version, 1);
        assert_eq!(l.kind, LessonKind::Fact, "exact, no option quality");
        assert_eq!(l.support, Support::Supported);
        assert_eq!(l.applies.roles, [Role::Bidder, Role::Declarer]);
        assert!(l.applies.games.is_empty(), "no game announced yet");
        assert_eq!(l.permitted, [EvidenceClass::Exact]);
    }
    let value = lesson(GAME_VALUE);
    assert_eq!(
        value.applies.phases,
        [Phase::Reizen, Phase::SkatAufnahme, Phase::Ansage]
    );
    assert_eq!(
        value.requires,
        [
            Condition::Fact(FactKind::GameValues),
            Condition::Fact(FactKind::Commitment)
        ]
    );
    assert_eq!(value.excludes, [Condition::OpeningCall]);
    let structure = lesson(HAND_STRUCTURE);
    assert_eq!(structure.applies.phases, [Phase::Reizen, Phase::Druecken]);
    assert_eq!(structure.requires, [Condition::Fact(FactKind::Structure)]);
    assert!(structure.excludes.is_empty());
}

#[test]
fn opening_call_teaches_the_structure_with_an_unknown_skat() {
    // MH opens the bidding: CJ HJ HA HT HK H9 H7 SA S7 D7. Unseen are the
    // other 22 cards, among them SJ, DJ, HQ and H8; any two may be the Skat.
    let v = view(deal_d(), &[]);
    assert_eq!(v.seat, Seat::Mittelhand);
    let e = evidence(&v);
    assert_eq!(e.snapshot.own_calls, 0);
    let buben = cards("CJ HJ");
    // Spitzen with own cards: CJ yes, SJ no → mit 1 in every game. The
    // most: SJ and DJ in the Skat give the run CJ SJ HJ DJ and then:
    // Kreuz CA missing → 4; Pik SA own, ST missing → 5; Herz HA HT HK own,
    // HQ missing → 7; Karo DA missing → 4; Grand (4 Buben) → 4. At most two
    // more Trumpf: Kreuz 2+2, Pik 4+2, Herz 7+2, Karo 3+2, Grand 2+2.
    let expect = [
        (KREUZ, "CJ HJ", 4, 4),
        (PIK, "CJ HJ SA S7", 5, 6),
        (HERZ, "CJ HJ HA HT HK H9 H7", 7, 9),
        (KARO, "CJ HJ D7", 4, 5),
        (GameType::Grand, "CJ HJ", 4, 4),
    ];
    let got = structures(&e);
    assert_eq!(got.len(), 5);
    for (s, (game, trumpf, most, trumpf_most)) in got.iter().zip(expect) {
        assert_eq!(
            *s,
            Structure {
                game,
                skat: SkatState::Unknown,
                buben,
                trumpf: cards(trumpf),
                skat_trumpf: CardSet::EMPTY,
                spitzen: mit(1),
                counts: Range { low: 1, high: most },
                trumpf_most,
            },
            "{game:?}"
        );
    }
    // Values, Hand still open: plain base × (1..most + 1), Hand one level
    // more, Hand Ouvert six levels more (Hand, Schneider ×2, Schwarz ×2,
    // Ouvert). Kreuz 12 × 2..5, Pik 11 × 2..6, Herz 10 × 2..8, Karo 9 ×
    // 2..5, Grand 24 × 2..5.
    let got = values(&e);
    let expect = [
        (KREUZ, r(24, 60), r(36, 72), None, r(96, 132)),
        (PIK, r(22, 66), r(33, 77), None, r(88, 132)),
        (HERZ, r(20, 80), r(30, 90), None, r(80, 140)),
        (KARO, r(18, 45), r(27, 54), None, r(72, 99)),
        (GameType::Grand, r(48, 120), r(72, 144), None, r(192, 264)),
        (GameType::Null, fixed(23), fixed(35), fixed(46), fixed(59)),
    ];
    assert_eq!(got.len(), 6);
    for (v, (game, plain, hand, ouvert, hand_ouvert)) in got.iter().zip(expect) {
        assert_eq!(
            *v,
            GameValues {
                game,
                skat: SkatState::Unknown,
                plain,
                hand,
                ouvert,
                hand_ouvert,
            },
            "{game:?}"
        );
    }
    // The first call commits to at least 18.
    assert_eq!(commitment(&e), Some((18, false)));
    let structure = of_kind(&e, FactKind::Structure);
    assert_eq!(
        lesson(HAND_STRUCTURE).check(&e, None),
        Check::Applies {
            items: structure.clone()
        }
    );
    for &i in &structure {
        assert_eq!(e.items[i].sources, [Source::OwnHand]);
    }
    // Counterexample: before the own first call the value lesson waits.
    assert_eq!(
        lesson(GAME_VALUE).check(&e, None),
        Check::Fails(Unmet::Excluded(Condition::OpeningCall))
    );
}

#[test]
fn ohne_spitzen_shrink_or_flip_with_an_unknown_skat() {
    // MH passes, HH opens: S9 S8 HQ H8 DA DT DK DQ D9 D8, no Bube.
    // Grand: ohne 4. Skat CJ → mit 1 (CJ SJ: mit 2), SJ alone → ohne 1, so
    // 1..4. Kreuz: no Kreuz card either → ohne 11; CJ in the Skat → mit 1,
    // no Kreuz Trumpf in the Skat → ohne 11 stays: 1..11.
    let v = view(deal_d(), &[Action::Pass]);
    assert_eq!(v.seat, Seat::Hinterhand);
    let e = evidence(&v);
    let s = structures(&e);
    let (kreuz, grand) = (s[0], s[4]);
    assert_eq!(grand.game, GameType::Grand);
    assert_eq!(grand.spitzen, ohne(4));
    assert_eq!(grand.counts, Range { low: 1, high: 4 });
    assert_eq!(grand.trumpf, CardSet::EMPTY);
    assert_eq!(grand.trumpf_most, 2);
    assert_eq!(kreuz.spitzen, ohne(11));
    assert_eq!(kreuz.counts, Range { low: 1, high: 11 });
    // Values: Grand 24 × 2..5, Kreuz 12 × 2..12.
    let v = values(&e);
    assert_eq!(v[4].plain, r(48, 120));
    assert_eq!(v[0].plain, r(24, 144));
    assert_eq!(
        lesson(HAND_STRUCTURE).check(&e, None),
        Check::Applies {
            items: of_kind(&e, FactKind::Structure)
        }
    );
}

#[test]
fn an_unknown_skat_cannot_change_four_buben_in_grand() {
    // Counterexample to "the Skat changes the Spitzen": MH holds all four
    // Buben, so Grand stays mit 4 and its values are fixed: 24 × 5 = 120,
    // Hand 144, Hand Ouvert 24 × 11 = 264. Kreuz still changes: CJ SJ HJ
    // DJ CA CT own, CK missing → mit 6; CK and CQ in the Skat → mit 8.
    let e = evidence(&view(deal_e(), &[]));
    let s = structures(&e);
    assert_eq!(s[4].spitzen, mit(4));
    assert_eq!(s[4].counts, Range { low: 4, high: 4 });
    assert_eq!(s[4].trumpf_most, 4, "no Bube unseen");
    assert_eq!(s[0].spitzen, mit(6));
    assert_eq!(s[0].counts, Range { low: 6, high: 8 });
    let v = values(&e);
    assert_eq!(v[4].plain, fixed(120));
    assert_eq!(v[4].hand, fixed(144));
    assert_eq!(v[4].hand_ouvert, fixed(264));
    assert_eq!(v[0].plain, r(84, 108), "12 × 7..9");
}

#[test]
fn a_later_call_teaches_the_values_against_the_next_bid() {
    // MH 18, VH holds: MH to bid again, at least 20. MH has called once.
    let v = view(deal_d(), &[Action::Bid(18), Action::Hold]);
    assert_eq!(v.seat, Seat::Mittelhand);
    let e = evidence(&v);
    assert_eq!(e.snapshot.own_calls, 1);
    assert_eq!(commitment(&e), Some((20, false)));
    let mut cited = of_kind(&e, FactKind::GameValues);
    cited.extend(of_kind(&e, FactKind::Commitment));
    cited.sort_unstable();
    assert_eq!(
        lesson(GAME_VALUE).check(&e, None),
        Check::Applies { items: cited }
    );
    let c = of_kind(&e, FactKind::Commitment)[0];
    assert_eq!(e.items[c].sources, [Source::Bidding, Source::Rules]);
    let g = of_kind(&e, FactKind::GameValues)[0];
    assert_eq!(e.items[g].sources, [Source::OwnHand, Source::Rules]);
    // Boundaries at 20: Herz plain 20..80 reaches it exactly; Karo plain
    // 18..45 only with a suitable Skat (mit 2 or more: 27); Karo Hand
    // 27..54 reaches it.
    let v = values(&e);
    use Reach::*;
    assert_eq!(reaches(&v[2], 20), [Reached, Reached, Reached]);
    assert_eq!(reaches(&v[3], 20), [Skat, Reached, Reached]);
    assert_eq!(reaches(&v[5], 20), [Reached; 4], "Null 23 and up");
    // The structure lesson would also apply; the value lesson comes first.
    assert!(matches!(
        lesson(HAND_STRUCTURE).check(&e, None),
        Check::Applies { .. }
    ));
}

#[test]
fn pickup_decision_shows_ueberreizen_and_the_hand_level() {
    // MH declarer at 48, deciding Hand or pickup. The ranges are those of
    // the opening call (same cards, Skat unknown); against 48:
    // Kreuz 24..60 / 36..72 / 96..132 → Skat, Skat, reached.
    // Karo 18..45 → only Schneider or Schwarz made in play can reach 48;
    //   Hand 27..54 → Skat; Hand Ouvert 72..99 → reached.
    // Grand 48..120 → reached (boundary: equal).
    // Null 23, 35, 46 → never (fixed values); Null Hand Ouvert 59 → reached.
    let v = view(deal_d(), &MH_AT_48);
    assert_eq!(v.phase, Phase::SkatAufnahme);
    let e = evidence(&v);
    assert_eq!(commitment(&e), Some((48, false)));
    let v = values(&e);
    use Reach::*;
    assert_eq!(reaches(&v[0], 48), [Skat, Skat, Reached]);
    assert_eq!(reaches(&v[3], 48), [Play, Skat, Reached]);
    assert_eq!(reaches(&v[4], 48), [Reached, Reached, Reached]);
    assert_eq!(reaches(&v[5], 48), [Never, Never, Never, Reached]);
    assert!(matches!(
        lesson(GAME_VALUE).check(&e, None),
        Check::Applies { .. }
    ));
    // Counterexample: not a structure phase.
    assert_eq!(
        lesson(HAND_STRUCTURE).check(&e, None),
        Check::Fails(Unmet::Phase)
    );
}

#[test]
fn twelve_cards_after_pickup_fix_the_spitzen_but_not_the_discard() {
    // MH picked up DJ C7: twelve cards, Skat held. Spitzen count all twelve:
    // CJ yes, SJ no → mit 1 everywhere, no range. Trumpf among the twelve:
    // Kreuz CJ HJ DJ C7, Pik CJ HJ DJ SA S7, Herz CJ HJ DJ + 5 Herz,
    // Karo CJ HJ DJ D7, Grand CJ HJ DJ.
    let mut actions = MH_AT_48.to_vec();
    actions.push(Action::TakeSkat);
    let v = view(deal_d(), &actions);
    assert_eq!(v.phase, Phase::Druecken);
    assert_eq!(v.hand.len(), 12);
    let e = evidence(&v);
    let expect = [
        (KREUZ, "CJ HJ DJ C7"),
        (PIK, "CJ HJ DJ SA S7"),
        (HERZ, "CJ HJ DJ HA HT HK H9 H7"),
        (KARO, "CJ HJ DJ D7"),
        (GameType::Grand, "CJ HJ DJ"),
    ];
    for (s, (game, trumpf)) in structures(&e).iter().zip(expect) {
        assert_eq!(
            *s,
            Structure {
                game,
                skat: SkatState::Held,
                buben: cards("CJ HJ DJ"),
                trumpf: cards(trumpf),
                skat_trumpf: CardSet::EMPTY,
                spitzen: mit(1),
                counts: Range { low: 1, high: 1 },
                trumpf_most: cards(trumpf).len() as u8,
            }
        );
    }
    assert!(matches!(
        lesson(HAND_STRUCTURE).check(&e, None),
        Check::Applies { .. }
    ));
    assert_eq!(
        lesson(GAME_VALUE).check(&e, None),
        Check::Fails(Unmet::Phase)
    );
}

#[test]
fn announcing_after_the_discard_uses_the_known_skat() {
    // MH put away DJ and H7. Spitzen still count them: mit 1 everywhere.
    // Plain values only (pickup): Kreuz 24, Pik 22, Herz 20, Karo 18,
    // Grand 48, Null 23 and Null Ouvert 46. Against 48: the suit games are
    // überreizt unless Schneider or Schwarz is made in play; Grand reaches
    // it exactly; Null never.
    let mut actions = MH_AT_48.to_vec();
    actions.push(Action::TakeSkat);
    actions.push(Action::Discard(cards("DJ H7")));
    let v = view(deal_d(), &actions);
    assert_eq!(v.phase, Phase::Ansage);
    let e = evidence(&v);
    let s = structures(&e);
    assert_eq!(s[2].skat, SkatState::Discarded);
    assert_eq!(s[2].trumpf, cards("CJ HJ HA HT HK H9"));
    assert_eq!(s[2].skat_trumpf, cards("DJ H7"));
    assert_eq!(s[2].buben, cards("CJ HJ DJ"));
    assert_eq!(s[2].spitzen, mit(1));
    assert_eq!(s[4].trumpf, cards("CJ HJ"));
    assert_eq!(s[4].skat_trumpf, cards("DJ"));
    let v = values(&e);
    let plain: Vec<_> = v.iter().map(|g| g.plain).collect();
    assert_eq!(
        plain,
        [
            fixed(24),
            fixed(22),
            fixed(20),
            fixed(18),
            fixed(48),
            fixed(23)
        ]
    );
    assert!(
        v.iter()
            .all(|g| g.hand.is_none() && g.hand_ouvert.is_none())
    );
    assert_eq!(v[5].ouvert, fixed(46));
    use Reach::*;
    for g in &v[..4] {
        assert_eq!(reaches(g, 48), [Play]);
    }
    assert_eq!(reaches(&v[4], 48), [Reached]);
    assert_eq!(reaches(&v[5], 48), [Never, Never]);
    let i = of_kind(&e, FactKind::GameValues)[0];
    assert_eq!(
        e.items[i].sources,
        [Source::OwnHand, Source::OwnSkat, Source::Rules]
    );
    assert!(matches!(
        lesson(GAME_VALUE).check(&e, None),
        Check::Applies { .. }
    ));
}

#[test]
fn announcing_a_hand_game_keeps_the_skat_ranges() {
    // Hand at 48: Skat unknown, only Hand contracts. Ranges as at the
    // opening call: Herz Hand 30..90 → Skat; Grand Hand 72..144 → reached;
    // Null Hand 35 → never; Null Hand Ouvert 59 → reached.
    let mut actions = MH_AT_48.to_vec();
    actions.push(Action::Hand);
    let v = view(deal_d(), &actions);
    assert_eq!((v.phase, v.hand_game), (Phase::Ansage, Some(true)));
    let e = evidence(&v);
    let v = values(&e);
    assert!(v.iter().all(|g| g.plain.is_none() && g.ouvert.is_none()));
    assert_eq!(v[2].hand, r(30, 90));
    assert_eq!(v[4].hand, r(72, 144));
    use Reach::*;
    assert_eq!(reaches(&v[2], 48), [Skat, Reached]);
    assert_eq!(reaches(&v[5], 48), [Never, Reached]);
    assert_eq!(structures(&e)[2].counts, Range { low: 1, high: 7 });
}

#[test]
fn kontra_re_changes_no_value() {
    // Same position with Kontra/Re configured: only the flag differs.
    let config = RuleConfig { kontra_re: true };
    let on = evidence(&to_move(&play(deal_d(), config, &MH_AT_48)));
    let off = evidence(&view(deal_d(), &MH_AT_48));
    assert_eq!(commitment(&on), Some((48, true)));
    assert_eq!(values(&on), values(&off));
    assert_eq!(structures(&on), structures(&off));
}

#[test]
fn only_the_bidder_or_declarer_gets_these_facts() {
    // While MH decides Hand or pickup, VH (a future defender) has no
    // decision and no structure or value facts; nor does anyone in card
    // play.
    let g = play(deal_d(), RuleConfig::default(), &MH_AT_48);
    let vh = g.view(Seat::Vorhand);
    let none = |v: &PlayerView| {
        let a = analyze(v, TINY, 1, None).expect("consistent");
        a.observations.iter().all(|o| {
            !matches!(
                o,
                Observation::Structure(_)
                    | Observation::GameValues(_)
                    | Observation::Commitment { .. }
            )
        })
    };
    assert!(none(&vh));
    let mut actions = MH_AT_48.to_vec();
    actions.push(Action::Hand);
    actions.push(Action::Announce(skat_core::Announcement::plain(
        GameType::Grand,
    )));
    let spiel = view(deal_d(), &actions);
    assert_eq!(spiel.phase, Phase::Spiel);
    assert!(none(&spiel));
    let e = evidence(&spiel);
    for key in [GAME_VALUE, HAND_STRUCTURE] {
        assert_eq!(lesson(key).check(&e, None), Check::Fails(Unmet::Phase));
    }
}

#[test]
fn hidden_cards_do_not_change_the_evidence() {
    // Swap SJ (VH) with HQ (HH), and DJ (Skat) with SK (VH): MH sees the
    // same view at its second call, so evidence and lessons are equal.
    let b = deal(
        "HQ CA CT CK CQ C9 C8 ST DJ SQ",
        "CJ HJ HA HT HK H9 H7 SA S7 D7",
        "S9 S8 SJ H8 DA DT DK DQ D9 D8",
        "SK C7",
    );
    let actions = [Action::Bid(18), Action::Hold];
    let views = [deal_d(), b].map(|d| view(d, &actions));
    assert_eq!(views[0], views[1], "precondition: one information set");
    let [a, b] = views.map(|v| evidence(&v));
    assert_eq!(a, b);
    for l in catalogue() {
        assert_eq!(l.check(&a, None), l.check(&b, None));
    }
}

#[test]
fn exact_facts_hold_in_an_incomplete_analysis() {
    let v = view(deal_d(), &[Action::Bid(18), Action::Hold]);
    let cancel = Cancel::new();
    cancel.cancel();
    let a = analyze(&v, TINY, 1, Some(&cancel)).expect("consistent");
    assert_ne!(a.status, Status::Complete);
    let cancelled = gather(&v, &a);
    let exact = |e: &Evidence| -> Vec<_> {
        e.items
            .iter()
            .filter(|i| i.class() == EvidenceClass::Exact)
            .cloned()
            .collect()
    };
    let full = evidence(&v);
    assert_eq!(exact(&cancelled), exact(&full));
    assert_eq!(
        lesson(GAME_VALUE).check(&cancelled, None),
        lesson(GAME_VALUE).check(&full, None)
    );
    assert!(matches!(
        lesson(GAME_VALUE).check(&cancelled, None),
        Check::Applies { .. }
    ));
}

#[test]
fn pre_play_facts_are_deterministic() {
    let v = view(deal_d(), &MH_AT_48);
    assert_eq!(evidence(&v), evidence(&v));
}

/// A small deterministic generator for the random deals below.
fn next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state >> 33
}

/// Trumpf order written out by hand: the four Buben, then for a suit game
/// Ass, Zehn, König, Dame, Neun, Acht, Sieben of that suit.
fn order(game: GameType) -> Vec<Card> {
    let mut out: Vec<Card> = [Suit::Kreuz, Suit::Pik, Suit::Herz, Suit::Karo]
        .into_iter()
        .map(|s| Card::new(s, Rank::Bube))
        .collect();
    if let GameType::Farbe(suit) = game {
        for rank in [
            Rank::Ass,
            Rank::Zehn,
            Rank::Koenig,
            Rank::Dame,
            Rank::Neun,
            Rank::Acht,
            Rank::Sieben,
        ] {
            out.push(Card::new(suit, rank));
        }
    }
    out
}

/// Length of the run held or missing from the top (ISkO 2.4).
fn run(order: &[Card], cards: CardSet) -> u16 {
    let first = cards.contains(order[0]);
    order
        .iter()
        .take_while(|&&c| cards.contains(c) == first)
        .count() as u16
}

#[test]
fn ranges_match_an_independent_enumeration() {
    // Oracle: plain code over every pair of cards MH does not hold, and the
    // real Skat of the deal, which must lie within the ranges.
    let mut state = 45;
    for _ in 0..40 {
        let deal = Deal::from_seed(next(&mut state));
        let g = Game::from_deal(deal, RuleConfig::default());
        let v = g.view(Seat::Mittelhand);
        let a = analyze(&v, TINY, 1, None).expect("consistent");
        let structures: Vec<Structure> = a
            .observations
            .iter()
            .filter_map(|o| match o {
                Observation::Structure(s) => Some(*s),
                _ => None,
            })
            .collect();
        let values: Vec<GameValues> = a
            .observations
            .iter()
            .filter_map(|o| match o {
                Observation::GameValues(v) => Some(*v),
                _ => None,
            })
            .collect();
        let unseen: Vec<Card> = (CardSet::ALL - v.hand).iter().collect();
        let real = g.omniscient_view().skat;
        for (i, game) in [KREUZ, PIK, HERZ, KARO, GameType::Grand]
            .into_iter()
            .enumerate()
        {
            let order = order(game);
            let base = match game {
                GameType::Farbe(Suit::Kreuz) => 12,
                GameType::Farbe(Suit::Pik) => 11,
                GameType::Farbe(Suit::Herz) => 10,
                GameType::Farbe(Suit::Karo) => 9,
                _ => 24,
            };
            let (mut low, mut high) = (u16::MAX, 0);
            for (x, &a) in unseen.iter().enumerate() {
                for &b in &unseen[x + 1..] {
                    let n = run(&order, v.hand | CardSet::single(a) | CardSet::single(b));
                    low = low.min(n);
                    high = high.max(n);
                }
            }
            let s = structures[i];
            assert_eq!(s.game, game);
            assert_eq!(s.counts, Range { low, high }, "{game:?}");
            let truth = run(&order, v.hand | real);
            assert!((low..=high).contains(&truth));
            assert_eq!(values[i].plain, r(base * (low + 1), base * (high + 1)));
            assert_eq!(values[i].hand, r(base * (low + 2), base * (high + 2)));
            assert_eq!(
                values[i].hand_ouvert,
                r(base * (low + 7), base * (high + 7))
            );
        }
    }
}
