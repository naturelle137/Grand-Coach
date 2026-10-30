// SPDX-License-Identifier: EUPL-1.2

//! The egui application: draws the [`TableView`] of `skat-app` and turns
//! clicks into [`Command`]s. No game logic here.

use eframe::egui::{self, Color32, RichText, Vec2};
use std::sync::mpsc::{Receiver, TryRecvError};

use skat_app::{
    Action, Announcement, AppError, AppEvent, Cancel, Card, CardSet, Command, Event, GameType,
    Level, OpponentMove, Opponents, Phase, PlayedCard, PlayerId, SeriesLength, Session,
    SessionConfig, Suit, TableView, reizwerte,
};

use crate::cards::{self, Deck};
use crate::i18n::{I18n, Language};
use crate::text;

const CARD: Vec2 = Vec2::new(78.0, 117.0);
const SMALL_CARD: Vec2 = Vec2::new(44.0, 66.0);
/// Gap between the cards of the trick triangle.
const TRICK_GAP: f32 = 8.0;
/// Height of the player name below a trick card.
const TRICK_LABEL_HEIGHT: f32 = 20.0;
const TABLE_GREEN: Color32 = Color32::from_rgb(34, 102, 58);
const TABLE_RED: Color32 = Color32::from_rgb(110, 22, 26);

/// Size of the trick triangle: left and right card on top, the human's card
/// below between them.
fn trick_area() -> Vec2 {
    Vec2::new(
        3.0 * CARD.x + 2.0 * TRICK_GAP,
        1.5 * CARD.y + TRICK_LABEL_HEIGHT,
    )
}

/// Offset of a player's card in the trick triangle, towards that player's
/// seat at the table: left opponent left, right opponent right, human below.
fn trick_slot(player: PlayerId) -> Vec2 {
    match player.0 {
        0 => Vec2::new(CARD.x + TRICK_GAP, 0.5 * CARD.y),
        1 => Vec2::ZERO,
        _ => Vec2::new(2.0 * (CARD.x + TRICK_GAP), 0.0),
    }
}

/// Draft of the "new series" dialog.
#[derive(Clone, Debug)]
struct NewSeries {
    free: bool,
    games: u16,
    kontra_re: bool,
    mode: Mode,
    levels: [Level; 2],
    seed: String,
}

/// Opponent choice in the "new series" dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Levels,
    Goldfish,
    Mephisto,
}

/// An opponent decision computed on a background thread. Dropping it
/// cancels the computation (new series).
struct Thinking {
    result: Receiver<OpponentMove>,
    cancel: Cancel,
}

impl Drop for Thinking {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

/// The Grand-Coach window.
pub struct GrandCoachApp {
    session: Session,
    i18n: I18n,
    deck: Deck,
    /// Cards chosen for Drücken.
    selected: CardSet,
    /// Game chosen in the announcement form.
    draft: Announcement,
    /// Value chosen in the jump-bid box.
    jump_bid: Option<u16>,
    error: Option<AppError>,
    /// Seconds between two opponent moves; tests set 0.
    pub opponent_delay: f64,
    next_move_at: f64,
    /// Opponent decision being computed.
    thinking: Option<Thinking>,
    /// Computed opponent move waiting for its turn to be shown.
    ready: Option<OpponentMove>,
    /// Until when the last completed trick stays in the middle.
    show_trick_until: f64,
    new_series: Option<NewSeries>,
    /// Developer option: show the seed in the status line and let the
    /// "new series" dialog set it.
    show_seed: bool,
}

impl GrandCoachApp {
    /// Creates the app and installs the SVG image loader.
    pub fn new(ctx: &egui::Context, config: SessionConfig) -> GrandCoachApp {
        egui_extras::install_image_loaders(ctx);
        GrandCoachApp {
            session: Session::new(config),
            i18n: I18n::new(Language::De),
            deck: Deck::French4,
            selected: CardSet::EMPTY,
            draft: Announcement::plain(GameType::Grand),
            jump_bid: None,
            error: None,
            opponent_delay: 0.8,
            next_move_at: 0.0,
            thinking: None,
            ready: None,
            show_trick_until: 0.0,
            new_series: None,
            show_seed: false,
        }
    }

    /// The running session (read-only, for tests and diagnostics).
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Switches the UI language; the game keeps running.
    pub fn set_language(&mut self, language: Language) {
        self.i18n.language = language;
    }

    /// Switches the card faces; the game keeps running.
    pub fn set_deck(&mut self, deck: Deck) {
        self.deck = deck;
    }

    /// Shows or hides the seed (developer option, off by default).
    pub fn set_show_seed(&mut self, show: bool) {
        self.show_seed = show;
    }

    /// Sends a command to the session and keeps a rejection for display.
    fn send(&mut self, command: Command, now: f64) {
        let result = self.session.execute(command);
        self.handle(result, now);
    }

    fn handle(&mut self, result: Result<Vec<AppEvent>, AppError>, now: f64) {
        match result {
            Ok(events) => {
                self.error = None;
                let trick_done = events.iter().any(|e| {
                    matches!(
                        e,
                        AppEvent::Game {
                            event: Event::TrickWon { .. },
                            ..
                        }
                    )
                });
                let pause = if trick_done {
                    2.0 * self.opponent_delay
                } else {
                    self.opponent_delay
                };
                if trick_done {
                    self.show_trick_until = now + pause;
                }
                self.next_move_at = now + pause;
                if events
                    .iter()
                    .any(|e| matches!(e, AppEvent::GameStarted { .. }))
                {
                    self.selected = CardSet::EMPTY;
                    self.jump_bid = None;
                    self.draft = Announcement::plain(GameType::Grand);
                }
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn act(&mut self, action: Action, now: f64) {
        self.send(Command::Act(action), now);
    }

    /// Lets the opponents move at a readable pace. Each decision is
    /// computed on a background thread, so the window stays responsive
    /// while an opponent thinks; it starts at once and overlaps the pause.
    fn drive_opponents(&mut self, ctx: &egui::Context, now: f64) {
        if !self.session.opponent_to_move() {
            self.thinking = None;
            self.ready = None;
            return;
        }
        if self.thinking.is_none()
            && self.ready.is_none()
            && let Some(job) = self.session.opponent_job()
        {
            let (sender, result) = std::sync::mpsc::channel();
            let cancel = Cancel::new();
            let (stop, repaint) = (cancel.clone(), ctx.clone());
            std::thread::spawn(move || {
                // The receiver is gone if the series was replaced meanwhile.
                let _ = sender.send(job.run(Some(&stop)));
                repaint.request_repaint();
            });
            self.thinking = Some(Thinking { result, cancel });
        }
        if let Some(thinking) = &self.thinking {
            match thinking.result.try_recv() {
                Ok(mv) => {
                    self.ready = Some(mv);
                    self.thinking = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => self.thinking = None,
            }
        }
        if now >= self.next_move_at
            && let Some(mv) = self.ready.take()
        {
            let result = self.session.finish_opponent(mv);
            self.handle(result, now);
        }
        let wait = (self.next_move_at - now).clamp(0.0, 0.05);
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(wait));
    }

    fn t(&self, id: &str) -> String {
        self.i18n.t(id)
    }

    fn player(&self, player: PlayerId) -> String {
        text::player(&self.i18n, player)
    }

    // ----------------------------------------------------------------------
    // Top: menu and status.

    fn menu_bar(&mut self, ui: &mut egui::Ui, view: &TableView, now: f64) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(self.t("menu-game"), |ui| {
                if ui.button(self.t("menu-new-series")).clicked() {
                    self.open_new_series();
                    ui.close();
                }
                if ui
                    .add_enabled(
                        view.can_end_series,
                        egui::Button::new(self.t("menu-end-series")),
                    )
                    .clicked()
                {
                    self.send(Command::EndSeries, now);
                    ui.close();
                }
            });
            ui.menu_button(self.t("menu-settings"), |ui| {
                ui.label(RichText::new(self.t("setting-language")).strong());
                for language in Language::ALL {
                    let id = match language {
                        Language::De => "language-de",
                        Language::En => "language-en",
                    };
                    let label = self.t(id);
                    ui.radio_value(&mut self.i18n.language, language, label);
                }
                ui.separator();
                ui.label(RichText::new(self.t("setting-deck")).strong());
                for deck in Deck::ALL {
                    let label = self.t(deck.message());
                    ui.radio_value(&mut self.deck, deck, label);
                }
                ui.separator();
                ui.label(RichText::new(self.t("setting-developer")).strong());
                let label = self.t("setting-show-seed");
                ui.checkbox(&mut self.show_seed, label)
                    .on_hover_text(self.t("seed-description"));
            });
        });
    }

    fn open_new_series(&mut self) {
        let config = self.session.config();
        let (free, games) = match config.series {
            SeriesLength::Fixed(n) => (false, n),
            SeriesLength::Free => (true, 36),
        };
        self.new_series = Some(NewSeries {
            free,
            games,
            kontra_re: config.kontra_re,
            mode: match config.opponents {
                Opponents::Levels(_) => Mode::Levels,
                Opponents::Goldfish => Mode::Goldfish,
                Opponents::Mephisto => Mode::Mephisto,
            },
            levels: match config.opponents {
                Opponents::Levels(levels) => levels,
                _ => [Level::Medium; 2],
            },
            seed: config.seed.wrapping_add(1).to_string(),
        });
    }

    fn status(&self, ui: &mut egui::Ui, view: &TableView) {
        let i = &self.i18n;
        ui.horizontal_wrapped(|ui| {
            let number = i64::from(view.game_number);
            let game = match view.config.series {
                SeriesLength::Fixed(total) => i.tf(
                    "status-game",
                    &[("number", number.into()), ("total", total.into())],
                ),
                SeriesLength::Free => i.tf("status-game-free", &[("number", number.into())]),
            };
            ui.label(RichText::new(game).strong());
            ui.separator();
            ui.label(text::phase(i, view.phase));
            ui.separator();
            ui.label(i.tf(
                "status-dealer",
                &[("player", self.player(view.dealer).into())],
            ));
            if let Some(acting) = view.acting {
                ui.separator();
                ui.label(
                    RichText::new(i.tf("status-turn", &[("player", self.player(acting).into())]))
                        .strong(),
                );
            }
            if view.bid > 0 {
                ui.separator();
                ui.label(i.tf("status-bid", &[("value", view.bid.into())]));
            }
            if let Some(declarer) = view.declarer {
                ui.separator();
                ui.label(i.tf(
                    "status-declarer",
                    &[("player", self.player(declarer).into())],
                ));
            }
            if let Some(contract) = view.contract {
                ui.separator();
                ui.label(RichText::new(text::contract(i, contract)).strong());
            } else if view.hand_game == Some(true) {
                ui.separator();
                ui.label(i.t("contract-hand"));
            }
            if let Some(kontra) = text::kontra(i, view.kontra) {
                ui.separator();
                ui.label(
                    RichText::new(kontra)
                        .color(Color32::from_rgb(200, 40, 40))
                        .strong(),
                );
            }
            ui.separator();
            ui.label(match view.config.opponents {
                Opponents::Levels([left, right]) => i.tf(
                    "status-levels",
                    &[
                        ("left", i.t(level_message(left)).into()),
                        ("right", i.t(level_message(right)).into()),
                    ],
                ),
                Opponents::Goldfish => {
                    i.tf("status-opponents", &[("mode", i.t("mode-goldfish").into())])
                }
                Opponents::Mephisto => {
                    i.tf("status-opponents", &[("mode", i.t("mode-mephisto").into())])
                }
            });
            if self.show_seed {
                ui.separator();
                ui.label(i.tf(
                    "status-seed",
                    &[("seed", view.config.seed.to_string().into())],
                ))
                .on_hover_text(i.t("seed-description"));
            }
        });
    }

    // ----------------------------------------------------------------------
    // Right: score sheet and last trick.

    fn side_panel(&self, ui: &mut egui::Ui, view: &TableView) {
        let i = &self.i18n;
        ui.heading(self.t("trick-last"));
        match &view.last_trick {
            Some(trick) => {
                ui.horizontal(|ui| {
                    for played in &trick.cards {
                        self.card_image(ui, played.card, SMALL_CARD);
                    }
                });
                ui.label(i.tf(
                    "trick-winner",
                    &[
                        ("player", self.player(trick.winner).into()),
                        ("augen", trick.augen.into()),
                    ],
                ));
            }
            None => {
                ui.label("–");
            }
        }
        ui.separator();
        ui.heading(self.t("sheet-title"));
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                egui::Grid::new("sheet").striped(true).show(ui, |ui| {
                    ui.strong(self.t("sheet-number"));
                    ui.strong(self.t("sheet-game"));
                    for p in PlayerId::ALL {
                        ui.strong(self.player(p));
                    }
                    ui.end_row();
                    for record in &view.sheet.records {
                        ui.label(record.number.to_string());
                        match (record.declarer, record.contract, record.result) {
                            (Some(declarer), Some(contract), Some(result)) => {
                                let mark = if result.won { "+" } else { "−" };
                                ui.label(format!(
                                    "{} {} {mark}{}",
                                    self.player(declarer),
                                    text::contract(i, contract),
                                    result.score.abs()
                                ));
                            }
                            _ => {
                                ui.label(self.t("sheet-eingepasst"));
                            }
                        }
                        for total in record.totals {
                            ui.label(total.to_string());
                        }
                        ui.end_row();
                    }
                    ui.strong(self.t("sheet-total"));
                    ui.label("");
                    for p in PlayerId::ALL {
                        ui.strong(view.players[p.index()].total.to_string());
                    }
                    ui.end_row();
                });
            });
    }

    // ----------------------------------------------------------------------
    // Centre: opponents, Skat and the trick.

    fn table(&mut self, ui: &mut egui::Ui, view: &TableView, now: f64) {
        ui.columns(3, |columns| {
            self.opponent(&mut columns[0], view, PlayerId(1));
            self.middle(&mut columns[1], view, now);
            self.opponent(&mut columns[2], view, PlayerId(2));
        });
    }

    fn opponent(&self, ui: &mut egui::Ui, view: &TableView, player: PlayerId) {
        let i = &self.i18n;
        let info = view.players[player.index()];
        ui.vertical(|ui| {
            let mut title = RichText::new(self.player(player))
                .heading()
                .color(Color32::WHITE);
            if view.acting == Some(player) {
                title = title.underline();
            }
            ui.label(title);
            let mut line = text::seat(i, info.seat);
            if view.declarer == Some(player) {
                line = format!("{line} · {}", self.t("label-declarer"));
            }
            ui.label(RichText::new(line).color(Color32::WHITE));
            if let Some(call) = info.last_call {
                ui.label(RichText::new(text::call(i, call)).color(Color32::LIGHT_YELLOW));
            }
            ui.label(
                RichText::new(i.tf(
                    "label-cards-tricks",
                    &[("cards", info.cards.into()), ("tricks", info.tricks.into())],
                ))
                .color(Color32::WHITE),
            );
            let open = view.declarer == Some(player);
            match (&view.declarer_cards, open) {
                (Some(cards), true) => {
                    ui.label(RichText::new(self.t("label-open-cards")).color(Color32::WHITE));
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        for &card in cards {
                            self.card_image(ui, card, SMALL_CARD);
                        }
                    });
                }
                _ => {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = -SMALL_CARD.x * 0.6;
                        for _ in 0..info.cards {
                            ui.add(
                                egui::Image::new(cards::image(self.back()))
                                    .fit_to_exact_size(SMALL_CARD),
                            );
                        }
                    });
                }
            }
        });
    }

    fn middle(&self, ui: &mut egui::Ui, view: &TableView, now: f64) {
        let i = &self.i18n;
        ui.vertical_centered(|ui| {
            if matches!(view.phase, Phase::Reizen | Phase::SkatAufnahme) {
                ui.label(RichText::new(self.t("label-skat")).color(Color32::WHITE));
                ui.horizontal(|ui| {
                    for _ in 0..2 {
                        ui.add(
                            egui::Image::new(cards::image(self.back()))
                                .fit_to_exact_size(SMALL_CARD),
                        );
                    }
                });
            }
            let (title, cards): (String, Vec<PlayedCard>) = if !view.current_trick.is_empty() {
                (self.t("trick-current"), view.current_trick.clone())
            } else if let Some(last) = view
                .last_trick
                .as_ref()
                .filter(|_| now < self.show_trick_until)
            {
                let winner = self.player(last.winner);
                (
                    i.tf(
                        "trick-winner",
                        &[("player", winner.into()), ("augen", last.augen.into())],
                    ),
                    last.cards.clone(),
                )
            } else {
                (String::new(), Vec::new())
            };
            if !cards.is_empty() {
                ui.label(RichText::new(title).color(Color32::WHITE));
                self.trick_triangle(ui, &cards);
            }
            if let Some(result) = &view.result {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.heading(self.t("result-title"));
                    for line in text::result_lines(i, result.declarer, &result.result) {
                        ui.label(line);
                    }
                    for p in PlayerId::ALL {
                        let points = result.points[p.index()];
                        if points != 0 {
                            ui.label(i.tf(
                                "result-points",
                                &[("player", self.player(p).into()), ("points", points.into())],
                            ));
                        }
                    }
                });
            }
            if view.phase == Phase::Eingepasst {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label(self.t("result-eingepasst"));
                });
            }
            if let Some(series) = &view.series_result {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.heading(i.tf("series-finished", &[("games", series.games.into())]));
                    for (rank, p) in series.ranking.iter().enumerate() {
                        ui.label(i.tf(
                            "series-rank",
                            &[
                                ("rank", (rank + 1).into()),
                                ("player", self.player(*p).into()),
                                ("total", series.totals[p.index()].into()),
                                ("won", series.won[p.index()].into()),
                                ("lost", series.lost[p.index()].into()),
                            ],
                        ));
                    }
                });
            }
        });
    }

    // ----------------------------------------------------------------------
    // Bottom: actions and the human's hand.

    fn actions(&mut self, ui: &mut egui::Ui, view: &TableView, now: f64) {
        let legal = view.legal_actions.clone();
        ui.horizontal_wrapped(|ui| {
            if let Some(error) = self.error {
                ui.label(
                    RichText::new(text::error(&self.i18n, error))
                        .color(Color32::from_rgb(220, 50, 50))
                        .strong(),
                );
                ui.separator();
            }
            if view.series_finished {
                if ui.button(self.t("menu-new-series")).clicked() {
                    self.open_new_series();
                }
                return;
            }
            if view.acting.is_none() {
                if view.can_next_game && ui.button(self.t("button-next-game")).clicked() {
                    self.send(Command::NextGame, now);
                }
                if view.can_end_series && ui.button(self.t("menu-end-series")).clicked() {
                    self.send(Command::EndSeries, now);
                }
                return;
            }
            if let Some(acting) = view.acting.filter(|p| !p.is_human()) {
                ui.label(
                    self.i18n
                        .tf("status-waiting", &[("player", self.player(acting).into())]),
                );
                return;
            }
            match view.phase {
                Phase::Reizen => self.bidding(ui, view, &legal, now),
                Phase::SkatAufnahme => {
                    if ui.button(self.t("button-hand")).clicked() {
                        self.act(Action::Hand, now);
                    }
                    if ui.button(self.t("button-take-skat")).clicked() {
                        self.act(Action::TakeSkat, now);
                    }
                }
                Phase::Druecken => {
                    ui.label(self.t("hint-discard"));
                    if ui.button(self.t("button-discard")).clicked() {
                        let cards = self.selected;
                        self.act(Action::Discard(cards), now);
                        if self.error.is_none() {
                            self.selected = CardSet::EMPTY;
                        }
                    }
                }
                Phase::Ansage => self.announcement(ui, view, now),
                Phase::Spiel => {
                    for (action, id) in [
                        (Action::Kontra, "button-kontra"),
                        (Action::KeinKontra, "button-kein-kontra"),
                        (Action::Re, "button-re"),
                        (Action::KeinRe, "button-kein-re"),
                    ] {
                        if legal.contains(&action) && ui.button(self.t(id)).clicked() {
                            self.act(action, now);
                        }
                    }
                    if legal.iter().any(|a| matches!(a, Action::Play(_))) {
                        ui.label(self.t("hint-play"));
                    }
                }
                Phase::Beendet | Phase::Eingepasst => {}
            }
        });
    }

    fn bidding(&mut self, ui: &mut egui::Ui, view: &TableView, legal: &[Action], now: f64) {
        if legal.contains(&Action::Hold) {
            if ui
                .button(self.i18n.tf("button-hold", &[("value", view.bid.into())]))
                .clicked()
            {
                self.act(Action::Hold, now);
            }
        } else {
            let values: Vec<u16> = legal
                .iter()
                .filter_map(|a| match a {
                    Action::Bid(v) => Some(*v),
                    _ => None,
                })
                .collect();
            if let Some(&next) = values.first()
                && ui
                    .button(self.i18n.tf("button-bid", &[("value", next.into())]))
                    .clicked()
            {
                self.act(Action::Bid(next), now);
            }
            if values.len() > 1 {
                ui.label(self.t("label-jump-bid"));
                let jump = self
                    .jump_bid
                    .filter(|v| values.contains(v))
                    .unwrap_or(values[0]);
                egui::ComboBox::from_id_salt("jump-bid")
                    .selected_text(jump.to_string())
                    .show_ui(ui, |ui| {
                        for &value in reizwerte().iter().filter(|v| values.contains(v)) {
                            ui.selectable_value(&mut self.jump_bid, Some(value), value.to_string());
                        }
                    });
                if ui.button(self.t("button-jump")).clicked() {
                    self.act(Action::Bid(jump), now);
                }
            }
        }
        if ui.button(self.t("button-pass")).clicked() {
            self.act(Action::Pass, now);
        }
    }

    fn announcement(&mut self, ui: &mut egui::Ui, view: &TableView, now: f64) {
        let i = &self.i18n;
        let hand = view.hand_game == Some(true);
        let mut draft = self.draft;
        ui.label(RichText::new(i.t("label-announce")).strong());
        let games = [
            GameType::Farbe(Suit::Kreuz),
            GameType::Farbe(Suit::Pik),
            GameType::Farbe(Suit::Herz),
            GameType::Farbe(Suit::Karo),
            GameType::Grand,
            GameType::Null,
        ];
        for game in games {
            ui.radio_value(&mut draft.game, game, text::game_type(i, game));
        }
        ui.separator();
        let trump_game = draft.game != GameType::Null;
        ui.add_enabled(
            hand && trump_game,
            egui::Checkbox::new(&mut draft.schneider, i.t("contract-schneider")),
        );
        ui.add_enabled(
            hand && trump_game,
            egui::Checkbox::new(&mut draft.schwarz, i.t("contract-schwarz")),
        );
        ui.add_enabled(
            hand || !trump_game,
            egui::Checkbox::new(&mut draft.ouvert, i.t("contract-ouvert")),
        );
        // Keep the implied levels consistent (Ouvert implies Schwarz implies
        // Schneider in suit and Grand games); the engine checks the rest.
        if trump_game {
            if !hand {
                draft.schneider = false;
                draft.schwarz = false;
                draft.ouvert = false;
            }
            if draft.ouvert {
                draft.schwarz = true;
            }
            if draft.schwarz {
                draft.schneider = true;
            }
        } else {
            draft.schneider = false;
            draft.schwarz = false;
        }
        self.draft = draft;
        if ui.button(self.t("button-announce")).clicked() {
            self.act(Action::Announce(draft), now);
        }
    }

    fn hand(&mut self, ui: &mut egui::Ui, view: &TableView, now: f64) {
        let playable: Vec<Card> = view
            .legal_actions
            .iter()
            .filter_map(|a| match a {
                Action::Play(c) => Some(*c),
                _ => None,
            })
            .collect();
        let discarding = view.phase == Phase::Druecken && view.acting == Some(PlayerId::HUMAN);
        let playing = view.phase == Phase::Spiel && view.acting == Some(PlayerId::HUMAN);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for &card in &view.hand {
                let selected = self.selected.contains(card);
                let dim = playing && !playable.contains(&card);
                let mut image =
                    egui::Image::new(cards::image(&self.face(card))).fit_to_exact_size(CARD);
                if dim {
                    image = image.tint(Color32::from_gray(150));
                }
                let response = ui.add(
                    egui::Button::image(image)
                        .selected(selected)
                        .frame(selected),
                );
                let label = text::card(&self.i18n, card);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, true, label.clone())
                });
                let response = response.on_hover_text(text::card(&self.i18n, card));
                if response.clicked() {
                    if discarding {
                        if !self.selected.remove(card) {
                            self.selected.insert(card);
                        }
                    } else if playing {
                        self.act(Action::Play(card), now);
                    }
                }
            }
        });
        if let Some(skat) = &view.own_skat {
            ui.horizontal(|ui| {
                ui.label(self.t("label-own-skat"));
                for &card in skat {
                    self.card_image(ui, card, SMALL_CARD);
                }
            });
        }
    }

    fn mephisto(&self) -> bool {
        self.session.config().opponents == Opponents::Mephisto
    }

    /// Face of `card` in the chosen deck (path below `assets/cards/`); the
    /// Mephisto variant while the session plays against Mephisto.
    pub fn face(&self, card: Card) -> String {
        cards::face_path(self.deck, card, self.mephisto())
    }

    /// Card back (path below `assets/cards/`); the Mephisto back while the
    /// session plays against Mephisto.
    pub fn back(&self) -> &'static str {
        cards::back_path(self.mephisto())
    }

    /// Table colour: red against Mephisto, green otherwise.
    pub fn table_color(&self) -> Color32 {
        if self.mephisto() {
            TABLE_RED
        } else {
            TABLE_GREEN
        }
    }

    /// Draws a trick as a triangle: every card lies on the side of the
    /// player who played it, in playing order (issue #24).
    fn trick_triangle(&self, ui: &mut egui::Ui, trick: &[PlayedCard]) {
        let (area, _) = ui.allocate_exact_size(trick_area(), egui::Sense::hover());
        for played in trick {
            let card = egui::Rect::from_min_size(area.min + trick_slot(played.player), CARD);
            let label = text::card(&self.i18n, played.card);
            ui.put(
                card,
                egui::Image::new(cards::image(&self.face(played.card))).fit_to_exact_size(CARD),
            )
            .on_hover_text(label);
            let name = egui::Rect::from_min_size(
                card.left_bottom(),
                Vec2::new(CARD.x, TRICK_LABEL_HEIGHT),
            );
            ui.put(
                name,
                egui::Label::new(RichText::new(self.player(played.player)).color(Color32::WHITE)),
            );
        }
    }

    fn card_image(&self, ui: &mut egui::Ui, card: Card, size: Vec2) {
        let label = text::card(&self.i18n, card);
        ui.add(egui::Image::new(cards::image(&self.face(card))).fit_to_exact_size(size))
            .on_hover_text(label);
    }

    // ----------------------------------------------------------------------
    // Dialog for a new series.

    fn new_series_window(&mut self, ctx: &egui::Context, now: f64) {
        let Some(mut draft) = self.new_series.clone() else {
            return;
        };
        let mut open = true;
        let mut start = false;
        let mut cancel = false;
        let seed = parse_seed(&draft.seed);
        egui::Window::new(self.t("new-series-title"))
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.radio_value(&mut draft.free, false, self.t("series-fixed"));
                ui.add_enabled(
                    !draft.free,
                    egui::Slider::new(&mut draft.games, 3..=72).text(self.t("series-games")),
                );
                ui.radio_value(&mut draft.free, true, self.t("series-free"));
                ui.checkbox(&mut draft.kontra_re, self.t("option-kontra-re"));
                ui.separator();
                ui.radio_value(&mut draft.mode, Mode::Levels, self.t("mode-levels"));
                for (k, id) in ["opponent-left", "opponent-right"].into_iter().enumerate() {
                    ui.add_enabled_ui(draft.mode == Mode::Levels, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(self.t(id));
                            for level in Level::ALL {
                                let label = self.t(level_message(level));
                                ui.radio_value(&mut draft.levels[k], level, label)
                                    .on_hover_text(self.t(level_description(level)));
                            }
                        });
                    });
                }
                ui.radio_value(&mut draft.mode, Mode::Goldfish, self.t("mode-goldfish"))
                    .on_hover_text(self.t("mode-goldfish-description"));
                ui.radio_value(&mut draft.mode, Mode::Mephisto, self.t("mode-mephisto"));
                let mut notes = Vec::new();
                match draft.mode {
                    Mode::Levels => {
                        notes.push(level_description(draft.levels[0]));
                        if draft.levels[1] != draft.levels[0] {
                            notes.push(level_description(draft.levels[1]));
                        }
                    }
                    Mode::Goldfish => notes.push("mode-goldfish-description"),
                    Mode::Mephisto => {}
                }
                for note in notes {
                    ui.label(RichText::new(self.t(note)).small().weak());
                }
                if self.show_seed {
                    ui.separator();
                    ui.horizontal(|ui| {
                        let label = ui
                            .label(self.t("series-seed"))
                            .on_hover_text(self.t("seed-description"));
                        ui.text_edit_singleline(&mut draft.seed)
                            .labelled_by(label.id)
                            .on_hover_text(self.t("seed-description"));
                    });
                    let note = if seed.is_some() {
                        RichText::new(self.t("seed-description")).weak()
                    } else {
                        RichText::new(self.t("seed-invalid")).color(Color32::from_rgb(200, 40, 40))
                    };
                    ui.label(note.small());
                }
                ui.horizontal(|ui| {
                    start = ui
                        .add_enabled(seed.is_some(), egui::Button::new(self.t("button-start")))
                        .clicked();
                    cancel = ui.button(self.t("button-cancel")).clicked();
                });
            });
        if start && let Some(seed) = seed {
            self.session = Session::new(SessionConfig {
                seed,
                series: if draft.free {
                    SeriesLength::Free
                } else {
                    SeriesLength::Fixed(draft.games)
                },
                kontra_re: draft.kontra_re,
                opponents: match draft.mode {
                    Mode::Levels => Opponents::Levels(draft.levels),
                    Mode::Goldfish => Opponents::Goldfish,
                    Mode::Mephisto => Opponents::Mephisto,
                },
            });
            self.thinking = None;
            self.ready = None;
            self.error = None;
            self.selected = CardSet::EMPTY;
            self.next_move_at = now + self.opponent_delay;
            self.new_series = None;
        } else if cancel || !open {
            self.new_series = None;
        } else {
            self.new_series = Some(draft);
        }
    }
}

/// The seed typed in the "new series" dialog: a whole number from 0 to
/// `u64::MAX`, surrounding spaces ignored.
fn parse_seed(text: &str) -> Option<u64> {
    text.trim().parse().ok()
}

/// Message id of a strength level's name.
fn level_message(level: Level) -> &'static str {
    match level {
        Level::Rookie => "level-rookie",
        Level::Easy => "level-easy",
        Level::Medium => "level-medium",
        Level::Strong => "level-strong",
    }
}

/// Message id of a strength level's description.
fn level_description(level: Level) -> &'static str {
    match level {
        Level::Rookie => "level-rookie-description",
        Level::Easy => "level-easy-description",
        Level::Medium => "level-medium-description",
        Level::Strong => "level-strong-description",
    }
}

impl eframe::App for GrandCoachApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let now = ui.input(|i| i.time);
        self.drive_opponents(ui.ctx(), now);
        let view = self.session.view();
        egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui, &view, now));
        egui::Panel::top("status").show(ui, |ui| self.status(ui, &view));
        egui::Panel::right("side")
            .default_size(320.0)
            .show(ui, |ui| self.side_panel(ui, &view));
        egui::Panel::bottom("hand").show(ui, |ui| {
            ui.add_space(4.0);
            self.actions(ui, &view, now);
            ui.add_space(4.0);
            self.hand(ui, &view, now);
            ui.add_space(4.0);
        });
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).fill(self.table_color()))
            .show(ui, |ui| self.table(ui, &view, now));
        let ctx = ui.ctx().clone();
        self.new_series_window(&ctx, now);
        if now < self.show_trick_until {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(
                self.show_trick_until - now,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(player: PlayerId) -> egui::Rect {
        egui::Rect::from_min_size(egui::Pos2::ZERO + trick_slot(player), CARD)
    }

    #[test]
    fn seeds_are_whole_numbers_within_u64() {
        assert_eq!(parse_seed(" 0 "), Some(0));
        assert_eq!(parse_seed("18446744073709551615"), Some(u64::MAX));
        for invalid in ["", " -1", "18446744073709551616", "1.5", "abc", "1 2"] {
            assert_eq!(parse_seed(invalid), None, "{invalid:?}");
        }
    }

    #[test]
    fn trick_cards_lie_towards_their_players() {
        let (human, left, right) = (rect(PlayerId::HUMAN), rect(PlayerId(1)), rect(PlayerId(2)));
        assert!(left.max.x < human.min.x && human.max.x < right.min.x);
        assert!(human.min.y > left.min.y && human.min.y > right.min.y);
        assert_eq!(left.min.y, right.min.y);
        assert_eq!(human.center().x, trick_area().x / 2.0);
    }

    #[test]
    fn trick_cards_and_names_fit_without_overlap() {
        let area = egui::Rect::from_min_size(egui::Pos2::ZERO, trick_area());
        let with_name = |p| {
            let r = rect(p);
            egui::Rect::from_min_max(r.min, r.max + Vec2::new(0.0, TRICK_LABEL_HEIGHT))
        };
        let all = PlayerId::ALL.map(with_name);
        for (i, a) in all.iter().enumerate() {
            assert!(area.contains_rect(*a), "{i} leaves the area");
            for b in &all[i + 1..] {
                assert!(!a.intersects(*b), "{a:?} overlaps {b:?}");
            }
        }
    }
}
