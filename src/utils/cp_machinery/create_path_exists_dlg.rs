use std::sync::mpsc::Sender;

use cursive::{
    theme::ColorStyle,
    view::{Nameable, Resizable},
    views::{Checkbox, Dialog, DummyView, LinearLayout, NamedView, ResizedView, TextView},
    Cursive,
};

use crate::{
    custom_views::horizontal_line::HorizontalLine,
    definitions::definitions::{CPY_ALL_CHCKBX_NAME, PATH_EXISTS_DLG_NAME},
};

use super::{cp_types::ExistingPathDilemma, cp_utils::close_dlg};
fn is_all_checked(s: &mut Cursive) -> bool {
    s.call_on_name(CPY_ALL_CHCKBX_NAME, |chckbx: &mut Checkbox| {
        chckbx.is_checked()
    })
    .unwrap()
}
pub fn create_path_exists_dlg(
    source: String,
    target: String,
    response_tx: Sender<ExistingPathDilemma>,
) -> NamedView<ResizedView<Dialog>> {
    let max_width = 81_usize;
    let skip_tx = response_tx.clone();
    let overwrite_tx = response_tx.clone();
    let replace_older_tx = response_tx.clone();
    let replace_newer_tx = response_tx.clone();
    let different_size_tx = response_tx.clone();
    let dlg = Dialog::around(
        LinearLayout::vertical()
            .child(
                LinearLayout::vertical()
                    .child(TextView::new("Source").style(ColorStyle::title_primary()))
                    .child(TextView::new(source)),
            )
            .child(DummyView)
            .child(
                LinearLayout::vertical()
                    .child(TextView::new("Target:").style(ColorStyle::title_primary()))
                    .child(TextView::new(target)),
            )
            .child(DummyView)
            .child(HorizontalLine::new("─", max_width))
            .child(
                LinearLayout::horizontal()
                    .child(Checkbox::new().with_name(CPY_ALL_CHCKBX_NAME))
                    .child(TextView::new("All")),
            )
            .child(DummyView),
    )
    .button_raw("[ Skip ]", move |s| {
        let apply_to_all = is_all_checked(s);
        if skip_tx
            .send(ExistingPathDilemma::Skip(apply_to_all))
            .is_err()
        {
            eprintln!("Err send: ExistingPathDilemma::Skip");
        }
        close_dlg(s, PATH_EXISTS_DLG_NAME);
    })
    .button_raw("[ Overwrite ]", move |s| {
        let apply_to_all = is_all_checked(s);

        if overwrite_tx
            .send(ExistingPathDilemma::Overwrite(apply_to_all))
            .is_err()
        {
            eprintln!("Err send: ExistingPathDilemma::Overwrite");
        }
        close_dlg(s, PATH_EXISTS_DLG_NAME);
    })
    .button_raw("[ Replace older ]", move |s| {
        let apply_to_all = is_all_checked(s);

        if replace_older_tx
            .send(ExistingPathDilemma::ReplaceOlder(apply_to_all))
            .is_err()
        {
            eprintln!("Err send: ExistingPathDilemma::ReplaceOlder");
        }
        close_dlg(s, PATH_EXISTS_DLG_NAME);
    })
    .button_raw("[ Replace newer ]", move |s| {
        let apply_to_all = is_all_checked(s);

        if replace_newer_tx
            .send(ExistingPathDilemma::ReplaceNewer(apply_to_all))
            .is_err()
        {
            eprintln!("Err send: ExistingPathDilemma::Replace newer");
        }
        close_dlg(s, PATH_EXISTS_DLG_NAME);
    })
    .button_raw("[ Different size ]", move |s| {
        let apply_to_all = is_all_checked(s);

        if different_size_tx
            .send(ExistingPathDilemma::DifferentSizes(apply_to_all))
            .is_err()
        {
            eprintln!("Err send: ExistingPathDilemma::Different size");
        }
        close_dlg(s, PATH_EXISTS_DLG_NAME);
    })
    .title("Path exists")
    .max_width(max_width)
    .with_name(PATH_EXISTS_DLG_NAME);
    dlg
}
