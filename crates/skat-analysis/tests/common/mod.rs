// SPDX-License-Identifier: EUPL-1.2

//! Test helpers: short card notation.
//!
//! Notation: suit `C` Kreuz, `S` Pik, `H` Herz, `D` Karo; rank `7 8 9 T J Q K A`.

#![allow(dead_code)]

use skat_core::{Card, CardSet, Deal, Rank, Suit};

pub fn c(s: &str) -> Card {
    let mut chars = s.chars();
    let suit = match chars.next() {
        Some('C') => Suit::Kreuz,
        Some('S') => Suit::Pik,
        Some('H') => Suit::Herz,
        Some('D') => Suit::Karo,
        _ => panic!("bad suit in {s}"),
    };
    let rank = match chars.next() {
        Some('7') => Rank::Sieben,
        Some('8') => Rank::Acht,
        Some('9') => Rank::Neun,
        Some('T') => Rank::Zehn,
        Some('J') => Rank::Bube,
        Some('Q') => Rank::Dame,
        Some('K') => Rank::Koenig,
        Some('A') => Rank::Ass,
        _ => panic!("bad rank in {s}"),
    };
    assert!(chars.next().is_none(), "bad card {s}");
    Card::new(suit, rank)
}

pub fn cards(s: &str) -> CardSet {
    s.split_whitespace().map(c).collect()
}

pub fn deal(vh: &str, mh: &str, hh: &str, skat: &str) -> Deal {
    Deal::new([cards(vh), cards(mh), cards(hh)], cards(skat)).expect("valid test deal")
}
