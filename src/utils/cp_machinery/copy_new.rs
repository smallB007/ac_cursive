use crate::utils::cp_machinery::cp_types::{copy_job, CopyJobs, GLOBAL_DATA};
use cursive::CbSink;
use once_cell::sync::Lazy;
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Condvar, Mutex,
    },
    thread::JoinHandle,
};

pub fn init_cp_sequence(copy_jobs_feed_rx: Receiver<CopyJobs>, cb_sink: CbSink) {
    server_thread(copy_jobs_feed_rx, cb_sink);
}

fn server_thread(copy_jobs_feed_rx: Receiver<CopyJobs>, cb_sink: CbSink) {
    std::thread::spawn(move || {
        eprintln!("[SERVER] Trying to get data");
        for copy_jobs in copy_jobs_feed_rx.try_iter() {
            eprintln!("[SERVER] Processing Data filled by client");
            for cp_job in copy_jobs {
                perform_op(cp_job);
            }
        }
        eprintln!("[SERVER] Exiting >>>>>>>>>>>>>>>>>>>>");
        if cb_sink
            .send(Box::new(|s| {
                s.set_user_data(());
            }))
            .is_err()
        {
            eprintln!("Err 1: cb_sink.send");
        }
    });
}

fn perform_op(data: copy_job) {
    eprintln!(
        "[LONG OP]>>>>>>>>>BEGIN: from: { } to: {}",
        data.source, data.target
    );
    std::thread::sleep(std::time::Duration::from_secs(5));
    eprintln!(
        "[LONG OP]>>>>>>>>>END: from: {} to: {}",
        data.source, data.target
    );
}
