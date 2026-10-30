// SPDX-License-Identifier: EUPL-1.2

//! Translated texts for the plain data types of `skat-app`.

use skat_app::{
    AppError, Call, Card, Contract, GameResult, GameType, KontraLevel, Phase, PlayerId, Rank,
    RuleError, Seat, Suit,
};

use crate::i18n::I18n;

/// Name of a player.
pub fn player(i18n: &I18n, player: PlayerId) -> String {
    i18n.t(match player.0 {
        0 => "player-human",
        1 => "player-left",
        _ => "player-right",
    })
}

/// Name of a seat.
pub fn seat(i18n: &I18n, seat: Seat) -> String {
    i18n.t(match seat {
        Seat::Vorhand => "seat-vorhand",
        Seat::Mittelhand => "seat-mittelhand",
        Seat::Hinterhand => "seat-hinterhand",
    })
}

/// Name of a phase.
pub fn phase(i18n: &I18n, phase: Phase) -> String {
    i18n.t(match phase {
        Phase::Reizen => "phase-reizen",
        Phase::SkatAufnahme => "phase-skataufnahme",
        Phase::Druecken => "phase-druecken",
        Phase::Ansage => "phase-ansage",
        Phase::Spiel => "phase-spiel",
        Phase::Beendet => "phase-beendet",
        Phase::Eingepasst => "phase-eingepasst",
    })
}

/// Name of a suit.
pub fn suit(i18n: &I18n, suit: Suit) -> String {
    i18n.t(match suit {
        Suit::Kreuz => "suit-kreuz",
        Suit::Pik => "suit-pik",
        Suit::Herz => "suit-herz",
        Suit::Karo => "suit-karo",
    })
}

/// Name of a rank.
pub fn rank(i18n: &I18n, rank: Rank) -> String {
    i18n.t(match rank {
        Rank::Sieben => "rank-sieben",
        Rank::Acht => "rank-acht",
        Rank::Neun => "rank-neun",
        Rank::Zehn => "rank-zehn",
        Rank::Bube => "rank-bube",
        Rank::Dame => "rank-dame",
        Rank::Koenig => "rank-koenig",
        Rank::Ass => "rank-ass",
    })
}

/// Name of a card, also its accessible label.
pub fn card(i18n: &I18n, card: Card) -> String {
    i18n.tf(
        "card-name",
        &[
            ("suit", suit(i18n, card.suit()).into()),
            ("rank", rank(i18n, card.rank()).into()),
        ],
    )
}

/// Name of a game type.
pub fn game_type(i18n: &I18n, game: GameType) -> String {
    match game {
        GameType::Farbe(s) => suit(i18n, s),
        GameType::Grand => i18n.t("game-grand"),
        GameType::Null => i18n.t("game-null"),
    }
}

/// A contract with its levels, e.g. "Grand Hand Schneider angesagt".
pub fn contract(i18n: &I18n, contract: Contract) -> String {
    let mut parts = vec![game_type(i18n, contract.game)];
    for (on, id) in [
        (contract.hand, "contract-hand"),
        (contract.schneider, "contract-schneider"),
        (contract.schwarz, "contract-schwarz"),
        (contract.ouvert, "contract-ouvert"),
    ] {
        if on {
            parts.push(i18n.t(id));
        }
    }
    parts.join(" ")
}

/// Kontra/Re level; `None` without Kontra.
pub fn kontra(i18n: &I18n, level: KontraLevel) -> Option<String> {
    match level {
        KontraLevel::None => None,
        KontraLevel::Kontra => Some(i18n.t("kontra-level-kontra")),
        KontraLevel::Re => Some(i18n.t("kontra-level-re")),
    }
}

/// A call in the bidding.
pub fn call(i18n: &I18n, call: Call) -> String {
    match call {
        Call::Bid(value) => i18n.tf("call-bid", &[("value", value.into())]),
        Call::Hold(value) => i18n.tf("call-hold", &[("value", value.into())]),
        Call::Pass => i18n.t("call-pass"),
    }
}

/// Details of a settled game, one line each.
pub fn result_lines(i18n: &I18n, declarer: PlayerId, result: &GameResult) -> Vec<String> {
    let o = result.outcome;
    let name = player(i18n, declarer);
    let game = contract(i18n, o.contract);
    let id = if result.won {
        "result-won"
    } else {
        "result-lost"
    };
    let mut lines = vec![i18n.tf(id, &[("player", name.into()), ("game", game.into())])];
    if o.contract.game != GameType::Null {
        lines.push(i18n.tf("result-augen", &[("augen", o.declarer_augen.into())]));
        if let Some(s) = o.spitzen {
            let id = if s.mit {
                "result-spitzen-mit"
            } else {
                "result-spitzen-ohne"
            };
            lines.push(i18n.tf(id, &[("count", s.count.into())]));
        }
    } else {
        lines.push(i18n.tf("result-tricks", &[("count", o.declarer_tricks.into())]));
    }
    if result.schneider {
        lines.push(i18n.t("result-schneider"));
    }
    if result.schwarz {
        lines.push(i18n.t("result-schwarz"));
    }
    lines.push(i18n.tf("result-value", &[("value", result.game_value.into())]));
    if result.ueberreizt {
        lines.push(i18n.tf(
            "result-ueberreizt",
            &[("bid", o.bid.into()), ("value", result.scored_value.into())],
        ));
    }
    if let Some(k) = kontra(i18n, o.kontra) {
        lines.push(k);
    }
    lines.push(i18n.tf("result-score", &[("score", result.score.into())]));
    lines
}

/// A translated, user-facing error message.
pub fn error(i18n: &I18n, error: AppError) -> String {
    match error {
        AppError::Rule(rule) => rule_error(i18n, rule),
        AppError::NotYourTurn => i18n.t("error-not-your-turn"),
        AppError::NoOpponentToMove => i18n.t("error-no-opponent-to-move"),
        AppError::StaleOpponentMove => i18n.t("error-stale-opponent-move"),
        AppError::GameInProgress => i18n.t("error-game-in-progress"),
        AppError::SeriesFinished => i18n.t("error-series-finished"),
        AppError::NotFreePlay => i18n.t("error-not-free-play"),
        AppError::NoDecision => i18n.t("error-no-decision"),
        AppError::CoachOff => i18n.t("error-coach-off"),
        AppError::NothingPending => i18n.t("error-nothing-pending"),
        AppError::ActionPending => i18n.t("error-action-pending"),
        AppError::StaleCoachResult => i18n.t("error-stale-coach-result"),
    }
}

fn rule_error(i18n: &I18n, error: RuleError) -> String {
    match error {
        RuleError::GameOver => i18n.t("error-game-over"),
        RuleError::UnexpectedAction { phase: p } => i18n.tf(
            "error-unexpected-action",
            &[("phase", phase(i18n, p).into())],
        ),
        RuleError::InvalidBidValue { value } => {
            i18n.tf("error-invalid-bid-value", &[("value", value.into())])
        }
        RuleError::BidTooLow { value, current } => i18n.tf(
            "error-bid-too-low",
            &[("value", value.into()), ("current", current.into())],
        ),
        RuleError::InvalidDiscard => i18n.t("error-invalid-discard"),
        RuleError::InvalidAnnouncement => i18n.t("error-invalid-announcement"),
        RuleError::NullValueBelowBid { value, bid } => i18n.tf(
            "error-null-value-below-bid",
            &[("value", value.into()), ("bid", bid.into())],
        ),
        RuleError::CardNotInHand { card: c } => {
            i18n.tf("error-card-not-in-hand", &[("card", card(i18n, c).into())])
        }
        RuleError::MustFollowSuit { card: c } => {
            i18n.tf("error-must-follow-suit", &[("card", card(i18n, c).into())])
        }
        RuleError::KontraNotAllowed => i18n.t("error-kontra-not-allowed"),
    }
}
