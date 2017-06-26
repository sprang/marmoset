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

use cell::{Cell, RenderData};
use config::{self, Config};
use core::card::Card;
use core::deck::Deck;
use core::shuffle::Shuffle;
use rules::Rules;

pub const COLUMNS: usize = 5;
pub const ROWS: usize = 4;

#[derive(Clone)]
pub struct GameState {
    pub deck: Deck,
    pub score: usize,
    pub tableau: Vec<Cell>,
    refill: Vec<usize>,
    hotkeys: Vec<char>,
}

impl GameState {
    pub fn with_config(config: Config) -> GameState {
        let rules = config.rules();
        let mut game_state = GameState {
            deck: Deck::new(),
            score: 0,
            tableau: vec!(Cell::Placeholder; ROWS * COLUMNS),
            refill: rules.deal_order(),
            hotkeys: "abcdefghijklmnopqrstuvwxyz".chars().collect(),
        };

        if config.deck == config::Deck::Simplified { game_state.deck.simplify() }
        game_state.tableau[0] = Cell::Deck;
        game_state.tableau[4] = Cell::Score;
        game_state.hotkeys.shuffle();

        game_state.deal(rules.initial_deal_size());
        game_state
    }

    /// Generate a list of `Card`s from the tableau.
    pub fn cards(&self) -> Vec<Card> {
        self.tableau.iter().filter_map(Cell::card).collect()
    }

    /// Finds the `Card` that matches a hotkey (if any)
    pub fn card_for_key(&self, key: char) -> Option<Card> {
        self.tableau.iter()
            .filter_map(|cell| cell.card_for_key(key))
            .nth(0)
    }

    pub fn card_count(&self) -> usize {
        self.tableau.iter().filter_map(Cell::card).count()
    }

    pub fn index_of_card(&self, card: Card) -> Option<usize> {
        self.tableau.iter().position(|cell| cell.card() == Some(card))
    }

    pub fn take_cards(&mut self, cards: &[Card], rules: &Rules) {
        self.score += 1; // woot!

        for (ix, mut cell) in self.tableau.iter_mut().enumerate().rev() {
            if let Cell::Card(data) = *cell {
                if cards.contains(&data.card) {
                    // reclaim hotkey and tableau index
                    self.hotkeys.push(data.hotkey);
                    self.refill.push(ix);
                    // remove the card
                    *cell = Cell::Placeholder;
                }
            }
        }

        self.hotkeys.shuffle();

        // replenish cards if we dropped below the initial deal size
        if self.card_count() < rules.initial_deal_size() {
            self.deal(rules.set_size());
        }
    }

    pub fn deal(&mut self, n: usize) {
        let cards = self.cards();
        let guarantee_set = n == 3 // this should probably be encoded in `Rules`
            && self.card_count() == 15 && self.deck.remainder() >= 6;

        let new_cards = if guarantee_set {
            self.deck.draw_guaranteeing_set(&cards).unwrap()
        } else {
            self.deck.draw(n)
        };

        for card in new_cards {
            let i = self.refill.pop().unwrap();
            let hotkey = self.hotkeys.pop().unwrap();
            self.tableau[i] = Cell::Card(RenderData::with_card_and_hotkey(card, hotkey));
        }
    }
}
