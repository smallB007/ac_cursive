use std::path::PathBuf;

use crate::cursive::view::{Nameable, Resizable};
use crate::tui_fn::create_table::{create_table, prepare_items_for_table_view};
use cursive::{
    views::{Dialog, HideableView, NamedView, ResizedView},
    Cursive,
};
use cursive_table_view::TableView;

use super::create_table::{BasicColumn, DirView};

fn create_name_for_table_view(name: &str) -> String {
    String::from(String::from(name) + "_tableview")
}
fn get_current_path_from_dialog_name(s: &mut Cursive, dialog_name: String) -> String {
    /*First get the dialog's title which is first path of dir */
    let current_path = s
        .call_on_name(&dialog_name, |s: &mut Dialog| {
            let title = String::from(s.get_title());
            title
        })
        .unwrap();

    current_path
}
fn traverse_up(s: &mut Cursive, dialog_name: String, table_view_name: String) {
    /*Third, combine them to form full path */
    let current_path = get_current_path_from_dialog_name(s, dialog_name.clone());
    let mut full_path = PathBuf::from(current_path);
    full_path.pop();
    if full_path.is_dir() {
        let new_dialog_title = full_path.into_os_string().into_string().unwrap();
        s.call_on_name(
            &table_view_name,
            |table: &mut TableView<DirView, BasicColumn>| {
                let items = prepare_items_for_table_view(&new_dialog_title);
                table.set_items(items);
            },
        );
        s.call_on_name(&dialog_name, |s: &mut Dialog| {
            s.set_title(new_dialog_title);
        });
    }
}
fn traverse_down(
    s: &mut Cursive,
    dialog_name: String,
    table_view_name: String,
    selected_item: String,
) {
    /*Third, combine them to form full path */
    let current_path = get_current_path_from_dialog_name(s, dialog_name.clone());
    let full_path = PathBuf::from(current_path.clone() + &selected_item);
    if full_path.is_dir() {
        let new_dialog_title = String::from(current_path + &selected_item);
        s.call_on_name(
            &table_view_name,
            |table: &mut TableView<DirView, BasicColumn>| {
                let items = prepare_items_for_table_view(&new_dialog_title);
                //table.remove_item(index);
                table.set_items(items);
            },
        );
        s.call_on_name(&dialog_name, |s: &mut Dialog| {
            s.set_title(new_dialog_title);
        });
    }
}
pub fn create_panel(name: &str, dir: &str) -> ResizedView<NamedView<Dialog>> {
    let table_view_name = create_name_for_table_view(name);
    let dialog_name = String::from(name);
    let table_view = create_table(dir); //.with_name(String::from(name) + "_tableview");
    let table_view = table_view.on_submit(move |s, r, index| {
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
        if selected_item == ".." {
            traverse_up(s, dialog_name.clone(), table_view_name.clone());
        } else {
            traverse_down(
                s,
                dialog_name.clone(),
                table_view_name.clone(),
                selected_item.clone(),
            );
        }
    });
    let table_view = table_view.with_name(create_name_for_table_view(name));
    let named_v: ResizedView<NamedView<Dialog>> = Dialog::around(table_view)
        .title(dir)
        .with_name(name)
        .full_screen();

    named_v
}
