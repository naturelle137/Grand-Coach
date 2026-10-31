// SPDX-License-Identifier: EUPL-1.2

//! Exact card-play mechanisms of one trick (#44, `docs/coach.md`).
//!
//! Two narrow deductions about the trick in progress after the deciding
//! seat's card, from its [`PlayerView`] only:
//!
//! - [`Abstich`]: an opponent who plays later in this trick can trump the
//!   seat's full side card (Ass or Zehn) that currently wins the trick. It
//!   can only if it holds no card of the led side suit **and** at least one
//!   Trumpf (follow-suit rule, ISkO). Farbspiel and Grand.
//! - [`Overtake`]: a defender plays second to the declarer's lead without
//!   taking the trick; the partner, who plays last, can legally beat the
//!   declarer's card: with a higher card of the led class, or, holding no
//!   card of a led side suit, with a Trumpf. Farbspiel and Grand.
//!
//! "Possible" means at least one distribution of the hidden cards that
//! satisfies every hard constraint of the view (sizes, seen cards, proven
//! voids; [`crate::sampling`]) meets the condition, "certain" means every
//! one does. Neither says the seat *will* play that way, which card is
//! better to play, or anything about later tricks. A remaining Trumpf
//! count alone proves no Abstich for a seat, and a void alone does not
//! prove that the partner wins: both conditions are checked jointly.
//!
//! Feasibility is decided exactly, without enumerating distributions:
//! hidden cards go to at most three places (two hands and an unknown Skat)
//! with fixed sizes, so a distribution exists if and only if, for every
//! set of places, the cards that may only go there fit into them (Hall's
//! condition).

use skat_core::{Action, Card, CardSet, Follow, GameType, Phase, PlayerView, Rank, Seat};

use crate::sampling::{Place, Slot, constraints};
use crate::trick::TrickState;

/// How sure a mechanism is, over the hidden distributions consistent with
/// the view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Certainty {
    /// At least one consistent distribution meets the condition, not all.
    Possible,
    /// Every consistent distribution meets it.
    Certain,
}

/// The kind of a [`Mechanism`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MechanismKind {
    /// [`Mechanism::Abstich`].
    Abstich,
    /// [`Mechanism::Overtake`].
    Overtake,
}

/// An opponent who plays later in this trick can trump the seat's side
/// card ([`Mechanism::Abstich`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Abstich {
    /// The opponent; it plays after the deciding seat in this trick.
    pub seat: Seat,
    /// The deciding seat's card: Ass or Zehn of the led side suit, winning
    /// the trick so far.
    pub card: Card,
    /// Whether `seat` can trump: possible or certain.
    pub certainty: Certainty,
    /// Cards of the led side suit the deciding seat has not seen.
    pub suit_unseen: CardSet,
    /// Trumpf the deciding seat has not seen.
    pub trumpf_unseen: CardSet,
}

/// The partner, playing last, can beat the declarer's card
/// ([`Mechanism::Overtake`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Overtake {
    /// The partner (a defender) who plays last in this trick.
    pub partner: Seat,
    /// The declarer, who led.
    pub declarer: Seat,
    /// The declarer's led card, still winning after the deciding seat's
    /// card.
    pub best: Card,
    /// Its class: Trumpf or the led side suit.
    pub led: Follow,
    /// The deciding seat's card.
    pub card: Card,
    /// Whether the partner can beat `best`: possible or certain.
    pub certainty: Certainty,
    /// Unseen cards of the led class that beat `best` and that the partner
    /// may hold.
    pub higher: CardSet,
    /// A side suit was led and the partner may hold none of it and a
    /// Trumpf.
    pub trumping: bool,
}

/// An exact mechanism of the trick after the deciding seat's card.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mechanism {
    /// See [`Abstich`].
    Abstich(Abstich),
    /// See [`Overtake`].
    Overtake(Overtake),
}

impl Mechanism {
    /// Its kind.
    pub fn kind(&self) -> MechanismKind {
        match self {
            Mechanism::Abstich(_) => MechanismKind::Abstich,
            Mechanism::Overtake(_) => MechanismKind::Overtake,
        }
    }
}

/// The mechanisms of the trick in progress after `view.seat` plays
/// `action`, from the view only. Empty for other actions, outside card play,
/// in Null, when neither mechanism's position applies, or when the view is
/// inconsistent.
pub fn mechanisms(view: &PlayerView, action: Action) -> Vec<Mechanism> {
    let Action::Play(card) = action else {
        return Vec::new();
    };
    let Some(position) = Position::of(view, card) else {
        return Vec::new();
    };
    let candidates = position.abstich_seats(view);
    let overtake = position.overtake(view);
    if candidates.is_empty() && overtake.is_none() {
        return Vec::new();
    }
    let Some(places) = Places::of(view) else {
        return Vec::new();
    };
    let mut out: Vec<Mechanism> = candidates
        .into_iter()
        .filter_map(|seat| position.abstich(&places, seat))
        .map(Mechanism::Abstich)
        .collect();
    if let Some((partner, declarer, best)) = overtake {
        out.extend(
            position
                .overtake_by(&places, partner, declarer, best)
                .map(Mechanism::Overtake),
        );
    }
    out
}

/// The trick in progress and the card about to be played.
struct Position {
    game: GameType,
    declarer: Seat,
    seat: Seat,
    card: Card,
    leader: Seat,
    /// Cards already in the trick, in play order.
    before: Vec<Card>,
}

impl Position {
    fn of(view: &PlayerView, card: Card) -> Option<Position> {
        let game = view.contract.map(|c| c.game)?;
        if view.phase != Phase::Spiel
            || game == GameType::Null
            || !view.hand.contains(card)
            || !view.legal_actions.contains(&Action::Play(card))
        {
            return None;
        }
        let before: Vec<Card> = view.current_trick.cards.iter().flatten().copied().collect();
        let leader = if before.is_empty() {
            view.seat
        } else {
            view.current_trick.leader?
        };
        Some(Position {
            game,
            declarer: view.declarer?,
            seat: view.seat,
            card,
            leader,
            before,
        })
    }

    /// The seats that play after the deciding seat in this trick.
    fn after(&self) -> impl Iterator<Item = Seat> + '_ {
        (self.before.len() + 1..3).map(|i| Seat::from_index(self.leader.index() + i))
    }

    fn opponent(&self, other: Seat) -> bool {
        other != self.seat && (self.seat == self.declarer || other == self.declarer)
    }

    /// Opponents playing later whose Abstich is worth checking: the card is
    /// the Ass or Zehn of the led side suit and wins the trick so far.
    fn abstich_seats(&self, view: &PlayerView) -> Vec<Seat> {
        let Follow::Farbe(_) = self.game.follow(self.card) else {
            return Vec::new();
        };
        let led = self.before.first().copied().unwrap_or(self.card);
        let winning =
            TrickState::of(self.game, &view.current_trick).beaten_by(self.game, self.card);
        if !matches!(self.card.rank(), Rank::Ass | Rank::Zehn)
            || self.game.follow(led) != self.game.follow(self.card)
            || !winning
        {
            return Vec::new();
        }
        self.after().filter(|&s| self.opponent(s)).collect()
    }

    fn abstich(&self, places: &Places, seat: Seat) -> Option<Abstich> {
        let suit = self.game.cards_of(self.game.follow(self.card));
        let trumps = self.game.trumps();
        let possible = places.feasible(seat, suit, Some(trumps));
        if !possible {
            return None;
        }
        let certain = !places.feasible(seat, CardSet::EMPTY, Some(suit))
            && !places.feasible(seat, trumps, None);
        Some(Abstich {
            seat,
            card: self.card,
            certainty: certainty(certain),
            suit_unseen: suit & places.unseen,
            trumpf_unseen: trumps & places.unseen,
        })
    }

    /// `(partner, declarer, declarer's card)` if a defender plays second to
    /// the declarer's lead and the declarer's card still wins after it.
    fn overtake(&self, view: &PlayerView) -> Option<(Seat, Seat, Card)> {
        let &[best] = self.before.as_slice() else {
            return None;
        };
        if self.seat == self.declarer || self.leader != self.declarer {
            return None;
        }
        let partner = self.after().next()?;
        let still_best =
            !TrickState::of(self.game, &view.current_trick).beaten_by(self.game, self.card);
        (partner != self.declarer && still_best).then_some((partner, self.declarer, best))
    }

    fn overtake_by(
        &self,
        places: &Places,
        partner: Seat,
        declarer: Seat,
        best: Card,
    ) -> Option<Overtake> {
        let class = self.game.follow(best);
        let follow = self.game.cards_of(class);
        let trumps = self.game.trumps();
        let beats = follow
            .iter()
            .filter(|&c| self.game.beats(c, best, best))
            .collect::<CardSet>();
        let higher: CardSet = (beats & places.unseen)
            .iter()
            .filter(|&c| places.feasible(partner, CardSet::EMPTY, Some(CardSet::single(c))))
            .collect();
        let side = matches!(class, Follow::Farbe(_));
        let trumping = side && places.feasible(partner, follow, Some(trumps));
        if higher.is_empty() && !trumping {
            return None;
        }
        // Cannot beat: holds the led class but nothing higher, or (side
        // suit) holds neither the led suit nor Trumpf.
        let lower = follow - beats;
        let cannot = places.feasible(partner, beats, Some(lower))
            || (side && places.feasible(partner, follow | trumps, None))
            || (!side && places.feasible(partner, beats, None));
        Some(Overtake {
            partner,
            declarer,
            best,
            led: class,
            card: self.card,
            certainty: certainty(!cannot),
            higher,
            trumping,
        })
    }
}

fn certainty(certain: bool) -> Certainty {
    if certain {
        Certainty::Certain
    } else {
        Certainty::Possible
    }
}

/// The hidden places of a view, for exact feasibility questions.
struct Places {
    slots: Vec<Slot>,
    known: [CardSet; 3],
    unseen: CardSet,
}

impl Places {
    /// The places of `view`; `None` if the view admits no distribution.
    fn of(view: &PlayerView) -> Option<Places> {
        let (known, unseen, slots) = constraints(view).ok()?;
        hall(&slots, unseen).then_some(Places {
            slots,
            known: known.hands,
            unseen,
        })
    }

    /// Whether some consistent distribution gives `seat` no card of
    /// `forbid` and, if `require` is given, at least one card of it.
    fn feasible(&self, seat: Seat, forbid: CardSet, require: Option<CardSet>) -> bool {
        let known = self.known[seat.index()];
        if !(known & forbid).is_empty() {
            return false;
        }
        let mut slots = self.slots.clone();
        let own = slots.iter().position(|s| s.place == Place::Seat(seat));
        if let Some(i) = own {
            slots[i].allowed -= forbid;
        }
        match require {
            None => hall(&slots, self.unseen),
            Some(require) if !(known & require).is_empty() => hall(&slots, self.unseen),
            Some(require) => {
                let Some(i) = own.filter(|&i| slots[i].size > 0) else {
                    return false;
                };
                (require & self.unseen & slots[i].allowed)
                    .iter()
                    .any(|card| {
                        let mut forced = slots.clone();
                        forced[i].size -= 1;
                        let mut unseen = self.unseen;
                        unseen.remove(card);
                        hall(&forced, unseen)
                    })
            }
        }
    }
}

/// Whether the `unseen` cards can fill `slots` exactly: the sizes add up,
/// and for every set of slots the cards allowed only there fit into them.
fn hall(slots: &[Slot], unseen: CardSet) -> bool {
    let total: usize = slots.iter().map(|s| usize::from(s.size)).sum();
    if total != unseen.len() {
        return false;
    }
    let masks: Vec<u8> = unseen
        .iter()
        .map(|card| {
            slots
                .iter()
                .enumerate()
                .filter(|(_, s)| s.allowed.contains(card))
                .fold(0u8, |m, (i, _)| m | 1 << i)
        })
        .collect();
    (0u8..1 << slots.len()).all(|set| {
        let only_there = masks.iter().filter(|&&m| m & !set == 0).count();
        let room: usize = slots
            .iter()
            .enumerate()
            .filter(|(i, _)| set & (1 << i) != 0)
            .map(|(_, s)| usize::from(s.size))
            .sum();
        only_there <= room
    })
}
