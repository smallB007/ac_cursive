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
use cursive::{align::HAlign, views::Button, Cursive};
use cursive::{direction::Orientation, views::CircularFocus};
use rand::Rng;

// Modules --------------------------------------------------------------------
// ----------------------------------------------------------------------------
use cursive_table_view::{TableView, TableViewItem};

mod definitions;
mod tui_fn;
mod utils;
use tui_fn::{
    create_classic_buttons::create_classic_buttons, create_classic_layout::create_classic_layout,
    create_menu::create_menubar, create_panel::create_panel,
    create_view_layout::create_view_layout,
};
fn main() {
    let mut siv = cursive::default();
    create_menubar(&mut siv);
    let classic_layout = create_classic_layout("/home", "/tmp");
    siv.add_fullscreen_layer(classic_layout);

    siv.run();
}

use cursive::event::{Event, Key};
use cursive::traits::*;
use cursive::views::{EditView, OnEventView, TextArea};

fn dmain() {
    let mut siv = cursive::default();

    // The main dialog will just have a textarea.
    // Its size expand automatically with the content.
    siv.add_fullscreen_layer(
        Dialog::new()
            .title("Describe your issue")
            .padding_lrtb(1, 1, 1, 0)
            .content(TextArea::new().with_name("text").min_height(30))
            .button("Ok", Cursive::quit),
    );

    // We'll add a find feature!
    siv.add_layer(Dialog::info("Hint: press Ctrl-F to find in text!"));

    siv.add_global_callback(Event::CtrlChar('f'), |s| {
        // When Ctrl-F is pressed, show the Find popup.
        // Pressing the Escape key will discard it.
        s.add_fullscreen_layer(
            OnEventView::new(
                Dialog::new()
                    .title("Find")
                    .content(
                        EditView::new()
                            .on_submit(find)
                            .with_name("edit")
                            .min_width(10),
                    )
                    .button("Ok", |s| {
                        let text = s
                            .call_on_name("edit", |view: &mut EditView| view.get_content())
                            .unwrap();
                        find(s, &text);
                    })
                    .dismiss_button("Cancel"),
            )
            .on_event(Event::Key(Key::Esc), |s| {
                s.pop_layer();
            }),
        )
    });

    siv.run();
}

fn find(siv: &mut Cursive, text: &str) {
    // First, remove the find popup
    siv.pop_layer();

    let res = siv.call_on_name("text", |v: &mut TextArea| {
        // Find the given text from the text area content
        // Possible improvement: search after the current cursor.
        if let Some(i) = v.get_content().find(text) {
            // If we found it, move the cursor
            v.set_cursor(i);
            Ok(())
        } else {
            // Otherwise, return an error so we can show a warning.
            Err(())
        }
    });

    if let Some(Err(())) = res {
        // If we didn't find anything, tell the user!
        siv.add_layer(Dialog::info(format!("`{}` not found", text)));
    }
}
