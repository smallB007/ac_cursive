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
use cursive::theme::ColorStyle;
use cursive::traits::*;
use cursive::views::{
    Dialog, DummyView, HideableView, LinearLayout, NamedView, ResizedView, StackView, TextView,
};
use cursive::{align::HAlign, views::Button};
use cursive::{direction::Orientation, views::CircularFocus};
use rand::Rng;

// Modules --------------------------------------------------------------------
// ----------------------------------------------------------------------------
use cursive_table_view::{TableView, TableViewItem};

mod tui_fn;
use tui_fn::{
    create_classic_buttons::create_classic_buttons, create_classic_layout::create_classic_layout,
    create_menu::create_menubar, create_panel::create_panel,
    create_view_layout::create_view_layout,
};
fn main() {
    let mut siv = cursive::default();
    create_menubar(&mut siv);
    let classic_layout = create_classic_layout();
    siv.add_fullscreen_layer(classic_layout);

    siv.run();
}
