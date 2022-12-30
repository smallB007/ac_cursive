use crate::cursive::view::{Nameable, Resizable};
use crossbeam::channel::{
    self, after, select, tick, Receiver as Crossbeam_Receiver, Sender as Crossbeam_Sender,
};
use cursive::{
    self,
    theme::{self, Theme},
    views,
    views::{
        Dialog, LayerPosition, LinearLayout, ListView, NamedView, ProgressBar, ResizedView,
        ScrollView, StackView, TextContent, TextView, ThemedView,
    },
    CbSink, Cursive, View, With,
};
use cursive_table_view::TableView;
use futures::SinkExt;
use once_cell::sync::Lazy;
use std::os::unix::prelude::PermissionsExt;
use std::time::SystemTime;
use std::{cmp::Ordering, fs::Permissions};
use std::{collections::VecDeque, path::PathBuf, sync::mpsc::Sender};

use crate::{
    definitions::definitions::*,
    tui_fn::create_table::{BasicColumn, DirView},
    utils::{
        common_utils::*,
        //cp_machinery::cp_client_main::cp_client_main,
        cp_machinery::cp_types::{copy_job, CopyJobs},
    },
};
fn deselect_copied_item(s: &mut Cursive, copied_item_inx: usize) {
    s.call_on_name(
        LEFT_TABLE_VIEW_NAME,
        |table: &mut TableView<DirView, BasicColumn>| {
            table.deselect_item(copied_item_inx);
        },
    );
}

pub fn update_cpy_dlg_progress(s: &mut Cursive, percent: u64) {
    //++artie, change name to update_progress
    //s.call_on_name("copied_n_of_x", |text_view: &mut TextView| {
    //    text_view.set_content(format!("{selected_item_n}",));
    //});
    //s.call_on_name("total_items", |text_view: &mut TextView| {
    //    text_view.set_content(format!("{total_items}",));
    //});
    s.call_on_all_named("cpy_progress", |progress_bar: &mut ProgressBar| {
        progress_bar.set_value(percent as usize);
    });
}

pub fn update_cpy_dlg_current_item_number_hlpr(cb_sink: CbSink, current_item_no: u64) {
    cb_sink.send(Box::new(move |s| {
        update_cpy_dlg_current_item_number(s, current_item_no);
    }));
}

pub fn update_cpy_dlg_current_item_number(s: &mut Cursive, current_item_no: u64) {
    s.call_on_name("copied_n_of_x", |text_view: &mut TextView| {
        text_view.set_content(format!("{current_item_no}",)); //++artie, must be number only, otherwise parsing will panic
    });
}

pub fn update_cpy_dlg_current_item_source_target_hlpr(
    cb_sink: CbSink,
    souce: String,
    target: String,
) {
    cb_sink.send(Box::new(move |s| {
        update_cpy_dlg_current_item_source_target(s, souce, target);
    }));
}
pub fn update_cpy_dlg_current_item_source_target(s: &mut Cursive, souce: String, target: String) {
    s.call_on_name("source_path", |text_view: &mut TextView| {
        text_view.set_content(souce); //++artie, must be number only, otherwise parsing will panic
    });
    s.call_on_name("target_path", |text_view: &mut TextView| {
        text_view.set_content(target); //++artie, must be number only, otherwise parsing will panic
    });
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
    //++artie refactor to show button + lbl
    s.call_on_name(CPY_DLG_NAME, move |dlg: &mut Dialog| {
        dlg.show_button("<Continue>", "<Pause>");
    });
}

pub fn cpy_dlg_show_pause_btn(s: &mut Cursive) {
    s.call_on_name(CPY_DLG_NAME, move |dlg: &mut Dialog| {
        dlg.show_button("<Pause>", "<Continue>");
    });
}
pub fn set_dlg_visible_hlpr(cb_sink: CbSink, dlg_name: &'static str, visible: bool) {
    cb_sink.send(Box::new(move |s| {
        set_dlg_visible(s, &dlg_name, visible);
    }));
}
#[cfg(unused)]
pub fn show_dlg_hlpr(cb_sink: CbSink, dlg_name: &'static str) {
    cb_sink.send(Box::new(move |s| {
        show_dlg(s, &dlg_name);
    }));
}
pub fn set_dlg_visible(s: &mut Cursive, dlg_name: &str, visible: bool) {
    match s.call_on_name(dlg_name, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(dlg_name)
            {
                Some(inx) => {
                    if visible {
                        s.screen_mut().move_to_front(LayerPosition::FromBack(inx));
                    } else {
                        s.screen_mut().move_to_back(LayerPosition::FromBack(inx));
                    }
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
pub fn show_dlg(s: &mut Cursive, dlg_name: &str) {
    //++artie, deprecated, use set_dlg_visible
    match s.call_on_name(dlg_name, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(dlg_name)
            {
                Some(inx) => {
                    s.screen_mut().move_to_front(LayerPosition::FromBack(inx));
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
pub fn hide_dlg(s: &mut Cursive, dlg_name: &str) {
    //++artie, deprecated, use set_dlg_visible
    match s.call_on_name(dlg_name, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(dlg_name)
            {
                Some(inx) => {
                    s.screen_mut().move_to_back(LayerPosition::FromBack(inx));
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
pub fn show_cpy_dlg(s: &mut Cursive) {
    //++artie, deprecated use, show_dlg
    s.call_on_name(
        //++artie rfctr
        "copy_stack_view",
        |copy_stack_view: &mut StackView| match copy_stack_view
            .find_layer_from_name_like_human_being("copy_progress_layout")
        {
            Some(inx) => {
                copy_stack_view.move_to_back(LayerPosition::FromBack(inx));
            }
            None => {}
        },
    );
    match s.call_on_name(CPY_DLG_NAME, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(CPY_DLG_NAME)
            {
                Some(inx) => {
                    s.screen_mut().move_to_front(LayerPosition::FromBack(inx));
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
#[cfg(unused)]
pub fn hide_dlg_hlpr(cb_sink: CbSink, dlg_name: &'static str) {
    cb_sink.send(Box::new(move |s| {
        hide_dlg(s, dlg_name);
    }));
}
#[cfg(unused)]
pub fn hide_cpy_dlg(s: &mut Cursive, show_progress_on_cpy_btn: bool) {
    //++artie, deprecated, use hide_dlg
    s.call_on_name(
        //++artie rfctr
        "copy_stack_view",
        |copy_stack_view: &mut StackView| match copy_stack_view
            .find_layer_from_name_like_human_being("copy_progress_layout")
        {
            Some(inx) => {
                if show_progress_on_cpy_btn {
                    copy_stack_view.move_to_front(LayerPosition::FromBack(inx));
                } else {
                    copy_stack_view.move_to_back(LayerPosition::FromBack(inx));
                }
            }
            None => {}
        },
    );
    match s.call_on_name(CPY_DLG_NAME, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(CPY_DLG_NAME)
            {
                Some(inx) => {
                    s.screen_mut().move_to_back(LayerPosition::FromBack(inx));
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
pub fn open_cpy_dlg(
    s: &mut Cursive,
    interrupt_tx_pause: Crossbeam_Sender<nix::sys::signal::Signal>,
    interrupt_tx_continue: Crossbeam_Sender<nix::sys::signal::Signal>,
    interrupt_tx_cancel: Crossbeam_Sender<nix::sys::signal::Signal>,
) {
    let cpy_dlg = create_cp_dlg(
        interrupt_tx_pause,
        interrupt_tx_continue,
        interrupt_tx_cancel,
    );
    s.add_layer(cpy_dlg);
}
pub fn close_dlg(s: &mut Cursive, dlg_name: &str) {
    match s.call_on_name(dlg_name, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(dlg_name)
            {
                Some(inx) => {
                    s.screen_mut().remove_layer(LayerPosition::FromBack(inx));
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
pub fn close_cpy_dlg(s: &mut Cursive) {
    //++artie, deprecated, use close_dlg
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
    match s.call_on_name(CPY_DLG_NAME, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(CPY_DLG_NAME)
            {
                Some(inx) => {
                    s.screen_mut().remove_layer(LayerPosition::FromBack(inx));
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
fn transfer_copying_jobs(
    copying_jobs: Vec<copy_job>,
    jobs_sender_tx: std::sync::mpsc::Sender<Vec<copy_job>>,
    rx_client_thread_started: std::sync::mpsc::Receiver<()>,
) {
    rx_client_thread_started.recv();
    jobs_sender_tx.send(copying_jobs);
}
#[cfg(unused)]
pub fn f5_handler_interprocess(s: &mut Cursive) {
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
    let selected_items = get_active_table_selected_items(s, src_table, true);
    //eprintln!("{:?}", selected_items);
    let dest_path = get_current_path_from_dialog_name(s, String::from(dest_panel));

    let mut copying_jobs: Vec<copy_job> = Vec::new();
    for (inx, selected_item) in selected_items {
        match PathBuf::from(&selected_item).file_name() {
            Some(file_name) => {
                //std::thread::scope(|scoped| {
                let full_dest_path =
                    format!("{}/{}", &dest_path, os_string_to_lossy_string(&file_name));

                let cb_sink = s.cb_sink().clone();
                copying_jobs.push(copy_job {
                    source: selected_item.clone(),
                    target: full_dest_path.clone(),
                    cb_sink,
                    inx,
                });
            }
            None => {
                eprintln!("Couldn't copy {selected_item}");
            }
        }
    }
    show_cpy_dlg(s);
    if s.user_data::<std::sync::mpsc::Sender<Vec<copy_job>>>()
        .is_some()
    {
        let sender: &mut std::sync::mpsc::Sender<Vec<copy_job>> = s.user_data().unwrap();
        sender.send(copying_jobs);
    } else {
        let (jobs_sender_tx, jobs_receiver_rx) = std::sync::mpsc::channel();
        let (client_thread_started_tx, client_thread_started_rx) = std::sync::mpsc::channel();
        let copying_jobs_clone = copying_jobs.clone();
        let jobs_sender_clone = jobs_sender_tx.clone();
        let transfer_copying_jobs_handle = std::thread::spawn(move || {
            transfer_copying_jobs(copying_jobs_clone, jobs_sender_tx, client_thread_started_rx);
        });
        s.set_user_data(jobs_sender_clone);

        //    if show_cpy_dlg(s) {
        //        return;
        //    }
        //eprintln!("dest_path: {}", dest_path);
        let (interrupt_tx, interrupt_rx) = crossbeam::channel::unbounded();
        //std::thread::spawn(move || {
        //    crate::utils::cp_machinery::signal_handlers::await_interrupt(interrupt_tx)
        //});
        let interrupt_tx_clone_1 = interrupt_tx.clone();
        let interrupt_tx_clone_2 = interrupt_tx.clone();
        create_cp_dlg(s, interrupt_tx, interrupt_tx_clone_1, interrupt_tx_clone_2);
        let cb_sink_clone = s.cb_sink().clone();

        /*Copying in separate thread so GUI isn't blocked*/
        let cb_sink = s.cb_sink().clone();
        let cb_sink_for_client_thread = s.cb_sink().clone();
        std::thread::spawn(move || {
            use crate::utils::cp_machinery::cp_utils::update_copy_dlg_with_error;
            let (snd, rcv) = std::sync::mpsc::channel();
            let srv_thread = std::thread::spawn(move || {
                // cp_server_main(snd, cb_sink, &update_copy_dlg_with_error, interrupt_rx)
            });
            let _ = rcv.recv();
            if let Err(e) = cp_client_main(
                copying_jobs,
                &update_cpy_dlg_progress,
                &show_cpy_dlg,
                &hide_cpy_dlg,
                jobs_receiver_rx,
                client_thread_started_tx,
                cb_sink_for_client_thread,
            ) {
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
    }
    /* std::thread::spawn(move || {
        let copying_jobs_len = copying_jobs.len();
        for (inx, copy_job) in copying_jobs.iter().enumerate() {
            let selected_item = copy_job.0.clone();
            let full_destination_path = copy_job.1.clone();
            let cb_sink = copy_job.2.clone();
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
fn prepare_cp_jobs(s: &mut Cursive) -> CopyJobs {
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
    let selected_items = get_active_table_selected_items(s, src_table, true);
    //eprintln!("{:?}", selected_items);
    let dest_path = get_current_path_from_dialog_name(s, String::from(dest_panel));

    let mut copying_jobs = CopyJobs::new();
    for (inx, selected_item) in selected_items {
        match PathBuf::from(&selected_item).file_name() {
            Some(file_name) => {
                let full_dest_path =
                    format!("{}/{}", &dest_path, os_string_to_lossy_string(&file_name));

                let cb_sink = s.cb_sink().clone();
                copying_jobs.push_back(copy_job {
                    source: selected_item.clone(),
                    target: full_dest_path.clone(),
                    cb_sink,
                    inx,
                });
            }
            None => {
                eprintln!("Couldn't copy {selected_item}");
            }
        }
    }

    copying_jobs
}

use super::{create_cp_dlg::create_cp_dlg, create_path_exists_dlg::create_path_exists_dlg};
use crate::utils::cp_machinery::copy_new::init_cp_sequence;

pub fn open_cpy_dlg_hlpr(cb_sink: CbSink) -> Crossbeam_Receiver<nix::sys::signal::Signal> {
    let (interrupt_tx_cancel, interrupt_rx) = crossbeam::channel::unbounded();
    let interrupt_tx_continue = interrupt_tx_cancel.clone();
    let interrupt_tx_pause = interrupt_tx_cancel.clone();
    cb_sink.send(Box::new(move |s| {
        open_cpy_dlg(
            s,
            interrupt_tx_pause,
            interrupt_tx_continue,
            interrupt_tx_cancel,
        );
    }));

    interrupt_rx
}
pub fn close_cpy_dlg_hlpr(cb_sink: CbSink) {
    if cb_sink
        .send(Box::new(|s| {
            s.set_user_data(());
            close_cpy_dlg(s);
        }))
        .is_err()
    {
        eprintln!("Err close_cpy_dlg_hlpr");
    }
}
pub fn show_cpy_dlg_hlpr(cb_sink: CbSink) {
    if cb_sink
        .send(Box::new(|s| {
            crate::utils::cp_machinery::cp_utils::show_cpy_dlg(s);
        }))
        .is_err()
    {
        eprintln!("Err show_cpy_dlg_hlpr");
    }
}
pub fn show_and_update_cpy_dlg_with_total_count(cb_sink: CbSink, total_count: u64) {
    let cb_sink_a = cb_sink.clone();
    let cb_sink_b = cb_sink_a.clone();
    show_cpy_dlg_hlpr(cb_sink_a);
    update_cpy_dlg_with_new_items_hlpr(cb_sink_b, total_count);
}
pub fn update_cpy_dlg_with_new_items_hlpr(cb_sink: CbSink, new_items_count: u64) {
    if cb_sink
        .send(Box::new(move |s| {
            crate::utils::cp_machinery::cp_utils::update_cpy_dlg_with_new_items(s, new_items_count);
        }))
        .is_err()
    {
        eprintln!("Err show_cpy_dlg_hlpr");
    }
}
pub fn update_cpy_dlg_with_new_items(s: &mut Cursive, total_items: u64) {
    s.call_on_name("total_items", |text_view: &mut TextView| {
        let total_so_far = match text_view.get_content().source().parse::<u64>() {
            Ok(val) => val,
            Err(_) => 0,
        };
        let new_total = total_so_far + total_items;
        text_view.set_content(format!("{new_total}",)); //++artie, must be number only, otherwise parsing will panic
    });
}

pub fn f5_handler(s: &mut Cursive) {
    let cp_jobs = prepare_cp_jobs(s);

    if s.user_data::<std::sync::mpsc::Sender<CopyJobs>>().is_some() {
        let tx_cp_jobs: &mut std::sync::mpsc::Sender<CopyJobs> = s.user_data().unwrap();
        show_and_update_cpy_dlg_with_total_count(cp_jobs[0].cb_sink.clone(), cp_jobs.len() as u64);
        if tx_cp_jobs.send(cp_jobs).is_err() {
            eprintln!("Send err 1: tx_cp_jobs.send(cp_jobs)");
        }
    } else {
        let (tx_cp_jobs, rx_cp_jobs) = std::sync::mpsc::channel();
        if tx_cp_jobs.send(cp_jobs).is_err() {
            eprintln!("Send err 2: tx_cp_jobs.send(cp_jobs)");
        }
        init_cp_sequence(rx_cp_jobs, s.cb_sink().clone());
        s.set_user_data(tx_cp_jobs);
    }
}

pub enum ExistingPathDilemma {
    Skip(bool /*all */),
    Overwrite(bool),
    ReplaceOlder(bool),
    ReplaceNewer(bool),
    DifferentSizes(bool),
}

pub fn show_path_exists_dlg_hlpr(
    cb_sink: CbSink,
    source: String,
    target: String,
    overwrite_current_tx: Sender<ExistingPathDilemma>,
) {
    cb_sink.send(Box::new(move |s| {
        show_path_exists_dlg(s, source, target, overwrite_current_tx);
    }));
}

pub fn show_path_exists_dlg(
    s: &mut Cursive,
    source: String,
    target: String,
    overwrite_current_tx: Sender<ExistingPathDilemma>,
) {
    let source_info = match file_info(&source) {
        Ok(info) => info,
        Err(e) => {
            format!("Couldn't get info for {}, reason: {}", source, e)
        }
    };
    let target_info = match file_info(&target) {
        Ok(info) => info,
        Err(e) => {
            format!("Couldn't get info for {}, reason: {}", source, e)
        }
    };

    let dlg = create_path_exists_dlg(source_info, target_info, overwrite_current_tx);
    show_error_themed_view(s, dlg);
}

fn show_error_themed_view<V: View>(s: &mut cursive::Cursive, dlg: V) {
    let theme = Lazy::new(|| {
        eprintln!("Lazy theme");
        let mut theme = Theme::default();

        theme.palette[theme::PaletteColor::View] = theme::Color::Dark(theme::BaseColor::Red);
        theme.palette[theme::PaletteColor::Primary] = theme::Color::Light(theme::BaseColor::White);
        theme.palette[theme::PaletteColor::TitlePrimary] =
            theme::Color::Light(theme::BaseColor::Yellow);
        theme.palette[theme::PaletteColor::Highlight] = theme::Color::Dark(theme::BaseColor::Green);

        theme
    });

    s.add_layer(views::ThemedView::new(
        theme.clone(),
        views::Layer::new(dlg),
    ));
}
#[cfg(target_os = "linux")]
pub fn file_info_size(file: &str) -> Result<u64, std::io::Error> {
    let metadata = std::fs::metadata(file)?;
    Ok(metadata.len())
}
pub fn compare_paths_for_size(path_a: &str, path_b: &str) -> Ordering {
    let ord = (|| {
        let size_a = file_info_size(path_a).ok()?;
        let size_b = file_info_size(path_b).ok()?;
        Some(size_a.cmp(&size_b))
    })()
    .unwrap_or(Ordering::Equal);
    #[cfg(unused)]
    match file_info_size(path_a) {
        Ok(size_a) => match file_info_size(path_b) {
            Ok(size_b) => match size_a.partial_cmp(&size_b) {
                Some(val) => val,
                None => Ordering::Equal,
            },
            Err(e) => Ordering::Equal,
        },
        Err(e) => Ordering::Equal,
    }
    ord
}
#[cfg(target_os = "linux")]
pub fn file_info_modification_time(file: &str) -> Result<SystemTime, std::io::Error> {
    let metadata = std::fs::metadata(file)?;
    metadata.modified()
}
pub fn compare_paths_for_modification_time(path_a: &str, path_b: &str) -> Ordering {
    let ord = (|| {
        let modification_time_a = file_info_modification_time(path_a).ok()?;
        let modification_time_b = file_info_modification_time(path_b).ok()?;
        modification_time_a.partial_cmp(&modification_time_b)
    })()
    .unwrap_or(Ordering::Equal);

    #[cfg(unused)]
    match file_info_modification_time(path_a) {
        Ok(a_systime) => match file_info_modification_time(path_b) {
            Ok(b_systime) => match a_systime.partial_cmp(&b_systime) {
                Some(ord) => ord,
                None => Ordering::Equal,
            },
            Err(e_b_systime) => Ordering::Equal, //++artie, not sure if that is the best solution, but for now it must do
        },
        Err(e_a_systime) => Ordering::Equal,
    }
    ord
}
#[cfg(target_os = "linux")]
pub fn file_info(file: &str) -> Result<String, std::io::Error> {
    let metadata = std::fs::metadata(file)?;

    let path = format!("Path: {}", file);
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

    Ok(path
        + "\n"
        + &file_type
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
