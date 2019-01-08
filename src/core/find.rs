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

#![allow(unknown_lints)]
#![allow(needless_range_loop)]

use crate::card::*;
use self::Iteration::*;

#[derive(PartialEq, Eq)]
enum Iteration { Continue, Break }

/// Trait for iterating through all the valid combinations within a container.
trait ForEach<T> {
    fn foreach<F>(&self, f: F) where F: FnMut(T) -> Iteration;

    fn find_first(&self) -> Option<T> {
        let mut first = None;
        self.foreach(|x| { first = Some(x); Break });
        first
    }

    fn find_all(&self) -> Vec<T> {
        let mut all = Vec::new();
        self.foreach(|x| { all.push(x); Continue });
        all
    }

    fn count(&self) -> usize {
        let mut num = 0;
        self.foreach(|_| { num += 1; Continue });
        num
    }

    fn contains_any(&self) -> bool {
        let mut contains = false;
        self.foreach(|_| { contains = true; Break });
        contains
    }
}

impl ForEach<Set> for [Card] {
    fn foreach<F>(&self, mut f: F) where F: FnMut(Set) -> Iteration {
        for a in 2..self.len() {
            for b in 1..a {
                for c in 0..b {
                    let triple = (self[a], self[b], self[c]);
                    if let Some(set) = triple.to_set() {
                        if f(set) == Break {
                            return;
                        }
                    }
                }
            }
        }
    }
}

impl ForEach<SuperSet> for [Card] {
    fn foreach<F>(&self, mut f: F) where F: FnMut(SuperSet) -> Iteration {
        for a in 3..self.len() {
            for b in 2..a {
                for c in 1..b {
                    for d in 0..c {
                        let quad = (self[a], self[b], self[c], self[d]);
                        if let Some(superset) = quad.to_superset() {
                            if f(superset) == Break {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// FindSets
////////////////////////////////////////////////////////////////////////////////

/// Convenience wrapper for `ForEach<Set>`
pub trait FindSets {
    fn find_first_set(&self) -> Option<Set>;
    fn find_all_sets(&self) -> Vec<Set>;
    fn count_sets(&self) -> usize;
    fn contains_set(&self) -> bool;
}

impl FindSets for [Card] {
    fn find_first_set(&self) -> Option<Set> {
        self.find_first()
    }

    fn find_all_sets(&self) -> Vec<Set> {
        self.find_all()
    }

    fn count_sets(&self) -> usize {
        ForEach::<Set>::count(self)
    }

    fn contains_set(&self) -> bool {
        ForEach::<Set>::contains_any(self)
    }
}

////////////////////////////////////////////////////////////////////////////////
// FindSuperSets
////////////////////////////////////////////////////////////////////////////////

/// Convenience wrapper for `ForEach<SuperSet>`
pub trait FindSuperSets {
    fn find_first_superset(&self) -> Option<SuperSet>;
    fn find_all_supersets(&self) -> Vec<SuperSet>;
    fn count_supersets(&self) -> usize;
    fn contains_superset(&self) -> bool;
}

impl FindSuperSets for [Card] {
    fn find_first_superset(&self) -> Option<SuperSet> {
        self.find_first()
    }

    fn find_all_supersets(&self) -> Vec<SuperSet> {
        self.find_all()
    }

    fn count_supersets(&self) -> usize {
        ForEach::<SuperSet>::count(self)
    }

    fn contains_superset(&self) -> bool {
        ForEach::<SuperSet>::contains_any(self)
    }
}
