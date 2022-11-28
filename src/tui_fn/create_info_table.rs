use crate::utils::common_utils::pretty_print_system_time;
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

#[cfg(target_os = "linux")]
fn file_info(file: &str) -> Result<String, std::io::Error> {
    use std::os::unix::prelude::PermissionsExt;

    let metadata = std::fs::metadata(file)?;

    let file_type = format!("File type: {:?}", metadata.file_type());
    let accessed = match metadata.accessed() {
        Ok(val) => format!("Access time: {:>25}", pretty_print_system_time(val)),
        Err(e) => {
            eprintln!("cannot get accessed time: {}", e);
            String::from("Access time: UNKNOWN")
        }
    };
    let created = match metadata.created() {
        Ok(val) => format!("Created time: {:>24}", pretty_print_system_time(val)),
        Err(e) => {
            eprintln!("cannot get created time: {}", e);
            String::from("Created time: UNKNOWN")
        }
    };
    let modified = match metadata.modified() {
        Ok(val) => format!("Modified time: {:>23}", pretty_print_system_time(val)),
        Err(e) => {
            eprintln!("cannot get modified time: {}", e);
            String::from("Modified time: UNKNOWN")
        }
    };

    let size_in_bytes = format!("Size in bytes: {}B", metadata.len());

    let permissions = metadata.permissions();
    let mode = format!(
        "mode: {}",
        <Permissions as PermissionsExt>::mode(&permissions)
    );

    Ok(file_type
        + "\n"
        + &accessed
        + "\n"
        + &created
        + "\n"
        + &modified
        + "\n"
        + &size_in_bytes
        + "\n"
        + &mode)
}

#[cfg(not(target_os = "linux"))]
fn file_info(file: &str) -> Result<String, std::io::Error> {
    std::io::Error
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
