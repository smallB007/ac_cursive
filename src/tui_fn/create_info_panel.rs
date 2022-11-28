use crate::cursive::view::{Nameable, Resizable};
use crate::tui_fn::create_info_table::create_info_table;
use cursive::views::{Dialog, HideableView, NamedView, ResizedView};
pub fn create_info_panel(name: &str, title: &str, path: &str) -> ResizedView<NamedView<Dialog>> {
    let named_v_left: ResizedView<NamedView<Dialog>> = Dialog::around(create_info_table(path))
        .title(title)
        .with_name(name)
        .full_screen();

    named_v_left
}
