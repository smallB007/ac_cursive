use crate::tui_fn::create_classic_buttons::create_classic_buttons;
use crate::{
    tui_fn::{create_panel::create_panel, create_view_panel::create_view_panel},
    utils::common_utils::{
        get_active_table_first_selected_index, get_active_table_first_selected_item,
        get_active_table_name, select_index,
    },
};

use cursive::{direction::Orientation, view::Nameable, views::CircularFocus};
use cursive::{
    views::{
        Dialog, DummyView, HideableView, LinearLayout, NamedView, ResizedView, StackView, TextView,
    },
    Cursive,
};
pub fn create_peek_layout(dir: &str, path: &str) -> LinearLayout {
    let mut layout_panes = LinearLayout::new(Orientation::Horizontal).with_name("PeekLinearLayout");

    layout_panes
        .get_mut()
        .add_child(create_panel("PeekPanelDir", dir, Some(peek_cb)));
    layout_panes
        .get_mut()
        .add_child(create_view_panel("PeekPanelPath", path, path));

    let layout_circular_panes = CircularFocus::new(layout_panes);
    let layout_circular_panes =
        layout_circular_panes.wrap_tab().wrap_up_down(/*won't go to the function keys */);

    let buttons = create_classic_buttons();
    let layout = LinearLayout::vertical()
        .child(layout_circular_panes)
        .child(buttons);

    layout
}

fn peek_cb(s: &mut Cursive, row: usize, col: usize) {
    //eprintln!("peek_cb: row: {}, col: {}", row, col);
    let selected_path = get_active_table_first_selected_item(s, "PeekPanelDir_tableview");
    //eprintln!("selected_path:{}", selected_path);
    s.call_on_name(
        "PeekLinearLayout",
        |layout: &mut NamedView<LinearLayout>| {
            layout.get_mut().remove_child(1);
            layout.get_mut().add_child(create_view_panel(
                "PeekPanelPath", //++artie rfctr
                &selected_path,
                &selected_path,
            ))
        },
    );
}
