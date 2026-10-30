// SPDX-License-Identifier: EUPL-1.2

//! Drives the real egui frontend headlessly with egui_kittest: every human
//! decision is a click on a widget found by its accessible label.

use eframe::egui::{self, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use gui_egui::{Deck, GrandCoachApp, Language};
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};
use skat_app::{
    Action, Announcement, Card, Command, GameType, Level, Opponents, Phase, PlayedCard, PlayerId,
    SeriesLength, Session, SessionConfig, Suit, TableView,
};

const TABLE_GREEN: egui::Color32 = egui::Color32::from_rgb(34, 102, 58);
const TABLE_RED: egui::Color32 = egui::Color32::from_rgb(110, 22, 26);

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
/// Opponents think on a background thread, so frames wait a little.
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
    perform(harness, &view, action);
    Some(action)
}

/// Clicks the widgets for one human action.
fn perform(harness: &mut Harness<'_, GrandCoachApp>, view: &TableView, action: Action) {
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
#[ignore = "slow: CI only (docs/testing.md)"]
fn series_played_entirely_by_clicks() {
    let mut contracts = Vec::new();
    for seed in 0..4 {
        let config = SessionConfig {
            seed,
            series: SeriesLength::Fixed(6),
            kontra_re: true,
            opponents: Opponents::Levels([Level::Easy; 2]),
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
        opponents: Opponents::Levels([Level::Easy; 2]),
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
        opponents: Opponents::Levels([Level::Easy; 2]),
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
            opponents: Opponents::Levels([Level::Easy; 2]),
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
    for mephisto in [false, true] {
        for deck in Deck::ALL {
            for index in 0..32 {
                let card = Card::from_index(index).unwrap();
                faces.push(gui_egui::cards::face_path(deck, card, mephisto));
            }
        }
    }
    for mephisto in [false, true] {
        faces.push(gui_egui::cards::back_path(mephisto).to_owned());
    }
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
        let deck = path.rsplit_once('/').map_or("", |(dir, _)| dir).to_owned();
        let pixels: Vec<[u8; 4]> = image.pixels.iter().map(|p| p.to_array()).collect();
        assert!(
            seen.insert((deck, pixels)),
            "{path} looks like another card"
        );
    }
}

/// Kind of a human decision, for coverage.
fn kind(action: Action) -> String {
    match action {
        Action::Bid(_) => "Bid".to_owned(),
        Action::Discard(_) => "Discard".to_owned(),
        Action::Play(_) => "Play".to_owned(),
        Action::Announce(a) => {
            let family = match a.game {
                GameType::Farbe(_) => "Farbe",
                GameType::Grand => "Grand",
                GameType::Null => "Null",
            };
            format!("{family} s={} z={} o={}", a.schneider, a.schwarz, a.ouvert)
        }
        other => format!("{other:?}"),
    }
}

#[test]
#[ignore = "slow: CI only (docs/testing.md)"]
fn every_decision_and_variant_is_clickable() {
    let mut seen = std::collections::BTreeSet::new();
    let mut hand_variants = std::collections::BTreeMap::new();
    let mut clicks = std::collections::BTreeMap::new();
    for seed in 100..140 {
        let config = SessionConfig {
            seed,
            series: SeriesLength::Fixed(6),
            kontra_re: true,
            opponents: Opponents::Levels([Level::Easy; 2]),
        };
        let mut harness = harness(config);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        while !view(&harness).series_finished {
            settle(&mut harness);
            if view(&harness).series_finished {
                break;
            }
            let v = view(&harness);
            // Announce the variant clicked least so far, to cover all.
            let least_seen = v
                .legal_actions
                .iter()
                .copied()
                .filter(|a| matches!(a, Action::Announce(_)))
                .min_by_key(|&a| {
                    let key = (kind(a), v.hand_game == Some(true));
                    hand_variants.get(&key).copied().unwrap_or(0)
                });
            // Likewise for the other decisions except card choices.
            let simple = v
                .legal_actions
                .iter()
                .copied()
                .filter(|a| {
                    !matches!(
                        a,
                        Action::Play(_) | Action::Discard(_) | Action::Announce(_)
                    )
                })
                .filter(|a| !matches!(a, Action::Bid(_)) || v.legal_actions.first() == Some(a))
                .min_by_key(|&a| clicks.get(&kind(a)).copied().unwrap_or(0));
            let action = match (least_seen, simple) {
                (Some(Action::Announce(a)), _) if v.acting == Some(PlayerId::HUMAN) => {
                    announce(&mut harness, a);
                    Some(Action::Announce(a))
                }
                (_, Some(a)) if v.acting == Some(PlayerId::HUMAN) => {
                    perform(&mut harness, &v, a);
                    Some(a)
                }
                _ => human_turn(&mut harness, &mut rng),
            };
            if let Some(a) = action {
                *clicks.entry(kind(a)).or_insert(0) += 1;
            }
            if let Some(action) = action {
                seen.insert(kind(action));
                if let (Action::Announce(_), Some(c)) = (action, view(&harness).contract) {
                    *hand_variants.entry((kind(action), c.hand)).or_insert(0) += 1;
                }
            }
        }
    }
    let mut wanted: Vec<String> = ["Bid", "Hold", "Pass", "Hand", "TakeSkat", "Discard", "Play"]
        .map(str::to_owned)
        .to_vec();
    // Kontra by the human; Re/KeinRe answer an opponent's Kontra. The
    // human's Ouvert Kontra round is rare and has its own test below.
    wanted.extend(["Kontra", "Re", "KeinRe"].map(str::to_owned));
    for family in ["Farbe", "Grand"] {
        for (s, z, o) in [
            (false, false, false),
            (true, false, false),
            (true, true, false),
            (true, true, true),
        ] {
            wanted.push(format!("{family} s={s} z={z} o={o}"));
        }
    }
    wanted.push("Null s=false z=false o=false".to_owned());
    wanted.push("Null s=false z=false o=true".to_owned());
    let missing: Vec<_> = wanted.iter().filter(|k| !seen.contains(*k)).collect();
    assert!(missing.is_empty(), "never clicked: {missing:?}");
    for hand in [false, true] {
        for ouvert in [false, true] {
            let null = (format!("Null s=false z=false o={ouvert}"), hand);
            assert!(hand_variants.contains_key(&null), "missing {null:?}");
        }
        let plain = ("Grand s=false z=false o=false".to_owned(), hand);
        assert!(hand_variants.contains_key(&plain), "missing {plain:?}");
    }
}

#[test]
fn table_shows_phase_turn_announcements_tricks_and_sheet() {
    let config = SessionConfig {
        seed: 21,
        series: SeriesLength::Fixed(3),
        kontra_re: true,
        opponents: Opponents::Levels([Level::Easy; 2]),
    };
    let mut harness = harness(config);
    settle(&mut harness);
    harness.get_by_label("Reizen");
    harness.get_by_label("Am Zug: Du");
    harness.get_by_label("Spielliste");
    harness.get_by_label("Letzter Stich");
    // The ten hand cards are buttons.
    assert!(harness.get_all_by_role(Role::Button).count() >= 10);
    let mut rng = ChaCha8Rng::seed_from_u64(21);
    let mut saw_current = false;
    let mut saw_announcement = false;
    let mut saw_last_winner = false;
    while view(&harness).acting.is_some() {
        settle(&mut harness);
        let v = view(&harness);
        if let Some(contract) = v.contract {
            let game = game_label(contract.game);
            saw_announcement |= harness.query_all_by_label_contains(game).next().is_some();
        }
        if !v.current_trick.is_empty() {
            saw_current |= harness
                .query_all_by_label("Aktueller Stich")
                .next()
                .is_some();
        }
        saw_last_winner |= harness
            .query_all_by_label_contains(" sticht (")
            .next()
            .is_some();
        if v.acting == Some(PlayerId::HUMAN) {
            human_turn(&mut harness, &mut rng);
        }
    }
    assert!(saw_current && saw_announcement && saw_last_winner);
    // After the game the result and the sheet line are visible.
    harness.get_by_label("Ergebnis");
    harness.get_by_label("Nächstes Spiel");
}

/// Checks that the shown trick lies as a triangle: left opponent's card
/// left, right opponent's card right, the human's card below between them.
fn assert_trick_triangle(harness: &Harness<'_, GrandCoachApp>, trick: &[PlayedCard]) {
    let played = |p: PlayerId| trick.iter().any(|c| c.player == p);
    // Names also head the opponent columns and the score sheet on the
    // right; in the middle of the table they are the innermost ones.
    let sheet = harness.get_by_label("Spielliste").rect().min.x;
    let innermost = |label: &str, rightmost: bool| {
        let rects = harness
            .query_all_by_label(label)
            .map(|n| n.rect())
            .filter(|r| r.max.x < sheet);
        if rightmost {
            rects.max_by(|a, b| a.min.x.total_cmp(&b.min.x))
        } else {
            rects.min_by(|a, b| a.min.x.total_cmp(&b.min.x))
        }
        .unwrap()
    };
    let left = played(PlayerId(1)).then(|| innermost("Links", true));
    let right = played(PlayerId(2)).then(|| innermost("Rechts", false));
    let human = played(PlayerId::HUMAN).then(|| innermost("Du", false));
    if let (Some(l), Some(r)) = (left, right) {
        assert!(l.max.x < r.min.x, "left {l:?}, right {r:?}");
        assert!((l.min.y - r.min.y).abs() < 1.0, "left {l:?}, right {r:?}");
    }
    if let Some(h) = human {
        if let Some(l) = left {
            assert!(
                l.max.x < h.min.x && l.min.y < h.min.y,
                "left {l:?}, human {h:?}"
            );
        }
        if let Some(r) = right {
            assert!(
                h.max.x < r.min.x && r.min.y < h.min.y,
                "human {h:?}, right {r:?}"
            );
        }
    }
}

#[test]
fn trick_cards_lie_on_the_side_of_their_player() {
    let config = SessionConfig {
        seed: 21,
        series: SeriesLength::Fixed(2),
        kontra_re: true,
        opponents: Opponents::Levels([Level::Easy; 2]),
    };
    let mut harness = harness(config);
    // Simulated time (0.25 s per frame): opponents wait 2 s after the human's
    // card, a completed trick stays in the middle for 4 s.
    harness.state_mut().opponent_delay = 2.0;
    let mut rng = ChaCha8Rng::seed_from_u64(21);
    let (mut right_led, mut complete) = (false, false);
    for _ in 0..200 {
        settle(&mut harness);
        let before = view(&harness);
        if before.series_finished {
            break;
        }
        if before.current_trick.len() == 2 {
            assert_trick_triangle(&harness, &before.current_trick);
        }
        let action = human_turn(&mut harness, &mut rng);
        if !matches!(action, Some(Action::Play(_))) {
            continue;
        }
        let after = view(&harness);
        let shown = if after.current_trick.is_empty() {
            // In the middle of the table as well as in the side panel.
            let titles = harness.query_all_by_label_contains(" sticht (").count();
            assert_eq!(titles, 2, "completed trick not shown in the middle");
            complete = true;
            after.last_trick.expect("trick just completed").cards
        } else {
            after.current_trick
        };
        assert_trick_triangle(&harness, &shown);
        // The case of issue #24: the right opponent leads, the human follows.
        right_led |= shown[0].player == PlayerId(2);
    }
    assert!(
        right_led && complete,
        "right_led {right_led}, complete {complete}"
    );
}

/// Finds, through `skat-app` alone, a series in which the human (holding 18
/// in the bidding) faces an opponent's Ouvert game, and returns its config
/// and the human's commands up to that point.
fn ouvert_kontra_round() -> (SessionConfig, Vec<Command>) {
    for seed in 0..400 {
        let config = SessionConfig {
            seed,
            series: SeriesLength::Fixed(36),
            kontra_re: true,
            opponents: Opponents::Levels([Level::Easy; 2]),
        };
        let mut session = Session::new(config);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut commands = Vec::new();
        loop {
            session.run_opponents();
            let v = session.view();
            if v.series_finished {
                break;
            }
            if v.legal_actions.contains(&Action::KeinKontra) {
                return (config, commands);
            }
            let legal = &v.legal_actions;
            let command = match v.phase {
                _ if legal.is_empty() => Command::NextGame,
                Phase::Reizen if legal.contains(&Action::Hold) && v.bid <= 18 => {
                    Command::Act(Action::Hold)
                }
                Phase::Reizen if legal.contains(&Action::Bid(18)) => Command::Act(Action::Bid(18)),
                Phase::Reizen => Command::Act(Action::Pass),
                _ => Command::Act(legal[rng.next_u32() as usize % legal.len()]),
            };
            session.execute(command).unwrap();
            commands.push(command);
        }
    }
    panic!("no Ouvert Kontra round for the human found");
}

#[test]
#[ignore = "slow: CI only (docs/testing.md)"]
fn kein_kontra_before_an_opponents_ouvert_game_by_click() {
    let (config, commands) = ouvert_kontra_round();
    let mut harness = harness(config);
    for command in commands {
        settle(&mut harness);
        let v = view(&harness);
        match command {
            Command::Act(action) => perform(&mut harness, &v, action),
            _ => button(&mut harness, "Nächstes Spiel"),
        }
    }
    settle(&mut harness);
    let v = view(&harness);
    assert!(v.contract.is_some_and(|c| c.ouvert));
    assert!(v.declarer_cards.is_none());
    harness.get_by_role_and_label(Role::Button, "Kontra");
    button(&mut harness, "Kein Kontra");
    settle(&mut harness);
    assert!(
        view(&harness).declarer_cards.is_some(),
        "cards not revealed"
    );
    harness.get_by_label("Offene Karten:");
}

/// Each opponent's level is chosen by clicks in the "new series" dialog
/// for a fixed series and for free play; the window stays responsive while
/// a searching opponent thinks on its background thread.
#[test]
fn opponent_levels_are_chosen_per_opponent() {
    let config = SessionConfig {
        seed: 1,
        series: SeriesLength::Fixed(36),
        kontra_re: false,
        opponents: Opponents::Levels([Level::Easy; 2]),
    };
    let mut harness = harness(config);
    let name = |level: Level| match level {
        Level::Rookie => "Grünschnabel",
        Level::Easy => "Leicht",
        Level::Medium => "Mittel",
        Level::Strong => "Stark",
    };
    for (free, left, right) in [
        (false, Level::Strong, Level::Rookie),
        (true, Level::Easy, Level::Medium),
    ] {
        button(&mut harness, "Spiel");
        button(&mut harness, "Neue Serie …");
        if free {
            click(&mut harness, Role::RadioButton, "Freies Spiel");
        }
        // Two rows of level buttons: the first row is the left opponent.
        for (row, level) in [(0, left), (1, right)] {
            let radios: Vec<_> = harness
                .get_all_by_role_and_label(Role::RadioButton, name(level))
                .collect();
            assert_eq!(radios.len(), 2);
            radios[row].click();
            harness.run_steps(4);
        }
        button(&mut harness, "Starten");
        let view = view(&harness);
        assert_eq!(view.config.opponents, Opponents::Levels([left, right]));
        assert_eq!(view.config.series == SeriesLength::Free, free);
        let left_name = name(left);
        let right_name = name(right);
        harness.get_by_label(&format!(
            "Gegner: {left_name} (links), {right_name} (rechts)"
        ));
        let card = view.hand[0];
        assert!(!harness.state().face(card).starts_with("mephisto/"));
        assert_eq!(harness.state().back(), "back.svg");
        assert_eq!(harness.state().table_color(), TABLE_GREEN);
    }
    // Free play against Medium on the right: frames keep coming while the
    // opponents think, and the game advances.
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    for _ in 0..3 {
        let start = std::time::Instant::now();
        settle(&mut harness);
        assert!(start.elapsed() < std::time::Duration::from_secs(120));
        if view(&harness).acting == Some(PlayerId::HUMAN) {
            human_turn(&mut harness, &mut rng);
        }
    }
    // The table modes: Goldfish, then Mephisto with its own card faces,
    // card back and a red table.
    for (label, opponents) in [
        ("Goldfisch", Opponents::Goldfish),
        ("Mephisto", Opponents::Mephisto),
    ] {
        button(&mut harness, "Spiel");
        button(&mut harness, "Neue Serie …");
        click(&mut harness, Role::RadioButton, label);
        button(&mut harness, "Starten");
        let view = view(&harness);
        assert_eq!(view.config.opponents, opponents);
        harness.get_by_label(&format!("Gegner: {label}"));
        let mephisto = opponents == Opponents::Mephisto;
        for &card in &view.hand {
            let face = harness.state().face(card);
            assert_eq!(face.starts_with("mephisto/"), mephisto, "{face}");
        }
        let (back, table) = if mephisto {
            ("mephisto/back.svg", TABLE_RED)
        } else {
            ("back.svg", TABLE_GREEN)
        };
        assert_eq!(harness.state().back(), back);
        assert_eq!(harness.state().table_color(), table);
    }
    // Back to single levels: green table and the usual back again.
    button(&mut harness, "Spiel");
    button(&mut harness, "Neue Serie …");
    click(&mut harness, Role::RadioButton, "Gegner einzeln wählen");
    button(&mut harness, "Starten");
    assert!(matches!(
        view(&harness).config.opponents,
        Opponents::Levels(_)
    ));
    assert_eq!(harness.state().back(), "back.svg");
    assert_eq!(harness.state().table_color(), TABLE_GREEN);
}

/// The seed is a developer option: hidden by default, shown and settable
/// after "Seed anzeigen", and only whole numbers within `u64` start a series.
#[test]
fn seed_is_shown_and_set_only_with_the_developer_option() {
    let config = SessionConfig {
        seed: 21,
        series: SeriesLength::Fixed(36),
        kontra_re: true,
        opponents: Opponents::Levels([Level::Easy; 2]),
    };
    let mut harness = harness(config);
    let seed_input = "Startwert (Seed):";
    assert!(harness.query_by_label("Seed 21").is_none());
    button(&mut harness, "Spiel");
    button(&mut harness, "Neue Serie …");
    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, seed_input)
            .is_none()
    );
    button(&mut harness, "Starten");
    assert_eq!(view(&harness).config.seed, 22);
    assert!(harness.query_by_label("Seed 22").is_none());

    button(&mut harness, "Einstellungen");
    click(&mut harness, Role::CheckBox, "Seed anzeigen");
    harness.key_press(egui::Key::Escape);
    harness.run_steps(4);
    harness.get_by_label("Seed 22");
    button(&mut harness, "Spiel");
    button(&mut harness, "Neue Serie …");
    harness.get_by_label_contains("ganze Zahlen von 0 bis 18446744073709551615");
    let set_seed = |harness: &mut Harness<'_, GrandCoachApp>, text: &str| {
        let input = harness.get_by_role_and_label(Role::TextInput, seed_input);
        input.focus();
        harness.run_steps(1);
        harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        harness
            .get_by_role_and_label(Role::TextInput, seed_input)
            .type_text(text);
        harness.run_steps(4);
    };
    let start_disabled = |harness: &Harness<'_, GrandCoachApp>| {
        harness
            .get_by_role_and_label(Role::Button, "Starten")
            .accesskit_node()
            .is_disabled()
    };
    assert!(!start_disabled(&harness));
    for invalid in ["abc", "-1", "18446744073709551616"] {
        set_seed(&mut harness, invalid);
        assert!(start_disabled(&harness), "{invalid}");
        harness.get_by_label_contains("Kein gültiger Seed");
    }
    set_seed(&mut harness, "18446744073709551615");
    assert!(!start_disabled(&harness));
    button(&mut harness, "Starten");
    assert_eq!(view(&harness).config.seed, u64::MAX);
    harness.get_by_label("Seed 18446744073709551615");
}

/// At launch the "new series" dialog shows the series settings (issue #25).
/// The opponents wait until it closes; starting it unchanged keeps the
/// series and its seed, cancelling keeps the series too.
#[test]
fn launch_shows_the_new_series_settings() {
    let config = |seed| SessionConfig {
        seed,
        series: SeriesLength::Fixed(12),
        kontra_re: false,
        opponents: Opponents::Levels([Level::Rookie, Level::Easy]),
    };
    // A deal in which an opponent decides first.
    let seed = (0..100)
        .find(|&s| Session::new(config(s)).view().opponent_to_move)
        .expect("an opponent moves first");
    for close in ["Starten", "Abbrechen"] {
        let mut harness = Harness::builder()
            .with_size([1600.0, 1000.0])
            .build_eframe(move |cc| {
                let mut app = GrandCoachApp::new(&cc.egui_ctx, config(seed));
                app.show_new_series();
                app
            });
        harness.state_mut().opponent_delay = 0.0;
        for _ in 0..20 {
            harness.run_steps(1);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(harness.state().new_series_open());
        harness.get_by_label("Neue Serie");
        let checked = |harness: &Harness<'_, GrandCoachApp>, role, label| {
            harness
                .get_by_role_and_label(role, label)
                .accesskit_node()
                .toggled()
                == Some(egui::accesskit::Toggled::True)
        };
        assert!(checked(
            &harness,
            Role::RadioButton,
            "Serie mit fester Spielzahl"
        ));
        assert!(!checked(&harness, Role::CheckBox, "Mit Kontra und Re"));
        assert!(view(&harness).bidding.is_empty(), "opponents waited");
        button(&mut harness, close);
        assert!(!harness.state().new_series_open());
        assert_eq!(view(&harness).config, config(seed));
        settle(&mut harness);
        assert!(!view(&harness).opponent_to_move);
    }
}
