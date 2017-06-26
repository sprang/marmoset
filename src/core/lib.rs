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

extern crate cairo;
extern crate num_traits;
extern crate rand;
#[macro_use] extern crate serde_derive;
extern crate time;

// model
pub mod card;
pub mod deck;
pub mod find;
pub mod pair_iter;
pub mod shuffle;

// rendering
pub mod geometry;
pub mod graphics;

// misc
pub mod utils;
