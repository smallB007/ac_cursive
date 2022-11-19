use std::{fs::DirEntry, path::PathBuf};

use cursive::Cursive;

use crate::tui_fn::create_table::{create_table, BasicColumn, DirView};
use cursive::views::{
    Dialog, DummyView, HideableView, LinearLayout, NamedView, ResizedView, StackView, TextView,
};
use cursive_table_view::{TableView, TableViewItem};
pub fn get_active_table_name(s: &mut Cursive) -> String {
    let left_focus_time = s
        .call_on_name(
            "Left_tableview",
            |table: &mut NamedView<TableView<DirView, BasicColumn>>| {
                table.get_mut().get_last_focus_time()
            },
        )
        .unwrap();

    let right_focus_time = s
        .call_on_name(
            "Right_tableview",
            |table: &mut NamedView<TableView<DirView, BasicColumn>>| {
                table.get_mut().get_last_focus_time()
            },
        )
        .unwrap();

    if left_focus_time > right_focus_time {
        String::from("Right_tableview")
    } else {
        String::from("Left_tableview")
    }
}

pub fn get_active_table_first_selected_item(s: &mut Cursive, active_table_name: &str) -> String {
    //++artie refactor, return ref to direntry
    let selected_item = s
        .call_on_name(
            active_table_name,
            |table: &mut NamedView<TableView<DirView, BasicColumn>>| {
                table.get_mut().get_selected_item().name.clone()
            },
        )
        .unwrap();

    selected_item.as_os_str().to_string_lossy().to_string()
}
