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
    create_classic_buttons::create_classic_buttons, create_menu, create_panel::create_panel,
};
fn main() {
    let mut siv = cursive::default();
    create_menu::create_menubar(&mut siv);
    let mut layout_panes = LinearLayout::new(Orientation::Horizontal);

    layout_panes.add_child(create_panel("Left", "LeftDialog"));
    layout_panes.add_child(create_panel("Right", "RightDialog"));

    let layout_circular_panes = CircularFocus::new(layout_panes);
    let layout_circular_panes = layout_circular_panes.wrap_tab();

    //  let aligned_center = AlignedView::with_center(stack_buttons);
    let classic_buttons = create_classic_buttons();
    let main_view = LinearLayout::vertical()
        .child(layout_circular_panes)
        .child(classic_buttons);

    siv.add_fullscreen_layer(main_view);

    siv.run();
}
