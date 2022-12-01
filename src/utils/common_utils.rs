use std::{ffi::OsStr, fs::DirEntry, path::PathBuf, time::SystemTime};

use cursive::Cursive;

use crate::definitions::definitions::{LEFT_TABLE_VIEW_NAME, RIGHT_TABLE_VIEW_NAME};
use crate::tui_fn::create_table::{create_table, BasicColumn, DirView};
use cursive::views::{
    Dialog, DummyView, HideableView, LinearLayout, NamedView, ResizedView, StackView, TextView,
};
use cursive_table_view::{TableView, TableViewItem};
pub fn get_active_table_name(s: &mut Cursive) -> String {
    let left_focus_time = s
        .call_on_name(
            LEFT_TABLE_VIEW_NAME,
            |table: &mut NamedView<TableView<DirView, BasicColumn>>| {
                table.get_mut().get_last_focus_time()
            },
        )
        .unwrap();

    let right_focus_time = s
        .call_on_name(
            RIGHT_TABLE_VIEW_NAME,
            |table: &mut NamedView<TableView<DirView, BasicColumn>>| {
                table.get_mut().get_last_focus_time()
            },
        )
        .unwrap();

    if left_focus_time > right_focus_time {
        String::from(RIGHT_TABLE_VIEW_NAME)
    } else {
        String::from(LEFT_TABLE_VIEW_NAME)
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

    pathbuf_to_lossy_string(&selected_item)
}

pub fn get_active_table_selected_items(s: &mut Cursive, active_table_name: &str) -> Vec<String> {
    let mut res = Vec::new();
    //++artie refactor, return ref to direntry
    s.call_on_name(
        active_table_name,
        |table: &mut NamedView<TableView<DirView, BasicColumn>>| {
            let table = table.get_mut();
            let selected_items = table.get_selected_items();
            for selected_item in selected_items {
                res.push(pathbuf_to_lossy_string(&selected_item.name));
            }
        },
    )
    .unwrap();

    res
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
    return String::from("Helloooo");
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

pub fn get_current_path_from_dialog_name(s: &mut Cursive, dialog_name: String) -> String {
    /*First get the dialog's title which is first path of dir */
    let current_path = s
        .call_on_name(&dialog_name, |s: &mut Dialog| {
            let title = String::from(s.get_title());
            title
        })
        .unwrap();

    current_path
}

pub fn copy_file(src: &str, dest: &str) -> std::io::Result<()> {
    std::fs::copy(src, dest)?;
    Ok(())
}

pub fn pathbuf_to_lossy_string(path_buf: &PathBuf) -> String {
    path_buf.as_os_str().to_string_lossy().to_string()
}

pub fn os_string_to_lossy_string(os_string: &OsStr) -> String {
    os_string.to_string_lossy().to_string()
}
