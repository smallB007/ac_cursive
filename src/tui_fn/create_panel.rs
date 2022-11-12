use std::path::PathBuf;

use crate::cursive::view::{Nameable, Resizable};
use crate::tui_fn::create_table::{create_table, prepare_items_for_table_view};
use cursive::views::{Dialog, HideableView, NamedView, ResizedView};
use cursive_table_view::TableView;

use super::create_table::{BasicColumn, DirView};

fn create_name_for_table_view(name: &str) -> String {
    String::from(String::from(name) + "_tableview")
}
pub fn create_panel(name: &str, dir: &str) -> ResizedView<NamedView<Dialog>> {
    let table_view_name = create_name_for_table_view(name);
    let dialog_title = String::from(name);
    let table_view = create_table(dir); //.with_name(String::from(name) + "_tableview");
    let table_view = table_view.on_submit(move |s, r, index| {
        /*First get the dialog's title which is first path of dir */
        let path = s
            .call_on_name(&dialog_title, |s: &mut Dialog| {
                let title = String::from(s.get_title());
                title
            })
            .unwrap();
        /*Second get the selected item */
        let selected_item = s
            .call_on_name(
                &table_view_name,
                |table: &mut TableView<DirView, BasicColumn>| {
                    //table.remove_item(index);
                    table.get_selected_item().name.clone()
                },
            )
            .unwrap();
        /*Third, combine them to form full path */

        let full_path = PathBuf::from(path.clone() + &selected_item);
        if full_path.is_dir() {
            let new_dialog_title = String::from(path + &selected_item);
            s.call_on_name(
                &table_view_name,
                |table: &mut TableView<DirView, BasicColumn>| {
                    let items = prepare_items_for_table_view(&new_dialog_title);
                    //table.remove_item(index);
                    table.set_items(items);
                },
            );
            s.call_on_name(&dialog_title, |s: &mut Dialog| {
                s.set_title(new_dialog_title);
            });
        }
    });
    let table_view = table_view.with_name(create_name_for_table_view(name));
    let named_v: ResizedView<NamedView<Dialog>> = Dialog::around(table_view)
        .title(dir)
        .with_name(name)
        .full_screen();

    named_v
}
