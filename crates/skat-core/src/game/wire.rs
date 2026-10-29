// SPDX-License-Identifier: EUPL-1.2

//! Deserialisation of [`Game`] (D-25). The wire form is checked for the
//! relationships between its fields, so a deserialised game is one that some
//! sequence of legal actions from its deal can reach.

use super::{CompletedTrick, Game, Phase, Step, Trick};
use crate::bidding::{BidOutcome, Bidding};
use crate::card::CardSet;
use crate::deal::Deal;
use crate::rules::{Announcement, Contract, GameType, legal_cards, trick_winner};
use crate::scoring::{GameResult, KontraLevel, settle};
use crate::seat::Seat;
use crate::RuleConfig;

/// Unvalidated wire form of [`Game`].
#[derive(serde::Deserialize)]
pub(super) struct GameData {
    config: RuleConfig,
    deal: Deal,
    hands: [CardSet; 3],
    skat: CardSet,
    phase: Phase,
    bidding: Bidding,
    declarer: Option<Seat>,
    bid: u16,
    hand_game: Option<bool>,
    discarded: bool,
    contract: Option<Contract>,
    tricks: Vec<CompletedTrick>,
    current: Trick,
    played: [bool; 3],
    kontra: KontraLevel,
    kontra_by: Option<Seat>,
    step: Step,
    revealed: bool,
    result: Option<GameResult>,
}

impl TryFrom<GameData> for Game {
    type Error = &'static str;

    fn try_from(data: GameData) -> Result<Game, &'static str> {
        let game = Game {
            config: data.config,
            deal: data.deal,
            hands: data.hands,
            skat: data.skat,
            phase: data.phase,
            bidding: data.bidding,
            declarer: data.declarer,
            bid: data.bid,
            hand_game: data.hand_game,
            discarded: data.discarded,
            contract: data.contract,
            tricks: data.tricks,
            current: data.current,
            played: data.played,
            kontra: data.kontra,
            kontra_by: data.kontra_by,
            step: data.step,
            revealed: data.revealed,
            result: data.result,
        };
        game.check_consistency()?;
        Ok(game)
    }
}

impl Game {
    fn check_consistency(&self) -> Result<(), &'static str> {
        let (bidding, outcome) =
            Bidding::replay(&self.bidding.history).ok_or("invalid bidding history")?;
        if bidding != self.bidding {
            return Err("bidding state does not match its history");
        }
        let (declarer, bid) = match outcome {
            BidOutcome::Continue if self.phase == Phase::Reizen => (None, 0),
            BidOutcome::Eingepasst if self.phase == Phase::Eingepasst => (None, 0),
            BidOutcome::Declarer(declarer, bid)
                if !matches!(self.phase, Phase::Reizen | Phase::Eingepasst) =>
            {
                (Some(declarer), bid)
            }
            _ => return Err("phase does not match the bidding outcome"),
        };
        if self.declarer != declarer || self.bid != bid {
            return Err("declarer or bid does not match the bidding");
        }
        let in_play = matches!(self.phase, Phase::Spiel | Phase::Beendet);
        if !in_play {
            let pristine = self.contract.is_none()
                && self.tricks.is_empty()
                && self.current == Trick::default()
                && self.played == [false; 3]
                && self.kontra == KontraLevel::None
                && self.kontra_by.is_none()
                && self.step == Step::Play
                && !self.revealed
                && self.result.is_none();
            if !pristine {
                return Err("play state set before the game started");
            }
        }
        let deal_hands = Seat::ALL.map(|s| self.deal.hand(s));
        let Some(declarer) = declarer else {
            let untouched = self.hand_game.is_none()
                && !self.discarded
                && self.hands == deal_hands
                && self.skat == self.deal.skat();
            return if untouched {
                Ok(())
            } else {
                Err("cards or choices set before the declarer is known")
            };
        };
        let all12 = deal_hands[declarer.index()] | self.deal.skat();
        // Hands at the start of card play.
        let mut start = deal_hands;
        match (self.phase, self.hand_game) {
            (Phase::SkatAufnahme, None) => {
                return if self.hands == deal_hands
                    && self.skat == self.deal.skat()
                    && !self.discarded
                {
                    Ok(())
                } else {
                    Err("cards changed before the Skat decision")
                };
            }
            (Phase::SkatAufnahme, Some(_)) | (_, None) => {
                return Err("Hand decision does not match the phase");
            }
            (Phase::Druecken, Some(false)) => {
                start[declarer.index()] = all12;
                return if self.hands == start && self.skat.is_empty() && !self.discarded {
                    Ok(())
                } else {
                    Err("cards do not match the picked-up Skat")
                };
            }
            (Phase::Druecken, Some(true)) => return Err("a Hand game cannot discard"),
            (_, Some(true)) => {
                if self.discarded || self.skat != self.deal.skat() {
                    return Err("the Skat of a Hand game must stay untouched");
                }
            }
            (_, Some(false)) => {
                if !self.discarded || self.skat.len() != 2 || !self.skat.is_subset(all12) {
                    return Err("discard does not fit the declarer's cards");
                }
                start[declarer.index()] = all12 - self.skat;
            }
        }
        if !in_play {
            return if self.hands == start {
                Ok(())
            } else {
                Err("hands do not match the deal")
            };
        }
        self.check_play(declarer, start)
    }

    /// Phases `Spiel` and `Beendet`.
    fn check_play(&self, declarer: Seat, start: [CardSet; 3]) -> Result<(), &'static str> {
        let contract = self.contract.ok_or("no contract during play")?;
        let announcement = Announcement {
            game: contract.game,
            schneider: contract.schneider,
            schwarz: contract.schwarz,
            ouvert: contract.ouvert,
        };
        if contract.hand != (self.hand_game == Some(true)) || !announcement.is_valid(contract.hand)
        {
            return Err("invalid contract");
        }
        if contract.null_value().is_some_and(|v| v < self.bid) {
            return Err("Null game below the bid");
        }
        let finished = self.phase == Phase::Beendet;
        // Cards played per seat; every card at most once.
        let mut played_cards = [CardSet::EMPTY; 3];
        // Hands as they were before each historical play, to check Bedienen.
        let mut remaining = start;
        let follows = |remaining: &mut [CardSet; 3], seat: Seat, card, led| {
            let hand = remaining[seat.index()];
            let ok = hand.contains(card) && legal_cards(contract.game, hand, led).contains(card);
            remaining[seat.index()].remove(card);
            ok
        };
        let mut count = 0;
        let mut expected_leader = Seat::Vorhand;
        for (i, trick) in self.tricks.iter().enumerate() {
            let winner = Seat::from_index(trick.leader.index() + trick_winner(contract.game, trick.cards));
            if trick.leader != expected_leader || trick.winner != winner {
                return Err("trick leader or winner is wrong");
            }
            let last = i + 1 == self.tricks.len();
            if contract.game == GameType::Null && winner == declarer && !(last && finished) {
                return Err("Null game continued after the declarer won a trick");
            }
            for (j, &card) in trick.cards.iter().enumerate() {
                if !follows(&mut remaining, trick.seat_of(j), card, Some(trick.cards[0])) {
                    return Err("a card was played against Bedienen");
                }
                played_cards[trick.seat_of(j).index()].insert(card);
                count += 1;
            }
            expected_leader = winner;
        }
        let in_trick = self.current.len();
        if in_trick > 2 || self.current.cards.iter().take(in_trick).any(Option::is_none) {
            return Err("trick in progress is malformed");
        }
        let leader_ok = if in_trick == 0 {
            self.current.leader == self.tricks.last().map(|t| t.winner)
        } else {
            self.current.leader == Some(expected_leader)
        };
        if !leader_ok {
            return Err("leader of the trick in progress is wrong");
        }
        for (i, card) in self.current.cards.iter().flatten().enumerate() {
            let seat = Seat::from_index(expected_leader.index() + i);
            if !follows(&mut remaining, seat, *card, self.current.led()) {
                return Err("a card was played against Bedienen");
            }
            played_cards[seat.index()].insert(*card);
            count += 1;
        }
        let total: usize = played_cards.iter().map(|c| c.len()).sum();
        if total != count || self.tricks.len() + usize::from(in_trick > 0) > 10 {
            return Err("a card was played twice or there are too many tricks");
        }
        for seat in Seat::ALL {
            let i = seat.index();
            let played = played_cards[i];
            if !self.hands[i].is_disjoint(played)
                || self.hands[i] | played != start[i]
                || self.played[i] == played.is_empty()
            {
                return Err("hands do not match the cards played");
            }
        }
        match &self.result {
            None if !finished => {
                if self.tricks.len() == 10 {
                    return Err("all tricks played but the game is not finished");
                }
            }
            Some(result) if finished => {
                let all_tricks = self.tricks.len() == 10;
                let null_lost = contract.game == GameType::Null
                    && self.tricks.last().is_some_and(|t| t.winner == declarer);
                if in_trick != 0 || !(all_tricks || null_lost) {
                    return Err("game finished too early");
                }
                if *result != settle(&self.game_outcome()) {
                    return Err("result does not match the game");
                }
            }
            _ => return Err("result does not match the phase"),
        }
        self.check_kontra(declarer, contract)
    }

    /// Kontra/Re, the Ouvert reveal and the sub-step.
    fn check_kontra(&self, declarer: Seat, contract: Contract) -> Result<(), &'static str> {
        let eligible = |s: Seat| s != declarer && self.bidding.kontra_eligible(s);
        match (self.kontra, self.kontra_by) {
            (KontraLevel::None, None) => {}
            (KontraLevel::Kontra | KontraLevel::Re, Some(seat))
                if self.config.kontra_re && eligible(seat) => {}
            _ => return Err("Kontra state is invalid"),
        }
        let before_play = self.tricks.is_empty() && self.current.is_empty();
        match self.step {
            Step::Play => {}
            Step::AwaitRe => {
                if self.phase != Phase::Spiel || self.kontra != KontraLevel::Kontra {
                    return Err("Re can only be asked after Kontra");
                }
            }
            Step::OuvertKontra(seat) => {
                let ok = self.phase == Phase::Spiel
                    && contract.ouvert
                    && !self.revealed
                    && before_play
                    && self.kontra == KontraLevel::None
                    && self.config.kontra_re
                    && eligible(seat);
                if !ok {
                    return Err("Ouvert Kontra round is invalid");
                }
            }
        }
        // An Ouvert game that is not yet revealed is still in the decision
        // round, before the first card.
        let waiting_for_reveal = contract.ouvert && !self.revealed;
        if (self.revealed && !contract.ouvert)
            || (waiting_for_reveal && (self.step == Step::Play || !before_play))
            || (self.phase == Phase::Beendet && self.step != Step::Play)
        {
            return Err("Ouvert reveal does not match the game");
        }
        Ok(())
    }
}
