// SPDX-License-Identifier: EUPL-1.2

//! Serialisation behind the `serde` feature (D-25).

#![cfg(feature = "serde")]

use skat_core::{Action, Card, Game, RuleConfig, Seat};

#[test]
fn game_and_view_roundtrip() {
    let mut game = Game::new(99, RuleConfig { kontra_re: true });
    for action in [
        Action::Pass,
        Action::Pass,
        Action::Bid(18),
        Action::TakeSkat,
    ] {
        game.apply(action).unwrap();
    }
    // First discard, first announcement, then a few cards.
    for _ in 0..5 {
        let action = game.legal_actions()[0];
        game.apply(action).unwrap();
    }
    let json = serde_json::to_string(&game).unwrap();
    assert_eq!(serde_json::from_str::<Game>(&json).unwrap(), game);
    let view = game.view(Seat::Mittelhand);
    let json = serde_json::to_string(&view).unwrap();
    assert_eq!(
        serde_json::from_str::<skat_core::PlayerView>(&json).unwrap(),
        view
    );
    let actions: Vec<Action> = game.legal_actions();
    let json = serde_json::to_string(&actions).unwrap();
    assert_eq!(serde_json::from_str::<Vec<Action>>(&json).unwrap(), actions);
}

#[test]
fn invalid_card_index_is_rejected() {
    assert_eq!(serde_json::from_str::<Card>("31").unwrap().index(), 31);
    assert!(serde_json::from_str::<Card>("32").is_err());
}
