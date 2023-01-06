// Crate Dependencies ---------------------------------------------------------
// ----------------------------------------------------------------------------
extern crate cursive;
extern crate cursive_table_view;
extern crate rand;

// STD Dependencies -----------------------------------------------------------
// ----------------------------------------------------------------------------
use std::{cmp::Ordering, rc::Rc};

// External Dependencies ------------------------------------------------------
// ----------------------------------------------------------------------------
use cursive::traits::*;
use cursive::views::{
    Dialog, DummyView, HideableView, LinearLayout, NamedView, ResizedView, StackView, TextView,
};
use cursive::{align::HAlign, views::Button, Cursive};
use cursive::{direction::Orientation, views::CircularFocus};
use cursive::{theme::ColorStyle, CursiveRunnable};
use rand::Rng;

// Modules --------------------------------------------------------------------
// ----------------------------------------------------------------------------
use cursive_table_view::{TableView, TableViewItem};

mod custom_views;
mod definitions;
mod tui_fn;
mod utils;
use cursive::event::{Event, Key};
use cursive::traits::*;
use cursive::views::{EditView, OnEventView, TextArea};
use tui_fn::{
    create_classic_buttons::create_classic_buttons, create_classic_layout::create_classic_layout,
    create_menu::create_menubar, create_panel::create_panel,
    create_view_layout::create_view_layout,
};
use utils::cp_machinery::cp_utils::f5_handler;
use utils::cp_machinery::cp_utils::{alt_f1_handler, quick_cd_handler};
fn main() {
    let mut siv = cursive::default();
    init_callbacks(&mut siv);
    create_menubar(&mut siv);

    let classic_layout =
        create_classic_layout("/home/artie/Desktop/Coumbo", "/tmp", siv.cb_sink().clone());
    siv.add_fullscreen_layer(classic_layout);

    siv.run();
}

fn init_callbacks(siv: &mut CursiveRunnable) {
    init_f5_handler(siv);
    init_alt_f1_handler(siv);
    init_quick_cd_handlers(siv);
}

fn init_alt_f1_handler(siv: &mut Cursive) {
    siv.add_global_callback(cursive::event::Event::Alt(Key::F1), |s| {
        alt_f1_handler(s);
    });
}
fn init_f5_handler(siv: &mut Cursive) {
    siv.add_global_callback(cursive::event::Event::Key(Key::F5), |s| {
        f5_handler(s);
    });
}

fn init_quick_cd_handlers(siv: &mut Cursive) {
    for c in '0'..='9' {
        siv.add_global_callback(cursive::event::Event::Char(c), move |s| {
            quick_cd_handler(c, s);
        });
    }
}
