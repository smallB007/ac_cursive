use cursive::{
    views::{Dialog, DummyView, LinearLayout, TextView},
    Cursive,
};

use super::cp_utils::show_info_themed_view;

pub fn show_no_paths_selected_dlg(s: &mut Cursive) {
    let info = Dialog::around(
        LinearLayout::vertical()
            .child(DummyView)
            .child(TextView::new("Please select path to copy")),
    )
    .title("No path selected")
    .button_raw("[ OK ]", |s| {
        s.pop_layer();
    });
    show_info_themed_view(s, info);
}
