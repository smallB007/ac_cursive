use crate::utils::{common_utils::pretty_print_system_time, cp_machinery::cp_utils::file_info};

use cursive::{align::HAlign, views::Button};
use cursive_table_view::{TableView, TableViewItem};
use rand::Rng;
use std::fs::Permissions;
use std::{cmp::Ordering, rc::Rc};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum InfoColumn {
    Content,
}

#[derive(Clone, Debug)]
pub struct InfoStruct {
    content: String,
}

impl TableViewItem<InfoColumn> for InfoStruct {
    fn to_column(&self, column: InfoColumn) -> String {
        self.content.clone()
    }

    fn cmp(&self, other: &Self, column: InfoColumn) -> Ordering
    where
        Self: Sized,
    {
        Ordering::Equal
    }
}

pub fn create_info_table(file: &str) -> TableView<InfoStruct, InfoColumn> {
    let content = match file_info(file) {
        Ok(content) => content,
        Err(e) => e.to_string(),
    };

    let mut items = Vec::new();
    for line in content.lines() {
        items.push(InfoStruct {
            content: String::from(line),
        });
    }

    TableView::<InfoStruct, InfoColumn>::new()
        .column(InfoColumn::Content, "", |c| c.align(HAlign::Left), false)
        .items(items)
}
