use std::{
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex},
};

use crate::{
    cursive::view::Nameable,
    definitions::definitions::{LEFT_TABLE_VIEW_NAME, RIGHT_TABLE_VIEW_NAME},
    utils::common_utils::{
        copy_file, get_active_table_first_selected_index, get_active_table_first_selected_item,
        get_active_table_name, get_current_path_from_dialog_name, os_string_to_lossy_string,
        select_index,
    },
};
use crate::{cursive::view::Resizable, utils::common_utils::get_active_table_selected_items};
use cursive::{direction::Orientation, views::CircularFocus};
use cursive::{theme::ColorStyle, Cursive};
use cursive::{
    views::{
        Button, Dialog, DummyView, HideableView, LinearLayout, NamedView, ResizedView, StackView,
        TextView,
    },
    CbSink,
};
use cursive_table_view::TableView;
use futures::channel::mpsc::Sender;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::new_debouncer_opt;

use crate::{
    definitions::definitions::{LEFT_PANEL_NAME, RIGHT_PANEL_NAME},
    tui_fn::{
        create_info_layout::create_info_layout,
        create_peek_layout,
        create_peek_layout::create_peek_layout,
        create_table::{create_table, BasicColumn, DirView},
        create_view_layout::create_view_layout,
    },
};
fn prepare_peek_view(s: &mut Cursive) {
    let active_table_name = get_active_table_name(s);
    let selected_item = get_active_table_first_selected_item(s, &active_table_name);
    let selected_item_inx = get_active_table_first_selected_index(s, &active_table_name);
    let dialog_name = LEFT_PANEL_NAME;
    let current_path = get_current_path_from_dialog_name(s, String::from(dialog_name)); //++artie, &str
    let peek_layout = create_peek_layout(&current_path, &selected_item);

    s.add_fullscreen_layer(peek_layout);
    /*In order for table to be "searchable" it must be added to cursive */
    select_index(s, "PeekPanelDir_tableview", selected_item_inx);
}
fn prepare_info_view(s: &mut Cursive) {
    let active_table_name = get_active_table_name(s);
    let selected_item = get_active_table_first_selected_item(s, &active_table_name);
    let selected_item_inx = get_active_table_first_selected_index(s, &active_table_name);
    let dialog_name = LEFT_PANEL_NAME;
    let current_path = get_current_path_from_dialog_name(s, String::from(dialog_name)); //++artie, &str
    let peek_layout = create_info_layout(&current_path, &selected_item);

    s.add_fullscreen_layer(peek_layout);
    /*In order for table to be "searchable" it must be added to cursive */
    select_index(s, "InfoPanelDir_tableview", selected_item_inx);
}
fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx.recv() {
        match res {
            Ok(event) => println!("changed: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn main_waiting_for_thread(selected_item: &str, full_dest_path: &str, cb_sink: CbSink) {
    let selected_item_clone = String::from(selected_item);
    let full_dest_path_clone = String::from(full_dest_path);
    let full_dest_path_clone_2 = String::from(full_dest_path);
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let cb_sink_clone = cb_sink.clone();
    // Inside of our lock, spawn a new thread, and then wait for it to start.
    thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        {
            let mut started = lock.lock().unwrap();
            *started = true;
            drop(started); //++artie, unbelievable, manual mem management...
        }
        // We notify the condvar that the value has changed.
        cvar.notify_one();
        match copy_file(&selected_item_clone, &full_dest_path_clone) {
            Ok(_) => {
                eprintln!("Copied");
            }
            Err(e) => {
                eprintln!("couldn't copy: {e}");
            }
        }
        tx.send(true);
        let _ = cb_sink_clone
            .send(Box::new(|s| {
                close_cpy_dlg(s);
            }))
            .unwrap();

        return;
    });

    // Wait for the thread to start up.
    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }
    drop(started); //++artie, unbelievable, manual mem management...

    println!("Copying thread started. Proceeding to spawn watch thread.");
    let _handle_read = std::thread::spawn(move || loop {
        match rx.try_recv() {
            Ok(res) => {
                if res {
                    eprintln!("Received end of copying msg");
                    break;
                }
            }
            Err(e) => {
                //eprintln!("Receiving error: {}", e);
            }
        }
        let full_dest_path_clone_2_clone = full_dest_path_clone_2.clone();
        match std::fs::File::open(full_dest_path_clone_2_clone) {
            Ok(f) => {
                let len = f.metadata().unwrap().len();
                //eprintln!("opened, len: {len}");
                let percent = len;
                cb_sink
                    .send(Box::new(move |siv| update_copy_dlg(siv, percent)))
                    .unwrap();
            }
            Err(e) => {
                // eprintln!("couldn't open: {e}");
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(250));
    });
}

fn update_copy_dlg(s: &mut Cursive, percent: u64) {
    s.call_on_name("cpy_percent", |text_view: &mut TextView| {
        text_view.set_content(format!("{percent}"));
    })
    .unwrap();
}
fn close_cpy_dlg(s: &mut Cursive) {
    match s.call_on_name("cpy_dlg", |_: &mut Dialog| true) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(v) => {
            if v == true {
                s.pop_layer();
            }
        }
        None => {}
    }
}
pub fn create_classic_buttons() -> ResizedView<StackView> {
    let help_tuple = (
        TextView::new("F1").style(ColorStyle::title_primary()),
        Button::new_raw("[ Info ]", |s| {}),
    );
    let help_layout = LinearLayout::horizontal()
        .child(TextView::new("F1").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Info ]", |s| {
            prepare_info_view(s);
        }));
    let menu_layout = LinearLayout::horizontal()
        .child(TextView::new("F2").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Popup ]", |s| {}));
    let view_layout = LinearLayout::horizontal()
        .child(TextView::new("F3").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ View/Edit ]", |s| {
            let active_table_name = get_active_table_name(s);
            let selected_item = get_active_table_first_selected_item(s, &active_table_name);
            let view_layout = create_view_layout(&selected_item);
            s.add_fullscreen_layer(view_layout);
        }));
    let edit_layout = LinearLayout::horizontal()
        .child(TextView::new("F4").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Peek ]", |s| {
            prepare_peek_view(s);
        }));
    let copy_layout = LinearLayout::horizontal()
        .child(TextView::new("F5").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Copy ]", |s| {
            let ((src_table, _), (_, dest_panel)) =
                if get_active_table_name(s) == LEFT_TABLE_VIEW_NAME {
                    (
                        //++artie only one item neede to return
                        (LEFT_TABLE_VIEW_NAME, LEFT_PANEL_NAME),
                        (RIGHT_TABLE_VIEW_NAME, RIGHT_PANEL_NAME),
                    )
                } else {
                    (
                        (RIGHT_TABLE_VIEW_NAME, RIGHT_PANEL_NAME),
                        (LEFT_TABLE_VIEW_NAME, LEFT_PANEL_NAME),
                    )
                };
            let selected_items = get_active_table_selected_items(s, src_table);
            //eprintln!("{:?}", selected_items);
            let dest_path = get_current_path_from_dialog_name(s, String::from(dest_panel));
            //eprintln!("dest_path: {}", dest_path);
            for selected_item in selected_items {
                match PathBuf::from(&selected_item).file_name() {
                    Some(file_name) => {
                        //std::thread::scope(|scoped| {
                        let full_dest_path =
                            format!("{}/{}", &dest_path, os_string_to_lossy_string(&file_name));
                        //eprintln!("full_dest_path: {full_dest_path}");
                        let dest_path_clone = dest_path.clone();
                        let full_dest_path_clone = full_dest_path.clone();
                        //let (tx, rx) = std::sync::mpsc::sync_channel(1);
                        let cpy_dlg = Dialog::around(TextView::new("").with_name("cpy_percent"))
                            .with_name("cpy_dlg");
                        s.add_layer(cpy_dlg);
                        let cb_sink = s.cb_sink().clone();
                        main_waiting_for_thread(&selected_item, &full_dest_path, cb_sink);

                        /*
                        let arc_cond_var = Arc::new((Mutex::new(false), Condvar::new()));
                        let arc_cond_var_clone = arc_cond_var.clone();

                        let _handle_copy = std::thread::spawn(move || {
                            let (lock, cvar) = &*arc_cond_var;
                            let mut started = lock.lock().unwrap();
                            *started = true;
                            // We notify the condvar that the value has changed.
                            cvar.notify_all();
                            match copy_file(&selected_item, &full_dest_path) {
                                Ok(_) => {
                                    eprintln!("Copied");
                                    tx.send(true);
                                    return;
                                }
                                Err(e) => {
                                    eprintln!("couldn't copy: {e}");
                                    tx.send(true);
                                    return;
                                }
                            }
                        });
                        /*First, lets wait for the readying thread to start */
                        let (lock, cond_var) = &*arc_cond_var_clone;
                        let mut started = lock.lock().unwrap();
                        while !*started {
                            started = cond_var.wait(started).unwrap();
                        }
                        let _handle_read = std::thread::spawn(move || {
                            loop {
                                match rx.try_recv() {
                                    Ok(res) => {
                                        if res {
                                            eprintln!("Received end of copying msg");
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Receiving error: {}", e);
                                        // break;
                                    }
                                }
                                let full_dest_path_clone_2 = full_dest_path_clone.clone();
                                match std::fs::File::open(full_dest_path_clone_2) {
                                    Ok(f) => {
                                        let len = f.metadata().unwrap().len();
                                        //eprintln!("opened, len: {len}");
                                    }
                                    Err(e) => {
                                        eprintln!("couldn't open: {e}");
                                    }
                                }

                                std::thread::sleep(std::time::Duration::from_secs(2));
                            }
                        });
                        */
                        //handle_copy.join();
                        //handle_read.join();
                        //std::thread::spawn(move || watch(&dest_path_clone));
                        //std::thread::spawn(move || {
                        //    match copy_file(&selected_item, &full_dest_path) {
                        //        Ok(_) => {
                        //            eprintln!("Copied")
                        //        }
                        //        Err(e) => {
                        //            eprintln!("Couldn't cpy: {e}")
                        //        }
                        //    }
                        //});
                        //scoped });
                    }
                    None => {
                        eprintln!("Couldn't copy {selected_item}");
                    }
                }
            }
        }));
    let rn_mv_layout = LinearLayout::horizontal()
        .child(TextView::new("F6").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Rnm/Mv ]", |s| {}));
    let mkdir_layout = LinearLayout::horizontal()
        .child(TextView::new("F8").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ MkDir ]", |s| {}));
    let pulldown_layout = LinearLayout::horizontal()
        .child(TextView::new("F9").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Find ]", move |s| {
            //s.call_on_name(
            //    "left_panel_hideable",
            //    |ob: &mut NamedView<ResizedView<HideableView<NamedView<Dialog>>>>| {
            //        ob.get_mut().get_inner_mut().hide();
            //    },
            //);
            //let mut layout_panes = LinearLayout::new(Orientation::Horizontal);
            //let named_v_right: NamedView<Dialog> = Dialog::around(create_table())
            //    .title("Left")
            //    .with_name("left_dialog");
            //let hide_v_right: HideableView<NamedView<Dialog>> = HideableView::new(named_v_right);
            //let hide_v_right_full_screed: NamedView<ResizedView<HideableView<NamedView<Dialog>>>> =
            //    hide_v_right.full_screen().with_name("right_panel_hideable");
            //layout_panes.add_child(hide_v_right_full_screed);
            //s.add_fullscreen_layer(layout_panes);
        }));

    let quit_layout = LinearLayout::horizontal()
        .child(TextView::new("F10").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Quit ]", |s| s.quit()));

    let classic_buttons = LinearLayout::horizontal()
        .child(help_layout)
        .child(DummyView.full_width())
        .child(menu_layout)
        .child(DummyView.full_width())
        .child(view_layout)
        .child(DummyView.full_width())
        .child(edit_layout)
        .child(DummyView.full_width())
        .child(copy_layout)
        .child(DummyView.full_width())
        .child(rn_mv_layout)
        .child(DummyView.full_width())
        .child(mkdir_layout)
        .child(DummyView.full_width())
        .child(pulldown_layout)
        .child(DummyView.full_width())
        .child(quit_layout);

    let mut stack_buttons = StackView::new().fixed_height(1);
    stack_buttons
        .get_inner_mut()
        .add_fullscreen_layer(classic_buttons.with_name("classic_buttons"));

    stack_buttons
}
