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

use std::cmp;
use num_traits::Float;

/// Constrain a value within a minimum and maximum range (inclusive).
pub fn clamp<T: Ord> (value: T, (min, max): (T, T)) -> T {
    cmp::min(cmp::max(value, min), max)
}

/// Constrain a float value within a minimum and maximum range (inclusive).
pub fn clamp_float<F: Float>(value: F, (min, max): (F, F)) -> F {
    F::min(F::max(value, min), max)
}

/// Returns a string representing `i` with thousands separated by underscores.
pub fn pretty_print(mut i: u64) -> String {
    let mut result: String = String::new();
    let separator = '_';

    // do once outside the loop to handle 0
    let mut chunks = vec![i % 1000];
    i /= 1000;

    while i != 0 {
        chunks.push(i % 1000);
        i /= 1000;
    }

    for (ix, n) in chunks.iter().rev().enumerate() {
        let digits = if ix == 0 { n.to_string() } else { format!("{:03}", n) };
        result.push_str(&digits);

        if ix + 1 != chunks.len() {
            result.push(separator);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_pretty_print() {
        assert_eq!(&pretty_print(0), "0");
        assert_eq!(&pretty_print(512), "512");
        assert_eq!(&pretty_print(1024), "1_024");
        assert_eq!(&pretty_print(16777216), "16_777_216");
    }

    #[test]
    fn check_clamp() {
        let range = (0, 255);
        assert_eq!(clamp( -1, range),   0);
        assert_eq!(clamp(100, range), 100);
        assert_eq!(clamp(256, range), 255);

        let range = (0.0, 1.0);
        assert_eq!(clamp_float(-1.0, range), 0.0);
        assert_eq!(clamp_float( 0.5, range), 0.5);
        assert_eq!(clamp_float( 2.0, range), 1.0);
    }
}
