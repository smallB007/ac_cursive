use std::{
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex},
};

use anyhow::Context;
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream, NameTypeSupport};
use std::{
    io::{self, prelude::*, BufReader},
    sync::mpsc::Sender,
};

use crate::{
    cursive::view::Nameable,
    definitions::definitions::{LEFT_TABLE_VIEW_NAME, RIGHT_TABLE_VIEW_NAME},
    utils::{
        common_utils::{
            copy_file, get_active_table_first_selected_index, get_active_table_first_selected_item,
            get_active_table_name, get_current_path_from_dialog_name, os_string_to_lossy_string,
            select_index,
        },
        cp_machinery::cp_client_main::cp_client_main,
        cp_machinery::cp_server_main::cp_server_main,
    },
};
use crate::{cursive::view::Resizable, utils::common_utils::get_active_table_selected_items};
use cursive::{
    direction::Orientation,
    views::{CircularFocus, ListView, ProgressBar, ScrollView, TextContent},
};
use cursive::{theme::ColorStyle, Cursive};
use cursive::{
    views::{
        Button, Dialog, DummyView, HideableView, LinearLayout, NamedView, ResizedView, StackView,
        TextView,
    },
    CbSink,
};
use cursive_table_view::TableView;
//use futures::channel::mpsc::Sender;
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

fn copying_engine(
    selected_item: &str,
    selected_item_n: u64,
    total_items: u64,
    full_dest_path: &str,
    cb_sink: CbSink,
) {
    std::thread::scope(|s| {
        let selected_item_clone = String::from(selected_item);
        let selected_item_len = match PathBuf::from(selected_item).metadata() {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                eprintln!("Couldn't get len for path: {}", selected_item);
                0
            }
        };
        let full_dest_path_clone = String::from(full_dest_path);
        let full_dest_path_clone_2 = String::from(full_dest_path);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        use std::sync::{Arc, Condvar, Mutex};
        use std::thread;

        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = Arc::clone(&pair);
        //let cb_sink_clone = cb_sink.clone();
        // Inside of our lock, spawn a new thread, and then wait for it to start.
        let _handle_cpy = s.spawn(move || {
            let (lock, cvar) = &*pair2;

            let mut started = lock.lock().unwrap();
            *started = true;

            // We notify the condvar that the value has changed.
            cvar.notify_one();
            drop(started); //++artie, unbelievable, manual mem management...
            match copy_file(&selected_item_clone, &full_dest_path_clone) {
                Ok(_) => {
                    eprintln!("Copied");
                }
                Err(e) => {
                    eprintln!("couldn't copy: {e}");
                }
            }
            tx.send(true);
        });

        // Wait for the thread to start up.
        let (lock, cvar) = &*pair;
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }
        drop(started); //++artie, unbelievable, manual mem management...

        println!("Copying thread started. Proceeding to spawn watch thread.");
        let _handle_read = s.spawn(move || loop {
            match rx.try_recv() {
                Ok(res) => {
                    if res {
                        //eprintln!("Received end of copying msg");
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
                    let percent = (len as f64 / selected_item_len as f64) * 100_f64;
                    cb_sink
                        .send(Box::new(move |siv| {
                            update_copy_dlg(siv, selected_item_n, total_items, percent as u64)
                        }))
                        .unwrap();
                }
                Err(e) => {
                    eprintln!("couldn't open: {e}");
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(250));
        });
    });
}

fn deselect_copied_item(s: &mut Cursive, copied_item_inx: usize) {
    s.call_on_name(
        LEFT_TABLE_VIEW_NAME,
        |table: &mut TableView<DirView, BasicColumn>| {
            table.deselect_item(copied_item_inx);
        },
    );
}

fn update_copy_dlg(s: &mut Cursive, selected_item_n: u64, total_items: u64, percent: u64) {
    s.call_on_name("copied_n_of_x", |text_view: &mut TextView| {
        text_view.set_content(format!("Copied {selected_item_n} of {total_items}",));
    });
    s.call_on_name("cpy_progress", |progress_bar: &mut ProgressBar| {
        progress_bar.set_value(percent as usize);
    });
    match s.call_on_name("cpy_percent", |text_view: &mut TextView| {
        text_view.set_content(format!("{percent}"));
    }) {
        Some(_) => {
            eprintln!("update_copy_dlg success: {}", percent)
        }
        None => {
            eprintln!("update_copy_dlg NOT success: {}", percent)
        }
    }
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
pub struct copying_job {
    pub source: String,
    pub target: String,
    pub cb_sink: CbSink,
    pub inx: usize,
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
            let (interrupt_tx, interrupt_rx) = crossbeam::channel::unbounded();
            //std::thread::spawn(move || {
            //    crate::utils::cp_machinery::signal_handlers::await_interrupt(interrupt_tx)
            //});
            let interrupt_tx_clone_1 = interrupt_tx.clone();
            let interrupt_tx_clone_2 = interrupt_tx.clone();

            let mut cpy_dlg = Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new("").with_name("copied_n_of_x"))
                    .child(
                        LinearLayout::horizontal()
                            .child(TextView::new("Copied: ")) //++artie, just format!
                            .child(TextView::new("").with_name("cpy_percent"))
                            .child(TextView::new("%")),
                    )
                    .child(ProgressBar::new().with_name("cpy_progress"))
                    .child(
                        LinearLayout::vertical()
                            .child(
                                TextView::new("Errors detected:")
                                    .max_height(0)
                                    .with_name("error_list_label"), /*++artie, 0 == invisible ;) */
                            )
                            .child(ScrollView::new(ListView::new().with_name("error_list"))),
                    ),
            )
            .button("Cancel", move |s| {
                eprintln!("Cancelling copy ops");
                interrupt_tx_clone_1.send(nix::sys::signal::Signal::SIGTERM);
            })
            .button("Pause", move |s| {
                interrupt_tx.send(nix::sys::signal::Signal::SIGSTOP);
            })
            .button_hidden("Continue", move |s| {
                interrupt_tx_clone_2.send(nix::sys::signal::Signal::SIGCONT);
            })
            .button("Background", |s| {})
            .title("Copy")
            .with_name("cpy_dlg");
            let cpy_dlg = cpy_dlg.max_height(15);
            s.add_layer(cpy_dlg);
            let cb_sink_clone = s.cb_sink().clone();
            let mut copying_jobs: Vec<copying_job> = Vec::new();
            for (inx, selected_item) in selected_items {
                match PathBuf::from(&selected_item).file_name() {
                    Some(file_name) => {
                        //std::thread::scope(|scoped| {
                        let full_dest_path =
                            format!("{}/{}", &dest_path, os_string_to_lossy_string(&file_name));
                        //eprintln!("full_dest_path: {full_dest_path}");
                        //let dest_path_clone = dest_path.clone();
                        //let full_dest_path_clone = full_dest_path.clone();
                        //let (tx, rx) = std::sync::mpsc::sync_channel(1);

                        let cb_sink = s.cb_sink().clone();
                        copying_jobs.push(copying_job {
                            source: selected_item.clone(),
                            target: full_dest_path.clone(),
                            cb_sink,
                            inx,
                            // selected_items.len(),
                        });
                        //copying_engine(&selected_item, &full_dest_path, cb_sink);

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
            /*Copying in separate thread so GUI isn't blocked*/
            let cb_sink = s.cb_sink().clone();
            std::thread::spawn(move || {
                use crate::utils::cp_machinery::cp_utils::update_copy_dlg_with_error;
                let (snd, rcv) = std::sync::mpsc::channel();
                let srv_thread = std::thread::spawn(move || {
                    cp_server_main(snd, cb_sink, &update_copy_dlg_with_error, interrupt_rx)
                });
                let _ = rcv.recv();
                if let Err(e) =
                    cp_client_main(copying_jobs, &update_copy_dlg, &deselect_copied_item)
                {
                    eprintln!("Error during copying:{}", e);
                }

                srv_thread.join();
                match cb_sink_clone.send(Box::new(|s| {
                    close_cpy_dlg(s);
                })) {
                    Ok(_) => {
                        eprintln!("Sending close_cpy_dlg successfull");
                    }
                    Err(e) => {
                        eprintln!("Sending close_cpy_dlg NOT successfull: {}", e);
                    }
                }
            });
            /* std::thread::spawn(move || {
                let copying_jobs_len = copying_jobs.len();
                for (inx, copying_job) in copying_jobs.iter().enumerate() {
                    let selected_item = copying_job.0.clone();
                    let full_destination_path = copying_job.1.clone();
                    let cb_sink = copying_job.2.clone();
                    //let cb_sink_clone = cb_sink.clone(); //++artie only needed at the end
                    let handle = std::thread::spawn(move || {
                        copying_engine(
                            &selected_item,
                            inx as u64,
                            copying_jobs_len as u64,
                            &full_destination_path,
                            cb_sink,
                        );
                    });
                    handle.join(); //and we make suer that we are copying in organized, well defined order
                    eprintln!("Finished copying: {}", inx);
                }

                match cb_sink_clone.send(Box::new(|s| {
                    close_cpy_dlg(s);
                })) {
                    Ok(_) => {
                        eprintln!("Sending close_cpy_dlg successfull")
                    }
                    Err(e) => {
                        eprintln!("Sending close_cpy_dlg NOT successfull: {}", e)
                    }
                }
            });
            */
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
