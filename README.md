# Marmoset &emsp; [![License: GPLv3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Marmoset is a single-player implementation of the card game [SET]&reg;. It uses [GTK+ 3] and is written in [Rust].

In addition to the classic [SET]&reg; rules, Marmoset provides another game variant called [SuperSet] that uses the same deck. Other features include a beginner's deck (solid cards only), undo and redo, hints, and a color palette intended to be playable by people with color vision deficiencies.

See [Quickstart] for gameplay rules.

![Marmoset Gameplay](/images/gameplay.png)

## Building

Check out the code and run `cargo build --release`.

### Desktop File and Icon

To install the desktop entry for Marmoset on Linux, edit `Marmoset.desktop` so that the `Icon=~/marmoset/icon.svg` entry reflects the actual path to `icon.svg` on your system, then:

`cp Marmoset.desktop ~/.local/share/applications`

Make sure the executable `target/release/marmoset` is somewhere in your `$PATH`.

## Examples

In addition to the Marmoset app, there are three command line programs that use the same underlying library. These were written to answer questions about Set and SuperSet gameplay, and to generate card images for documentation.

### count

The `count` program generates all possible deals to determine the smallest hand that is guaranteed to contain a SuperSet. The conclusion is that any deal of 10 cards will contain at least one SuperSet. This result informed the layout and behavior of the SuperSet variant in Marmoset.

Run `count` with `cargo run --release --example count -- [OPTIONS]`.

```
USAGE:
	count [OPTIONS]

FLAGS:
	-h, --help       Prints help information
	-V, --version    Prints version information

OPTIONS:
	-t, --threads <THREADS>    Sets number of threads
```

### simulate

The `simulate` program simulates games using classic [SET]&reg; rules. It tallies the number of hands that contain no Sets and the number of cards remaining when the game ends. This program was used to determine how often a game was likely to get wedged with 18 cards in play. Since this turned out to be exceedingly rare (approximately once every 1.6 million games), Marmoset always guarantees that an 18 card deal contains at least 1 Set. This means we only need 18 card positions on the tableau rather than 21.

Run `simulate` with `cargo run --release --example simulate -- [OPTIONS]`.

```
USAGE:
	simulate [OPTIONS]

FLAGS:
	-h, --help       Prints help information
	-V, --version    Prints version information

OPTIONS:
	-g, --games <GAMES>        Sets number of games to simulate (default: 1_000_000)
	-t, --threads <THREADS>    Sets number of threads
```

### genpng

The `genpng` program generates a PNG image for each card in the Marmoset deck.

Run `genpng` with `cargo run --release --example genpng -- [FLAGS] [OPTIONS] <DIRECTORY>`.

```
USAGE:
	genpng [FLAGS] [OPTIONS] <DIRECTORY>

FLAGS:
	-c, --classic-colors       Uses classic SET colors
	-v, --render-vertically    Orients cards vertically
	-h, --help                 Prints help information
	-V, --version              Prints version information

OPTIONS:
	-b, --border <BORDER>    Sets the border width in pixels
	-w, --width <WIDTH>      Sets the card width in pixels

ARGS:
	<DIRECTORY>    Sets the directory in which to place the images
```

## License

Marmoset is released under the [GNU General Public License v3].

[SET]: http://setgame.com/set
[GNU General Public License v3]: https://www.gnu.org/licenses/gpl-3.0.en.html
[GTK+ 3]: http://www.gtk.org/
[Rust]: https://www.rust-lang.org
[SuperSet]: http://magliery.com/Set/SuperSet.html
[QUICKSTART]: QUICKSTART.md
