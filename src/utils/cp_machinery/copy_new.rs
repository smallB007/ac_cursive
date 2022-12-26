use crate::utils::cp_machinery::{
    cp_types::{copy_job, CopyJobs},
    cp_utils::{close_cpy_dlg, open_cpy_dlg, update_cpy_dlg},
};
use cursive::CbSink;
use nix::sys::signal::Signal;
use once_cell::sync::Lazy;
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Condvar, Mutex,
    },
    thread::JoinHandle,
};

pub fn init_cp_sequence(copy_jobs_feed_rx: Receiver<CopyJobs>, cb_sink: CbSink) {
    server_thread(copy_jobs_feed_rx, cb_sink);
}
fn open_cpy_dlg_hlpr(cb_sink: CbSink) -> Crossbeam_Receiver<nix::sys::signal::Signal> {
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
fn close_cpy_dlg_hlpr(cb_sink: CbSink) {
    if cb_sink
        .send(Box::new(|s| {
            s.set_user_data(());
            close_cpy_dlg(s);
        }))
        .is_err()
    {
        eprintln!("Err 1: cb_sink.send");
    }
}
fn rm_dest(target: &str) {}
fn enter_cpy_loop(interrupt_rx: Crossbeam_Receiver<Signal>, copy_jobs_feed_rx: Receiver<CopyJobs>) {
    eprintln!("[SERVER] Trying to get data");
    for copy_jobs in copy_jobs_feed_rx.try_iter() {
        eprintln!("[SERVER] Processing Data filled by client");
        for cp_job in copy_jobs {
            execute_process("rm", &["-f", &cp_job.target], None);
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
    let watch_progress_handle = create_watch_progress_thread(
        tx_progress,
        job.source.clone(),
        job.target.clone(),
        job.cb_sink.clone(),
    );
    rx_progress.recv();
    cp_path_new(job, &interrupt_rx);
    watch_progress_handle.join();
    eprintln!("[COPYING] FINISHED");
}

use crate::utils::cp_machinery::cp_types::Cp_error;
use crossbeam::channel::{
    self, after, select, tick, Receiver as Crossbeam_Receiver, Sender as Crossbeam_Sender,
};
use signal_hook::consts::*;
use signal_hook::iterator::Signals;
use std::io::prelude::*;
use std::process::{Command, Stdio};
fn cp_path_new(
    //++artie, use execute process
    job: copy_job,
    interrupt_rx: &Crossbeam_Receiver<nix::sys::signal::Signal>,
) -> Cp_error {
    let mut process = match Command::new("cp")
        .arg("-f")
        .arg(job.source)
        .arg(job.target)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Err(why) => {
            return Cp_error::CP_COULDNOT_START;
        }
        Ok(process) => process,
    };

    let timeout = tick(std::time::Duration::from_secs(2));
    loop {
        select! {
            recv(interrupt_rx) -> interrupt_rx_result => {
                println!("Received interrupt notification:{:?}",interrupt_rx_result);
                let id = process.id();
                match interrupt_rx_result
                {
                    Ok(nix::sys::signal::Signal::SIGSTOP)=>{
                        nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGSTOP);
                        job.cb_sink.send(Box::new(|s|{crate::utils::cp_machinery::cp_utils::cpy_dlg_show_continue_btn(s)}));
                    },
                    Ok(nix::sys::signal::Signal::SIGCONT)=>{
                        nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGCONT);
                        job.cb_sink.send(Box::new(|s|{crate::utils::cp_machinery::cp_utils::cpy_dlg_show_pause_btn(s)}));
                    },
                    Ok(nix::sys::signal::Signal::SIGTERM)=>{
                        nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGCONT);
                        nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGTERM);
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
                    Err(e) => {eprintln!("error attempting to wait: {e}");break;},
                }
            }
        }
        eprintln!("AFTER SELECT>>>>>>>>>>>>>>>>>>>>>>");
    }
    eprintln!("AFTER LOOP>>>>>>>>>>>>>>>>>>>>>>");

    {
        let mut buf = String::new();
        match process.stderr.unwrap().read_to_string(&mut buf) {
            Err(why) => {
                return Cp_error::CP_COULDNOT_READ_STDERR;
            }
            Ok(_) => {
                if buf.len() != 0 {
                    return Cp_error::CP_EXIT_STATUS_ERROR(buf);
                }
            }
        }
    }
    Cp_error::CP_EXIT_STATUS_SUCCESS
}

fn create_watch_progress_thread(
    snd_progress_watch: Sender<()>,
    selected_item: String,
    full_dest_path: String,
    cb_sink: CbSink,
    //break_condition: Arc<Mutex<bool>>,
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
            let full_dest_path_clone = full_dest_path.clone();
            match std::fs::File::open(full_dest_path_clone) {
                Ok(f) => {
                    let len = f.metadata().unwrap().len();
                    let percent = if len == selected_item_len || selected_item_len == 0 {
                        100 //case where original file is zero length
                    } else {
                        ((len as f64 / selected_item_len as f64) * 100_f64) as u64
                    };

                    // eprintln!("percent,  {percent}");
                    cb_sink
                        .send(Box::new(move |siv| {
                            update_cpy_dlg(
                                siv, /*selected_item_n*/ 0, /*total_items */ 0, percent,
                            );
                        }))
                        .unwrap();

                    if percent >= 100 {
                        eprintln!("exiting percent,  {percent}");
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("couldn't open: {e}");
                }
            }

            //{
            //    match break_condition.try_lock() {
            //        Ok(mutex_guard) => {
            //            if *mutex_guard == true {
            //                break;
            //            }
            //        }
            //        Err(_) => {}
            //    }
            //}
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    });

    progress_watch_thread_handle
}
struct InterruptComponents<'a> {
    job: copy_job,
    interrupt_rx: &'a Crossbeam_Receiver<nix::sys::signal::Signal>,
}
fn execute_process(
    process: &str,
    args: &[&str],
    interrupt_component: Option<InterruptComponents>,
) -> Cp_error {
    let mut process = match Command::new(process)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Err(why) => {
            return Cp_error::CP_COULDNOT_START;
        }
        Ok(process) => process,
    };
    match interrupt_component {
        Some(interrupt_component) => {
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
                                nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGCONT);
                                nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGTERM);
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
                            Err(e) => {eprintln!("error attempting to wait: {e}");break;},
                        }
                    }
                }
                eprintln!("AFTER SELECT>>>>>>>>>>>>>>>>>>>>>>");
            }
        }
        None => {}
    }

    eprintln!("AFTER LOOP>>>>>>>>>>>>>>>>>>>>>>");

    {
        let mut buf = String::new();
        match process.stderr.unwrap().read_to_string(&mut buf) {
            Err(why) => {
                return Cp_error::CP_COULDNOT_READ_STDERR;
            }
            Ok(_) => {
                if buf.len() != 0 {
                    return Cp_error::CP_EXIT_STATUS_ERROR(buf);
                }
            }
        }
    }
    Cp_error::CP_EXIT_STATUS_SUCCESS
}
