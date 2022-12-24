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

#[derive(Clone)]
pub struct copy_job {
    pub source: String,
    pub target: String,
    pub cb_sink: CbSink,
    pub inx: usize,
}

pub type CopyJobs = VecDeque<copy_job>;

pub static GLOBAL_DATA: Lazy<Mutex<CopyJobs>> = Lazy::new(|| {
    let m = CopyJobs::new();
    Mutex::new(m)
});
