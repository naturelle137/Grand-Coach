// SPDX-License-Identifier: EUPL-1.2

//! Serialisation behind the `serde` feature (D-25).

#![cfg(feature = "serde")]

mod common;

use skat_core::{
    Action, Announcement, Card, CardSet, Deal, Game, GameType, Rank, RuleConfig, Seat, Suit,
};

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

/// Mittelhand plays Herz-Neun on a Kreuz lead while still holding Kreuz-Zehn.
/// Card ownership, hand sizes and conservation all stay intact.
#[test]
fn off_suit_play_in_the_current_trick_is_rejected() {
    let mut game = Game::new(0, RuleConfig::default());
    for action in [
        Action::Pass,
        Action::Pass,
        Action::Bid(18),
        Action::Hand,
        Action::Announce(Announcement::plain(GameType::Grand)),
    ] {
        game.apply(action).unwrap();
    }
    let c8 = Card::new(Suit::Kreuz, Rank::Acht);
    let ct = Card::new(Suit::Kreuz, Rank::Zehn);
    let h9 = Card::new(Suit::Herz, Rank::Neun);
    game.apply(Action::Play(c8)).unwrap();
    assert!(game.validate(Action::Play(h9)).is_err());
    game.apply(Action::Play(ct)).unwrap();

    let mut json = serde_json::to_value(&game).unwrap();
    assert!(serde_json::from_value::<Game>(json.clone()).is_ok());
    json["current"]["cards"][1] = serde_json::json!(h9.index());
    let hand = (game.view(Seat::Mittelhand).hand - CardSet::single(h9)) | CardSet::single(ct);
    json["hands"][Seat::Mittelhand.index()] = serde_json::json!(hand.bits());
    assert!(serde_json::from_value::<Game>(json).is_err());
}

/// The same violation inside a completed trick: Hinterhand plays Herz-Dame
/// on a Karo-Bube lead (Trumpf) while holding Karo.
#[test]
fn off_suit_play_in_a_completed_trick_is_rejected() {
    let mut game = common::game(common::standard_deal(), false);
    common::run(
        &mut game,
        &[
            Action::Pass,
            Action::Bid(18),
            Action::Pass,
            Action::Hand,
            Action::Announce(Announcement::plain(GameType::Farbe(Suit::Karo))),
        ],
    );
    common::play(&mut game, "CJ S7 D8  SJ S8 D9");
    let d9 = common::c("D9");
    let hq = common::c("HQ");

    let mut json = serde_json::to_value(&game).unwrap();
    assert!(serde_json::from_value::<Game>(json.clone()).is_ok());
    json["tricks"][1]["cards"][2] = serde_json::json!(hq.index());
    let hand = (game.view(Seat::Hinterhand).hand - CardSet::single(hq)) | CardSet::single(d9);
    json["hands"][Seat::Hinterhand.index()] = serde_json::json!(hand.bits());
    assert!(serde_json::from_value::<Game>(json).is_err());
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
