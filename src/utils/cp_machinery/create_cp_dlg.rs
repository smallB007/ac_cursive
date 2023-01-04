use crossbeam::channel::{
    self, after, select, tick, Receiver as Crossbeam_Receiver, Sender as Crossbeam_Sender,
};
use cursive::{
    view::{Nameable, Resizable},
    views::{Dialog, LinearLayout, NamedView, ProgressBar, ResizedView, TextView},
};

use crate::definitions::definitions::{CPY_DLG_NAME, CPY_PROGRESSBAR_NAME};

use super::cp_utils::set_dlg_visible;

pub fn create_cp_dlg(
    interrupt_tx_pause: Crossbeam_Sender<nix::sys::signal::Signal>,
    interrupt_tx_continue: Crossbeam_Sender<nix::sys::signal::Signal>,
    interrupt_tx_cancel: Crossbeam_Sender<nix::sys::signal::Signal>,
) -> ResizedView<NamedView<Dialog>> {
    let cpy_dlg = Dialog::around(
        LinearLayout::vertical()
            .child(
                LinearLayout::horizontal()
                    .child(TextView::new("Copying: "))
                    .child(TextView::new("").with_name("copied_n_of_x"))
                    .child(TextView::new(" of "))
                    .child(TextView::new("").with_name("total_items")),
            )
            .child(
                LinearLayout::vertical()
                    .child(TextView::new("From: "))
                    .child(TextView::new("").with_name("source_path")),
            )
            .child(
                LinearLayout::vertical()
                    .child(TextView::new("To: "))
                    .child(TextView::new("").with_name("target_path")),
            )
            .child(ProgressBar::new().with_name(CPY_PROGRESSBAR_NAME)),
    )
    .button("Cancel", move |s| {
        eprintln!("Cancelling copy ops");
        interrupt_tx_cancel.send(nix::sys::signal::Signal::SIGTERM);
    })
    .button("Pause", move |s| {
        interrupt_tx_pause.send(nix::sys::signal::Signal::SIGSTOP);
    })
    .button_hidden("Continue", move |s| {
        interrupt_tx_continue.send(nix::sys::signal::Signal::SIGCONT);
    })
    .button("Background", |s| {
        set_dlg_visible(s, CPY_DLG_NAME, false);
    })
    .title("Copy")
    .with_name(CPY_DLG_NAME);
    let cpy_dlg = cpy_dlg.max_height(15);

    cpy_dlg
}
