use crate::cursive::view::Resizable;
use crate::{
    cursive::view::Nameable,
    utils::common_utils::{get_active_table_first_selected_item, get_active_table_name},
};
use cursive::theme::ColorStyle;
use cursive::views::{
    Button, Dialog, DummyView, HideableView, LinearLayout, NamedView, ResizedView, StackView,
    TextView,
};
use cursive::{direction::Orientation, views::CircularFocus};
use cursive_table_view::TableView;

use crate::tui_fn::{
    create_table::{create_table, BasicColumn, DirView},
    create_view_layout::create_view_layout,
};

pub fn create_classic_buttons() -> ResizedView<StackView> {
    let help_tuple = (
        TextView::new("F1").style(ColorStyle::title_primary()),
        Button::new_raw("[ Info ]", |s| {}),
    );
    let help_layout = LinearLayout::horizontal()
        .child(TextView::new("F1").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Info ]", |s| {}));
    let menu_layout = LinearLayout::horizontal()
        .child(TextView::new("F2").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Popup ]", |s| {}));
    let view_layout = LinearLayout::horizontal()
        .child(TextView::new("F3").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ View/Edit ]", |s| {
            let active_table_name = get_active_table_name(s);
            let selected_item = get_active_table_first_selected_item(s, &active_table_name);
            let view_layout = create_view_layout();
            s.add_fullscreen_layer(view_layout);
        }));
    let edit_layout = LinearLayout::horizontal()
        .child(TextView::new("F4").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Peek ]", |s| {}));
    let copy_layout = LinearLayout::horizontal()
        .child(TextView::new("F5").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Copy ]", |s| {}));
    let rn_mv_layout = LinearLayout::horizontal()
        .child(TextView::new("F6").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Rnm/Mv ]", |s| {}));
    let mkdir_layout = LinearLayout::horizontal()
        .child(TextView::new("F8").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ MkDir ]", |s| {}));
    let pulldown_layout = LinearLayout::horizontal()
        .child(TextView::new("F9").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Find ]", move |s| {
            //s.call_on_name(
            //    "left_panel_hideable",
            //    |ob: &mut NamedView<ResizedView<HideableView<NamedView<Dialog>>>>| {
            //        ob.get_mut().get_inner_mut().hide();
            //    },
            //);
            //let mut layout_panes = LinearLayout::new(Orientation::Horizontal);
            //let named_v_right: NamedView<Dialog> = Dialog::around(create_table())
            //    .title("Left")
            //    .with_name("left_dialog");
            //let hide_v_right: HideableView<NamedView<Dialog>> = HideableView::new(named_v_right);
            //let hide_v_right_full_screed: NamedView<ResizedView<HideableView<NamedView<Dialog>>>> =
            //    hide_v_right.full_screen().with_name("right_panel_hideable");
            //layout_panes.add_child(hide_v_right_full_screed);
            //s.add_fullscreen_layer(layout_panes);
        }));

    let quit_layout = LinearLayout::horizontal()
        .child(TextView::new("F10").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Quit ]", |s| s.quit()));

    let classic_buttons = LinearLayout::horizontal()
        .child(help_layout)
        .child(DummyView.full_width())
        .child(menu_layout)
        .child(DummyView.full_width())
        .child(view_layout)
        .child(DummyView.full_width())
        .child(edit_layout)
        .child(DummyView.full_width())
        .child(copy_layout)
        .child(DummyView.full_width())
        .child(rn_mv_layout)
        .child(DummyView.full_width())
        .child(mkdir_layout)
        .child(DummyView.full_width())
        .child(pulldown_layout)
        .child(DummyView.full_width())
        .child(quit_layout);

    let mut stack_buttons = StackView::new().fixed_height(1);
    stack_buttons
        .get_inner_mut()
        .add_fullscreen_layer(classic_buttons.with_name("classic_buttons"));

    stack_buttons
}
