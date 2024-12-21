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
extern crate core;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate num_traits;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

pub mod cell;
pub mod config;
pub mod controller;
pub mod game_state;
pub mod rules;

use gdk::prelude::*;
use gdk::ModifierType;
use gdk_pixbuf::{Pixbuf, PixbufLoader};
use glib::{clone, Error};
use gtk::prelude::*;
use gtk::{AccelGroup, Application, ApplicationWindow, MenuItem};
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{Config, Deck, Variant};
use crate::controller::Controller;
use core::graphics::ColorScheme::{Classic, CMYK};

/// A convenience type for passing data to menu building functions
type MenuData<'a> = (
    &'a ApplicationWindow,
    &'a AccelGroup,
    &'a Rc<RefCell<Controller>>,
);

fn main() {
    let app = Application::new(Some("org.nybble.marmoset"), Default::default());

    app.connect_activate(|app| init(app));
    app.run();
}

fn init(app: &Application) {
    // load app configuration
    let config = Config::load();

    // create controller and drawing area
    let controller = Controller::shared_with_config(config);
    let drawing_area = controller.borrow().get_drawing_area();
    // create window
    let window = build_window(app, &controller);

    // create menu bar
    let menu_bar = gtk::MenuBar::new();
    let accel_group = AccelGroup::new();
    let menu_data = (&window, &accel_group, &controller);

    window.add_accel_group(&accel_group);
    menu_bar.append(&build_game_menu(menu_data));
    menu_bar.append(&build_control_menu(menu_data));
    menu_bar.append(&build_help_menu(&window));

    // add the widgets to the window
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    v_box.pack_start(&menu_bar, false, false, 0);
    v_box.pack_start(&drawing_area, true, true, 0);
    window.add(&v_box);

    window.show_all();
}

fn build_window(app: &Application, controller: &Rc<RefCell<Controller>>) -> ApplicationWindow {
    let config = controller.borrow().config;
    let window = ApplicationWindow::new(app);
    let (width, height) = config.window_size;

    window.set_title(config.rules().name());
    window.set_default_size(width, height);

    // quit if the window is closed
    window.connect_delete_event(
        clone!(@strong controller, @weak window => @default-return Inhibit(false), move |_, _| {
            // save the current window size in the config
            let config = &mut controller.borrow_mut().config;
            config.set_window_size(window.size());
            Inhibit(false)
        }),
    );

    window
}

////////////////////////////////////////////////////////////////////////////////
// Menu Helpers
////////////////////////////////////////////////////////////////////////////////

macro_rules! build_menu {
    ($menu:expr, [$( $e:expr ),*]) => {{
	let menu = MenuItem::with_mnemonic($menu);
	let submenu = gtk::Menu::new();
	$( submenu.append(&$e); )*
	menu.set_submenu(Some(&submenu));
	menu
    }}
}

fn make_menu_item(
    mnemonic: &str,
    accel_group: &AccelGroup,
    modifier: ModifierType,
    keys: &[char],
) -> MenuItem {
    let item = MenuItem::with_mnemonic(mnemonic);
    for &key in keys.iter() {
        item.add_accelerator(
            "activate",
            accel_group,
            key as u32,
            modifier,
            gtk::AccelFlags::VISIBLE,
        );
    }
    item
}

////////////////////////////////////////////////////////////////////////////////
// Game Menu
////////////////////////////////////////////////////////////////////////////////

fn build_game_menu(menu_data: MenuData) -> MenuItem {
    let (window, accel_group, controller) = menu_data;

    // create menu items
    let new_game = make_menu_item("_New Game", accel_group, ModifierType::CONTROL_MASK, &['N']);
    let restart = MenuItem::with_mnemonic("_Restart Game");
    let close = make_menu_item("_Close", accel_group, ModifierType::CONTROL_MASK, &['W']);

    new_game.connect_activate(
        clone!(@strong controller => move |_| controller.borrow_mut().new_game()),
    );

    restart
        .connect_activate(clone!(@strong controller => move |_| controller.borrow_mut().restart()));

    close.connect_activate(clone!(@weak window => move |_| window.close()));

    // disable restart menu by default
    restart.set_sensitive(false);
    // update restart status based on undo stack changes
    controller.borrow_mut().add_undo_observer(
        clone!(@weak restart => move |controller| restart.set_sensitive(controller.can_undo())),
    );

    build_menu!(
        "_Game",
        [
            new_game,
            restart,
            gtk::SeparatorMenuItem::new(),
            build_variant_submenu(menu_data),
            build_deck_submenu(menu_data),
            gtk::SeparatorMenuItem::new(),
            close
        ]
    )
}

////////////////////////////////////////////////////////////////////////////////
// Variant Submenu
////////////////////////////////////////////////////////////////////////////////

fn build_variant_submenu(menu_data: MenuData) -> MenuItem {
    let (window, _accel_group, controller) = menu_data;

    // create menu items
    let set_variant = gtk::RadioMenuItem::with_mnemonic("_Set");
    let superset_variant = gtk::RadioMenuItem::with_mnemonic("S_uperSet");
    superset_variant.join_group(Some(&set_variant));

    // reflect config settings
    match controller.borrow().config.variant {
        Variant::Set => set_variant.set_active(true),
        Variant::SuperSet => superset_variant.set_active(true),
    }

    set_variant.connect_toggled(clone!(@strong controller, @weak window => move |_| {
        controller.borrow_mut().set_variant(Variant::Set);
        window.set_title("Set");
    }));

    superset_variant.connect_toggled(clone!(@strong controller, @weak window => move |_| {
        controller.borrow_mut().set_variant(Variant::SuperSet);
        window.set_title("SuperSet");
    }));

    build_menu!("_Variant", [set_variant, superset_variant])
}

////////////////////////////////////////////////////////////////////////////////
// Deck Submenu
////////////////////////////////////////////////////////////////////////////////

fn build_deck_submenu(menu_data: MenuData) -> MenuItem {
    let (_window, _accel_group, controller) = menu_data;

    // create menu items
    let beginner_deck = gtk::RadioMenuItem::with_mnemonic("_Beginner");
    let full_deck = gtk::RadioMenuItem::with_mnemonic("_Full");
    full_deck.join_group(Some(&beginner_deck));

    // reflect config settings
    match controller.borrow().config.deck {
        Deck::Simplified => beginner_deck.set_active(true),
        Deck::Full => full_deck.set_active(true),
    }

    beginner_deck.connect_toggled(clone!(@strong controller => move |_|
	       controller.borrow_mut().set_deck(Deck::Simplified)));

    full_deck.connect_toggled(clone!(@strong controller => move |_|
	       controller.borrow_mut().set_deck(Deck::Full)));

    build_menu!("_Deck", [beginner_deck, full_deck])
}

////////////////////////////////////////////////////////////////////////////////
// Undo Menu Items
////////////////////////////////////////////////////////////////////////////////

fn connect_undo_redo(controller: &Rc<RefCell<Controller>>, undo: &MenuItem, redo: &MenuItem) {
    undo.connect_activate(clone!(@strong controller => move |_| controller.borrow_mut().undo()));

    redo.connect_activate(clone!(@strong controller => move |_| controller.borrow_mut().redo()));

    // undo and redo are disabled by default
    undo.set_sensitive(false);
    redo.set_sensitive(false);

    // update undo/redo status based on undo stack changes
    controller
        .borrow_mut()
        .add_undo_observer(clone!(@weak undo, @weak redo => move |controller| {
            undo.set_sensitive(controller.can_undo());
            redo.set_sensitive(controller.can_redo());

            if let Some(action) = controller.undo_action_name() {
                undo.set_label(&format!("_Undo {}", action));
            } else {
                undo.set_label("_Undo");
            }

            if let Some(action) = controller.redo_action_name() {
                redo.set_label(&format!("_Redo {}", action));
            } else {
                redo.set_label("_Redo");
            }
        }));
}

////////////////////////////////////////////////////////////////////////////////
// Control Menu
////////////////////////////////////////////////////////////////////////////////

fn build_control_menu(menu_data: MenuData) -> MenuItem {
    let (window, accel_group, controller) = menu_data;
    let ctrl_shift = ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK;
    let no_modifier = ModifierType::empty();
    let config = controller.borrow().config;

    // create menu items
    let undo = make_menu_item("_Undo", accel_group, ModifierType::CONTROL_MASK, &['Z']);
    let redo = make_menu_item("_Redo", accel_group, ctrl_shift, &['Z']);
    let hint = make_menu_item("_Hint", accel_group, no_modifier, &['?', '/']);
    let deal_more = make_menu_item("_Deal More Cards", accel_group, no_modifier, &['+', '=']);
    let tidy_layout = gtk::CheckMenuItem::with_mnemonic("_Tidy Layout");
    let classic_colors = gtk::CheckMenuItem::with_mnemonic("_Classic Colors");

    // reflect config settings
    tidy_layout.set_active(config.tidy_layout);
    classic_colors.set_active(config.color_scheme == Classic);

    // undo and redo require a bit more setup than other menu items
    connect_undo_redo(controller, &undo, &redo);

    hint.connect_activate(clone!(@strong controller, @weak window => move |_| {
        let message = controller.borrow_mut().show_hint();
        show_message_dialog(message, &window);
    }));

    deal_more.connect_activate(clone!(@strong controller, @weak window => move |_| {
        let message = controller.borrow_mut().deal_more_cards();
        show_message_dialog(message, &window);
    }));

    tidy_layout.connect_toggled(clone!(@strong controller => move |w|
	       controller.borrow_mut().set_tidy_layout(w.is_active())));

    classic_colors.connect_toggled(clone!(@strong controller => move |w|  {
        let scheme = if w.is_active() { Classic } else { CMYK };
        controller.borrow_mut().set_color_scheme(scheme);
    }));

    build_menu!(
        "_Control",
        [
            undo,
            redo,
            gtk::SeparatorMenuItem::new(),
            hint,
            deal_more,
            gtk::SeparatorMenuItem::new(),
            tidy_layout,
            classic_colors
        ]
    )
}

////////////////////////////////////////////////////////////////////////////////
// Help Menu
////////////////////////////////////////////////////////////////////////////////

static VERSION: &str = env!("CARGO_PKG_VERSION");
static LICENSE: &str = include_str!("../resources/short_license.txt");
static COMMENT: &str = include_str!("../resources/description.txt");

fn logo_loader() -> Result<PixbufLoader, Error> {
    let data = include_bytes!("../resources/logo.png");

    let loader = PixbufLoader::with_type("png")?;
    loader.write(data)?;
    loader.close()?;

    Ok(loader)
}

fn logo() -> Option<Pixbuf> {
    match logo_loader() {
        Ok(loader) => loader.pixbuf(),
        Err(_) => None,
    }
}

fn build_help_menu(window: &ApplicationWindow) -> MenuItem {
    let about = MenuItem::with_mnemonic("_About");
    about.connect_activate(clone!(@weak window => move |_| {
        let a = gtk::AboutDialog::new();
        a.set_program_name("Marmoset");
        a.set_logo(logo().as_ref());
        a.set_comments(Some(COMMENT));
        a.set_copyright(Some("Copyright © 2017-2020 Steve Sprang"));
        a.set_license_type(gtk::License::Gpl30);
        a.set_license(Some(LICENSE));
        a.set_website(Some("https://github.com/sprang/marmoset"));
        a.set_website_label(Some("Marmoset Website"));
        a.set_version(Some(VERSION));
        a.set_transient_for(Some(&window));
        a.run();
        unsafe {
        a.destroy();
        }
    }));

    build_menu!("_Help", [about])
}

////////////////////////////////////////////////////////////////////////////////
// Game Messages
////////////////////////////////////////////////////////////////////////////////

fn show_message_dialog(message: Option<String>, window: &ApplicationWindow) {
    if let Some(string) = message {
        let md = gtk::MessageDialog::new(
            Some(window),
            gtk::DialogFlags::empty(),
            gtk::MessageType::Info,
            gtk::ButtonsType::Ok,
            &string,
        );
        md.set_markup(&format!("<big>{}</big>", string));
        md.run();
        unsafe {
            md.destroy();
        }
    }
}
