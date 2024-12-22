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

extern crate cairo;
extern crate clap;
extern crate core;

use cairo::{Context, Format, ImageSurface, Operator, Rectangle};
use clap::Parser;
use std::f64::consts::FRAC_PI_2;
use std::fs::File;
use std::mem;

use core::deck::cards;
use core::graphics::*;
use core::utils::clamp;

const CARD_ASPECT_RATIO: f64 = 3.5 / 2.25;

#[derive(Parser)]
#[command(version)]
#[command(about = "Generate an image for each Marmoset card.")]
struct Cli {
    /// The directory in which to write the images
    directory: String,

    /// Set the card width in pixels
    #[arg(short, long)]
    width: Option<i32>,

    /// Set the border width in pixels
    #[arg(short, long)]
    border: Option<i32>,

    /// Orient cards vertically
    #[arg(short, long)]
    vertical: bool,

    /// Use classic SET colors
    #[arg(short, long)]
    classic: bool,
}

fn generate_card_images(
    path: &str,
    card_width: i32,
    border: i32,
    vertical: bool,
    scheme: ColorScheme,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let card_height = (card_width as f64 / CARD_ASPECT_RATIO).ceil() as i32;
    // offset by (border, border)
    let card_rect = Rectangle::new(
        border as f64,
        border as f64,
        card_width as f64,
        card_height as f64,
    );

    // add space for the border on each edge
    let mut ctx_width = card_width + border * 2;
    let mut ctx_height = card_height + border * 2;
    if vertical {
        mem::swap(&mut ctx_width, &mut ctx_height);
    }

    // create the surface and context
    let surface = ImageSurface::create(Format::ARgb32, ctx_width, ctx_height)
        .expect("Could not create surface.");
    let ctx = Context::new(&surface)?;
    if vertical {
        // adjust the transform to account for the vertical orientation
        ctx.rotate(FRAC_PI_2);
        ctx.translate(0.0, -ctx_width as f64);
    }

    for card in cards() {
        // completely clear the context to avoid accumulating color on
        // any edge that antialiases over the transparent background
        // (e.g. rounded card corners)
        ctx.save()?;
        ctx.set_operator(Operator::Clear);
        ctx.paint()?;
        ctx.restore()?;

        if border > 0 {
            ctx.rounded_rect(card_rect, card_corner_radius(card_rect));
            ctx.set_source_gray(0.0);
            // half the stroke will be covered by the card
            ctx.set_line_width(border as f64 * 2.);
            ctx.stroke()?;
        }

        ctx.draw_card(card, card_rect, None, scheme)?;

        let filename = format!("{}/{}.png", path, card.index());
        let mut image = File::create(&filename)?;

        surface
            .write_to_png(&mut image)
            .unwrap_or_else(|_| println!("Error writing {}", filename));
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let path = &cli.directory;
    let width = cli.width.unwrap_or(350);
    let border = cli.border.unwrap_or(0);
    let render_vertically = cli.vertical;
    let classic_colors = cli.classic;

    // keep values within reasonable ranges
    let width = clamp(width, (64, 6400));
    let border = clamp(border, (0, 64));
    let scheme = if classic_colors {
        ColorScheme::Classic
    } else {
        ColorScheme::CMYK
    };

    generate_card_images(path, width, border, render_vertically, scheme)
        .unwrap_or_else(|e| println!("{}", e));
}
