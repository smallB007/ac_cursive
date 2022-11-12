use cursive::{align::HAlign, views::Button};
use cursive_table_view::{TableView, TableViewItem};
use rand::Rng;
use std::{cmp::Ordering, path::PathBuf, rc::Rc};
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BasicColumn {
    Name,
    Count,
    Rate,
}

#[derive(Clone, Debug)]
pub struct DirView {
    pub name: String,
    count: usize,
    rate: usize,
}

impl TableViewItem<BasicColumn> for DirView {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.to_string(),
            BasicColumn::Count => format!("{}", self.count),
            BasicColumn::Rate => format!("{}", self.rate),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Name if self.name == ".." => Ordering::Greater,
            //BasicColumn::Name if self.name.starts_with("/.") => Ordering::Less,
            BasicColumn::Name => self.name.cmp(&other.name),
            BasicColumn::Count => self.count.cmp(&other.count),
            BasicColumn::Rate => self.rate.cmp(&other.rate),
        }
    }
}

pub fn create_table(dir: &str) -> TableView<DirView, BasicColumn> {
    let dir_entries = list_dir_content(dir).unwrap(); //++artie, unwrap, deal with error, disp dialog
    let mut items = Vec::new();
    let is_root = PathBuf::from(dir).parent().is_none();
    if !is_root {
        items.push(DirView {
            name: String::from(".."),
            count: 0,
            rate: 0,
        });
    }
    for entry in dir_entries {
        let path = if entry.path().is_dir() {
            format!("/{}", entry.path().file_name().unwrap().to_str().unwrap())
        } else {
            String::from(entry.path().file_name().unwrap().to_str().unwrap())
        };

        items.push(DirView {
            name: path,
            count: 0,
            rate: 0,
        });
    }

    TableView::<DirView, BasicColumn>::new()
        .column(BasicColumn::Name, "Name", |c| c.width_percent(20))
        .column(BasicColumn::Count, "Count", |c| c.align(HAlign::Center))
        .column(BasicColumn::Rate, "Rate", |c| {
            c.ordering(Ordering::Greater)
                .align(HAlign::Right)
                .width_percent(20)
        })
        .items(items)
}

use std::fs::{self, DirEntry};

fn list_dir_content(dir: &str) -> Result<Vec<DirEntry>, std::io::Error> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        entries.push(entry);
        //println!("entry: {:?}", entry);
        //let path = entry.path();

        //let metadata = fs::metadata(&path)?;
        //let last_modified = metadata.modified()?.elapsed()?.as_secs();
        //
        //if last_modified < 24 * 3600 && metadata.is_file() {
        //    println!(
        //        "Last modified: {:?} seconds, is read only: {:?}, size: {:?} bytes, filename: {:?}",
        //        last_modified,
        //        metadata.permissions().readonly(),
        //        metadata.len(),
        //        path.file_name().ok_or("No filename")?
        //    );
        //}
    }

    Ok(entries)
}
