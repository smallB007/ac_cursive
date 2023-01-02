use crossbeam::channel::{
    self, after, select, tick, Receiver as Crossbeam_Receiver, Sender as Crossbeam_Sender,
};
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
use thiserror::Error;

#[derive(Clone)]
pub struct copy_job {
    pub source: String,
    pub target: String,
    pub cb_sink: CbSink,
    pub inx: usize, //++artie, not needed
}

pub type CopyJobs = VecDeque<copy_job>; //++artie, Vec?

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

pub struct InterruptComponents<'a> {
    pub job: copy_job,
    pub interrupt_rx: &'a Crossbeam_Receiver<nix::sys::signal::Signal>,
    pub break_condition: Arc<Mutex<bool>>,
}

pub enum ExistingPathDilemma {
    Skip(bool /*all */),
    Overwrite(bool),
    ReplaceOlder(bool),
    ReplaceNewer(bool),
    DifferentSizes(bool),
}

#[derive(Clone)]
pub struct UpdateInfo {
    pub table_view_name: String,
    pub path: String,
}
