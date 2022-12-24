use crate::utils::cp_machinery::cp_types::{copy_job, CopyJobs, GLOBAL_DATA};
use once_cell::sync::Lazy;
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Condvar, Mutex,
    },
    thread::JoinHandle,
};

pub fn init_cp_sequence(copy_jobs_feed_rx: Receiver<CopyJobs>) {
    std::thread::spawn(|| {
        server_thread(copy_jobs_feed_rx);
    });
}

fn server_thread(copy_jobs_feed_rx: Receiver<CopyJobs>) {
    std::thread::scope(|s| {
        s.spawn(move || {
            eprintln!("[SERVER] Trying to get data");
            for copy_jobs in copy_jobs_feed_rx.try_iter() {
                eprintln!("[SERVER] Processing Data filled by client");
                for cp_job in copy_jobs {
                    perform_op(cp_job);
                }
                eprintln!("[SERVER] Waiting for 2 sec for new data");
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
        });
    });

    eprintln!("[SERVER] Exiting >>>>>>>>>>>>>>>>>>>>");
}

fn perform_op(data: copy_job) {
    eprintln!(
        "[LONG OP]>>>>>>>>>BEGIN: from: { } to: {}",
        data.source, data.target
    );
    std::thread::sleep(std::time::Duration::from_secs(1));
    eprintln!(
        "[LONG OP]>>>>>>>>>END: from: {} to: {}",
        data.source, data.target
    );
}
