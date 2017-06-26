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

use core::card::{Card, ToSet};
use core::find::{FindSets, FindSuperSets};
use core::shuffle::Shuffle;

pub struct Set;
pub struct SuperSet;

pub trait Rules {
    fn name(&self) -> &'static str;
    /// Stack of tableau indices: top indices are dealt first.
    fn deal_order(&self) -> Vec<usize>;
    fn initial_deal_size(&self) -> usize;
    fn set_size(&self) -> usize;
    fn valid_set(&self, selection: &[Card]) -> bool;
    fn hint(&self, cards: &[Card]) -> Option<Vec<Card>>;
    fn stuck(&self, cards: &[Card]) -> bool;
    fn count_sets(&self, cards: &[Card]) -> usize;
}

impl Rules for Set {
    fn name(&self) -> &'static str {
        "Set"
    }

    fn deal_order(&self) -> Vec<usize> {
        //
        //  XX   1   2   3  XX
        //   5   6   7   8   9
        //  10  11  12  13  14
        //  15  16  17  18  19
        //
        vec![19, 14, 9, 15, 10, 5, 18, 17, 16, 13, 12, 11, 8, 7, 6, 3, 2, 1]
    }

    fn initial_deal_size(&self) -> usize { 12 }
    fn set_size(&self) -> usize { 3 }

    fn valid_set(&self, cards: &[Card]) -> bool {
        assert_eq!(cards.len(), self.set_size());
        let triple = (cards[0], cards[1], cards[2]);
        triple.to_set().is_some()
    }

    fn hint(&self, cards: &[Card]) -> Option<Vec<Card>> {
        let mut shuffled = cards.to_owned();
        // By shuffling here, we randomize both the order of the discovered
        // sets, as well as the order of the cards within the returned hint
        // pair. Otherwise we favor sets and cards earlier in the layout.
        shuffled.shuffle();

        if let Some(set) = shuffled.find_first_set() {
            let (a,b,_) = set.cards();
            Some(vec![a, b])
        } else {
            None
        }
    }

    fn stuck(&self, cards: &[Card]) -> bool {
        !cards.contains_set()
    }

    fn count_sets(&self, cards: &[Card]) -> usize {
        cards.count_sets()
    }
}

impl Rules for SuperSet {
    fn name(&self) -> &'static str {
        "SuperSet"
    }

    fn deal_order(&self) -> Vec<usize> {
        //
        //  XX   1   2   3  XX
        //  XX   6  XX   8  XX
        //  XX  11  XX  13  XX
        //  XX  16  17  18  XX
        //
        vec![18, 17, 16, 13, 11, 8, 6, 3, 2, 1]
    }

    fn initial_deal_size(&self) -> usize { 10 }
    fn set_size(&self) -> usize { 4 }

    fn valid_set(&self, cards: &[Card]) -> bool {
        assert_eq!(cards.len(), self.set_size());
        cards.contains_superset()
    }

    fn hint(&self, cards: &[Card]) -> Option<Vec<Card>> {
        let mut shuffled = cards.to_owned();
        // Same rationale for randomizing as in rules::Set::hint().
        shuffled.shuffle();

        if let Some(superset) = shuffled.find_first_superset() {
            let (a,b) = superset.left(); // or right
            Some(vec![a, b])
        } else {
            None
        }
    }

    fn stuck(&self, cards: &[Card]) -> bool {
        !cards.contains_superset()
    }

    fn count_sets(&self, cards: &[Card]) -> usize {
        cards.count_supersets()
    }
}
