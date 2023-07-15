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

use cairo::Rectangle;
use std::f64;

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
    Rectangle::new(0., 0., 0., 0.)
}

impl RectangleExt for Rectangle {
    #[inline]
    fn center(&self) -> Point {
        (self.x() + self.width() / 2., self.y() + self.height() / 2.)
    }

    #[inline]
    fn max_x(&self) -> f64 {
        self.x() + self.width()
    }

    #[inline]
    fn max_y(&self) -> f64 {
        self.y() + self.height()
    }

    #[inline]
    fn inset(&self, dx: f64, dy: f64) -> Rectangle {
        Rectangle::new(
            self.x() + dx / 2.,
            self.y() + dy / 2.,
            self.width() - dx,
            self.height() - dy,
        )
    }

    #[inline]
    fn offset(&self, dx: f64, dy: f64) -> Rectangle {
        Rectangle::new(self.x() + dx, self.y() + dy, self.width(), self.height())
    }

    #[inline]
    fn round(&self) -> Rectangle {
        Rectangle::new(
            f64::round(self.x()),
            f64::round(self.y()),
            f64::round(self.width()),
            f64::round(self.height()),
        )
    }

    #[inline]
    fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x() && x <= self.max_x() && y >= self.y() && y <= self.max_y()
    }
}
