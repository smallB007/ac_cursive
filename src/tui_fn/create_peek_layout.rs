use crate::tui_fn::create_classic_buttons::create_classic_buttons;
use crate::tui_fn::{create_panel::create_panel, create_view_panel::create_view_panel};
use cursive::views::{
    Dialog, DummyView, HideableView, LinearLayout, NamedView, ResizedView, StackView, TextView,
};
use cursive::{direction::Orientation, views::CircularFocus};
pub fn create_peek_layout(dir: &str, path: &str) -> LinearLayout {
    let mut layout_panes = LinearLayout::new(Orientation::Horizontal);

    layout_panes.add_child(create_panel("PeekPanelDir", dir));
    layout_panes.add_child(create_view_panel("PeekPanelPath", path, path));

    let layout_circular_panes = CircularFocus::new(layout_panes);
    let layout_circular_panes =
        layout_circular_panes.wrap_tab().wrap_up_down(/*won't go to the function keys */);

    let buttons = create_classic_buttons();
    let layout = LinearLayout::vertical()
        .child(layout_circular_panes)
        .child(buttons);

    layout
}
