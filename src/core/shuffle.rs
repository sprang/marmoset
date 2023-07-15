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

//! Shuffle vectors and slices.
//!
//! Uses the Fisher-Yates algorithm:
//! https://en.wikipedia.org/wiki/Fisher-Yates_shuffle
//!

use rand::{thread_rng, Rng};

pub trait Shuffle {
    fn shuffle(&mut self);
}

impl<T> Shuffle for [T] {
    fn shuffle(&mut self) {
        let mut rng = thread_rng();
        let n = self.len();

        for i in (1..n).rev() {
            let j = rng.gen_range(0..i + 1);
            self.swap(i, j);
        }
    }
}
