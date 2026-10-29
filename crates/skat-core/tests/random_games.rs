// SPDX-License-Identifier: EUPL-1.2

//! Seeded random games through a test-only legal-action driver.
//!
//! Every step is checked against rules re-implemented independently here
//! (Trumpf, Bedienen, Stich winner, turn order, Augen). A failure message
//! names the seed and the action sequence so the game can be replayed.

use std::collections::BTreeMap;

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};
use skat_core::{
    Action, Announcement, Card, CardSet, Event, Game, GameType, KontraLevel, Phase, Rank,
    RuleConfig, Seat, Suit, reizwerte,
};

/// Default number of games; `SKAT_RANDOM_GAMES` overrides it for the scheduled
/// deep run (see `docs/testing.md`).
const GAMES: u64 = 10_000;

fn game_count() -> u64 {
    match std::env::var("SKAT_RANDOM_GAMES") {
        Ok(n) => n.parse().expect("SKAT_RANDOM_GAMES must be a number"),
        Err(_) => GAMES,
    }
}

// ---------------------------------------------------------------------------
// Independent rule model (deliberately not using skat_core's rule functions).

fn notation(card: Card) -> String {
    let suit = match card.suit() {
        Suit::Kreuz => 'C',
        Suit::Pik => 'S',
        Suit::Herz => 'H',
        Suit::Karo => 'D',
    };
    let rank = match card.rank() {
        Rank::Sieben => '7',
        Rank::Acht => '8',
        Rank::Neun => '9',
        Rank::Zehn => 'T',
        Rank::Bube => 'J',
        Rank::Dame => 'Q',
        Rank::Koenig => 'K',
        Rank::Ass => 'A',
    };
    format!("{suit}{rank}")
}

fn suit_letter(game: GameType) -> Option<char> {
    match game {
        GameType::Farbe(Suit::Kreuz) => Some('C'),
        GameType::Farbe(Suit::Pik) => Some('S'),
        GameType::Farbe(Suit::Herz) => Some('H'),
        GameType::Farbe(Suit::Karo) => Some('D'),
        _ => None,
    }
}

/// Trumps from highest to lowest, as notation strings.
fn trump_list(game: GameType) -> Vec<String> {
    if game == GameType::Null {
        return Vec::new();
    }
    let mut list: Vec<String> = ["CJ", "SJ", "HJ", "DJ"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if let Some(s) = suit_letter(game) {
        list.extend("ATKQ987".chars().map(|r| format!("{s}{r}")));
    }
    list
}

/// "Colour" a card counts as: "T" for Trumpf, otherwise its suit letter.
fn colour(game: GameType, card: Card) -> char {
    let n = notation(card);
    if trump_list(game).contains(&n) {
        'T'
    } else {
        n.chars().next().unwrap()
    }
}

/// Higher is stronger within the same colour.
fn strength(game: GameType, card: Card) -> usize {
    let n = notation(card);
    let trumps = trump_list(game);
    if let Some(i) = trumps.iter().position(|t| *t == n) {
        return 100 - i;
    }
    let order = if game == GameType::Null {
        "AKQJT987"
    } else {
        "ATKQ987"
    };
    50 - order.find(n.chars().nth(1).unwrap()).unwrap()
}

fn expected_winner(game: GameType, cards: [Card; 3]) -> usize {
    let led = colour(game, cards[0]);
    let any_trump = cards.iter().any(|&c| colour(game, c) == 'T');
    let target = if any_trump { 'T' } else { led };
    (0..3)
        .filter(|&i| colour(game, cards[i]) == target)
        .max_by_key(|&i| strength(game, cards[i]))
        .unwrap()
}

fn may_play(game: GameType, hand: CardSet, led: Option<Card>, card: Card) -> bool {
    if !hand.contains(card) {
        return false;
    }
    match led {
        None => true,
        Some(led) => {
            let need = colour(game, led);
            colour(game, card) == need || hand.iter().all(|c| colour(game, c) != need)
        }
    }
}

// ---------------------------------------------------------------------------
// Driver.

fn choose(rng: &mut ChaCha8Rng, legal: &[Action]) -> Action {
    let roll = rng.next_u32() % 100;
    let pick = |rng: &mut ChaCha8Rng, n: usize| (rng.next_u32() as usize) % n;
    if legal.contains(&Action::Hold) {
        return if roll < 65 {
            Action::Hold
        } else {
            Action::Pass
        };
    }
    if legal.contains(&Action::Pass) {
        let bids: Vec<Action> = legal
            .iter()
            .copied()
            .filter(|a| matches!(a, Action::Bid(_)))
            .collect();
        return if roll < 40 || bids.is_empty() {
            Action::Pass
        } else if roll < 95 {
            bids[pick(rng, bids.len().min(4))]
        } else {
            bids[pick(rng, bids.len())]
        };
    }
    if legal.contains(&Action::Hand) {
        return if roll < 30 {
            Action::Hand
        } else {
            Action::TakeSkat
        };
    }
    if legal.contains(&Action::Kontra) && roll < 30 {
        return Action::Kontra;
    }
    if legal.contains(&Action::Re) {
        return if roll < 50 {
            Action::Re
        } else {
            Action::KeinRe
        };
    }
    if legal.contains(&Action::KeinKontra) {
        return Action::KeinKontra;
    }
    let options: Vec<Action> = legal
        .iter()
        .copied()
        .filter(|&a| a != Action::Kontra)
        .collect();
    options[pick(rng, options.len())]
}

/// A random action from the whole action space, legal or not.
fn arbitrary(rng: &mut ChaCha8Rng) -> Action {
    let card = |rng: &mut ChaCha8Rng| Card::from_index((rng.next_u32() % 32) as u8).unwrap();
    let values = reizwerte();
    let games = [
        GameType::Farbe(Suit::Kreuz),
        GameType::Farbe(Suit::Karo),
        GameType::Grand,
        GameType::Null,
    ];
    match rng.next_u32() % 12 {
        0 => Action::Bid(values[(rng.next_u32() as usize) % values.len()]),
        1 => Action::Bid((rng.next_u32() % 300) as u16),
        2 => Action::Hold,
        3 => Action::Pass,
        4 => Action::Hand,
        5 => Action::TakeSkat,
        6 => Action::Discard(CardSet::single(card(rng)) | CardSet::single(card(rng))),
        7 => Action::Announce(Announcement {
            game: games[(rng.next_u32() % 4) as usize],
            schneider: rng.next_u32().is_multiple_of(2),
            schwarz: rng.next_u32().is_multiple_of(2),
            ouvert: rng.next_u32().is_multiple_of(2),
        }),
        8 => Action::Kontra,
        9 => Action::KeinKontra,
        10 => [Action::Re, Action::KeinRe][(rng.next_u32() % 2) as usize],
        _ => Action::Play(card(rng)),
    }
}

// ---------------------------------------------------------------------------
// Checks.

struct Ctx<'a> {
    seed: u64,
    config: RuleConfig,
    actions: &'a [Action],
}

impl Ctx<'_> {
    fn fail(&self, what: &str) -> ! {
        panic!(
            "seed {} config {:?}: {what}\nactions ({}): {:?}",
            self.seed,
            self.config,
            self.actions.len(),
            self.actions
        );
    }

    fn check(&self, ok: bool, what: impl FnOnce() -> String) {
        if !ok {
            self.fail(&what());
        }
    }
}

fn check_conservation(ctx: &Ctx, game: &Game) {
    let all = game.omniscient_view();
    let mut parts: Vec<CardSet> = all.hands.to_vec();
    parts.push(all.skat);
    for trick in game.tricks() {
        parts.push(trick.cards.iter().copied().collect());
    }
    parts.push(
        game.current_trick()
            .cards
            .iter()
            .flatten()
            .copied()
            .collect(),
    );
    let total: usize = parts.iter().map(|p| p.len()).sum();
    let union = parts.iter().fold(CardSet::EMPTY, |a, &b| a | b);
    ctx.check(total == 32 && union == CardSet::ALL, || {
        format!("cards not conserved: {parts:?}")
    });

    let sizes = all.hands.map(|h| h.len());
    let played = game.tricks().len() * 3 + game.current_trick().len();
    let declarer = game.declarer().map(Seat::index);
    match game.phase() {
        Phase::Reizen | Phase::SkatAufnahme | Phase::Eingepasst => ctx
            .check(sizes == [10, 10, 10] && all.skat.len() == 2, || {
                format!("sizes {sizes:?}")
            }),
        Phase::Druecken => {
            let d = declarer.unwrap();
            ctx.check(sizes[d] == 12 && all.skat.is_empty(), || {
                format!("Drücken sizes {sizes:?}")
            });
        }
        Phase::Ansage | Phase::Spiel | Phase::Beendet => {
            ctx.check(all.skat.len() == 2, || "Skat size".into());
            ctx.check(sizes.iter().sum::<usize>() + played == 30, || {
                format!("sizes {sizes:?}")
            });
            for seat in Seat::ALL {
                let own = game.tricks().len()
                    + usize::from(game.current_trick().leader.is_some_and(|l| {
                        (seat.index() + 3 - l.index()) % 3 < game.current_trick().len()
                    }));
                ctx.check(sizes[seat.index()] == 10 - own, || {
                    format!("{seat:?} holds {sizes:?}")
                });
            }
        }
    }
}

#[derive(Default)]
struct Stats(BTreeMap<String, u32>);

impl Stats {
    fn add(&mut self, key: impl Into<String>) {
        *self.0.entry(key.into()).or_default() += 1;
    }
}

/// Plays one game; returns its action sequence.
fn play_game(seed: u64, config: RuleConfig, stats: &mut Stats) -> (Vec<Action>, Game) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x5EED_CAFE);
    let mut game = Game::new(seed, config);
    let mut actions: Vec<Action> = Vec::new();
    let mut expected_leader = Seat::Vorhand;
    let mut declarer_augen = 0u32;

    while let Some(seat) = game.current_seat() {
        let legal = game.legal_actions();
        let ctx = Ctx {
            seed,
            config,
            actions: &actions,
        };
        ctx.check(!legal.is_empty(), || "no legal action".into());
        for &a in &legal {
            ctx.check(game.validate(a).is_ok(), || {
                format!("listed {a:?} fails validation")
            });
        }
        for _ in 0..2 {
            let probe = arbitrary(&mut rng);
            if legal.contains(&probe) {
                continue;
            }
            let before = game.clone();
            let result = game.apply(probe);
            ctx.check(result.is_err(), || format!("unlisted {probe:?} accepted"));
            ctx.check(game == before, || {
                format!("rejected {probe:?} changed state")
            });
        }

        let action = choose(&mut rng, &legal);
        let before = game.omniscient_view();
        let phase = game.phase();
        let contract = game.contract();
        let trick_before = game.current_trick();
        let events = match game.apply(action) {
            Ok(events) => events,
            Err(e) => ctx.fail(&format!("legal {action:?} rejected: {e:?}")),
        };
        actions.push(action);
        let ctx = Ctx {
            seed,
            config,
            actions: &actions,
        };
        if phase == Phase::Spiel
            && let Action::Play(card) = action
        {
            let game_type = contract.unwrap().game;
            let hand = before.hands[seat.index()];
            ctx.check(may_play(game_type, hand, trick_before.led(), card), || {
                format!("{seat:?} played {} illegally", notation(card))
            });
            let position = trick_before.len();
            let expected_seat = Seat::from_index(expected_leader.index() + position);
            ctx.check(seat == expected_seat, || {
                format!("{seat:?} played out of turn")
            });
            if position == 2 {
                let trick = *game.tricks().last().unwrap();
                ctx.check(trick.leader == expected_leader, || "wrong leader".into());
                let winner = Seat::from_index(
                    expected_leader.index() + expected_winner(game_type, trick.cards),
                );
                ctx.check(trick.winner == winner, || {
                    format!("trick {:?} won by {:?}", trick, trick.winner)
                });
                if Some(winner) == game.declarer() {
                    declarer_augen += trick
                        .cards
                        .iter()
                        .map(|c| u32::from(c.augen()))
                        .sum::<u32>();
                }
                expected_leader = winner;
            }
        }
        for event in &events {
            if let Event::Kontra { .. } = event {
                stats.add("kontra said");
            }
        }
        check_conservation(&ctx, &game);
    }

    let ctx = Ctx {
        seed,
        config,
        actions: &actions,
    };
    match game.phase() {
        Phase::Eingepasst => {
            stats.add("eingepasst");
            ctx.check(game.result().is_none() && game.tricks().is_empty(), || {
                "Einpassen".into()
            });
        }
        Phase::Beendet => {
            let r = *game.result().unwrap();
            let o = r.outcome;
            let all = game.omniscient_view();
            let n = game.tricks().len();
            let declarer = o.declarer;
            if o.contract.game == GameType::Null {
                let first_lost = game.tricks().iter().position(|t| t.winner == declarer);
                ctx.check(first_lost.map_or(n == 10, |i| i + 1 == n), || {
                    "Null end".into()
                });
                ctx.check(r.won == first_lost.is_none(), || "Null result".into());
                if n < 10 {
                    stats.add("null early end");
                }
            } else {
                ctx.check(n == 10, || "trump game ended early".into());
                let total: u32 = game
                    .tricks()
                    .iter()
                    .flat_map(|t| t.cards)
                    .map(|c| u32::from(c.augen()))
                    .sum::<u32>()
                    + u32::from(all.skat.augen());
                ctx.check(total == 120, || format!("Augen total {total}"));
            }
            ctx.check(
                u32::from(o.declarer_augen) == declarer_augen + u32::from(all.skat.augen()),
                || format!("declarer Augen {}", o.declarer_augen),
            );
            ctx.check(o.tricks_played as usize == n, || "tricks played".into());
            ctx.check((r.score > 0) == r.won, || {
                format!("score {} won {}", r.score, r.won)
            });
            ctx.check(o.kontra == KontraLevel::None || config.kontra_re, || {
                "Kontra while disabled".into()
            });
            let sf = r.seeger_fabian();
            ctx.check(
                if r.won {
                    sf.iter().filter(|&&p| p != 0).count() == 1
                } else {
                    sf.iter().filter(|&&p| p == 40).count() == 2
                },
                || format!("Seeger-Fabian {sf:?}"),
            );
            let c = o.contract;
            let family = match c.game {
                GameType::Farbe(_) => "farbe",
                GameType::Grand => "grand",
                GameType::Null => "null",
            };
            stats.add(format!("{family} hand={} ouvert={}", c.hand, c.ouvert));
            if c.schneider && !c.schwarz {
                stats.add(format!("{family} schneider angesagt"));
            }
            if c.schwarz && !c.ouvert {
                stats.add(format!("{family} schwarz angesagt"));
            }
            stats.add(format!(
                "kontra {:?} enabled={}",
                o.kontra, config.kontra_re
            ));
            stats.add(if r.won { "won" } else { "lost" });
            if r.ueberreizt {
                stats.add("ueberreizt");
            }
        }
        phase => ctx.fail(&format!("stopped in {phase:?}")),
    }
    (actions, game)
}

#[test]
fn ten_thousand_seeded_games() {
    let mut stats = Stats::default();
    for seed in 0..game_count() {
        let config = RuleConfig {
            kontra_re: seed.is_multiple_of(2),
        };
        let (actions, game) = play_game(seed, config, &mut stats);
        // Same seed and actions: identical states and results.
        let mut replay = Game::new(seed, config);
        for &a in &actions {
            replay.apply(a).unwrap();
        }
        assert_eq!(replay, game, "seed {seed}: replay differs");
    }
    let mut expected = vec![
        "eingepasst".to_string(),
        "null early end".into(),
        "kontra said".into(),
        "won".into(),
        "lost".into(),
        "ueberreizt".into(),
    ];
    for family in ["farbe", "grand"] {
        for (hand, ouvert) in [(false, false), (true, false), (true, true)] {
            expected.push(format!("{family} hand={hand} ouvert={ouvert}"));
        }
        expected.push(format!("{family} schneider angesagt"));
        expected.push(format!("{family} schwarz angesagt"));
    }
    for (hand, ouvert) in [(false, false), (true, false), (false, true), (true, true)] {
        expected.push(format!("null hand={hand} ouvert={ouvert}"));
    }
    for level in ["None", "Kontra", "Re"] {
        expected.push(format!("kontra {level} enabled=true"));
    }
    expected.push("kontra None enabled=false".into());
    for key in &expected {
        assert!(
            stats.0.get(key).is_some_and(|&n| n > 0),
            "never exercised: {key}\n{:?}",
            stats.0
        );
    }
    assert!(
        !stats
            .0
            .keys()
            .any(|k| k.contains("enabled=false") && !k.contains("None"))
    );
}

#[test]
fn same_seed_same_game() {
    let config = RuleConfig { kontra_re: true };
    let mut stats = Stats::default();
    let (a, game_a) = play_game(4711, config, &mut stats);
    let (b, game_b) = play_game(4711, config, &mut stats);
    assert_eq!(a, b);
    assert_eq!(game_a, game_b);
    assert_eq!(game_a.result(), game_b.result());
    assert_ne!(Game::new(4711, config), Game::new(4712, config));
}
