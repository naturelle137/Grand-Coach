// SPDX-License-Identifier: EUPL-1.2

//! Drives the real egui frontend headlessly with egui_kittest: every human
//! decision is a click on a widget found by its accessible label.

use eframe::egui::{self, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use gui_egui::{Deck, GrandCoachApp, Language};
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};
use skat_app::{
    Action, Announcement, Card, GameType, Phase, PlayerId, SeriesLength, SessionConfig, Suit,
    TableView,
};

fn harness(config: SessionConfig) -> Harness<'static, GrandCoachApp> {
    let mut harness = Harness::builder()
        .with_size([1600.0, 1000.0])
        .build_eframe(move |cc| GrandCoachApp::new(&cc.egui_ctx, config));
    harness.state_mut().opponent_delay = 0.0;
    harness.run_steps(2);
    harness
}

fn view(harness: &Harness<'_, GrandCoachApp>) -> TableView {
    harness.state().session().view()
}

fn click(harness: &mut Harness<'_, GrandCoachApp>, role: Role, label: &str) {
    harness.get_by_role_and_label(role, label).click();
    // Panels resize over a few frames; let the layout settle.
    harness.run_steps(4);
}

fn button(harness: &mut Harness<'_, GrandCoachApp>, label: &str) {
    click(harness, Role::Button, label);
}

/// German card label as shown to assistive technology.
fn card_label(card: Card) -> String {
    let suit = ["Kreuz", "Pik", "Herz", "Karo"][usize::from(card.suit().index())];
    let rank = [
        "Sieben", "Acht", "Neun", "Zehn", "Bube", "Dame", "König", "Ass",
    ][usize::from(card.rank().index())];
    format!("{suit}-{rank}")
}

fn game_label(game: GameType) -> &'static str {
    match game {
        GameType::Farbe(Suit::Kreuz) => "Kreuz",
        GameType::Farbe(Suit::Pik) => "Pik",
        GameType::Farbe(Suit::Herz) => "Herz",
        GameType::Farbe(Suit::Karo) => "Karo",
        GameType::Grand => "Grand",
        GameType::Null => "Null",
    }
}

/// Lets the opponents move until the human acts or the game is over.
fn settle(harness: &mut Harness<'_, GrandCoachApp>) {
    for _ in 0..500 {
        if !view(harness).opponent_to_move {
            harness.run_steps(3);
            return;
        }
        harness.run_steps(1);
    }
    panic!("opponents did not finish their moves");
}

/// Performs one human decision by clicking, chosen with `rng` among the
/// legal actions. Returns the action, or `None` for "next game".
fn human_turn(harness: &mut Harness<'_, GrandCoachApp>, rng: &mut ChaCha8Rng) -> Option<Action> {
    let view = view(harness);
    if view.acting.is_none() {
        button(harness, "Nächstes Spiel");
        return None;
    }
    let legal = &view.legal_actions;
    let pick =
        |rng: &mut ChaCha8Rng, options: &[Action]| options[rng.next_u32() as usize % options.len()];
    let action = match view.phase {
        Phase::Reizen => {
            let mut options: Vec<Action> = legal
                .iter()
                .copied()
                .filter(|a| matches!(a, Action::Hold | Action::Pass))
                .collect();
            if let Some(&bid) = legal.iter().find(|a| matches!(a, Action::Bid(_))) {
                options.push(bid);
                options.push(bid);
            }
            pick(rng, &options)
        }
        Phase::Druecken => {
            let a = view.hand[rng.next_u32() as usize % 12];
            let b = view.hand[(view.hand.iter().position(|&c| c == a).unwrap() + 1) % 12];
            Action::Discard(skat_app::CardSet::single(a) | skat_app::CardSet::single(b))
        }
        _ => pick(rng, legal),
    };
    match action {
        Action::Bid(value) => button(harness, &format!("{value} sagen")),
        Action::Hold => button(harness, &format!("Ja, {} halten", view.bid)),
        Action::Pass => button(harness, "Passe"),
        Action::Hand => button(harness, "Hand spielen"),
        Action::TakeSkat => button(harness, "Skat aufnehmen"),
        Action::Discard(cards) => {
            for card in cards {
                button(harness, &card_label(card));
            }
            button(harness, "Drücken");
        }
        Action::Announce(a) => announce(harness, a),
        Action::Play(card) => button(harness, &card_label(card)),
        Action::Kontra => button(harness, "Kontra"),
        Action::KeinKontra => button(harness, "Kein Kontra"),
        Action::Re => button(harness, "Re"),
        Action::KeinRe => button(harness, "Kein Re"),
    }
    Some(action)
}

fn announce(harness: &mut Harness<'_, GrandCoachApp>, a: Announcement) {
    click(harness, Role::RadioButton, game_label(a.game));
    if a.ouvert {
        click(harness, Role::CheckBox, "Ouvert");
    } else if a.schwarz {
        click(harness, Role::CheckBox, "Schwarz angesagt");
    } else if a.schneider {
        click(harness, Role::CheckBox, "Schneider angesagt");
    }
    button(harness, "Ansagen");
}

#[test]
fn series_played_entirely_by_clicks() {
    let mut contracts = Vec::new();
    for seed in 0..4 {
        let config = SessionConfig {
            seed,
            series: SeriesLength::Fixed(6),
            kontra_re: true,
        };
        let mut harness = harness(config);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        for _ in 0..2000 {
            settle(&mut harness);
            let before = view(&harness);
            if before.series_finished {
                break;
            }
            let action = human_turn(&mut harness, &mut rng);
            let after = view(&harness);
            assert_ne!(before, after, "click for {action:?} had no effect");
            if let Some(Action::Announce(a)) = action {
                assert_eq!(
                    after
                        .contract
                        .map(|c| (c.game, c.schneider, c.schwarz, c.ouvert)),
                    Some((a.game, a.schneider, a.schwarz, a.ouvert))
                );
            }
        }
        let view = view(&harness);
        assert!(view.series_finished, "series {seed} did not finish");
        assert_eq!(view.sheet.records.len(), 6);
        // The final ranking is on screen.
        harness.get_by_label_contains("Serie beendet nach 6 Spielen");
        contracts.extend(
            view.sheet
                .records
                .iter()
                .filter(|r| r.declarer == Some(PlayerId::HUMAN))
                .filter_map(|r| r.contract),
        );
    }
    assert!(!contracts.is_empty(), "the human never played a game");
}

#[test]
fn free_play_ends_on_request() {
    let config = SessionConfig {
        seed: 3,
        series: SeriesLength::Free,
        kontra_re: false,
    };
    let mut harness = harness(config);
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    while view(&harness).acting.is_some() || view(&harness).opponent_to_move {
        settle(&mut harness);
        if view(&harness).acting == Some(PlayerId::HUMAN) {
            human_turn(&mut harness, &mut rng);
        }
    }
    button(&mut harness, "Serie beenden");
    let view = view(&harness);
    assert!(view.series_finished);
    assert_eq!(view.sheet.records.len(), 1);
}

#[test]
fn language_and_faces_switch_without_touching_the_game() {
    let config = SessionConfig {
        seed: 12,
        series: SeriesLength::Fixed(36),
        kontra_re: true,
    };
    let mut harness = harness(config);
    settle(&mut harness);
    let before = view(&harness);
    assert_eq!(before.acting, Some(PlayerId::HUMAN));
    harness.get_by_role_and_label(Role::Button, "Passe");

    button(&mut harness, "Einstellungen");
    click(&mut harness, Role::RadioButton, "English");
    assert_eq!(view(&harness), before);
    harness.get_by_role_and_label(Role::Button, "Pass");

    button(&mut harness, "Settings");
    click(
        &mut harness,
        Role::RadioButton,
        "German suits (Altenburg pattern)",
    );
    assert_eq!(view(&harness), before);
    for deck in Deck::ALL {
        harness.state_mut().set_deck(deck);
        harness.run_steps(1);
    }
    harness.state_mut().set_language(Language::De);
    harness.run_steps(1);
    assert_eq!(view(&harness), before);
    // The game continues normally afterwards.
    button(&mut harness, "Passe");
    assert_ne!(view(&harness), before);
}

#[test]
fn rejected_commands_show_translated_errors() {
    for seed in 0..100 {
        let config = SessionConfig {
            seed,
            series: SeriesLength::Fixed(1),
            kontra_re: false,
        };
        let mut harness = harness(config);
        // Bid until the human is declarer or out.
        loop {
            settle(&mut harness);
            let v = view(&harness);
            if v.phase != Phase::Reizen || v.acting != Some(PlayerId::HUMAN) {
                break;
            }
            if v.legal_actions.contains(&Action::Hold) {
                button(&mut harness, &format!("Ja, {} halten", v.bid));
            } else {
                button(&mut harness, "18 sagen");
            }
        }
        if view(&harness).phase != Phase::SkatAufnahme {
            continue;
        }
        button(&mut harness, "Skat aufnehmen");
        let before = view(&harness);
        // Discarding without choosing two cards is rejected.
        button(&mut harness, "Drücken");
        assert_eq!(view(&harness), before);
        harness.get_by_label("Drücke genau zwei Karten aus deiner Hand.");
        harness.state_mut().set_language(Language::En);
        harness.run_steps(2);
        harness.get_by_label("Discard exactly two cards from your hand.");
        return;
    }
    panic!("no seed made the human declarer");
}

#[test]
fn every_card_face_renders() {
    let ctx = egui::Context::default();
    egui_extras::install_image_loaders(&ctx);
    let mut faces = Vec::new();
    for deck in Deck::ALL {
        for index in 0..32 {
            let card = Card::from_index(index).unwrap();
            faces.push(gui_egui::cards::face_path(deck, card));
        }
    }
    faces.push(gui_egui::cards::BACK.to_owned());
    let mut seen = std::collections::BTreeSet::new();
    for path in faces {
        let egui::ImageSource::Bytes { uri, bytes } = gui_egui::cards::image(&path) else {
            unreachable!()
        };
        ctx.include_bytes(uri.clone(), bytes);
        let poll = ctx
            .try_load_image(&uri, egui::load::SizeHint::Width(100))
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        let egui::load::ImagePoll::Ready { image } = poll else {
            panic!("{path} not rendered synchronously");
        };
        assert_eq!(image.size, [100, 150], "{path}");
        let inked = image
            .pixels
            .iter()
            .filter(|p| p.a() > 0 && (p.r() < 200 || p.g() < 200 || p.b() < 200))
            .count();
        assert!(inked > 500, "{path} is nearly empty");
        // The top-left index corner carries ink (rank and suit).
        let corner = (0..40)
            .flat_map(|y| (0..25).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let p = image.pixels[y * 100 + x];
                p.a() > 0 && (p.r() < 200 || p.g() < 200 || p.b() < 200)
            })
            .count();
        assert!(corner > 20, "{path} has no visible index");
        // Within one deck no two faces look alike.
        let deck = path.split('/').next().unwrap().to_owned();
        let pixels: Vec<[u8; 4]> = image.pixels.iter().map(|p| p.to_array()).collect();
        assert!(
            seen.insert((deck, pixels)),
            "{path} looks like another card"
        );
    }
}
