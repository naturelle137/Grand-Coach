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

use eframe::egui::{Key, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use gui_egui::GrandCoachApp;
use skat_app::coach_model::Judgement;
use skat_app::{
    Action, Advice, Budget, Card, CoachLevel, CoachSettings, Level, Opponents, Pending,
    SeriesLength, SessionConfig, TableView,
};

const SIZE: [f32; 2] = [1280.0, 820.0];

type Scenario = fn(&mut Harness<'_, GrandCoachApp>);

const SCENARIOS: &[(&str, Scenario)] = &[
    ("launch", launch),
    ("bidding", bidding),
    ("trick", trick),
    ("trick-complete", trick_complete),
    ("result", result),
    ("settings", settings),
    ("new-series", new_series),
    ("new-series-seed", new_series_seed),
    ("coach", coach),
    ("coach-no-figures", coach_no_figures),
    ("coach-warning", coach_warning),
];

/// The window at launch: the "new series" dialog shows the series settings
/// and the opponents wait (`harness` opens it like `main`).
fn launch(harness: &mut Harness<'_, GrandCoachApp>) {
    harness.run_steps(4);
}

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

/// The settings menu with the pointer on the developer option "Seed
/// anzeigen", whose tooltip explains the seed.
fn settings(harness: &mut Harness<'_, GrandCoachApp>) {
    settle(harness);
    click(harness, Role::Button, "Einstellungen");
    harness
        .get_by_role_and_label(Role::CheckBox, "Seed anzeigen")
        .hover();
    // Tooltips appear after a short delay.
    harness.run_steps(8);
}

/// The "new series" dialog by default: no seed.
fn new_series(harness: &mut Harness<'_, GrandCoachApp>) {
    settle(harness);
    click(harness, Role::Button, "Spiel");
    click(harness, Role::Button, "Neue Serie …");
}

/// The "new series" dialog with "Seed anzeigen" on and an invalid seed.
fn new_series_seed(harness: &mut Harness<'_, GrandCoachApp>) {
    settle(harness);
    click(harness, Role::Button, "Einstellungen");
    click(harness, Role::CheckBox, "Seed anzeigen");
    harness.key_press(Key::Escape);
    harness.run_steps(4);
    click(harness, Role::Button, "Spiel");
    click(harness, Role::Button, "Neue Serie …");
    let input = "Startwert (Seed):";
    harness
        .get_by_role_and_label(Role::TextInput, input)
        .focus();
    harness.run_steps(1);
    harness
        .get_by_role_and_label(Role::TextInput, input)
        .type_text("x");
    harness.run_steps(4);
}

/// The coach panel after "Was soll ich bedenken?" in the second trick:
/// facts, estimates, compared options with figures.
fn coach(harness: &mut Harness<'_, GrandCoachApp>) {
    play_until(harness, |view, _| {
        view.tricks_played >= 1 && view.legal_actions.len() >= 2
    });
    click(harness, Role::Button, "Was soll ich bedenken?");
    await_coach(harness, |view| {
        matches!(view.coach.advice, Advice::Ready(_) | Advice::Provisional(_))
    });
}

/// The same explanation with the figures hidden.
fn coach_no_figures(harness: &mut Harness<'_, GrandCoachApp>) {
    coach(harness);
    let settings = view(harness).coach.settings;
    harness.state_mut().set_coach(CoachSettings {
        show_figures: false,
        ..settings
    });
    harness.run_steps(4);
}

/// The warning before a clearly weaker card (coach at every move).
fn coach_warning(harness: &mut Harness<'_, GrandCoachApp>) {
    harness.state_mut().set_coach(CoachSettings {
        level: CoachLevel::EveryMove,
        warnings: true,
        show_figures: true,
    });
    for _ in 0..400 {
        settle(harness);
        await_coach(harness, |view| {
            !view.coach.analysing && !matches!(view.coach.advice, Advice::Computing)
        });
        let v = view(harness);
        if v.series_finished {
            break;
        }
        let weak = match &v.coach.advice {
            Advice::Ready(e) => e
                .options
                .iter()
                .find(|o| o.judgement == Judgement::Weak)
                .map(|o| o.action),
            _ => None,
        };
        if let Some(Action::Play(card)) = weak {
            click(harness, Role::Button, &card_label(card));
            await_coach(harness, |view| {
                matches!(view.coach.pending, Some(Pending::Warning { .. }))
            });
            return;
        }
        let label = if v.acting.is_none() {
            "Nächstes Spiel".to_owned()
        } else if v.legal_actions.contains(&Action::Pass) {
            "Passe".to_owned()
        } else if v.legal_actions.contains(&Action::KeinKontra) {
            "Kein Kontra".to_owned()
        } else if let Some(&Action::Play(card)) = v.legal_actions.first() {
            card_label(card)
        } else {
            panic!("no scripted click for {:?}", v.legal_actions);
        };
        click(harness, Role::Button, &label);
        if view(harness).coach.pending.is_some() {
            click(harness, Role::Button, "Trotzdem spielen");
        }
    }
    panic!("no weak card found");
}

/// Runs frames until the coach state satisfies `done`.
fn await_coach(harness: &mut Harness<'_, GrandCoachApp>, done: impl Fn(&TableView) -> bool) {
    for _ in 0..20_000 {
        harness.run_steps(1);
        if done(&view(harness)) {
            harness.run_steps(4);
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    panic!("the coach did not finish");
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
        let mut harness = harness(*name == "launch");
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

/// The app with a small series; `launch` opens the "new series" dialog
/// before the first frame, as `main` does.
fn harness(launch: bool) -> Harness<'static, GrandCoachApp> {
    let config = SessionConfig {
        seed: 21,
        series: SeriesLength::Fixed(2),
        kontra_re: true,
        opponents: Opponents::Levels([Level::Easy; 2]),
    };
    let mut harness = Harness::builder()
        .with_size(SIZE)
        .wgpu()
        .build_eframe(move |cc| {
            let mut app = GrandCoachApp::new(&cc.egui_ctx, config);
            if launch {
                app.show_new_series();
            }
            app
        });
    // Simulated time (0.25 s per frame): opponents wait 2 s after the human's
    // card, so a completed trick stays visible in the middle.
    harness.state_mut().opponent_delay = 2.0;
    // A smaller coach budget than interactive keeps the debug build quick.
    harness.state_mut().set_coach_budget(Budget {
        samples: 8,
        nodes_per_task: 20_000_000,
    });
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

fn click(harness: &mut Harness<'_, GrandCoachApp>, role: Role, label: &str) {
    harness.get_by_role_and_label(role, label).click();
    harness.run_steps(4);
}

/// German card label as shown to assistive technology.
fn card_label(card: Card) -> String {
    let suit = ["Kreuz", "Pik", "Herz", "Karo"][usize::from(card.suit().index())];
    let rank = [
        "Sieben", "Acht", "Neun", "Zehn", "Bube", "Dame", "König", "Ass",
    ][usize::from(card.rank().index())];
    format!("{suit}-{rank}")
}
