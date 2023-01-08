use cursive::{
    view::{Nameable, Resizable},
    views::{
        Dialog, DummyView, EditView, LinearLayout, SelectView, TextContent, TextContentRef,
        TextView,
    },
    Cursive,
};

use crate::{
    definitions::definitions::{FIND_CONTENT, FIND_FILE_NAME, FIND_STARTING_DIR_NAME},
    utils::cp_machinery::{
        copy::execute_process,
        cp_utils::{show_error_themed_view, show_info_themed_view, show_result_themed_view},
    },
};

fn create_find_dlg() -> Dialog {
    Dialog::new()
        .title("Find")
        // Padding is (left, right, top, bottom)
        .padding_lrtb(1, 1, 1, 0)
        .content(
            LinearLayout::vertical()
                .child(
                    LinearLayout::horizontal()
                        .child(TextView::new("Start dir:"))
                        .child(DummyView)
                        .child(
                            EditView::new()
                                .content("/home/artie/Desktop/Coumbo")
                                // Call `show_popup` when the user presses `Enter`
                                .on_submit(show_popup)
                                // Give the `EditView` a name so we can refer to it later.
                                .with_name(FIND_STARTING_DIR_NAME) // Wrap this in a `ResizedView` with a fixed width.
                                // Do this _after_ `with_name` or the name will point to the
                                // `ResizedView` instead of `EditView`!
                                .min_width(20),
                        ),
                )
                .child(DummyView)
                .child(
                    LinearLayout::horizontal()
                        .child(TextView::new("File name:"))
                        .child(DummyView)
                        .child(
                            EditView::new()
                                .content("1.txt")
                                // Call `show_popup` when the user presses `Enter`
                                .on_submit(show_popup)
                                // Give the `EditView` a name so we can refer to it later.
                                .with_name(FIND_FILE_NAME) // Wrap this in a `ResizedView` with a fixed width.
                                // Do this _after_ `with_name` or the name will point to the
                                // `ResizedView` instead of `EditView`!
                                .min_width(20),
                        ),
                )
                .child(DummyView)
                .child(
                    LinearLayout::horizontal()
                        .child(TextView::new("Content:"))
                        .child(DummyView)
                        .child(
                            EditView::new()
                                // Call `show_popup` when the user presses `Enter`
                                .on_submit(show_popup)
                                // Give the `EditView` a name so we can refer to it later.
                                .with_name(FIND_CONTENT) // Wrap this in a `ResizedView` with a fixed width.
                                // Do this _after_ `with_name` or the name will point to the
                                // `ResizedView` instead of `EditView`!
                                .min_width(20),
                        ),
                ),
        )
        .button("Ok", |s| {
            let starting_dir = get_find_X(s, FIND_STARTING_DIR_NAME);
            eprintln!("Starting dir: {}", starting_dir);
            let file_name = get_find_X(s, FIND_FILE_NAME);
            eprintln!("File name: {}", file_name);
            let content = get_find_X(s, FIND_CONTENT);
            eprintln!("Content: {}", content);
            let output = execute_process("fd", &["--glob", &file_name, &starting_dir], None);
            if output.std_err.len() != 0 {
                let dlg = Dialog::around(TextView::new(output.std_err))
                    .title("Errors detected")
                    .dismiss_button("OK");
                show_error_themed_view(s, dlg);
            } else {
                let mut select_view = SelectView::new();
                for item in output.std_out.lines() {
                    select_view.add_item_str(item);
                }
                let dlg = Dialog::around(select_view)
                    .title("Find results")
                    .dismiss_button("OK");
                show_result_themed_view(s, dlg);
            }
        })
}
pub fn open_find_dlg(s: &mut Cursive) {
    let dlg = create_find_dlg();
    s.add_layer(dlg);
}

// This will replace the current layer with a new popup.
// If the name is empty, we'll show an error message instead.
fn show_popup(s: &mut Cursive, name: &str) {
    if name.is_empty() {
        // Try again as many times as we need!
        s.add_layer(Dialog::info("Please enter a name!"));
    } else {
        let content = format!("Hello {}!", name);
        // Remove the initial popup
        s.pop_layer();
        // And put a new one instead
        s.add_layer(Dialog::around(TextView::new(content)).button("Quit", |s| s.quit()));
    }
}

fn get_find_X(s: &mut Cursive, view_name: &str) -> String {
    s.call_on_name(view_name, |text_view: &mut EditView| {
        (*text_view.get_content()).clone()
    })
    .unwrap_or_default()
}
