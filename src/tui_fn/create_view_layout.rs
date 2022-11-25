use crate::tui_fn::{
    create_classic_buttons::create_classic_buttons, create_panel::create_panel,
    create_view_buttons::create_view_buttons, create_view_panel::create_view_panel,
};

use cursive::views::{
    Dialog, DummyView, HideableView, LinearLayout, NamedView, ResizedView, StackView, TextView,
};
use cursive::{direction::Orientation, views::CircularFocus};
pub fn create_view_layout(path: &str) -> LinearLayout {
    let mut layout_panes = LinearLayout::new(Orientation::Horizontal);

    layout_panes.add_child(create_view_panel("Left", "LeftDialog", path));

    let buttons = create_view_buttons();
    let layout = LinearLayout::vertical().child(layout_panes).child(buttons);

    layout
}
