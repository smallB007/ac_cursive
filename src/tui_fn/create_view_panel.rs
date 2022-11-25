use crate::cursive::view::{Nameable, Resizable};
use crate::tui_fn::create_view_table::create_view_table;
use cursive::views::{Dialog, HideableView, NamedView, ResizedView};
pub fn create_view_panel(name: &str, title: &str, path: &str) -> ResizedView<NamedView<Dialog>> {
    let named_v_left: ResizedView<NamedView<Dialog>> = Dialog::around(create_view_table(path))
        .title(title)
        .with_name(name)
        .full_screen();

    named_v_left
}
