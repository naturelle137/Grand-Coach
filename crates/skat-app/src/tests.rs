// SPDX-License-Identifier: EUPL-1.2

//! Information boundary of the human's view, checked against the full game
//! state that only the session itself holds.

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};
use skat_core::{CardSet, Seat};

use crate::{Command, PlayerId, SeriesLength, Session, SessionConfig, TableView};

/// Every card the view shows.
fn shown_cards(view: &TableView) -> CardSet {
    let mut shown: CardSet = view.hand.iter().copied().collect();
    shown |= view.own_skat.iter().flatten().copied().collect();
    shown |= view.declarer_cards.iter().flatten().copied().collect();
    shown |= view.current_trick.iter().map(|p| p.card).collect();
    if let Some(trick) = &view.last_trick {
        shown |= trick.cards.iter().map(|p| p.card).collect();
    }
    shown
}

/// Cards the human may not see right now: the opponents' hands (except the
/// revealed cards of an Ouvert declarer) and the Skat unless the human
/// discarded it.
fn hidden_cards(session: &Session) -> CardSet {
    let omniscient = session.game.omniscient_view();
    let human = PlayerId::HUMAN.seat(session.dealer);
    let mut hidden = CardSet::EMPTY;
    for seat in Seat::ALL.into_iter().filter(|&s| s != human) {
        let revealed = session.game.is_revealed() && session.game.declarer() == Some(seat);
        if !revealed {
            hidden |= omniscient.hands[seat.index()];
        }
    }
    let human_discarded = omniscient.seats[human.index()].own_skat.is_some();
    if !human_discarded {
        hidden |= omniscient.skat;
    }
    hidden
}

#[test]
fn human_view_never_shows_hidden_cards() {
    let mut checked = 0;
    for seed in 0..30 {
        let mut session = Session::new(SessionConfig {
            seed,
            series: SeriesLength::Fixed(12),
            kontra_re: true,
        });
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        while !session.view().series_finished {
            let view = session.view();
            assert!(shown_cards(&view).is_disjoint(hidden_cards(&session)));
            assert_eq!(view.hand.len(), usize::from(view.players[0].cards));
            checked += 1;
            let command = if view.opponent_to_move {
                Command::AdvanceOpponent
            } else if view.legal_actions.is_empty() {
                Command::NextGame
            } else {
                let i = rng.next_u32() as usize % view.legal_actions.len().min(6);
                Command::Act(view.legal_actions[i])
            };
            session.execute(command).unwrap();
        }
    }
    assert!(checked > 5000);
}
