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

//! Tableau cells.

use cairo::{Matrix, MatrixTrait, Rectangle};
use rand::{Rng, thread_rng};
use core::card::Card;
use core::geometry::*;

#[derive(Clone, Copy)]
pub enum Cell {
    Deck,
    Score,
    Placeholder,
    Card(RenderData)
}

impl Cell {
    /// Convenience method for extracting `Card`
    pub fn card(&self) -> Option<Card> {
        if let Cell::Card(data) = *self {
            Some(data.card)
        } else {
            None
        }
    }

    /// Convenience method for matching a hotkey to a `Card`
    pub fn card_for_key(&self, hotkey: char) -> Option<Card> {
        if let Cell::Card(data) = *self {
            if data.hotkey == hotkey { Some(data.card) } else { None }
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// RenderData
////////////////////////////////////////////////////////////////////////////////

const MAX_ROTATION: f64 = 3.0;

/// Returns a random angle between `-max` and `max` degrees, rounded
/// to the nearest fifth of a degree, and converted to radians.
fn random_angle(max: f64) -> f64 {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0, max * 2.0) - max;
    // round to nearest fifth of a degree
    let degrees = f64::round(angle * 5.) / 5.;
    degrees.to_radians()
}

#[derive(Clone, Copy, Debug)]
pub struct RenderData {
    pub card: Card,
    pub hotkey: char,
    pub angle: f64,
}

impl RenderData {
    pub fn with_card_and_hotkey(card: Card, hotkey: char) -> RenderData {
        let angle = random_angle(MAX_ROTATION);
        RenderData { card, hotkey, angle }
    }

    pub fn point_in_rect(&self, x: f64, y: f64, rect: Rectangle, transform: bool) -> bool {
        if transform {
            let (cx, cy) = rect.center();
            let mut transform = Matrix::identity();

            transform.translate(cx, cy);
            transform.rotate(self.angle);
            transform.translate(-cx, -cy);
            transform.invert();

            let (tx, ty) = transform.transform_point(x, y);
            rect.contains_point(tx, ty)
        } else {
            rect.contains_point(x, y)
        }
    }
}
