use crate::cursive::view::{Nameable, Resizable};
use crate::tui_fn::create_table::create_table;
use cursive::views::{Dialog, HideableView, NamedView, ResizedView};
pub fn create_panel(name: &str, dir: &str) -> ResizedView<NamedView<Dialog>> {
    let named_v: ResizedView<NamedView<Dialog>> =
        Dialog::around(create_table(dir).with_name(String::from(name) + "_tableview"))
            .title(dir)
            .with_name(name)
            .full_screen();

    named_v
}
