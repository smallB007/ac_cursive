use cursive::{align::HAlign, views::Button};
use cursive_table_view::{TableView, TableViewItem};
use rand::Rng;
use std::{any::Any, cmp::Ordering, path::PathBuf, rc::Rc};
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BasicColumn {
    Name,
    Count,
    Rate,
}

#[derive(Debug)]
pub struct DirView {
    pub name: String,
    pub size: u64,
}

impl TableViewItem<BasicColumn> for DirView {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.clone(),
            BasicColumn::Count => format!("{}", self.size),
            BasicColumn::Rate => format!("{}", 0),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Name if self.name == ".." || other.name == ".." => Ordering::Greater,
            //Folders
            BasicColumn::Name if self.name.ends_with('/') && other.name.ends_with('/') => {
                if !self.name.starts_with(".") && other.name.starts_with(".") {
                    Ordering::Less
                } else if self.name.starts_with(".") && !other.name.starts_with(".") {
                    Ordering::Greater
                } else {
                    self.name.cmp(&other.name) //seems OK
                }
            }
            //Folder file
            BasicColumn::Name if self.name.ends_with('/') && !other.name.ends_with('/') => {
                Ordering::Greater
            }
            BasicColumn::Name if !self.name.ends_with('/') && other.name.ends_with('/') => {
                Ordering::Less
            }
            //Files
            BasicColumn::Name if !self.name.ends_with('/') && !other.name.ends_with('/') => {
                if self.name.starts_with(".") && !other.name.starts_with(".") {
                    Ordering::Greater
                } else if !self.name.starts_with(".") && other.name.starts_with(".") {
                    Ordering::Less
                } else {
                    self.name.cmp(&other.name)
                }
            }
            BasicColumn::Name => self.name.cmp(&other.name),
            //BasicColumn::Name if other.name.ends_with('/') => Ordering::Less,
            //BasicColumn::Name if self.name.ends_with('/') => Ordering::Greater,
            // BasicColumn::Name => {
            //     if self.name != ".." && self.name.starts_with('.') && other.name.starts_with('.') {
            //         self.name.cmp(&other.name)
            //     } else if self.name != ".."
            //         && self.name.starts_with('.')
            //         && !other.name.starts_with('.')
            //     {
            //         Ordering::Less
            //     } else {
            //         Ordering::Greater
            //     }
            // }
            BasicColumn::Count => Ordering::Equal,
            BasicColumn::Rate => Ordering::Equal,
        }
    }
}
pub fn prepare_items_for_table_view(dir: &str) -> (usize, Vec<DirView>) {
    let mut longest_path = 0_usize;
    let dir_entries = Dir_entry_list_dir_content(dir).unwrap(); //++artie, unwrap, deal with error, disp dialog
    let mut items = Vec::new();
    let has_parent = PathBuf::from(dir).parent().is_some();
    if has_parent {
        let level_up_dir_entry = String::from("..");
        items.push(DirView {
            name: level_up_dir_entry,
            size: 0,
        });
    }
    for entry in dir_entries {
        let path = if entry.is_dir() {
            format!("{}/", entry.file_name().unwrap().to_str().unwrap())
        } else {
            String::from(entry.file_name().unwrap().to_str().unwrap())
        };
        if path.len() > longest_path {
            longest_path = path.len();
        }
        eprintln!(">>entries: {:?}", entry);
        if entry.is_symlink() {
            match fs::symlink_metadata(&entry) {
                Ok(meta) => {
                    items.push(DirView {
                        name: path,
                        size: meta.len(),
                    });
                }
                Err(e) => {
                    panic!("meta:{:?}, entry:{:?}", e, entry);
                }
            }
        } else {
            match fs::metadata(&entry) {
                Ok(meta) => {
                    items.push(DirView {
                        name: path,
                        size: meta.len(),
                    });
                }
                Err(e) => {
                    panic!("meta:{:?}, entry:{:?}", e, entry);
                }
            }
        }
    }

    (longest_path, items)
}
pub fn create_table(dir: &str) -> TableView<DirView, BasicColumn> {
    let (longest_path, items) = prepare_items_for_table_view(dir);
    TableView::<DirView, BasicColumn>::new()
        .column(BasicColumn::Name, "Name", |c| {
            //if longest_path < 50 {
            //    c.width(longest_path)
            //} else {
            //    c.width_percent(70)
            //}
            c
        })
        .column(BasicColumn::Count, "Size", |c| c.align(HAlign::Center))
        .column(BasicColumn::Rate, "Modify Time", |c| {
            c.ordering(Ordering::Greater).align(HAlign::Right)
            //.width_percent(20)
        })
        .items(items)
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
