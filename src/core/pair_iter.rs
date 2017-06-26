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

//! Iterate through all the possible pairs in a vector.
//!
//! There are (n choose 2) == n * (n - 1) / 2 possible combinations,
//! where n is the the length of the vector.

pub struct PairIterator<'a, T: 'a> {
    items: &'a [T],
    next: (usize, usize),
}

impl<'a, T> Iterator for PairIterator<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<(&'a T, &'a T)> {
        let (x, y) = self.next;

        if x >= self.items.len() {
            None
        } else {
            self.next = if y + 1 == x { (x + 1, 0) } else { (x, y + 1) };
            Some((&self.items[x], &self.items[y]))
        }
    }
}

pub trait PairIter<'a, T> {
    fn pairs(&'a self) -> PairIterator<'a, T>;
}

impl<'a, T> PairIter<'a, T> for [T] {
    fn pairs(&'a self) -> PairIterator<'a, T> {
        PairIterator { items: self, next: (1, 0) }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_pair_iter() {
        let nums = [0, 1, 2, 3, 4];

        let pairs = nums.pairs()
            .map(|(&a, &b)| (a, b))
            .collect::<Vec<_>>();

        let expected = [(1, 0),
                        (2, 0), (2, 1),
                        (3, 0), (3, 1), (3, 2),
                        (4, 0), (4, 1), (4, 2), (4, 3)];

        assert_eq!(pairs, expected);

        // can't make any pairs from a single element slice
        assert_eq!([1].pairs().count(), 0);

        // can't make any pairs from a zero element slice
        let empty: [usize; 0] = [];
        assert_eq!(empty.pairs().count(), 0);

        // should get exactly one pair from a 2-element slice
        let two = [0, 1];
        assert_eq!(two.pairs().count(), 1);

        if let Some((&a, &b)) = two.pairs().nth(0) {
            assert_eq!((a, b), (1, 0));
        } else {
            panic!();
        }
    }
}
