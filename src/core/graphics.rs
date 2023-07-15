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

#![allow(clippy::cast_lossless)]

use crate::card::{Card, Color, Shading, Shape};
use crate::geometry::RectangleExt;
use cairo::{Context, Rectangle};
use rand::{thread_rng, Rng};
use std::f64;
use std::f64::consts::{FRAC_PI_2, PI};

const CORNER_RADIUS_PERCENTAGE: f64 = 0.08;
const BADGE_BACKGROUND_GRAY: f64 = 0.68;
const CARD_LABEL_GRAY: f64 = 0.75;
const PLACEHOLDER_GRAY: f64 = 0.75;
const TABLEAU_BACKGROUND_GRAY: f64 = 0.8;
const MOCK_STRIPE_TRANSLUCENCY: f64 = 0.4;

#[inline]
pub fn card_corner_radius(Rectangle { height, .. }: Rectangle) -> f64 {
    CORNER_RADIUS_PERCENTAGE * height
}

////////////////////////////////////////////////////////////////////////////////
// ColorScheme
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    CMYK,
    Classic,
}

impl ColorScheme {
    pub fn card_color(self, card: Card) -> (f64, f64, f64) {
        let (r, g, b) = match self {
            // This scheme is intended to be friendlier to those with
            // color vision deficiencies
            ColorScheme::CMYK => match card.color() {
                Color::A => (0, 200, 220), // cyan
                Color::B => (192, 0, 192), // magenta
                Color::C => (220, 200, 0), // yellow
            },

            ColorScheme::Classic => match card.color() {
                Color::A => (0, 151, 0),   // green
                Color::B => (130, 0, 140), // purple
                Color::C => (240, 0, 0),   // red
            },
        };

        (r as f64 / 255., g as f64 / 255., b as f64 / 255.)
    }
}

////////////////////////////////////////////////////////////////////////////////
// ContextExt
////////////////////////////////////////////////////////////////////////////////

pub trait ContextExt {
    /// Perform transform operations around a pivot point.
    fn with_pivot<F>(&self, pivot: (f64, f64), f: F)
    where
        F: Fn() -> ();

    fn set_source_gray(&self, g: f64);
    fn set_source_random_rgb(&self);

    fn rounded_rect(&self, rect: Rectangle, radius: f64);
    fn diamond_in_rect(&self, rect: Rectangle);
    fn squiggle_in_rect(&self, rect: Rectangle);

    fn draw_badge(&self, rect: Rectangle, count: usize, label: &str);
    fn draw_card_background(&self, rect: Rectangle, label: Option<&str>, gray: f64);
    fn draw_card_placeholder(&self, rect: Rectangle);
    fn draw_card_selection(&self, rect: Rectangle);
    fn draw_card(&self, card: Card, rect: Rectangle, label: Option<&str>, scheme: ColorScheme);
}

impl ContextExt for Context {
    fn with_pivot<F>(&self, (px, py): (f64, f64), f: F)
    where
        F: Fn() -> (),
    {
        self.translate(px, py);
        f();
        self.translate(-px, -py);
    }

    fn set_source_gray(&self, g: f64) {
        self.set_source_rgb(g, g, g);
    }

    fn set_source_random_rgb(&self) {
        let mut rng = thread_rng();
        let r = rng.gen_range(0.0..1.0);
        let g = rng.gen_range(0.0..1.0);
        let b = rng.gen_range(0.0..1.0);

        self.set_source_rgb(r, g, b);
    }

    fn rounded_rect(&self, rect: Rectangle, radius: f64) {
        let Rectangle {
            x,
            y,
            width,
            height,
        } = rect;
        let r = f64::min(radius, f64::min(width / 2., height / 2.));

        self.new_sub_path();
        self.arc(x + width - r, y + r, r, -FRAC_PI_2, 0.);
        self.arc(x + width - r, y + height - r, r, 0., FRAC_PI_2);
        self.arc(x + r, y + height - r, r, FRAC_PI_2, PI);
        self.arc(x + r, y + r, r, PI, FRAC_PI_2 * 3.);
        self.close_path();
    }

    fn diamond_in_rect(&self, rect: Rectangle) {
        let Rectangle {
            x,
            y,
            width,
            height,
        } = rect;
        let half_width = width / 2.;
        let half_height = height / 2.;

        self.new_sub_path();
        self.move_to(x + half_width, y);
        self.line_to(x + width, y + half_height);
        self.line_to(x + half_width, y + height);
        self.line_to(x, y + half_height);
        self.close_path();
    }

    fn squiggle_in_rect(&self, rect: Rectangle) {
        let Rectangle {
            x,
            y,
            width,
            height,
        } = rect;

        self.new_sub_path();
        self.move_to(x + width / 3., y);

        self.curve_to(
            x + width * 4. / 5.,
            y,
            x + width,
            y + height / 6.,
            x + width,
            y + height / 3.,
        );

        self.curve_to(
            x + width,
            y + height / 2.,
            x + width * 5. / 6.,
            y + height / 2.,
            x + width * 5. / 6.,
            y + height * 2. / 3.,
        );

        self.curve_to(
            x + width * 5. / 6.,
            y + height * 5. / 6.,
            x + width,
            y + height * 5. / 6.,
            x + width,
            y + height * 11. / 12.,
        );

        self.curve_to(
            x + width,
            y + height * 23. / 24.,
            x + width * 5. / 6.,
            y + height,
            x + width * 2. / 3.,
            y + height,
        );

        self.curve_to(
            x + width / 5.,
            y + height,
            x,
            y + height * 5. / 6.,
            x,
            y + height * 2. / 3.,
        );

        self.curve_to(
            x,
            y + height / 2.,
            x + width / 6.,
            y + height / 2.,
            x + width / 6.,
            y + height / 3.,
        );

        self.curve_to(
            x + width / 6.,
            y + height / 6.,
            x,
            y + height / 6.,
            x,
            y + height / 12.,
        );

        self.curve_to(x, y + height / 24., x + width / 6., y, x + width / 3., y);

        self.close_path();
    }

    fn draw_badge(&self, rect: Rectangle, count: usize, label: &str) {
        let badge_height = rect.height * (2. / 3.);
        let label_height = rect.height - badge_height;
        let count_string = count.to_string();

        let padding = rect.width * 0.2;
        let badge_rect = Rectangle {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: badge_height,
        }
        .inset(padding, padding / 8.);

        // draw badge background
        self.set_source_gray(BADGE_BACKGROUND_GRAY);
        self.rounded_rect(badge_rect.round(), f64::INFINITY);
        self.fill();

        // draw the label (same gray as badge background)
        self.set_font_size(label_height * 0.9);
        let extents = self.text_extents(label);
        let x = rect.x + (rect.width - extents.width) / 2.;
        let y = rect.max_y() - (label_height - extents.height) / 3.;

        self.move_to(x, y);
        self.show_text(label);

        // draw count
        self.set_font_size(badge_height * 0.75);
        let extents = self.text_extents(&count_string);
        let x = rect.x + (rect.width - extents.width) / 2. - extents.x_bearing;
        let y = badge_rect.max_y() - (badge_rect.height - extents.height) / 2.;

        self.move_to(x, y);
        self.set_source_gray(TABLEAU_BACKGROUND_GRAY);
        self.show_text(&count_string);
    }

    fn draw_card_background(&self, rect: Rectangle, label: Option<&str>, gray: f64) {
        let corner_radius = card_corner_radius(rect);
        self.rounded_rect(rect, corner_radius);
        self.set_source_gray(gray);
        self.fill();

        if let Some(text) = label {
            let font_size = f64::min(rect.height * 0.15, 24.);
            self.set_font_size(font_size);
            self.move_to(rect.x + corner_radius, rect.max_y() - corner_radius);
            self.set_source_gray(CARD_LABEL_GRAY);
            self.show_text(text);
        }
    }

    fn draw_card_placeholder(&self, rect: Rectangle) {
        self.draw_card_background(rect, None, PLACEHOLDER_GRAY);
    }

    fn draw_card_selection(&self, rect: Rectangle) {
        let Rectangle { height, .. } = rect;
        let corner_radius = card_corner_radius(rect);
        let selection_width = (height * 0.035).round() * 2.;

        self.rounded_rect(rect, corner_radius);
        self.set_source_gray(0.);
        self.set_line_width(selection_width);
        self.stroke();
    }

    fn draw_card(&self, card: Card, rect: Rectangle, label: Option<&str>, scheme: ColorScheme) {
        let Rectangle {
            x,
            y,
            width,
            height,
        } = rect;
        // render the background
        self.draw_card_background(rect, label, 1.0);

        // calculate shape bounds and margins
        let vertical_margin = 0.15 * height;
        let spacing = vertical_margin / 2.;
        let shape_height = height - (vertical_margin * 2.);
        let shape_width = shape_height / 2.1;

        let count = card.count();
        // total width of all shapes including spacing
        let shape_extent = (count as f64) * (shape_width + spacing) - spacing;
        let horizontal_margin = (width - shape_extent) / 2.;

        // bounds of a single shape
        let mut shape_rect = Rectangle {
            x: x + horizontal_margin,
            y: y + vertical_margin,
            width: shape_width,
            height: shape_height,
        };

        // add the shapes to the context
        for _ in 0..count {
            match card.shape() {
                Shape::Oval => self.rounded_rect(shape_rect, f64::INFINITY),
                Shape::Squiggle => self.squiggle_in_rect(shape_rect),
                Shape::Diamond => self.diamond_in_rect(shape_rect),
            }
            shape_rect = shape_rect.offset(shape_width + spacing, 0.);
        }

        // determine card color
        let (r, g, b) = scheme.card_color(card);
        self.set_source_rgb(r, g, b);

        // compute base outline width
        let stroke_width = shape_width / 11.;

        // finally, do the rendering based on the shading
        match card.shading() {
            Shading::Solid => self.fill(),
            Shading::Outlined => {
                // clip to the path so that the stroked shape has the
                // same footprint as the filled shape
                self.clip_preserve();
                // double the width since half the stroke is clipped away
                self.set_line_width(stroke_width * 2.);
                self.stroke();
                self.reset_clip();
            }
            Shading::Striped => {
                // a translucent fill is more attractive than stripes
                self.set_source_rgba(r, g, b, MOCK_STRIPE_TRANSLUCENCY);
                self.fill_preserve();

                // draw a white band between the stroke and the translucent fill
                self.set_source_gray(1.0);
                self.set_line_width(stroke_width * 3.);
                self.stroke_preserve();

                // draw the outside stroke in the card color
                self.set_source_rgb(r, g, b);
                self.set_line_width(stroke_width * 4. / 3.);
                // clip to the path so that the stroked shape has the
                // same footprint as the filled shape
                self.clip_preserve();
                self.stroke();
                self.reset_clip();
            }
        }
    }
}
