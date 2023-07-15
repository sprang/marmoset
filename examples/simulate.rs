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

//! Gather statistics for simulated games of SET.
//!
//! Results on *AMD Ryzen 7 1800x @ 3.9GHz* with 16 threads:
//!
//! Simulating 1_000_000_000 games. This may take some time...
//! PT1280.803583379S seconds elapsed.
//!
//!  hand |           sets |       no sets |          total |   ratio | % with no sets
//! ------+----------------+---------------+----------------+---------+----------------
//!     6 |     12_229_414 |   468_288_234 |    480_517_648 |    1:38 |     97.45495 %
//!     9 |    480_517_648 |   445_045_030 |    925_562_678 |     1:1 |     48.08373 %
//!    12 | 22_385_130_429 | 1_597_587_780 | 23_982_718_209 |    14:1 |      6.66141 %
//!    15 |  1_523_150_458 |    17_280_709 |  1_540_431_167 |    88:1 |      1.12181 %
//!    18 |     16_513_441 |         1_082 |     16_514_523 | 15262:1 |      0.00655 %
//!
//!  cards left | occurrences | % of games
//! ------------+-------------+------------
//!           0 |  12_229_414 |  1.22294 %
//!           6 | 468_288_234 | 46.82882 %
//!           9 | 445_045_030 | 44.50450 %
//!          12 |  73_670_054 |  7.36701 %
//!          15 |     766_796 |  0.07668 %
//!          18 |         472 |  0.00005 %
//!
//! As an optimization, this program makes use of the fact that there is an
//! isomorphism between a `core::Card` and its index. It only uses `core::Card`
//! objects directly when initializing the `SETS` lookup table, and otherwise just
//! works with the cards by index.

#[macro_use]
extern crate clap;
extern crate core;
extern crate num_cpus;
#[macro_use]
extern crate prettytable;
extern crate rand;
extern crate time;

use prettytable::format::consts;
use prettytable::Table;
use rand::{thread_rng, Rng};
use std::cmp;
use std::sync::mpsc;
use std::thread;
use time::PreciseTime;

use core::card::*;
use core::deck::cards;
use core::pair_iter::PairIter;
use core::shuffle::Shuffle;
use core::utils::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NUM_GAMES: u64 = 1_000_000;
const INITIAL_DEAL: usize = 12;
const MAX_DEAL: usize = 22;
const SET_SIZE: usize = 3;

////////////////////////////////////////////////////////////////////////////////
// Counts
////////////////////////////////////////////////////////////////////////////////

struct Counts {
    /// Playable hand.
    sets: [u64; MAX_DEAL],
    /// Stuck hand.
    no_sets: [u64; MAX_DEAL],
    /// Game over.
    remainder: [u64; MAX_DEAL],
}

impl Counts {
    fn zero() -> Counts {
        Counts {
            sets: [0; MAX_DEAL],
            no_sets: [0; MAX_DEAL],
            remainder: [0; MAX_DEAL],
        }
    }

    fn add(&mut self, other: &Counts) {
        for i in 0..MAX_DEAL {
            self.sets[i] += other.sets[i];
            self.no_sets[i] += other.no_sets[i];
            self.remainder[i] += other.remainder[i];
        }
    }

    fn num_simulated(&self) -> u64 {
        self.remainder.iter().sum()
    }

    fn print_hand_stats(&self) {
        let mut table = Table::new();
        table.set_format(*consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        table.set_titles(row![r => "hand", "sets", "no sets", "total", "ratio", "% with no sets"]);

        let iter = self.sets.iter().zip(self.no_sets.iter()).enumerate();

        for (hand_size, (&sets, &no_sets)) in iter {
            if hand_size == 0 || no_sets == 0 {
                continue;
            }

            let total_hands = sets + no_sets;
            // no sets as a percentage of all hands of this size
            let percentage = (no_sets as f64 / total_hands as f64) * 100.0;

            // ratio of sets : no sets
            let ratio = sets as f64 / no_sets as f64;
            let ratio_string = if ratio > 1.0 {
                format!("{}:1", ratio.round() as usize)
            } else {
                format!("1:{}", (1.0 / ratio).round() as usize)
            };

            table.add_row(row![r => &hand_size.to_string(),
                               &pretty_print(sets),
                               &pretty_print(no_sets),
                               &pretty_print(total_hands),
                               &ratio_string,
                               &format!("{:.5} %", percentage)]);
        }

        table.printstd();
    }

    fn print_end_game_stats(&self) {
        let mut table = Table::new();
        table.set_format(*consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        table.set_titles(row![r => "cards left", "occurrences", "% of games"]);

        let num_games = self.num_simulated();

        for (hand_size, &count) in self.remainder.iter().enumerate() {
            if count == 0 {
                continue;
            }

            let percentage = (count as f64 / num_games as f64) * 100.0;
            table.add_row(row![r => &hand_size.to_string(),
                               &pretty_print(count),
                               &format!("{:.5} %", percentage)]);
        }

        table.printstd();
    }
}

////////////////////////////////////////////////////////////////////////////////
// IndexDeck
////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Clone)]
pub struct IndexDeck {
    stock: Vec<usize>,
}

// A deck of indices rather than cards. It didn't seem worth
// generalizing `core::Deck` for the optimizations used in this
// program, so a bit of code duplication here.
impl IndexDeck {
    /// Returns a shuffled `IndexDeck`.
    pub fn new() -> IndexDeck {
        let mut indices = (0..81).collect::<Vec<_>>();
        indices.shuffle();
        IndexDeck { stock: indices }
    }

    pub fn is_empty(&self) -> bool {
        self.stock.is_empty()
    }

    pub fn remainder(&self) -> usize {
        self.stock.len()
    }

    pub fn draw(&mut self, n: usize) -> Vec<usize> {
        let r = self.remainder();
        let x = cmp::min(n, r);
        self.stock.split_off(r - x)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Support Functions
////////////////////////////////////////////////////////////////////////////////

/// Lookup table for Sets.
static mut SETS: [[usize; 81]; 81] = [[0; 81]; 81];

fn build_lookup() {
    let cards = cards();

    for (&a, &b) in (0..81).collect::<Vec<_>>().pairs() {
        let c = (cards[a], cards[b]).complete_set().index();
        unsafe {
            SETS[a][b] = c;
            // `complete_set()` is commutative
            SETS[b][a] = c;
        }
    }
}

#[inline(always)]
fn is_set(a: usize, b: usize, c: usize) -> bool {
    unsafe { *SETS.get_unchecked(a).get_unchecked(b) == c }
}

fn find_random_set(hand: &[usize]) -> Option<(usize, usize, usize)> {
    let mut sets = Vec::new();

    for x in 2..hand.len() {
        let a = hand[x];
        for y in 1..x {
            let b = hand[y];
            for &c in hand.iter().take(y) {
                if is_set(a, b, c) {
                    sets.push((a, b, c));
                }
            }
        }
    }

    if sets.is_empty() {
        None
    } else {
        let mut rng = thread_rng();
        let random_ix = rng.gen_range(0..sets.len());
        Some(sets[random_ix])
    }
}

////////////////////////////////////////////////////////////////////////////////
// Simulate
////////////////////////////////////////////////////////////////////////////////

fn simulate_game(counts: &mut Counts) {
    let mut deck = IndexDeck::new();
    let mut hand = deck.draw(INITIAL_DEAL);

    'game: loop {
        if let Some((a, b, c)) = find_random_set(&hand) {
            counts.sets[hand.len()] += 1;

            // remove the set
            hand.retain(|&x| x != a && x != b && x != c);

            if hand.len() < INITIAL_DEAL {
                // deal more cards to replace removed set
                hand.append(&mut deck.draw(SET_SIZE));
            }
        } else {
            counts.no_sets[hand.len()] += 1;

            if deck.is_empty() {
                // no sets and no stock remaining: game over
                counts.remainder[hand.len()] += 1;
                break 'game;
            } else {
                // deal more cards to increase odds of set
                hand.append(&mut deck.draw(SET_SIZE));
            }
        }
    }
}

fn run_simulations(num_games: u64, num_threads: u64) {
    let start_time = PreciseTime::now();
    let (tx, rx) = mpsc::channel();
    let (thread_chunk, rem) = (num_games / num_threads, num_games % num_threads);

    // initialize set lookup table
    build_lookup();

    // launch threads
    for ix in 0..num_threads {
        let tx = tx.clone();
        let num = thread_chunk + if ix == 0 { rem } else { 0 };

        thread::spawn(move || {
            let mut counts = Counts::zero();
            for _ in 0..num {
                simulate_game(&mut counts)
            }
            tx.send(counts).unwrap();
        });
    }

    // collate results
    let mut totals = Counts::zero();
    for _ in 0..num_threads {
        let counts = rx.recv().unwrap();
        totals.add(&counts);
    }

    // summary
    println!("{} seconds elapsed.\n", start_time.to(PreciseTime::now()));
    totals.print_hand_stats();
    println!();
    totals.print_end_game_stats();
}

////////////////////////////////////////////////////////////////////////////////
// main
////////////////////////////////////////////////////////////////////////////////

fn main() {
    let games_help = &format!(
        "Sets number of games to simulate (default: {})",
        pretty_print(NUM_GAMES)
    );

    let matches = clap_app!(simulate =>
        (version: VERSION)
        (about: "Gather statistics for simulated games of SET.")
        (@arg GAMES: -g --games +takes_value games_help)
        (@arg THREADS: -t --threads +takes_value "Sets number of threads")
    )
    .get_matches();

    let num_games = value_t!(matches, "GAMES", u64).unwrap_or(NUM_GAMES);
    let num_threads = value_t!(matches, "THREADS", u64).unwrap_or(num_cpus::get() as u64);

    println!(
        "Simulating {} games. This may take some time...",
        pretty_print(num_games)
    );
    run_simulations(num_games, num_threads);
}
