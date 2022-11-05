use crate::cursive::view::{Nameable, Resizable};
use crate::tui_fn::create_table::create_table;
use cursive::views::{Dialog, HideableView, NamedView, ResizedView};
pub fn create_panel(name: &str, title: &str) -> ResizedView<NamedView<Dialog>> {
    let named_v_left: ResizedView<NamedView<Dialog>> = Dialog::around(create_table())
        .title(title)
        .with_name(name)
        .full_screen();

    named_v_left
}
