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
    pub inx: usize, //++artie, not needed
}

pub type CopyJobs = VecDeque<copy_job>;
#[cfg(unused)]
pub static GLOBAL_DATA: Lazy<Mutex<CopyJobs>> = Lazy::new(|| {
    let m = CopyJobs::new();
    Mutex::new(m)
});

use thiserror::Error;
#[derive(Error, Debug)]
pub enum Cp_error {
    #[error("Source does not exist")]
    CP_SOURCE_DOESNOT_EXIST,
    #[error("Target does not exist")]
    CP_TARGET_DOESNOT_EXIST,
    #[error("Could not start cp process")]
    CP_COULDNOT_START,
    #[error("Could not read stderr")]
    CP_COULDNOT_READ_STDERR,
    #[error("Could not read stdout")]
    CP_COULDNOT_READ_STDOUT,
    #[error("")]
    CP_EXIT_STATUS_ERROR(String),
    #[error("")]
    CP_EXIT_STATUS_SUCCESS,
}
