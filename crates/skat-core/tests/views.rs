// SPDX-License-Identifier: EUPL-1.2

//! Information sets: a player's view reveals only what that seat may know.

mod common;

use common::*;
use skat_core::{Action, Announcement, CardSet, Game, GameType, Phase, PlayerView, Seat, Suit};

/// Every card a view exposes.
fn visible(view: &PlayerView) -> CardSet {
    let mut set =
        view.hand | view.own_skat.unwrap_or_default() | view.declarer_cards.unwrap_or_default();
    for trick in &view.tricks {
        set |= trick.cards.iter().copied().collect::<CardSet>();
    }
    set |= view
        .current_trick
        .cards
        .iter()
        .flatten()
        .copied()
        .collect::<CardSet>();
    for action in &view.legal_actions {
        match action {
            Action::Play(card) => set.insert(*card),
            Action::Discard(cards) => {
                set |= *cards;
                true
            }
            _ => false,
        };
    }
    set
}

/// What `seat` may legitimately know, computed from the omniscient state.
fn allowed(game: &Game, seat: Seat) -> CardSet {
    let all = game.omniscient_view();
    let mut set = all.hands[seat.index()];
    for trick in game.tricks() {
        set |= trick.cards.iter().copied().collect::<CardSet>();
    }
    set |= game
        .current_trick()
        .cards
        .iter()
        .flatten()
        .copied()
        .collect::<CardSet>();
    if game.declarer() == Some(seat) && game.view(seat).hand_game == Some(false) {
        set |= all.original_skat | all.skat;
    }
    if game.is_revealed() {
        set |= all.hands[game.declarer().unwrap().index()];
    }
    set
}

fn check_all_views(game: &Game) {
    for seat in Seat::ALL {
        let view = game.view(seat);
        let all = game.omniscient_view();
        assert_eq!(view.hand, all.hands[seat.index()]);
        assert!(
            visible(&view).is_subset(allowed(game, seat)),
            "{seat:?} sees too much: {view:?}"
        );
        if game.current_seat() != Some(seat) {
            assert!(view.legal_actions.is_empty());
        }
        if game.declarer() != Some(seat) {
            assert_eq!(view.own_skat, None);
        }
        for other in Seat::ALL {
            if other != seat && !(game.is_revealed() && game.declarer() == Some(other)) {
                assert!(view.hand.is_disjoint(all.hands[other.index()]));
                assert!(visible(&view).is_disjoint(all.hands[other.index()]));
            }
        }
        if !(game.declarer() == Some(seat) && view.hand_game == Some(false)) {
            assert!(visible(&view).is_disjoint(all.skat));
        }
    }
}

fn drive(game: &mut Game, actions: &[Action]) {
    check_all_views(game);
    for &action in actions {
        game.apply(action).unwrap();
        check_all_views(game);
    }
    while game.phase() == Phase::Spiel {
        let action = game.legal_actions()[0];
        game.apply(action).unwrap();
        check_all_views(game);
    }
}

#[test]
fn closed_game_with_pickup() {
    let mut g = game(standard_deal(), true);
    drive(
        &mut g,
        &[
            Action::Pass,
            Action::Pass,
            Action::Bid(18),
            Action::TakeSkat,
            Action::Discard(cards("SJ CA")),
            Action::Announce(Announcement::plain(GameType::Farbe(Suit::Kreuz))),
        ],
    );
    assert_eq!(g.view(Seat::Vorhand).own_skat, Some(cards("SJ CA")));
    assert_eq!(g.phase(), Phase::Beendet);
}

#[test]
fn hand_game_hides_the_skat_from_everyone() {
    let mut g = game(standard_deal(), true);
    drive(
        &mut g,
        &[
            Action::Bid(18),
            Action::Pass,
            Action::Pass,
            Action::Hand,
            Action::Announce(Announcement::plain(GameType::Grand)),
        ],
    );
    for seat in Seat::ALL {
        let view = g.view(seat);
        assert_eq!(view.own_skat, None);
        assert!(visible(&view).is_disjoint(cards("C7 D7")));
    }
}

#[test]
fn ouvert_shows_declarer_cards_only_after_reveal() {
    let mut g = game(standard_deal(), true);
    let grand_ouvert = Announcement {
        schneider: true,
        schwarz: true,
        ouvert: true,
        ..Announcement::plain(GameType::Grand)
    };
    run(
        &mut g,
        &[
            Action::Bid(18),
            Action::Hold,
            Action::Bid(20),
            Action::Pass,
            Action::Pass,
        ],
    );
    run(&mut g, &[Action::Hand, Action::Announce(grand_ouvert)]);
    assert!(!g.is_revealed());
    check_all_views(&g);
    assert_eq!(g.view(Seat::Vorhand).declarer_cards, None);
    drive(&mut g, &[Action::KeinKontra, Action::KeinKontra]);
    assert!(g.is_revealed());
}

#[test]
fn null_ouvert_after_pickup_hides_the_discard() {
    let mut g = game(standard_deal(), false);
    let null_ouvert = Announcement {
        ouvert: true,
        ..Announcement::plain(GameType::Null)
    };
    drive(
        &mut g,
        &[
            Action::Bid(18),
            Action::Pass,
            Action::Pass,
            Action::TakeSkat,
            Action::Discard(cards("SA ST")),
            Action::Announce(null_ouvert),
        ],
    );
    for seat in Seat::ALL {
        assert!(
            g.view(seat)
                .declarer_cards
                .is_some_and(|d| d.is_disjoint(cards("SA ST")))
        );
    }
}
