use crossbeam::channel::{
    self, after, select, Receiver as Crossbeam_Receiver, Sender as Crossbeam_Sender,
};
use signal_hook::consts::*;
use signal_hook::iterator::Signals;

pub fn await_interrupt(interrupt_notification_channel: Crossbeam_Sender<()>) {
    let mut signals = Signals::new(&[
        // 1
        SIGINT,
    ])
    .unwrap();

    for s in &mut signals {
        // 2
        interrupt_notification_channel.send(()); // 3
    }
}
