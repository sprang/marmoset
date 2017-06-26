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

use std::f64;
use cairo::Rectangle;

pub type Point = (f64, f64);

pub trait RectangleExt {
    fn center(&self) -> Point;
    fn max_x(&self) -> f64;
    fn max_y(&self) -> f64;
    fn inset(&self, dx: f64, dy: f64) -> Rectangle;
    fn offset(&self, dx: f64, dy: f64) -> Rectangle;
    fn round(&self) -> Rectangle;
    fn contains_point(&self, x: f64, y: f64) -> bool;
}

#[inline]
pub fn zero_rect() -> Rectangle {
    Rectangle { x: 0., y: 0., width: 0., height: 0. }
}

impl RectangleExt for Rectangle {
    #[inline]
    fn center(&self) -> Point {
        (self.x + self.width / 2., self.y + self.height / 2.)
    }

    #[inline]
    fn max_x(&self) -> f64 {
        self.x + self.width
    }

    #[inline]
    fn max_y(&self) -> f64 {
        self.y + self.height
    }

    #[inline]
    fn inset(&self, dx: f64, dy: f64) -> Rectangle {
        Rectangle {
            x: self.x + dx / 2.,
            y: self.y + dy / 2.,
            width: self.width - dx,
            height: self.height - dy
        }
    }

    #[inline]
    fn offset(&self, dx: f64, dy: f64) -> Rectangle {
        Rectangle {
            x: self.x + dx,
            y: self.y + dy,
            width: self.width,
            height: self.height
        }
    }

    #[inline]
    fn round(&self) -> Rectangle {
        Rectangle {
            x: f64::round(self.x),
            y: f64::round(self.y),
            width: f64::round(self.width),
            height: f64::round(self.height)
        }
    }

    #[inline]
    fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.max_x() &&
            y >= self.y && y <= self.max_y()
    }
}
