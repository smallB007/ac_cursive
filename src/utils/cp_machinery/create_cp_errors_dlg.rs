use cursive::{
    view::{Nameable, Resizable},
    views::{
        Dialog, DummyView, LinearLayout, ListView, NamedView, ScrollView, SelectView, TextContent,
        TextView,
    },
    Cursive,
};

use crate::definitions::definitions::{CPY_ERRORS_DLG_NAME, ERRORS_LIST_NAME};

use super::{
    cp_types::ExitInfo,
    cp_utils::{close_dlg, show_error_themed_view},
};

pub fn display_cp_errors_dlg(s: &mut Cursive, errors: Vec<ExitInfo>) {
    let dlg = create_cp_errors_dlg(errors);

    show_error_themed_view(s, dlg);
}
fn create_cp_errors_dlg(errors: Vec<ExitInfo>) -> NamedView<Dialog> {
    let mut list_view = SelectView::new();
    for error in &errors {
        let lbl: String = error.process.to_string();
        let content: String = format!("{}", "error.exit_status");
        list_view.add_item(
            &lbl,
            //&error.process,
            TextView::new_with_content(TextContent::new(content)),
        );
    }
    Dialog::around(
        LinearLayout::vertical()
            .child(DummyView)
            .child(TextView::new("Errors during copying detected"))
            .child(DummyView)
            .child(ScrollView::new(list_view).max_height(3))
            .child(DummyView),
    )
    .button_raw("[Dismiss]", |s| {
        close_dlg(s, CPY_ERRORS_DLG_NAME);
    })
    .title("Errors detected")
    .with_name(CPY_ERRORS_DLG_NAME)
}
