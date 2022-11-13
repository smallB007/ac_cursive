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

#[derive(Debug)]
pub struct DirView {
    pub name: String,
}

impl TableViewItem<BasicColumn> for DirView {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.clone(),
            BasicColumn::Count => format!("{}", 0),
            BasicColumn::Rate => format!("{}", 0),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Name if self.name == ".." => Ordering::Greater,
            BasicColumn::Name if self.name.ends_with('/') && other.name.ends_with('/') => {
                Ordering::Equal
            }
            BasicColumn::Name if other.name.ends_with('/') => Ordering::Less,
            BasicColumn::Name if self.name.ends_with('/') => Ordering::Greater,
            BasicColumn::Name => {
                eprintln!("other.name{}", &other.name);
                self.name.cmp(&other.name)
            }
            BasicColumn::Count => Ordering::Equal,
            BasicColumn::Rate => Ordering::Equal,
        }
    }
}
pub fn prepare_items_for_table_view(dir: &str) -> Vec<DirView> {
    let dir_entries = Dir_entry_list_dir_content(dir).unwrap(); //++artie, unwrap, deal with error, disp dialog
    let mut items = Vec::new();
    let has_parent = PathBuf::from(dir).parent().is_some();
    if has_parent {
        let level_up_dir_entry = String::from("..");
        items.push(DirView {
            name: level_up_dir_entry,
        });
    }
    for entry in dir_entries {
        let path = if entry.is_dir() {
            format!("{}/", entry.file_name().unwrap().to_str().unwrap())
        } else {
            String::from(entry.file_name().unwrap().to_str().unwrap())
        };

        items.push(DirView { name: path });
    }

    items
}
pub fn create_table(dir: &str) -> TableView<DirView, BasicColumn> {
    TableView::<DirView, BasicColumn>::new()
        .column(BasicColumn::Name, "Name", |c| c.width_percent(20))
        .column(BasicColumn::Count, "Count", |c| c.align(HAlign::Center))
        .column(BasicColumn::Rate, "Rate", |c| {
            c.ordering(Ordering::Greater)
                .align(HAlign::Right)
                .width_percent(20)
        })
        .items(prepare_items_for_table_view(dir))
}

use std::fs::{self, DirEntry};
use walkdir::WalkDir;

fn Dir_entry_list_dir_content(dir: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut res = Vec::new();
    for entry in WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .skip(1) //to skip printout of the dir name we are iterating
        .filter_map(|e| e.ok())
    {
        //println!("{}", entry.path().display());
        res.push(entry.path().to_owned())
    }
    Ok(res)
}
fn list_dir_content(dir: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        entries.push(entry.path());
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
