// SPDX-License-Identifier: EUPL-1.2

//! Serialisation behind the `serde` feature (D-25).

#![cfg(feature = "serde")]

use skat_core::{Action, Card, Deal, Game, RuleConfig, Seat};

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

#[test]
fn invalid_deal_is_rejected() {
    for json in [
        r#"{"hands":[0,0,0],"skat":0}"#,
        r#"{"hands":[1023,1047552,1072693248],"skat":0}"#,
    ] {
        assert!(serde_json::from_str::<Deal>(json).is_err(), "{json}");
    }
    let deal = Deal::from_seed(5);
    let json = serde_json::to_string(&deal).unwrap();
    assert_eq!(serde_json::from_str::<Deal>(&json).unwrap(), deal);
}

#[test]
fn game_with_invalid_deal_is_rejected() {
    let mut json = serde_json::to_value(Game::new(1, RuleConfig::default())).unwrap();
    json["deal"] = serde_json::json!({"hands": [0, 0, 0], "skat": 0});
    assert!(serde_json::from_value::<Game>(json).is_err());
}

#[test]
fn inconsistent_game_is_rejected() {
    let fresh = serde_json::to_value(Game::new(1, RuleConfig::default())).unwrap();
    let mutations = [
        ("phase", serde_json::json!("Spiel")),
        ("phase", serde_json::json!("Beendet")),
        ("phase", serde_json::json!("Eingepasst")),
        ("declarer", serde_json::json!("Vorhand")),
        ("hands", serde_json::json!([0, 0, 0])),
        ("revealed", serde_json::json!(true)),
    ];
    for (field, value) in mutations {
        let mut json = fresh.clone();
        json[field] = value;
        assert!(
            serde_json::from_value::<Game>(json).is_err(),
            "{field} accepted"
        );
    }
}

/// Every game reached by legal actions survives the round trip. Changing any
/// single field either is rejected or leaves a game that can still be queried
/// without panicking.
#[test]
fn reachable_games_roundtrip_and_mutations_never_panic() {
    for seed in 0..40 {
        let mut game = Game::new(seed, RuleConfig { kontra_re: true });
        loop {
            let json = serde_json::to_value(&game).unwrap();
            assert_eq!(serde_json::from_value::<Game>(json.clone()).unwrap(), game);
            for (field, other) in json.as_object().unwrap() {
                for value in [
                    serde_json::json!(null),
                    serde_json::json!("Spiel"),
                    serde_json::json!(0),
                    serde_json::json!(true),
                ] {
                    if &value == other {
                        continue;
                    }
                    let mut mutated = json.clone();
                    mutated[field] = value;
                    if let Ok(g) = serde_json::from_value::<Game>(mutated) {
                        let _ = g.legal_actions();
                        let _ = g.current_seat();
                        for seat in Seat::ALL {
                            let _ = g.view(seat);
                        }
                    }
                }
            }
            let actions = game.legal_actions();
            if actions.is_empty() {
                break;
            }
            let pick = actions[(seed as usize * 7 + game.tricks().len()) % actions.len()];
            game.apply(pick).unwrap();
        }
    }
}
