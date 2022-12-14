use crate::{
    cursive::view::{Nameable, Resizable},
    utils::cp_machinery::cp_utils::hide_cpy_dlg,
};
use crossbeam::channel::{
    self, after, select, tick, Receiver as Crossbeam_Receiver, Sender as Crossbeam_Sender,
};
use cursive::views::{
    Dialog, LayerPosition, LinearLayout, ListView, NamedView, ProgressBar, ScrollView, TextView,
};
pub fn create_cp_dlg(
    s: &mut cursive::Cursive,
    interrupt_tx: Crossbeam_Sender<nix::sys::signal::Signal>,
    interrupt_tx_clone_2: Crossbeam_Sender<nix::sys::signal::Signal>,
    interrupt_tx_clone_1: Crossbeam_Sender<nix::sys::signal::Signal>,
) {
    let mut cpy_dlg = Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new("").with_name("copied_n_of_x"))
            .child(
                LinearLayout::horizontal()
                    .child(TextView::new("Copied: ")) //++artie, just format!
                    .child(TextView::new("").with_name("cpy_percent"))
                    .child(TextView::new("%")),
            )
            .child(ProgressBar::new().with_name("cpy_progress"))
            .child(
                LinearLayout::vertical()
                    .child(
                        TextView::new("Errors detected:")
                            .max_height(0)
                            .with_name("error_list_label"), /*++artie, 0 == invisible ;) */
                    )
                    .child(ScrollView::new(ListView::new().with_name("error_list"))),
            ),
    )
    .button("Cancel", move |s| {
        eprintln!("Cancelling copy ops");
        interrupt_tx_clone_1.send(nix::sys::signal::Signal::SIGTERM);
    })
    .button("Pause", move |s| {
        interrupt_tx.send(nix::sys::signal::Signal::SIGSTOP);
    })
    .button_hidden("Continue", move |s| {
        interrupt_tx_clone_2.send(nix::sys::signal::Signal::SIGCONT);
    })
    .button("Background", |s| {
        hide_cpy_dlg(s);
    })
    .title("Copy")
    .with_name("cpy_dlg");
    let cpy_dlg = cpy_dlg.max_height(15);
    s.add_layer(cpy_dlg);
}
