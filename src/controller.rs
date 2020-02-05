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

use cairo::{Context, Rectangle};
use gdk::{self, EventMask};
use gtk::prelude::*;
use gtk::{Allocation, DrawingArea};
use num_traits::ToPrimitive;
use std::cell::RefCell;
use std::{f64, i32};
use std::rc::Rc;

use crate::cell::Cell;
use crate::config::{self, Config};
use core::card::Card;
use core::geometry::{RectangleExt, zero_rect};
use core::graphics::{ContextExt, ColorScheme};
use crate::game_state::{GameState, ROWS, COLUMNS};
use crate::rules::Rules;

const CARD_WIDTH: f64 = 3.5;
const CARD_HEIGHT: f64 = 2.25;
/// for debugging dirty rects
const VISUALIZE_REDRAWS: bool = false;
/// scaling factor used when hovering over a card
const EXPLODE: f64 = 1.04;

/// Callback for undo status changes
type Notification = Box<dyn Fn(&Controller) -> ()>;

pub struct Controller {
    /// Settings
    pub config: Config,
    /// Game Mechanics
    state: GameState,
    rules: Box<dyn Rules>,
    selected: Vec<Card>,
    /// Undo Stacks
    undo_stack: Vec<UndoItem>,
    redo_stack: Vec<UndoItem>,
    undo_observers: Vec<Notification>,
    /// Layout
    tableau_bounds: Rectangle,
    cell_rects: Vec<Rectangle>,
    /// Widget
    view: DrawingArea,
    /// Event Bookkeeping
    clicked_card: Option<Card>,
    // is the mouse inside the click card?
    inside_clicked_card: bool,
    exploded_cell: Option<usize>,
}

impl Controller {
    pub fn shared_with_config(config: Config) -> Rc<RefCell<Controller>> {
	let drawing_area = Controller::new_drawing_area();
	let controller = Controller {
	    config,
	    state: GameState::with_config(config),
	    rules: config.rules(),
	    selected: vec!(),
	    undo_stack: vec!(),
	    redo_stack: vec!(),
	    undo_observers: vec!(),
	    tableau_bounds: zero_rect(),
	    cell_rects: vec![zero_rect(); ROWS*COLUMNS],
	    view: drawing_area.clone(),
	    clicked_card: None,
	    inside_clicked_card: false,
	    exploded_cell: None,
	};

	// need a shared reference that can be moved into event callbacks
	let shared_controller = Rc::new(RefCell::new(controller));

	macro_rules! connect {
	    ($connect:ident :> $action:ident) => {{
		let controller = shared_controller.clone();
		drawing_area.$connect(
		    move |a, b| controller.borrow_mut().$action(a, b)
		);
	    }}
	}

	connect!(connect_draw :> draw);
	connect!(connect_size_allocate :> layout);
	connect!(connect_button_press_event :> button_press);
	connect!(connect_button_release_event :> button_release);
	connect!(connect_key_press_event :> key_press);
	connect!(connect_key_release_event :> key_release);
	connect!(connect_motion_notify_event :> motion_notify);

	shared_controller
    }

    fn new_drawing_area() -> DrawingArea {
	let drawing_area = DrawingArea::new();
	let event_mask = EventMask::POINTER_MOTION_MASK
	    | EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK
	    | EventMask::KEY_PRESS_MASK | EventMask::KEY_RELEASE_MASK;

	drawing_area.set_can_focus(true);
	drawing_area.add_events(event_mask);

	// establish a reasonable minimum view size
	drawing_area.set_size_request(800, 450);
	drawing_area
    }

    pub fn get_drawing_area(&self) -> DrawingArea {
	self.view.clone()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Actions
////////////////////////////////////////////////////////////////////////////////

impl Controller {
    fn new_game_with_state(&mut self, start_state: Option<GameState>) {
	if let Some(state) = start_state {
	    self.state = state;
	}

	self.selected.clear();
	self.reset_undo_stacks();
	self.redraw();
    }

    pub fn restart(&mut self) {
	// state will be None if the undo stack is empty
	let state = self.undo_stack.first().map(|item| item.state.clone());
	self.new_game_with_state(state);
    }

    pub fn new_game(&mut self) {
	let state = GameState::with_config(self.config);
	self.new_game_with_state(Some(state));
    }

    pub fn show_hint(&mut self) -> Option<String> {
	self.deselect_all();

	if let Some(hint_cards) = self.rules.hint(&self.state.cards()) {
	    self.selected = hint_cards;
	    self.redraw();
	    None
	} else if self.state.deck.is_empty() {
	    Some("No more moves!".to_string())
	} else {
	    self.deal_more_cards()
	}
    }

    pub fn deal_more_cards(&mut self) -> Option<String> {
	if self.rules.stuck(&self.state.cards()) {
	    if self.state.deck.is_empty() {
		return Some("No more moves!".to_string());
	    } else {
		self.register_undo("Deal More Cards");
		self.state.deal(self.rules.set_size());
		self.redraw();
	    }

	    None
	} else {
	    let num_in_play = self.rules.count_sets(&self.state.cards());
	    let string = if num_in_play == 1 {
		format!("There is 1 {} available.", self.rules.name())
	    } else {
		format!("There are {} {}s available.", num_in_play, self.rules.name())
	    };

	    Some(string)
	}
    }

    fn check_for_set(&mut self) {
	if self.selected.len() == self.rules.set_size() {
	    // if we found a valid set, remove it, otherwise deselect the last selected card
	    if self.rules.valid_set(&self.selected) {
		let action_name = self.rules.name();
		self.register_undo(action_name);

		self.state.take_cards(&self.selected, &*self.rules);
		self.deselect_all();
	    } else if let Some(card) = self.selected.pop() {
		self.redraw_cell(self.state.index_of_card(card));
	    }
	}
    }
}

////////////////////////////////////////////////////////////////////////////////
// Config
////////////////////////////////////////////////////////////////////////////////

impl Controller {
    pub fn set_deck(&mut self, deck: config::Deck) {
	self.config.set_deck(deck);
	self.new_game();
    }

    pub fn set_variant(&mut self, variant: config::Variant) {
	self.config.set_variant(variant);
	self.rules = self.config.rules();
	self.new_game();
    }

    pub fn set_tidy_layout(&mut self, tidy: bool) {
	self.config.set_tidy_layout(tidy);
	self.redraw();
    }

    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
	self.config.set_color_scheme(scheme);
	self.redraw();
    }
}

////////////////////////////////////////////////////////////////////////////////
// Selection
////////////////////////////////////////////////////////////////////////////////

impl Controller {
    fn deselect_all(&mut self) {
	if !self.selected.is_empty() {
	    self.selected.clear();
	    self.redraw();
	}
    }

    fn toggle_selected(&mut self, card: Card) {
	if self.is_selected(card) {
	    self.selected.retain(|&c| c != card);
	} else if self.selected.len() < self.rules.set_size() {
	    self.selected.push(card);
	}

	self.redraw_cell(self.state.index_of_card(card));
    }

    fn is_selected(&self, card: Card) -> bool {
	// this is an O(n) test, but n is <= 4
	self.selected.contains(&card)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Undo
////////////////////////////////////////////////////////////////////////////////

struct UndoItem {
    state: GameState,
    action_name: &'static str
}

/// Undo and Redo are symmetrical operations. This is implemented from
/// the undo perspective, but redo is the same operation with the
/// corresponding parameters swapped.
macro_rules! create_do {
    ($name:ident, $undo_stack:ident, $redo_stack:ident) => {
	pub fn $name(&mut self) {
	    if let Some(prev) = self.$undo_stack.pop() {
		// push the current state onto the redo stack
		let redo = UndoItem {
		    state: self.state.clone(),
		    action_name: prev.action_name
		};
		self.$redo_stack.push(redo);

		// set the current state to the undo state
		self.state = prev.state;
		self.selected.clear();
		self.redraw();

		self.undo_status_changed();
	    }
	}
    }
}

impl Controller {
    fn register_undo(&mut self, action_name: &'static str) {
	let item = UndoItem {
	    state: self.state.clone(),
	    action_name
	};
	self.undo_stack.push(item);
	self.redo_stack.clear();
	self.undo_status_changed();
    }

    fn reset_undo_stacks(&mut self) {
	self.undo_stack.clear();
	self.redo_stack.clear();
	self.undo_status_changed();
    }

    fn undo_status_changed(&self) {
	// post undo nofifications
	for f in &self.undo_observers { f(self) }
    }

    pub fn add_undo_observer<F>(&mut self, f: F) where F: Fn(&Controller) -> () + 'static {
	self.undo_observers.push(Box::new(f));
    }

    pub fn can_undo(&self) -> bool {
	!self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
	!self.redo_stack.is_empty()
    }

    pub fn undo_action_name(&self) -> Option<&str> {
	self.undo_stack.last().map(|item| item.action_name)
    }

    pub fn redo_action_name(&self) -> Option<&str> {
	self.redo_stack.last().map(|item| item.action_name)
    }

    // pub fn undo(&mut self);
    create_do!(undo, undo_stack, redo_stack);

    // pub fn redo(&mut self);
    create_do!(redo, redo_stack, undo_stack);
}

////////////////////////////////////////////////////////////////////////////////
// Event Handling
////////////////////////////////////////////////////////////////////////////////

impl Controller {
    fn card_for_point(&self, x: f64, y: f64) -> Option<Card> {
	// calculate the tableau row and column of the mouse location
	let cell_width = self.tableau_bounds.width / COLUMNS as f64;
	let cell_height = self.tableau_bounds.height / ROWS as f64;

	let col = ((x - self.tableau_bounds.x) / cell_width) as i32;
	let row = ((y - self.tableau_bounds.y) / cell_height) as i32;

	let col_valid = 0 <= col && col < COLUMNS as i32;
	let row_valid = 0 <= row && row < ROWS as i32;

	if col_valid && row_valid {
	    let cell_index = row as usize * COLUMNS + col as usize;
	    let cell = self.state.tableau[cell_index];
	    let cell_rect = self.cell_rects[cell_index];

	    if let Cell::Card(data) = cell {
		let transform = !self.config.tidy_layout;
		if data.point_in_rect(x, y, cell_rect, transform) {
		    return Some(data.card);
		}
	    }
	}

	None
    }

    fn set_exploded_cell(&mut self, cell: Option<usize>) {
	if self.exploded_cell != cell {
	    // redisplay old cell
	    self.redraw_cell(self.exploded_cell);
	    self.exploded_cell = cell;
	    // redisplay new cell
	    self.redraw_cell(self.exploded_cell);
	}
    }

    fn set_inside_clicked_card(&mut self, flag: bool) {
	if self.inside_clicked_card != flag {
	    self.inside_clicked_card = flag;

	    if let Some(card) = self.clicked_card {
		// we transitioned in or out of the clicked card, so
		// we need to toggle its selection state
		self.toggle_selected(card);
	    }
	}
    }

    fn motion_notify(&mut self, _widget: &DrawingArea, event: &gdk::EventMotion) -> Inhibit {
	let (x, y) = event.get_position();
	let mouse_down_in_card = self.clicked_card.is_some();
	let mut inside = false;

	if let Some(card) = self.card_for_point(x, y) {
	    inside = Some(card) == self.clicked_card;
	    if !mouse_down_in_card || inside {
		let ix = self.state.index_of_card(card);
		self.set_exploded_cell(ix);
	    }
	} else {
	    self.set_exploded_cell(None);
	}

	self.set_inside_clicked_card(inside);

	Inhibit(false)
    }

    fn button_press(&mut self, _widget: &DrawingArea, event: &gdk::EventButton) -> Inhibit {
	let single = event.get_event_type() == gdk::EventType::ButtonPress;
	let primary = event.get_button() == 1;

	if single && primary {
	    let (x, y) = event.get_position();

	    if let Some(card) = self.card_for_point(x, y) {
		self.clicked_card = Some(card);
		self.inside_clicked_card = true;
		self.toggle_selected(card);
	    }
	}

	Inhibit(false)
    }

    fn button_release(&mut self, _widget: &DrawingArea, event: &gdk::EventButton) -> Inhibit {
	if event.get_button() == 1 {
	    self.clicked_card = None;
	    self.inside_clicked_card = false;
	    self.check_for_set();
	}

	Inhibit(false)
    }

    fn key_press(&mut self, _widget: &DrawingArea, event: &gdk::EventKey) -> Inhibit {
	if let Some(byte) = event.get_keyval().to_u8() {
	    let letter = byte as char;

	    // only pay attention to lowercase letters with no modifiers
	    if letter.is_alphabetic() && event.get_state().is_empty() {
		if let Some(hotkey) = letter.to_lowercase().next() {
		    if let Some(card) = self.state.card_for_key(hotkey) {
			self.toggle_selected(card);
		    }
		}
	    }
	}

	// make sure we don't lose focus
	let inhibit = event.get_keyval() == gdk::enums::key::Tab;
	Inhibit(inhibit)
    }

    fn key_release(&mut self, _widget: &DrawingArea, _event: &gdk::EventKey) -> Inhibit {
	self.check_for_set();
	Inhibit(false)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Rendering
////////////////////////////////////////////////////////////////////////////////

/// Returns the space needed for `n` `items` separated by `spacing`. Assumes
/// spacing is applied between items, at the beginning, and at the end.
fn span(n: usize, item: f64, spacing: f64) -> f64 {
    (item + spacing) * (n as f64) + spacing
}

impl Controller {
    fn layout(&mut self, _widget: &DrawingArea, allocation: &Allocation) {
	let (w, h) = (allocation.width, allocation.height);

	// figure out the tableau aspect ratio
	let spacing_percentage = 0.15;
	let tableau_spacing = CARD_WIDTH * spacing_percentage;
	let tableau_width = span(COLUMNS, CARD_WIDTH, tableau_spacing);
	let tableau_height = span(ROWS, CARD_HEIGHT, tableau_spacing);
	let tableau_aspect_ratio = tableau_width / tableau_height;

	// figure out the view aspect ratio
	let (view_width, view_height) = (f64::from(w), f64::from(h));
	let view_aspect_ratio = view_width / view_height;

	// now squeeze the tableau into the view
	let effective_view_width = if view_aspect_ratio > tableau_aspect_ratio {
	    // height constrained...
	    view_height * tableau_aspect_ratio
	} else {
	    view_width
	};

	let card_width = effective_view_width / span(COLUMNS, 1., spacing_percentage);
	let card_height = CARD_HEIGHT / CARD_WIDTH * card_width;
	let spacing = card_width * spacing_percentage;

	// ... and center it
	let offset_x = (view_width - span(COLUMNS, card_width, spacing)) / 2.;
	let offset_y = (view_height - span(ROWS, card_height, spacing)) / 2.;

	for y in 0..ROWS {
	    let dy = offset_y + span(y, card_height, spacing);
	    for x in 0..COLUMNS {
		let dx = offset_x + span(x, card_width, spacing);
		let rect = Rectangle {
		    x: dx,
		    y: dy,
		    width: card_width,
		    height: card_height
		};
		self.cell_rects[y * COLUMNS + x] = rect.round();
	    }
	}

	let bounds = Rectangle {
	    x: offset_x,
	    y: offset_y,
	    width: span(COLUMNS, card_width, spacing),
	    height: span(ROWS, card_height, spacing)
	};

	self.tableau_bounds = bounds.inset(spacing, spacing);
    }

    fn draw(&self, _widget: &DrawingArea, ctx: &Context) -> Inhibit {
	let remainder = self.state.deck.remainder();
	let remainder_label = if remainder == 1 { "card left" } else { "cards left" };
	let scheme = self.config.color_scheme;

	// view background
	if VISUALIZE_REDRAWS { ctx.set_source_random_rgb() } else { ctx.set_source_gray(0.8) }
	ctx.paint();

	let iter = self.state.tableau.iter().zip(self.cell_rects.iter());
	for (ix, (&cell, &rect)) in iter.enumerate() {
	    match cell {
		Cell::Deck => ctx.draw_badge(rect, remainder, remainder_label),
		Cell::Score => ctx.draw_badge(rect, self.state.score, "found"),
		Cell::Placeholder => ctx.draw_card_placeholder(rect),
		Cell::Card(data) => {
		    ctx.save();
		    ctx.with_pivot(rect.center(), || {
			if self.exploded_cell == Some(ix) { ctx.scale(EXPLODE, EXPLODE) }
			if !self.config.tidy_layout { ctx.rotate(data.angle) }
		    });
		    if self.is_selected(data.card) { ctx.draw_card_selection(rect) }
		    ctx.draw_card(data.card, rect, Some(&data.hotkey.to_string()), scheme);
		    ctx.restore();
		}
	    }
	}

	Inhibit(false)
    }

    fn redraw(&self) {
	self.view.queue_draw();
    }

    fn redraw_in_rect(&self, rect: Rectangle) {
	let integral_rect = rect.round();
	self.view.queue_draw_area(integral_rect.x as i32,
				  integral_rect.y as i32,
				  integral_rect.width as i32,
				  integral_rect.height as i32);
    }

    fn redraw_cell(&self, cell_index: Option<usize>) {
	if let Some(ix) = cell_index {
	    let rect = self.cell_rects[ix];
	    let padding = rect.width * 0.2;
	    self.redraw_in_rect(rect.inset(-padding, -padding));
	}
    }
}
