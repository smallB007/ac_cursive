use std::path::PathBuf;

use cursive::{
    views::{
        Dialog, LayerPosition, ListView, NamedView, ProgressBar, ResizedView, StackView,
        TextContent, TextView,
    },
    CbSink, Cursive,
};

use crossbeam::channel::{
    self, after, select, tick, Receiver as Crossbeam_Receiver, Sender as Crossbeam_Sender,
};
use cursive_table_view::TableView;

use crate::{
    definitions::definitions::*,
    tui_fn::{
        create_cp_dlg::create_cp_dlg,
        create_table::{BasicColumn, DirView},
    },
    utils::{
        common_utils::*,
        cp_machinery::{cp_client_main::cp_client_main, cp_server_main::cp_server_main},
    },
};

pub struct copying_job {
    pub source: String,
    pub target: String,
    pub cb_sink: CbSink,
    pub inx: usize,
}
fn deselect_copied_item(s: &mut Cursive, copied_item_inx: usize) {
    s.call_on_name(
        LEFT_TABLE_VIEW_NAME,
        |table: &mut TableView<DirView, BasicColumn>| {
            table.deselect_item(copied_item_inx);
        },
    );
}
pub fn update_copy_dlg(s: &mut Cursive, selected_item_n: u64, total_items: u64, percent: u64) {
    s.call_on_name("copied_n_of_x", |text_view: &mut TextView| {
        text_view.set_content(format!("Copied {selected_item_n} of {total_items}",));
    });
    s.call_on_all_named("cpy_progress", |progress_bar: &mut ProgressBar| {
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

pub fn update_copy_dlg_with_error(s: &mut Cursive, error: String) {
    s.call_on_name(
        "error_list_label",
        |text_view: &mut ResizedView<TextView>| {
            text_view.set_height(cursive::view::SizeConstraint::Fixed((1)))
        },
    );
    s.call_on_name("error_list", |list_view: &mut ListView| {
        list_view.add_child("label", TextView::new_with_content(TextContent::new(error)));
    });
}

pub fn cpy_dlg_show_continue_btn(s: &mut Cursive) {
    s.call_on_name("cpy_dlg", move |dlg: &mut Dialog| {
        dlg.show_button("<Continue>", "<Pause>");
    });
}

pub fn cpy_dlg_show_pause_btn(s: &mut Cursive) {
    s.call_on_name("cpy_dlg", move |dlg: &mut Dialog| {
        dlg.show_button("<Pause>", "<Continue>");
    });
}
pub fn show_cpy_dlg(s: &mut Cursive) -> bool {
    s.call_on_name(
        //++artie rfctr
        "copy_stack_view",
        |copy_stack_view: &mut StackView| match copy_stack_view
            .find_layer_from_name("copy_progress_layout")
        {
            Some(inx) => {
                if inx == LayerPosition::FromFront(1) {
                    copy_stack_view.move_to_back(LayerPosition::FromFront(0));
                }
            }
            None => {}
        },
    );
    match s.call_on_name("cpy_dlg", |_: &mut Dialog| true) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(v) => {
            if v == true {
                s.screen_mut().move_layer(
                    cursive::views::LayerPosition::FromBack(0),
                    cursive::views::LayerPosition::FromFront(0),
                );
            }

            v
        }
        None => false,
    }
}

pub fn hide_cpy_dlg(s: &mut Cursive) -> bool {
    s.call_on_name(
        //++artie rfctr
        "copy_stack_view",
        |copy_stack_view: &mut StackView| match copy_stack_view
            .find_layer_from_name_like_human_being("copy_progress_layout")
        {
            Some(inx) => {
                if inx == 0 {
                    copy_stack_view.move_to_front(LayerPosition::FromBack(0));
                }
            }
            None => {}
        },
    );
    match s.call_on_name("cpy_dlg", |_: &mut Dialog| true) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(v) => {
            if v == true {
                s.screen_mut().move_to_back(LayerPosition::FromFront(0));
            }

            v
        }
        None => false,
    }
}

pub fn close_cpy_dlg(s: &mut Cursive) {
    s.call_on_name(
        //++artie rfctr
        "copy_stack_view",
        |copy_stack_view: &mut StackView| match copy_stack_view
            .find_layer_from_name_like_human_being("copy_progress_layout")
        {
            Some(inx) => {
                if inx == 1 {
                    copy_stack_view.move_to_back(LayerPosition::FromFront(0));
                }
            }
            None => {}
        },
    );
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

pub fn f5_handler(s: &mut Cursive) {
    if show_cpy_dlg(s) {
        return;
    }
    let ((src_table, _), (_, dest_panel)) = if get_active_table_name(s) == LEFT_TABLE_VIEW_NAME {
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
    create_cp_dlg(s, interrupt_tx, interrupt_tx_clone_1, interrupt_tx_clone_2);
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
        if let Err(e) = cp_client_main(copying_jobs, &update_copy_dlg, &deselect_copied_item) {
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
}
