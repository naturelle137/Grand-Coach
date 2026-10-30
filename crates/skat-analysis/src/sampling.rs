// SPDX-License-Identifier: EUPL-1.2

//! Hidden-card distributions of one seat's information set and uniform
//! sampling of complete hypothetical deals.
//!
//! Hard constraints, never violated: every card is in exactly one place,
//! hand and Skat sizes, cards the seat has seen (own hand, played cards,
//! own discard, revealed Ouvert cards) and voids proven by not following
//! suit. Within these constraints every distribution is equally likely;
//! heuristic evidence only weights samples afterwards ([`crate::evidence`]).
//!
//! Sampling is exact: the number of consistent distributions is counted
//! by dynamic programming over groups of cards with the same allowed places,
//! and each sample is drawn with probability 1 / count.

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::Rng;
use skat_core::{Card, CardSet, OmniscientView, PlayerView, Seat};

use crate::knowledge::Knowledge;

/// A place a hidden card can be in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Place {
    /// The current hand of a seat.
    Seat(Seat),
    /// The Skat.
    Skat,
}

/// One complete hypothetical distribution of the cards not yet played:
/// current hands and the current Skat (empty while the declarer holds
/// twelve cards). A simulation input, never the real deal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct World {
    /// Current hand of every seat.
    pub hands: [CardSet; 3],
    /// Current Skat.
    pub skat: CardSet,
}

impl World {
    /// The real distribution of a game. Full knowledge: only for uses that
    /// may see every card (open mode, post-game analysis, the Mephisto
    /// opponent); the argument type makes every use visible (D-23).
    pub fn from_omniscient(all: &OmniscientView) -> World {
        World {
            hands: all.hands,
            skat: all.skat,
        }
    }
}

/// Why a view does not describe a consistent information set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Diagnostic {
    /// A card appears twice among the cards the seat has seen.
    CardTwice(Card),
    /// A seat's hand size does not fit its played cards.
    HandSize(Seat),
    /// The unseen cards do not fill the places exactly.
    CardCount {
        /// Cards the seat has not seen.
        unseen: u8,
        /// Free places in other hands and the Skat.
        places: u8,
    },
    /// A known card contradicts a proven void of its holder.
    VoidViolated {
        /// Holder.
        seat: Seat,
        /// The card.
        card: Card,
    },
    /// No distribution satisfies every constraint.
    NoConsistentDeal,
    /// The public history cannot be replayed (bidding, Hand, discard or
    /// contract missing or out of order).
    InvalidHistory,
}

/// A place to fill: where, how many cards, and which cards may go there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Slot {
    /// The place.
    pub place: Place,
    /// Number of hidden cards it holds.
    pub size: u8,
    /// Cards allowed there by the hard constraints.
    pub allowed: CardSet,
}

/// Capacity per place is at most 12, so 13 values per dimension.
const DIM: usize = 13;

/// The hidden cards of one seat's information set.
#[derive(Clone, Debug)]
pub struct HiddenCards {
    known: World,
    unseen: CardSet,
    slots: Vec<Slot>,
    /// Unseen cards grouped by the set of slots they may go to (bit mask).
    groups: Vec<(u8, Vec<Card>)>,
    /// `ways[g][caps]`: distributions of groups `g..` into capacities `caps`.
    ways: Vec<Vec<u128>>,
}

impl HiddenCards {
    /// The hidden cards as `view.seat` sees them.
    pub fn from_view(view: &PlayerView) -> Result<HiddenCards, Vec<Diagnostic>> {
        let k = Knowledge::from_view(view);
        let mut diagnostics = Vec::new();
        let mut seen = CardSet::EMPTY;
        let revealed = view
            .declarer_cards
            .filter(|_| view.declarer != Some(view.seat));
        let parts = [
            k.own,
            k.played,
            view.own_skat.unwrap_or_default(),
            revealed.unwrap_or_default(),
        ];
        for part in parts {
            diagnostics.extend((seen & part).iter().map(Diagnostic::CardTwice));
            seen |= part;
        }
        if view.hand.len() != usize::from(view.hand_sizes[view.seat.index()]) {
            diagnostics.push(Diagnostic::HandSize(view.seat));
        }
        let mut played_by = [0usize; 3];
        for trick in &view.tricks {
            for i in 0..3 {
                played_by[trick.seat_of(i).index()] += 1;
            }
        }
        if let Some(leader) = view.current_trick.leader {
            for i in 0..view.current_trick.len() {
                played_by[Seat::from_index(leader.index() + i).index()] += 1;
            }
        }
        let declarer_holds_12 = view
            .declarer
            .is_some_and(|d| view.hand_sizes[d.index()] == 12);
        for seat in Seat::ALL {
            let full = if declarer_holds_12 && view.declarer == Some(seat) {
                12
            } else {
                10
            };
            if usize::from(view.hand_sizes[seat.index()]) + played_by[seat.index()] != full {
                diagnostics.push(Diagnostic::HandSize(seat));
            }
        }

        let unseen = CardSet::ALL - seen;
        let mut known = World {
            hands: [CardSet::EMPTY; 3],
            skat: view.own_skat.unwrap_or_default(),
        };
        known.hands[view.seat.index()] = k.own;
        let mut slots = Vec::new();
        for seat in Seat::ALL {
            if seat == view.seat {
                continue;
            }
            if let (Some(cards), true) = (revealed, view.declarer == Some(seat)) {
                known.hands[seat.index()] = cards;
                for card in cards & k.voids[seat.index()] {
                    diagnostics.push(Diagnostic::VoidViolated { seat, card });
                }
                continue;
            }
            slots.push(Slot {
                place: Place::Seat(seat),
                size: view.hand_sizes[seat.index()],
                allowed: unseen - k.voids[seat.index()],
            });
        }
        if view.own_skat.is_none() && !declarer_holds_12 {
            slots.push(Slot {
                place: Place::Skat,
                size: 2,
                allowed: unseen,
            });
        }
        let places: u8 = slots.iter().map(|s| s.size).sum();
        if usize::from(places) != unseen.len() {
            diagnostics.push(Diagnostic::CardCount {
                unseen: unseen.len() as u8,
                places,
            });
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        let hidden = HiddenCards::build(known, unseen, slots);
        if hidden.count() == 0 {
            return Err(vec![Diagnostic::NoConsistentDeal]);
        }
        Ok(hidden)
    }

    fn build(known: World, unseen: CardSet, slots: Vec<Slot>) -> HiddenCards {
        let mut groups: Vec<(u8, Vec<Card>)> = Vec::new();
        for card in unseen {
            let mask = slots
                .iter()
                .enumerate()
                .filter(|(_, s)| s.allowed.contains(card))
                .fold(0u8, |m, (i, _)| m | 1 << i);
            match groups.iter_mut().find(|(m, _)| *m == mask) {
                Some((_, cards)) => cards.push(card),
                None => groups.push((mask, vec![card])),
            }
        }
        let mut ways = vec![vec![0u128; DIM * DIM * DIM]; groups.len() + 1];
        ways[groups.len()][0] = 1;
        for g in (0..groups.len()).rev() {
            for caps in 0..DIM * DIM * DIM {
                let caps = decode(caps);
                let mut total = 0;
                for_each_split(groups[g].0, groups[g].1.len() as u8, caps, |x| {
                    let rest = encode([caps[0] - x[0], caps[1] - x[1], caps[2] - x[2]]);
                    total += multinomial(x) * ways[g + 1][rest];
                });
                ways[g][encode(caps)] = total;
            }
        }
        HiddenCards {
            known,
            unseen,
            slots,
            groups,
            ways,
        }
    }

    fn sizes(&self) -> [u8; 3] {
        let mut caps = [0u8; 3];
        for (i, slot) in self.slots.iter().enumerate() {
            caps[i] = slot.size;
        }
        caps
    }

    /// Cards the seat has not seen.
    pub fn unseen(&self) -> CardSet {
        self.unseen
    }

    /// The places to fill and their constraints.
    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }

    /// The cards whose place is known (own hand, own Skat, revealed cards).
    pub fn known(&self) -> World {
        self.known
    }

    /// Number of distributions consistent with every hard constraint.
    pub fn count(&self) -> u128 {
        self.ways[0][encode(self.sizes())]
    }

    /// `true` if `world` satisfies every hard constraint.
    pub fn admits(&self, world: &World) -> bool {
        let all = world.hands[0] | world.hands[1] | world.hands[2] | world.skat;
        let total = world.hands.iter().map(|h| h.len()).sum::<usize>() + world.skat.len();
        if total != all.len() || all != self.unseen | self.known_cards() {
            return false;
        }
        for seat in Seat::ALL {
            if !self.known.hands[seat.index()].is_subset(world.hands[seat.index()]) {
                return false;
            }
        }
        if !self.known.skat.is_subset(world.skat) {
            return false;
        }
        self.slots.iter().all(|slot| {
            let cards = match slot.place {
                Place::Seat(seat) => world.hands[seat.index()],
                Place::Skat => world.skat,
            };
            cards.len() == usize::from(slot.size) && cards.is_subset(slot.allowed)
        }) && self.unknown_places_hold_nothing_else(world)
    }

    fn known_cards(&self) -> CardSet {
        self.known.hands[0] | self.known.hands[1] | self.known.hands[2] | self.known.skat
    }

    /// Places without a slot hold exactly their known cards.
    fn unknown_places_hold_nothing_else(&self, world: &World) -> bool {
        let has_slot = |place: Place| self.slots.iter().any(|s| s.place == place);
        Seat::ALL.iter().all(|&s| {
            has_slot(Place::Seat(s)) || world.hands[s.index()] == self.known.hands[s.index()]
        }) && (has_slot(Place::Skat) || world.skat == self.known.skat)
    }

    /// Draws one distribution uniformly among all consistent ones.
    pub fn sample(&self, rng: &mut ChaCha8Rng) -> World {
        let mut world = self.known;
        let mut caps = self.sizes();
        for (g, (mask, cards)) in self.groups.iter().enumerate() {
            let total = self.ways[g][encode(caps)];
            let mut pick = uniform_below(rng, total);
            let mut chosen = None;
            for_each_split(*mask, cards.len() as u8, caps, |x| {
                if chosen.is_some() {
                    return;
                }
                let rest = [caps[0] - x[0], caps[1] - x[1], caps[2] - x[2]];
                let weight = multinomial(x) * self.ways[g + 1][encode(rest)];
                if pick < weight {
                    chosen = Some(x);
                } else {
                    pick -= weight;
                }
            });
            let x = chosen.expect("weights sum to the total");
            let mut shuffled = cards.clone();
            for i in (1..shuffled.len()).rev() {
                let j = uniform_below(rng, i as u128 + 1) as usize;
                shuffled.swap(i, j);
            }
            let mut next = shuffled.into_iter();
            for (i, &n) in x.iter().enumerate() {
                for card in next.by_ref().take(usize::from(n)) {
                    self.put(&mut world, i, card);
                }
            }
            caps = [caps[0] - x[0], caps[1] - x[1], caps[2] - x[2]];
        }
        world
    }

    fn put(&self, world: &mut World, slot: usize, card: Card) {
        match self.slots[slot].place {
            Place::Seat(seat) => world.hands[seat.index()].insert(card),
            Place::Skat => world.skat.insert(card),
        };
    }

    /// Every consistent distribution, if there are at most `limit`.
    pub fn enumerate(&self, limit: usize) -> Option<Vec<World>> {
        if self.count() > limit as u128 {
            return None;
        }
        let mut out = Vec::new();
        let mut cards: Vec<(Card, u8)> = Vec::new();
        for (mask, group) in &self.groups {
            cards.extend(group.iter().map(|&c| (c, *mask)));
        }
        self.enumerate_from(&cards, self.known, self.sizes(), &mut out);
        Some(out)
    }

    fn enumerate_from(
        &self,
        cards: &[(Card, u8)],
        world: World,
        caps: [u8; 3],
        out: &mut Vec<World>,
    ) {
        let Some(&(card, mask)) = cards.first() else {
            out.push(world);
            return;
        };
        for i in 0..self.slots.len() {
            if mask & (1 << i) != 0 && caps[i] > 0 {
                let mut w = world;
                self.put(&mut w, i, card);
                let mut c = caps;
                c[i] -= 1;
                self.enumerate_from(&cards[1..], w, c, out);
            }
        }
    }
}

fn encode(caps: [u8; 3]) -> usize {
    (usize::from(caps[0]) * DIM + usize::from(caps[1])) * DIM + usize::from(caps[2])
}

fn decode(index: usize) -> [u8; 3] {
    [
        (index / (DIM * DIM)) as u8,
        (index / DIM % DIM) as u8,
        (index % DIM) as u8,
    ]
}

/// Calls `f` for every split of `n` cards over the slots in `mask` within
/// `caps`.
fn for_each_split(mask: u8, n: u8, caps: [u8; 3], mut f: impl FnMut([u8; 3])) {
    let limit = |i: usize| if mask & (1 << i) != 0 { caps[i] } else { 0 };
    for x0 in 0..=limit(0).min(n) {
        for x1 in 0..=limit(1).min(n - x0) {
            let x2 = n - x0 - x1;
            if x2 <= limit(2) {
                f([x0, x1, x2]);
            }
        }
    }
}

fn factorial(n: u8) -> u128 {
    (1..=u128::from(n)).product()
}

fn multinomial(x: [u8; 3]) -> u128 {
    factorial(x[0] + x[1] + x[2]) / (factorial(x[0]) * factorial(x[1]) * factorial(x[2]))
}

/// Uniform value in `0..n` (`n > 0`) without modulo bias.
fn uniform_below(rng: &mut ChaCha8Rng, n: u128) -> u128 {
    let zone = u128::MAX - u128::MAX % n;
    loop {
        let x = u128::from(rng.next_u64()) << 64 | u128::from(rng.next_u64());
        if x < zone {
            return x % n;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multinomials_and_splits() {
        assert_eq!(multinomial([2, 1, 0]), 3);
        assert_eq!(multinomial([10, 10, 2]), 42_678_636);
        let mut splits = Vec::new();
        for_each_split(0b011, 3, [2, 2, 2], |x| splits.push(x));
        assert_eq!(splits, vec![[1, 2, 0], [2, 1, 0]]);
        for i in [0, 5, 100, DIM * DIM * DIM - 1] {
            assert_eq!(encode(decode(i)), i);
        }
    }

    #[test]
    fn impossible_constraints_count_zero() {
        let a = Card::new(skat_core::Suit::Herz, skat_core::Rank::Ass);
        let b = Card::new(skat_core::Suit::Pik, skat_core::Rank::Ass);
        let two = CardSet::single(a) | CardSet::single(b);
        let known = World {
            hands: [CardSet::EMPTY; 3],
            skat: CardSet::EMPTY,
        };
        let slot = |place, allowed| Slot {
            place,
            size: 1,
            allowed,
        };
        // Both slots only accept the Herz Ace.
        let hidden = HiddenCards::build(
            known,
            two,
            vec![
                slot(Place::Seat(Seat::Vorhand), CardSet::single(a)),
                slot(Place::Skat, CardSet::single(a)),
            ],
        );
        assert_eq!(hidden.count(), 0);
        assert_eq!(hidden.enumerate(10), Some(Vec::new()));
        let fine = HiddenCards::build(
            known,
            two,
            vec![
                slot(Place::Seat(Seat::Vorhand), two),
                slot(Place::Skat, CardSet::single(a)),
            ],
        );
        assert_eq!(fine.count(), 1);
    }
}
