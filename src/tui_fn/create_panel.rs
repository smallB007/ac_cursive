use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    cursive::view::{Nameable, Resizable},
    utils::common_utils::{init_watcher, pathbuf_to_lossy_string, update_watcher},
};
use crate::{
    tui_fn::create_table::{create_table, prepare_items_for_table_view},
    utils::common_utils::get_current_path_from_dialog_name,
};
use cursive::{
    view::ViewWrapper,
    views::{Dialog, HideableView, NamedView, ResizedView},
    Cursive,
};
use cursive_table_view::TableView;
use notify::{INotifyWatcher, RecommendedWatcher, Watcher};

use super::create_table::{BasicColumn, DirView};

fn create_name_for_table_view(name: &str) -> String {
    String::from(String::from(name) + "_tableview")
}

pub fn update_table(s: &mut Cursive, dialog_name: &String, table_view_name: &String) {
    s.call_on_name(
        &table_view_name,
        |table: &mut TableView<DirView, BasicColumn>| {
            let (longest_path, items) = prepare_items_for_table_view(&dialog_name);
            table.set_items(items);
        },
    );
}

fn traverse_up(
    s: &mut Cursive,
    dialog_name: String,
    table_view_name: String,
    watcher: &mut Option<Rc<RefCell<INotifyWatcher>>>,
) {
    /*Third, combine them to form full path */
    let current_path = get_current_path_from_dialog_name(s, &dialog_name);
    let old_path = current_path.clone();
    let mut full_path = PathBuf::from(current_path);
    full_path.pop();
    if full_path.is_dir() {
        let new_path = pathbuf_to_lossy_string(&full_path);
        let new_path_clone = new_path.clone();
        if watcher.is_some() {
            let watcher = watcher.as_mut().unwrap();
            if update_watcher(watcher.clone(), old_path, new_path).is_err() {
                eprintln!("Err: if update_watcher");
            }
        }
        update_table(s, &new_path_clone, &table_view_name);
        s.call_on_name(&dialog_name, |dlg: &mut Dialog| {
            dlg.set_title(new_path_clone);
        });
    }
}
fn traverse_down(
    s: &mut Cursive,
    dialog_name: String,
    table_view_name: String,
    selected_item: PathBuf,
    watcher: &mut Option<Rc<RefCell<INotifyWatcher>>>,
) {
    let current_path = PathBuf::from(get_current_path_from_dialog_name(s, &dialog_name));
    let old_path = pathbuf_to_lossy_string(&current_path);
    let full_path = current_path.join(&selected_item);

    if full_path.is_dir() {
        let new_path = pathbuf_to_lossy_string(&full_path);
        let new_path_clone = new_path.clone();
        if watcher.is_some() {
            let watcher = watcher.as_mut().unwrap();
            if update_watcher(watcher.clone(), old_path, new_path).is_err() {
                eprintln!("Err: if update_watcher");
            }
        }
        s.call_on_name(
            &table_view_name,
            |table: &mut TableView<DirView, BasicColumn>| {
                let (longest_path, items) = prepare_items_for_table_view(&new_path_clone);
                //table.remove_item(index);
                table.set_items(items);
            },
        );
        s.call_on_name(&dialog_name, |s: &mut Dialog| {
            s.set_title(new_path_clone);
        });
    }
}
pub fn create_panel(
    name: &str,
    dir: &str,
    cb_peek: Option<fn(&mut Cursive, usize, usize)>,
    watcher: Option<Rc<RefCell<RecommendedWatcher>>>,
) -> ResizedView<NamedView<Dialog>> {
    let table_view_name = create_name_for_table_view(name);
    let table_view_name_clone = table_view_name.clone();
    let dialog_name = String::from(name);
    let table_view = create_table(dir); //.with_name(String::from(name) + "_tableview");
    let table_view = if cb_peek.is_some() {
        table_view.on_peek(cb_peek.unwrap())
    } else {
        table_view
    };
    let table_view = table_view.on_submit(move |s, r, index| {
        /*Second get the selected item */
        let selected_item = s
            .call_on_name(
                &table_view_name_clone,
                |table: &mut TableView<DirView, BasicColumn>| {
                    //table.remove_item(index);
                    table.get_focused_item().name.clone()
                },
            )
            .unwrap();
        if selected_item == PathBuf::from("..") {
            traverse_up(
                s,
                dialog_name.clone(),
                table_view_name_clone.clone(),
                &mut watcher.clone(),
            );
        } else {
            traverse_down(
                s,
                dialog_name.clone(),
                table_view_name_clone.clone(),
                selected_item.clone(),
                &mut watcher.clone(),
            );
        }
    });
    let table_view = table_view.with_name(table_view_name);
    let named_v = Dialog::around(table_view)
        .title(dir)
        .with_name(name)
        .full_screen();
    named_v
}
#[cfg(unused)]
pub struct PanelWithWatcher {
    view: ResizedView<NamedView<Dialog>>,
    watcher: Option<RecommendedWatcher>,
}
#[cfg(unused)]
impl PanelWithWatcher {
    fn update_watcher(&mut self, old_path: &str, new_path: &str) {
        if self.watcher.is_some() {
            let watcher = self.watcher.as_mut().unwrap();

            if watcher.unwatch(old_path.as_ref()).is_err() {
                eprintln!("Error: watcher.unwatch");
            }
            if watcher
                .watch(new_path.as_ref(), notify::RecursiveMode::NonRecursive)
                .is_err()
            {
                eprintln!("Error: watcher.watch");
            }
        }
    }
}
#[cfg(unused)]
impl ViewWrapper for PanelWithWatcher {
    type V = ResizedView<NamedView<Dialog>>;
    fn with_view<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Self::V) -> R,
    {
        Some(f(&self.view))
    }
    fn with_view_mut<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut Self::V) -> R,
    {
        Some(f(&mut self.view))
    }
}
