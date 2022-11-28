use std::{fs::DirEntry, path::PathBuf, time::SystemTime};

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

pub fn get_active_table_first_selected_index(s: &mut Cursive, active_table_name: &str) -> usize {
    //++artie refactor, return ref to direntry
    let selected_index = s
        .call_on_name(
            active_table_name,
            |table: &mut NamedView<TableView<DirView, BasicColumn>>| match table.get_mut().item() {
                Some(inx) => inx,
                None => 0,
            },
        )
        .unwrap();

    selected_index
}

pub fn select_index(s: &mut Cursive, active_table_name: &str, item_index: usize) {
    s.call_on_name(
        active_table_name,
        |table: &mut NamedView<TableView<DirView, BasicColumn>>| {
            table.get_mut().set_selected_item(item_index)
        },
    );
    //.unwrap();
}
const DATE_FORMAT_STR: &'static str = "[day]-[month repr:short]-[year] [hour]:[minute]";
pub const FORMAT: &[time::format_description::FormatItem<'_>] = time::macros::format_description!(
    "[day]-[month repr:short]-[year repr:last_two] [hour]:[minute]"
);
pub fn pretty_print_system_time(t: SystemTime) -> String {
    // readableBytes(21111024);
    let mut res = Vec::new(); //++artie, with_capacity

    let utc = time::OffsetDateTime::UNIX_EPOCH
        + time::Duration::try_from(t.duration_since(std::time::UNIX_EPOCH).unwrap()).unwrap();
    let local = utc.to_offset(time::UtcOffset::local_offset_at(utc).unwrap());
    local
        .format_into(
            //&mut std::io::stdout().lock(),
            &mut res,
            FORMAT, //time::macros::format_description!(
                   //    // "[day]-[month repr:numerical]-[year] [hour]:[minute]:[second]"
                   //),
                   //&time::format_description::parse(
                   //    // "[day]-[month repr:numerical]-[year] [hour]:[minute]:[second]"
                   //    DATE_FORMAT_STR,
                   //)
                   //.unwrap(),
        )
        .unwrap();
    String::from_utf8(res).unwrap()
}

pub fn readableBytes(bytes: usize) -> String {
    static SIZES: [&str; 9] = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let inx = f64::floor(f64::log2(bytes as f64) / f64::log2(1024.0)) as usize;

    if inx != 0 {
        format!(
            "{:.2}{}",
            bytes as f64 / f64::powf(1024 as f64, inx as f64),
            SIZES[inx]
        )
    } else {
        format!(
            "{}{}",
            bytes as f64 / f64::powf(1024 as f64, inx as f64),
            SIZES[inx]
        )
    }
}
