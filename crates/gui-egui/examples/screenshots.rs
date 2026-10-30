// SPDX-License-Identifier: EUPL-1.2

//! Renders screenshots of the real egui frontend headlessly (D-45).
//!
//! ```sh
//! cargo run --locked -p gui-egui --example screenshots -- OUT_DIR [SCENARIO...]
//! ```
//!
//! Without scenario names, all scenarios are rendered. Each one writes
//! `OUT_DIR/<name>.png`. egui_kittest renders with wgpu and prefers software
//! rasterisers (lavapipe on Linux, WARP on Windows), so no display is needed.
//! The human's decisions are clicks on widgets found by their accessible
//! labels, as in `tests/gui.rs`. Add a scenario when a frontend change needs
//! its own picture.

use eframe::egui::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use gui_egui::GrandCoachApp;
use skat_app::{Action, Card, Level, Opponents, SeriesLength, SessionConfig, TableView};

const SIZE: [f32; 2] = [1280.0, 820.0];

type Scenario = fn(&mut Harness<'_, GrandCoachApp>);

const SCENARIOS: &[(&str, Scenario)] = &[
    ("bidding", bidding),
    ("trick", trick),
    ("trick-complete", trick_complete),
    ("result", result),
];

/// The first decision of the series: the human bids.
fn bidding(harness: &mut Harness<'_, GrandCoachApp>) {
    settle(harness);
}

/// The human has added a card to a trick that is not complete yet.
fn trick(harness: &mut Harness<'_, GrandCoachApp>) {
    play_until(harness, |view, played| {
        played && !view.current_trick.is_empty()
    });
}

/// The human's card completed a trick, which stays in the middle.
fn trick_complete(harness: &mut Harness<'_, GrandCoachApp>) {
    play_until(harness, |view, played| {
        played && view.current_trick.is_empty() && view.last_trick.is_some()
    });
}

/// The first game is over and its result is shown.
fn result(harness: &mut Harness<'_, GrandCoachApp>) {
    play_until(harness, |view, _| view.acting.is_none());
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(out) = args.next() else {
        let names: Vec<_> = SCENARIOS.iter().map(|(name, _)| *name).collect();
        eprintln!("usage: screenshots OUT_DIR [SCENARIO...]; scenarios: {names:?}");
        std::process::exit(2);
    };
    let wanted: Vec<String> = args.collect();
    if let Some(unknown) = wanted
        .iter()
        .find(|w| !SCENARIOS.iter().any(|(name, _)| name == w))
    {
        eprintln!("unknown scenario {unknown:?}");
        std::process::exit(2);
    }
    std::fs::create_dir_all(&out).expect("create output directory");
    for (name, scenario) in SCENARIOS {
        if !wanted.is_empty() && !wanted.iter().any(|w| w == name) {
            continue;
        }
        let mut harness = harness();
        scenario(&mut harness);
        harness.run_steps(4);
        let path = format!("{out}/{name}.png");
        harness
            .render()
            .expect("render with wgpu")
            .save(&path)
            .expect("write PNG");
        println!("{path}");
    }
}

fn harness() -> Harness<'static, GrandCoachApp> {
    let config = SessionConfig {
        seed: 21,
        series: SeriesLength::Fixed(2),
        kontra_re: true,
        opponents: Opponents::Levels([Level::Easy; 2]),
    };
    let mut harness = Harness::builder()
        .with_size(SIZE)
        .wgpu()
        .build_eframe(move |cc| GrandCoachApp::new(&cc.egui_ctx, config));
    // Simulated time (0.25 s per frame): opponents wait 2 s after the human's
    // card, so a completed trick stays visible in the middle.
    harness.state_mut().opponent_delay = 2.0;
    harness.run_steps(2);
    harness
}

fn view(harness: &Harness<'_, GrandCoachApp>) -> TableView {
    harness.state().session().view()
}

/// Lets the opponents move until the human acts or the game is over.
fn settle(harness: &mut Harness<'_, GrandCoachApp>) {
    for _ in 0..5000 {
        if !view(harness).opponent_to_move {
            harness.run_steps(3);
            return;
        }
        harness.run_steps(1);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("opponents did not finish their moves");
}

/// Plays the first game as a defender (the human passes, says no Kontra and
/// plays the first legal card) until `stop(view, card_just_played)` holds.
fn play_until(harness: &mut Harness<'_, GrandCoachApp>, stop: impl Fn(&TableView, bool) -> bool) {
    for _ in 0..200 {
        settle(harness);
        let before = view(harness);
        if stop(&before, false) {
            return;
        }
        let legal = &before.legal_actions;
        let (label, played) = if before.acting.is_none() {
            ("Nächstes Spiel".to_owned(), false)
        } else if legal.contains(&Action::Pass) {
            ("Passe".to_owned(), false)
        } else if legal.contains(&Action::KeinKontra) {
            ("Kein Kontra".to_owned(), false)
        } else if let Some(&Action::Play(card)) = legal.first() {
            (card_label(card), true)
        } else {
            panic!("no scripted click for {legal:?}");
        };
        harness.get_by_role_and_label(Role::Button, &label).click();
        harness.run_steps(4);
        if stop(&view(harness), played) {
            return;
        }
    }
    panic!("scenario did not reach its state");
}

/// German card label as shown to assistive technology.
fn card_label(card: Card) -> String {
    let suit = ["Kreuz", "Pik", "Herz", "Karo"][usize::from(card.suit().index())];
    let rank = [
        "Sieben", "Acht", "Neun", "Zehn", "Bube", "Dame", "König", "Ass",
    ][usize::from(card.rank().index())];
    format!("{suit}-{rank}")
}
