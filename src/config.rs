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

use serde_yaml;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::{env, error, fmt, result};

use core::graphics::ColorScheme;
use rules::{self, Rules};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Variant { Set, SuperSet }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Deck { Simplified, Full }

////////////////////////////////////////////////////////////////////////////////
// Config
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Game Variant: Set vs SuperSet
    pub variant: Variant,
    /// Deck type: Beginner vs Full
    pub deck: Deck,
    /// Layout neatly or sloppily
    pub tidy_layout: bool,
    /// Classic vs CMYK
    pub color_scheme: ColorScheme,
    /// Store last used window size
    pub window_size: (i32, i32)
}

impl Config {
    pub fn new() -> Config {
        Config {
            variant: Variant::Set,
            deck: Deck::Full,
            tidy_layout: false,
            color_scheme: ColorScheme::CMYK,
            window_size: (1200, 700)
        }
    }

    pub fn rules(&self) -> Box<Rules> {
        match self.variant {
            Variant::Set => Box::new(rules::Set),
            Variant::SuperSet => Box::new(rules::SuperSet)
        }
    }

    pub fn config_path() -> ConfigResult<PathBuf> {
        let home_dir = env::var("HOME")?;
        let path = PathBuf::from(&home_dir).join(".config/marmoset/");

        if !path.exists() {
            // make sure parent directories exist
            fs::create_dir_all(&path)?;
        }

        Ok(path.join("marmoset.yml"))
    }

    pub fn load() -> Config {
        let mut serialized = String::new();

        Config::config_path()
            .and_then(|path| File::open(&path)
                      .map_err(ConfigError::Io))
            .and_then(|mut file| file.read_to_string(&mut serialized)
                      .map_err(ConfigError::Io))
            .and_then(|_| serde_yaml::from_str(&serialized)
                      .map_err(ConfigError::Yaml))
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let serialized = serde_yaml::to_string(&self).unwrap();

        Config::config_path()
            .and_then(|path| File::create(&path)
                      .map_err(ConfigError::Io))
            .and_then(|mut file| file.write_all(serialized.as_bytes())
                      .map_err(ConfigError::Io))
            .unwrap_or_else(|err| {
                println!("Could not save app settings.");
                println!("{}", err);
            });
    }
}

////////////////////////////////////////////////////////////////////////////////
// Config: Setters
////////////////////////////////////////////////////////////////////////////////

/// Create setter methods that automatically save the config.
macro_rules! make_setter {
    ($name:ident, $field:ident: $t:ty) => {
        pub fn $name(&mut self, $field: $t) {
            self.$field = $field;
            self.save();
        }
    }
}

impl Config {
    make_setter!(set_variant, variant: Variant);
    make_setter!(set_deck, deck: Deck);
    make_setter!(set_tidy_layout, tidy_layout: bool);
    make_setter!(set_color_scheme, color_scheme: ColorScheme);
    make_setter!(set_window_size, window_size: (i32, i32));
}

////////////////////////////////////////////////////////////////////////////////
// Config: Default
////////////////////////////////////////////////////////////////////////////////

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

////////////////////////////////////////////////////////////////////////////////
// ConfigError
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ConfigError {
    NoHomeDir(env::VarError),
    Io(io::Error),
    Yaml(serde_yaml::Error),
}

pub type ConfigResult<T> = result::Result<T, ConfigError>;

////////////////////////////////////////////////////////////////////////////////
// ConfigError: Display
////////////////////////////////////////////////////////////////////////////////

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::NoHomeDir(ref err) =>
                write!(f, "Could not find $HOME: {}", err),
            ConfigError::Io(ref err) =>
                write!(f, "Config IO error: {}", err),
            ConfigError::Yaml(ref err) =>
                write!(f, "Config parse error: {:?}", err),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// ConfigError: Error
////////////////////////////////////////////////////////////////////////////////

impl error::Error for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::NoHomeDir(ref err) => err.description(),
            ConfigError::Io(ref err) => err.description(),
            ConfigError::Yaml(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ConfigError::NoHomeDir(ref err) => Some(err),
            ConfigError::Io(ref err) => Some(err),
            ConfigError::Yaml(ref err) => Some(err),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// ConfigError: From<T>
////////////////////////////////////////////////////////////////////////////////

impl From<env::VarError> for ConfigError {
    fn from(err: env::VarError) -> ConfigError {
        ConfigError::NoHomeDir(err)
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::Io(err)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> ConfigError {
        ConfigError::Yaml(err)
    }
}
