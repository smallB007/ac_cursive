use crate::{
    definitions::definitions::CPY_DLG_NAME,
    utils::cp_machinery::{
        cp_types::{copy_job, CopyJobs, ExistingPathDilemma, EXIT_PROCESS_STATUS},
        cp_utils::{
            check_if_path_exists, close_cpy_dlg_hlpr, compare_paths_for_modification_time,
            compare_paths_for_size, open_cpy_dlg_hlpr, set_dlg_visible_hlpr,
            show_and_update_cpy_dlg_with_total_count, show_path_exists_dlg_hlpr,
            update_cpy_dlg_current_item_number_hlpr,
            update_cpy_dlg_current_item_source_target_hlpr, update_cpy_dlg_progress,
        },
    },
};
use crossbeam::channel::{
    self, after, select, tick, Receiver as Crossbeam_Receiver, Sender as Crossbeam_Sender,
};
use cursive::CbSink;
use nix::sys::signal::Signal;
use once_cell::sync::Lazy;
use signal_hook::consts::*;
use signal_hook::iterator::Signals;
use std::process::{ChildStderr, Command, Stdio};
use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Condvar, Mutex,
    },
    thread::JoinHandle,
};
use std::{io::prelude::*, process::ChildStdout};

use super::cp_types::InterruptComponents;

pub fn init_cp_sequence(copy_jobs_feed_rx: Receiver<CopyJobs>, cb_sink: CbSink) {
    server_thread(copy_jobs_feed_rx, cb_sink);
}

fn enter_cpy_loop(interrupt_rx: Crossbeam_Receiver<Signal>, copy_jobs_feed_rx: Receiver<CopyJobs>) {
    eprintln!("[SERVER] Trying to get data");
    let mut overwrite_all_flag = false;
    let mut skip_all_flag = false;
    let mut replace_all_newer_flag = false;
    let mut replace_all_older_flag = false;
    let mut replace_all_different_size_flag = false;
    for copy_jobs in copy_jobs_feed_rx.try_iter() {
        eprintln!("[SERVER] Processing Data filled by client");
        show_and_update_cpy_dlg_with_total_count(
            copy_jobs[0].cb_sink.clone(),
            copy_jobs.len() as u64,
        );
        for (inx, cp_job) in copy_jobs.into_iter().enumerate() {
            if !overwrite_all_flag {
                if check_if_path_exists(&cp_job.target) {
                    if skip_all_flag {
                        continue;
                    }
                    let source_target =
                        compare_paths_for_modification_time(&cp_job.source, &cp_job.target);
                    eprintln!("Ordering:{:?}", source_target);
                    let is_target_older_than_source = source_target == Ordering::Greater;
                    let is_target_newer_than_source = source_target == Ordering::Less;
                    let is_source_and_target_different_size =
                        compare_paths_for_size(&cp_job.source, &cp_job.target) != Ordering::Equal;
                    let replace_oldr_combo = replace_all_older_flag && is_target_older_than_source;
                    let replace_nwr_combo = replace_all_newer_flag && is_target_newer_than_source;
                    let replace_sizes_combo =
                        replace_all_different_size_flag && is_source_and_target_different_size;
                    let whole_exp = !(replace_all_older_flag && is_target_older_than_source)
                        && !(replace_all_newer_flag && is_target_newer_than_source)
                        && !(replace_all_different_size_flag
                            && is_source_and_target_different_size);
                    if !(replace_all_older_flag && is_target_older_than_source)
                        && !(replace_all_newer_flag && is_target_newer_than_source)
                        && !(replace_all_different_size_flag && is_source_and_target_different_size)
                    {
                        let (tx, rx) = std::sync::mpsc::channel();
                        set_dlg_visible_hlpr(cp_job.cb_sink.clone(), CPY_DLG_NAME, false);

                        show_path_exists_dlg_hlpr(
                            cp_job.cb_sink.clone(),
                            cp_job.source.to_owned(),
                            cp_job.target.to_owned(),
                            tx,
                        );
                        match rx.recv() {
                            Ok(existing_path_dilemma) => match existing_path_dilemma {
                                ExistingPathDilemma::Overwrite(true) => {
                                    eprintln!("Overwrite all");
                                    overwrite_all_flag = true;
                                }
                                ExistingPathDilemma::Overwrite(false) => {
                                    eprintln!("Overwrite current");
                                }
                                ExistingPathDilemma::Skip(true) => {
                                    eprintln!("Skip all");
                                    skip_all_flag = true;
                                    continue;
                                }
                                ExistingPathDilemma::Skip(false) => {
                                    eprintln!("Skip current");
                                    continue;
                                }
                                ExistingPathDilemma::ReplaceOlder(true) => {
                                    replace_all_older_flag = true;
                                    if !is_target_older_than_source {
                                        continue;
                                    }
                                }
                                ExistingPathDilemma::ReplaceOlder(false) => {
                                    if !is_target_older_than_source {
                                        continue;
                                    }
                                }
                                ExistingPathDilemma::ReplaceNewer(true) => {
                                    replace_all_newer_flag = true;
                                    if !is_target_newer_than_source {
                                        continue;
                                    }
                                }
                                ExistingPathDilemma::ReplaceNewer(false) => {
                                    if !is_target_newer_than_source {
                                        continue;
                                    }
                                }
                                ExistingPathDilemma::DifferentSizes(true) => {
                                    replace_all_different_size_flag = true;
                                    if !is_source_and_target_different_size {
                                        continue;
                                    }
                                }
                                ExistingPathDilemma::DifferentSizes(false) => {
                                    if !is_source_and_target_different_size {
                                        continue;
                                    }
                                }
                            },
                            Err(e) => {
                                return;
                            }
                        }
                    }
                }

                set_dlg_visible_hlpr(cp_job.cb_sink.clone(), CPY_DLG_NAME, true);
            }
            execute_process("rm", &["-f", &cp_job.target], None);
            update_cpy_dlg_current_item_number_hlpr(cp_job.cb_sink.clone(), (inx + 1) as u64);
            update_cpy_dlg_current_item_source_target_hlpr(
                cp_job.cb_sink.clone(),
                cp_job.source.clone(),
                cp_job.target.clone(),
            );
            perform_op(cp_job, &interrupt_rx);
        }
    }
    eprintln!("[SERVER] Exiting >>>>>>>>>>>>>>>>>>>>");
}

fn server_thread(copy_jobs_feed_rx: Receiver<CopyJobs>, cb_sink: CbSink) {
    std::thread::spawn(move || {
        let interrupt_rx = open_cpy_dlg_hlpr(cb_sink.clone());
        enter_cpy_loop(interrupt_rx, copy_jobs_feed_rx);
        close_cpy_dlg_hlpr(cb_sink);
    });
}

fn perform_op(job: copy_job, interrupt_rx: &Crossbeam_Receiver<nix::sys::signal::Signal>) {
    eprintln!("[COPYING] START: from: { } to: {}", job.source, job.target);
    let (tx_progress, rx_progress) = std::sync::mpsc::channel();
    let break_condition = Arc::new(Mutex::new(false));
    let break_condition_clone_1 = break_condition.clone();
    let break_condition_clone_2 = break_condition.clone();
    let watch_progress_handle = create_watch_progress_thread(
        tx_progress,
        job.source.clone(),
        job.target.clone(),
        job.cb_sink.clone(),
        break_condition_clone_1,
    );
    rx_progress.recv();
    execute_process(
        "cp",
        &["-f", &job.source.clone(), &job.target.clone()],
        Some(InterruptComponents {
            job,
            interrupt_rx,
            break_condition: break_condition_clone_2,
        }),
    );

    watch_progress_handle.join();
}

fn create_watch_progress_thread(
    snd_progress_watch: Sender<()>,
    selected_item: String,
    full_dest_path: String,
    cb_sink: CbSink,
    break_condition: Arc<Mutex<bool>>,
) -> JoinHandle<()> {
    let progress_watch_thread_handle = std::thread::spawn(move || {
        snd_progress_watch.send(()); //sync point, let know that the thread started
        let selected_item_len = match PathBuf::from(&selected_item).metadata() {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                eprintln!("Couldn't get len for path: {}", selected_item);
                0
            }
        };
        loop {
            match break_condition.try_lock() {
                Ok(mutex_guard) => {
                    if *mutex_guard == true {
                        break;
                    }
                }
                _ => {}
            }

            //let full_dest_path_clone = full_dest_path.clone();
            match std::fs::File::open(&full_dest_path) {
                Ok(f) => {
                    let len = f.metadata().unwrap().len();
                    let percent = if len == selected_item_len || selected_item_len == 0 {
                        100 //case where original file is zero length
                    } else {
                        ((len as f64 / selected_item_len as f64) * 100_f64) as u64
                    };
                    cb_sink
                        .send(Box::new(move |siv| {
                            update_cpy_dlg_progress(siv, percent);
                        }))
                        .unwrap();

                    if percent >= 100 {
                        eprintln!("exiting percent,  {percent}");
                        return;
                    }
                }
                Err(e) => {
                    //panic!("couldn't open: {e}");
                    eprintln!("couldn't open: {e}");
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    });

    progress_watch_thread_handle
}

fn execute_process(
    process_name: &str,
    args: &[&str],
    interrupt_component: Option<InterruptComponents>,
) -> EXIT_PROCESS_STATUS {
    let mut exit_process_status = EXIT_PROCESS_STATUS::EXIT_STATUS_SUCCESS;
    let mut process = match Command::new(process_name)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Err(why) => {
            exit_process_status = EXIT_PROCESS_STATUS::COULD_NOT_START;
            None
        }
        Ok(process) => Some(process),
    };

    match process {
        Some(mut process) => {
            match interrupt_component {
                Some(ref interrupt_component) => {
                    let timeout = tick(std::time::Duration::from_secs(2));
                    loop {
                        select! {
                            recv(interrupt_component.interrupt_rx) -> interrupt_rx_result => {
                                println!("Received interrupt notification:{:?}",interrupt_rx_result);
                                let id = process.id();
                                match interrupt_rx_result
                                {
                                    Ok(nix::sys::signal::Signal::SIGSTOP)=>{
                                        nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGSTOP);
                                        interrupt_component.job.cb_sink.send(Box::new(|s|{crate::utils::cp_machinery::cp_utils::cpy_dlg_show_continue_btn(s)}));
                                    },
                                    Ok(nix::sys::signal::Signal::SIGCONT)=>{
                                        nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGCONT);
                                        interrupt_component.job.cb_sink.send(Box::new(|s|{crate::utils::cp_machinery::cp_utils::cpy_dlg_show_pause_btn(s)}));
                                    },
                                    Ok(nix::sys::signal::Signal::SIGTERM)=>{
                                        /*Two steps:
                                        a) kill process
                                        b) set flag to true so the watch progress thread can finish */
                                        nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGCONT);
                                        nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGTERM);
                                        exit_process_status = EXIT_PROCESS_STATUS::CANCELLED;
                                        break;
                                    },
                                    _=>{}
                                }
                              },
                            recv(timeout) -> _ => {
                                eprintln!("Checking if we finished the long task");
                                match process.try_wait() {
                                    Ok(Some(status)) =>{ eprintln!("exited with: {status}");break;},
                                    Ok(None) => {
                                        eprintln!("status not ready yet");
                                    }
                                    Err(e) => {eprintln!("error attempting to wait: {e}");
                                    exit_process_status = EXIT_PROCESS_STATUS::EXIT_STATUS_ERROR(format!("Could not wait: {}",e));
                                    break;},
                                }
                            }
                        }
                        eprintln!("AFTER SELECT>>>>>>>>>>>>>>>>>>>>>>");
                    }
                }

                None => {}
            }

            eprintln!("AFTER LOOP>>>>>>>>>>>>>>>>>>>>>>");

            if exit_process_status == EXIT_PROCESS_STATUS::EXIT_STATUS_SUCCESS {
                read_std_stream(
                    process.stderr,
                    &mut exit_process_status,
                    EXIT_PROCESS_STATUS::COULD_NOT_READ_STDERR("".to_owned()),
                );
            }
            if exit_process_status == EXIT_PROCESS_STATUS::EXIT_STATUS_SUCCESS {
                read_std_stream(
                    process.stdout,
                    &mut exit_process_status,
                    EXIT_PROCESS_STATUS::COULD_NOT_READ_STDOUT("".to_owned()),
                );
            }
        }
        None => {}
    }

    if interrupt_component.is_some() {
        //++artie, so progresswatch thread is definitely cancelled
        signal_flag(&interrupt_component.unwrap());
    }
    eprintln!("{}", format!("{} FINISHED", process_name));
    exit_process_status
}

fn signal_flag(interrupt_component: &InterruptComponents) {
    let mut mutex_guard = interrupt_component.break_condition.lock().unwrap();
    *mutex_guard = true;
}

fn read_std_stream<T: std::io::Read>(
    std_stream: Option<T>,
    exit_process_status: &mut EXIT_PROCESS_STATUS,
    err_flag: EXIT_PROCESS_STATUS,
) {
    let mut buf = String::new();

    match std_stream.unwrap().read_to_string(&mut buf) {
        Err(why) => {
            if err_flag == EXIT_PROCESS_STATUS::COULD_NOT_READ_STDERR("".to_owned()) {
                *exit_process_status = EXIT_PROCESS_STATUS::COULD_NOT_READ_STDERR(why.to_string());
            } else if err_flag == EXIT_PROCESS_STATUS::COULD_NOT_READ_STDOUT("".to_owned()) {
                *exit_process_status = EXIT_PROCESS_STATUS::COULD_NOT_READ_STDOUT(why.to_string());
            }
        }
        Ok(_) => {
            if buf.len() != 0 {
                *exit_process_status = EXIT_PROCESS_STATUS::EXIT_STATUS_ERROR(buf.clone());
            }
        }
    }
}
