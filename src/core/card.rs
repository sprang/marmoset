// Copyright (C) 2017 Steve Sprang
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//! Card, Set, and SuperSet implementation.
//!
//! A `Card` has four features, each of which has three possible
//! values. As such, it can be represented by a four-element vector
//! where each element is a ternary digit (trit): 0, 1, or 2. The
//! implementation here packs this vector into the four bytes of a
//! `u32`.
//!

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Card(u32);

impl Card {
    pub fn new(mut index: usize) -> Card {
        // Convert the index to ternary and pack each of the resulting
        // four trits into the bytes of a u32. The least significant
        // trit ends up in the leftmost byte.
        let mut value = index % 3;

        for _ in 0..3 {
            value <<= 8;
            index /= 3;
            value |= index % 3;
        }

        Card(value as u32)
    }

    /// Maps the card value back to the index from which it was derived.
    pub fn index(self) -> usize {
        let mut value = self.0;
        let mut result = value & 0xff;

        for _ in 0..3 {
            value >>= 8;
            result *= 3;
            result += value & 0xff;
        }

        result as usize
    }
}

////////////////////////////////////////////////////////////////////////////////
// Card: Feature Extraction
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Feature { Count, Shape, Color, Shading }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape { Oval, Squiggle, Diamond }

/// Unlike the other features, Color doesn't have a fixed interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color { A, B, C }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shading { Solid, Striped, Outlined }

impl Card {
    /// Extracts the byte corresponding to the given `Feature`. Since
    /// the bytes represent ternary digits, the returned value will
    /// always be in the interval [0,2].
    fn feature(self, feature: Feature) -> u8 {
        (self.0 >> (feature as u32 * 8) & 0xff) as u8
    }

    /// Returns a shape count in the interval [1,3]
    pub fn count(self) -> u8 {
        self.feature(Feature::Count) + 1
    }

    pub fn shape(self) -> Shape {
        match self.feature(Feature::Shape) {
            0 => Shape::Oval,
            1 => Shape::Squiggle,
            _ => Shape::Diamond,
        }
    }

    pub fn color(self) -> Color {
        match self.feature(Feature::Color) {
            0 => Color::A,
            1 => Color::B,
            _ => Color::C,
        }
    }

    pub fn shading(self) -> Shading {
        match self.feature(Feature::Shading) {
            0 => Shading::Solid,
            1 => Shading::Striped,
            _ => Shading::Outlined,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Card: Debug
////////////////////////////////////////////////////////////////////////////////

use std::fmt;

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Card({})", self.index())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Set
////////////////////////////////////////////////////////////////////////////////

type Triple = (Card, Card, Card);

/// Validated Set
pub struct Set { cards: Triple }

impl Set {
    pub fn cards(&self) -> Triple {
        self.cards
    }
}

pub trait ToSet {
    fn to_set(self) -> Option<Set>;
}

impl ToSet for Triple {
    /// Three cards form a `Set` if their sum is (0,0,0,0) modulo 3.
    fn to_set(self) -> Option<Set> {
        let (a, b, c) = self;
        let mut sum = a.0 + b.0 + c.0;
        let mut result = true;

        while sum != 0 {
            // see if each byte is divisible by 3
            result &= (sum & 0xff) % 3 == 0;
            sum >>= 8;
        }

        if result {
            let set = Set { cards: self };
            Some(set)
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// SuperSet
////////////////////////////////////////////////////////////////////////////////

type Pair = (Card, Card);

/// Validated SuperSet
pub struct SuperSet { left: Pair, right: Pair }

impl SuperSet {
    pub fn left(&self) -> Pair {
        self.left
    }

    pub fn right(&self) -> Pair {
        self.right
    }
}

pub trait ToSuperSet {
    fn to_superset(self) -> Option<SuperSet>;
}

impl ToSuperSet for (Card, Card, Card, Card) {
    fn to_superset(self) -> Option<SuperSet> {
        let (a, b, c, d) = self;
        let pair_combos = &[((a, b), (c, d)),
                            ((a, c), (b, d)),
                            ((a, d), (b, c))];

        for &(left, right) in pair_combos {
            // Two pairs of cards form a SuperSet if and only if the
            // Set-completing card for each pair is the same
            if left.complete_set() == right.complete_set() {
                let superset = SuperSet { left, right };
                return Some(superset);
            }
        }

        None
    }
}

////////////////////////////////////////////////////////////////////////////////
// CompleteSet
////////////////////////////////////////////////////////////////////////////////

pub trait CompleteSet {
    fn complete_set(self) -> Card;
}

impl CompleteSet for Pair {
    /// Given a pair of cards, returns the third card needed to
    /// complete the `Set`.
    ///
    /// This method works by taking the trit-wise sum of the cards in
    /// the pair. Each unique sum maps to a matching trit which is the
    /// appropriate component of the missing card:
    ///
    ///   A | B | A+B | Match
    ///  ---+---+-----+-------
    ///   0 | 0 |   0 |     0
    ///   0 | 1 |   1 |     2
    ///   0 | 2 |   2 |     1
    ///   1 | 1 |   2 |     1
    ///   1 | 2 |   3 |     0
    ///   2 | 2 |   4 |     2
    ///
    fn complete_set(self) -> Card {
        const MATCHES: [u32; 5] = [0, 2, 1, 0, 2];

        let (a, b) = self;
        let mut sum = (a.0 + b.0) as usize;

        let mut value = MATCHES[sum & 0xff];
        for _ in 0..3 {
            value <<= 8;
            sum >>= 8;
            value |= MATCHES[sum & 0xff];
        }

        let reversed = value.swap_bytes();
        Card(reversed)
    }
}

impl<'a> CompleteSet for (&'a Card, &'a Card) {
    fn complete_set(self) -> Card {
        let (&a, &b) = self;
        (a, b).complete_set()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::cards;
    use crate::pair_iter::PairIter;

    #[test]
    fn check_card_conversions() {
        for i in 0..81 {
            let card = Card::new(i);
            assert_eq!(i, card.index());
        }
    }

    #[test]
    fn check_set_completion() {
        let cards = cards();
        let mut set_count = 0;

        for (&a, &b) in cards.pairs() {
            let c = (a, b).complete_set();
            let triple = (a, b, c);
            assert!(triple.to_set().is_some());
            set_count += 1;
        }
        // each set is encountered thrice
        assert_eq!(set_count, 1080 * 3)
    }
}
