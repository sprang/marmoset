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

use card::*;
use find::FindSets;
use pair_iter::PairIter;
use shuffle::Shuffle;
use std::cmp;

pub const DECK_SIZE: usize = 81;

/// Returns a vector containing all the cards in a Set deck.
pub fn cards() -> Vec<Card> {
    (0..DECK_SIZE).map(Card::new).collect()
}

#[derive(Default, Clone)]
pub struct Deck { stock: Vec<Card> }

impl Deck {
    /// Returns a shuffled `Deck`.
    pub fn new() -> Deck {
        let mut cards = cards();
        cards.shuffle();
        Deck { stock: cards }
    }

    /// Removes all cards from the deck that do not have a solid
    /// shading. This is useful as a deck for beginners.
    pub fn simplify(&mut self) {
        self.stock.retain(|card| card.shading() == Shading::Solid);
    }

    pub fn is_empty(&self) -> bool {
        self.stock.is_empty()
    }

    pub fn remainder(&self) -> usize {
        self.stock.len()
    }

    pub fn draw(&mut self, n: usize) -> Vec<Card> {
        let r = self.remainder();
        let x = cmp::min(n, r);
        self.stock.split_off(r - x)
    }
}

impl Deck {
    /// The smallest number of cards guaranteed to contain a `Set` is
    /// 21. However, the odds that there are no sets in 18 cards is so
    /// low that it almost never happens in practice. As long as there
    /// are at least 6 cards in the stock, we can doctor the deck to
    /// guarantee that 18 cards will contain a `Set`:
    ///
    /// 15 (table) + 6 (stock) == 21
    ///
    /// If there are only 3 more cards to deal, you could still get
    /// stuck with 18 cards, but at that point the game would be over.
    ///
    /// In the worst case scenario, 15 cards are on the table, 6 remain
    /// in the stock, and there's exactly 1 `Set` amongst those 21
    /// cards.* There are 3 cases we need to handle:
    ///
    /// 1) Two cards from the `Set` are on the table, and one is in the
    /// stock. We need to make sure that the one card in the stock is in
    /// the next draw.
    ///
    /// 2) One card from the `Set` is on the table, and two are in the
    /// stock. We need to make sure that both cards in the stock are in
    /// the next draw.
    ///
    /// 3) All three cards in the `Set` are in the stock. We need to put
    /// those three cards into the next draw.
    ///
    /// *NOTE: It's possible that 21 cards always contain 2 or more
    /// sets. As far as I know, that's an open question.
    ///
    pub fn draw_guaranteeing_set(&mut self, hand: &[Card]) -> Option<Vec<Card>> {
        assert_eq!(hand.len(), 15);
        assert!(self.stock.len() >= 6);

        // Check to see if simply drawing the next 3 cards is okay.
        // This will almost always work.
        let mut draw = self.draw(3);
        let mut test = hand.to_owned();
        test.append(&mut draw.clone());

        if test.contains_set() {
            return Some(draw);
        } else {
            // return the draw to the stock so we can doctor the deck
            self.stock.append(&mut draw);
        }

        self.fix_one_card(hand)
            .or_else(|| self.fix_two_cards(hand))
            .or_else(|| self.fix_three_cards())
    }

    fn fix_one_card(&mut self, hand: &[Card]) -> Option<Vec<Card>> {
        // shuffle the cards in the hand so we don't favor cards at
        // the front of the layout
        let mut hand = hand.to_owned();
        hand.shuffle();

        for c in hand.pairs().map(|pair| pair.complete_set()) {
            if let Some(ix) = self.stock.iter().position(|&obj| obj == c) {
                // swap the matching card with the top card
                let last_ix = self.stock.len() - 1;
                self.stock.swap(ix, last_ix);

                // now just draw normally
                let mut draw = self.draw(3);

                // shuffle to randomize the position of the found card
                draw.shuffle();
                return Some(draw);
            }
        }

        None
    }

    fn fix_two_cards(&mut self, hand: &[Card]) -> Option<Vec<Card>> {
        if let Some((&a, &b)) = self.stock.pairs()
            .filter(|&pair| hand.contains(&pair.complete_set()))
            .nth(0)
        {
            let mut result = vec![a, b];
            // remove the found pair from the stock
            self.stock.retain(|&n| n != a && n != b);

            // need 1 more card... any will do
            result.append(&mut self.draw(1));

            // randomize the order
            result.shuffle();
            Some(result)
        } else {
            None
        }
    }

    fn fix_three_cards(&mut self) -> Option<Vec<Card>> {
        if let Some(set) = self.stock.find_first_set() {
            let (a,b,c) = set.cards();
            self.stock.retain(|&n| n != a && n != b && n != c);
            Some(vec![a, b, c])
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use card::Card;
    use find::{FindSets, FindSuperSets};

    #[test]
    fn count_sets() {
        let sets = cards().find_all_sets();
        assert_eq!(sets.len(), 1080);
        assert_eq!(cards().count_sets(), 1080);
    }

    #[test]
    fn count_supersets() {
        let supersets = cards().find_all_supersets();
        assert_eq!(supersets.len(), 63180);
        assert_eq!(cards().count_supersets(), 63180);
    }

    #[test]
    fn check_draw_cards() {
        let mut deck = Deck::new();
        let mut r = deck.remainder();

        let mut deal = deck.draw(12);
        assert_eq!(deal.len(), 12);
        assert_eq!(deck.remainder(), r - 12);

        r = deck.remainder();
        deal = deck.draw(r + 10);
        assert_eq!(deal.len(), r);
        assert!(deck.is_empty());

        deal = deck.draw(3);
        assert!(deal.is_empty());
    }

    trait AsCards {
        fn as_cards(&self) -> Vec<Card>;
    }

    impl AsCards for [usize] {
        fn as_cards(&self) -> Vec<Card> {
            self.iter()
                .map(|&ix| Card::new(ix))
                .collect()
        }
    }

    #[test]
    fn check_fixers() {
        // these cards are a set
        const A: usize = 21;
        const B: usize = 41;
        const C: usize = 58;
        let set = [A, B, C].as_cards();
        assert!(set.contains_set());

        // 9 cards that contain exactly one set: (A, B, C)
        let indices = vec![11, 19, 31, 34, 64, 72, A, B, C];
        let cards = indices.as_cards();
        assert_eq!(cards.count_sets(), 1);

        ////////////////////////////////////////////////////////////////////////////////
        // TEST fix_one_card
        ////////////////////////////////////////////////////////////////////////////////

        let hand = [11, A, B].as_cards(); // two cards from the set in the hand
        let stock = [C, 19, 31, 34, 64, 72].as_cards(); // one in the stock

        assert!(!hand.contains_set());
        assert!(!stock.contains_set());

        let mut deck = Deck { stock: stock };
        match deck.fix_one_card(&hand) {
            None => panic!("Could not guarantee set!"),
            Some(mut draw) => {
                let mut test = hand.clone();
                test.append(&mut draw);
                assert!(test.contains_set());
                test.sort_by_key(|c| c.index());
                assert_eq!(test, [11, A, 34, B, C, 64].as_cards());
                assert_eq!(deck.stock, [72, 19, 31].as_cards());
            }
        }

        ////////////////////////////////////////////////////////////////////////////////
        // TEST fix_two_cards
        ////////////////////////////////////////////////////////////////////////////////

        let hand = [11, 19, A].as_cards(); // one card from the set in the hand
        let stock = [B, 31, 34, C, 64, 72].as_cards(); // two in the stock

        assert!(!hand.contains_set());
        assert!(!stock.contains_set());

        let mut deck = Deck { stock: stock };
        match deck.fix_two_cards(&hand) {
            None => panic!("Could not guarantee set!"),
            Some(mut draw) => {
                let mut test = hand.clone();
                test.append(&mut draw);
                assert!(test.contains_set());
                test.sort_by_key(|c| c.index());
                assert_eq!(test, [11, 19, A, B, C, 72].as_cards());
                assert_eq!(deck.stock, [31, 34, 64].as_cards());
            }
        }

        ////////////////////////////////////////////////////////////////////////////////
        // TEST fix_three_cards
        ////////////////////////////////////////////////////////////////////////////////

        let stock = [34, A, B, C, 64, 72].as_cards(); // three cards from the set in the stock

        let mut deck = Deck { stock: stock };
        match deck.fix_three_cards() {
            None => panic!("Could not guarantee set!"),
            Some(mut draw) => {
                assert!(draw.contains_set());
                draw.sort_by_key(|c| c.index());
                assert_eq!(draw, set);
                assert_eq!(deck.stock, [34, 64, 72].as_cards());
            }
        }
    }

    #[test]
    fn check_guarantee() {
        // 20 cards that contain no sets
        let indices = vec![0,1,3,4,9,13,14,15,19,34,38,39,40,44,49,50,52,53,60,74];

        // find the remaining cards in the deck
        let mut complement: Vec<usize> = (0..81).collect();
        for &ix in indices.iter().rev() {
            complement.remove(ix);
        }

        // convert indices to Cards
        let mut hand = indices.as_cards();
        let rest = complement.as_cards();

        assert_eq!(hand.len(), 20);
        assert!(!hand.contains_set());

        let mut stock = hand.split_off(15);
        assert_eq!(stock.len(), 5);

        // shift the stock to the right, so we can put our test cards at the bottom
        stock.insert(0, rest[0]);

        // add each of the remaining cards to the stock one by one,
        // making sure that we can find sets
        for &x in &rest {
            stock[0] = x;
            let mut deck = Deck { stock: stock.clone() };

            match deck.draw_guaranteeing_set(&hand) {
                None => panic!("Could not guarantee set!"),
                Some(mut draw) => {
                    let mut test = hand.clone();
                    test.append(&mut draw);
                    assert_eq!(test.len(), 18);
                    assert!(test.contains_set());
                }
            }
        }
    }
}
