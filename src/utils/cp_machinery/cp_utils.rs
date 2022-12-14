use cursive::{
    views::{
        Dialog, LayerPosition, ListView, NamedView, ProgressBar, ResizedView, StackView,
        TextContent, TextView,
    },
    Cursive,
};

pub fn update_copy_dlg(s: &mut Cursive, selected_item_n: u64, total_items: u64, percent: u64) {
    s.call_on_name("copied_n_of_x", |text_view: &mut TextView| {
        text_view.set_content(format!("Copied {selected_item_n} of {total_items}",));
    });
    s.call_on_all_named("cpy_progress", |progress_bar: &mut ProgressBar| {
        progress_bar.set_value(percent as usize);
    });
    match s.call_on_name("cpy_percent", |text_view: &mut TextView| {
        text_view.set_content(format!("{percent}"));
    }) {
        Some(_) => {
            eprintln!("update_copy_dlg success: {}", percent)
        }
        None => {
            eprintln!("update_copy_dlg NOT success: {}", percent)
        }
    }
}

pub fn update_copy_dlg_with_error(s: &mut Cursive, error: String) {
    s.call_on_name(
        "error_list_label",
        |text_view: &mut ResizedView<TextView>| {
            text_view.set_height(cursive::view::SizeConstraint::Fixed((1)))
        },
    );
    s.call_on_name("error_list", |list_view: &mut ListView| {
        list_view.add_child("label", TextView::new_with_content(TextContent::new(error)));
    });
}
use crossbeam::channel::{
    self, after, select, tick, Receiver as Crossbeam_Receiver, Sender as Crossbeam_Sender,
};
pub fn cpy_dlg_show_continue_btn(s: &mut Cursive) {
    s.call_on_name("cpy_dlg", move |dlg: &mut Dialog| {
        dlg.show_button("<Continue>", "<Pause>");
    });
}

pub fn cpy_dlg_show_pause_btn(s: &mut Cursive) {
    s.call_on_name("cpy_dlg", move |dlg: &mut Dialog| {
        dlg.show_button("<Pause>", "<Continue>");
    });
}
pub fn show_cpy_dlg(s: &mut Cursive) -> bool {
    s.call_on_name("copy_stack_view", |copy_stack_view: &mut StackView| {
        copy_stack_view.move_to_front(LayerPosition::FromBack(0));
    });
    match s.call_on_name("cpy_dlg", |_: &mut Dialog| true) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(v) => {
            if v == true {
                s.screen_mut().move_layer(
                    cursive::views::LayerPosition::FromBack(0),
                    cursive::views::LayerPosition::FromFront(0),
                );
            }

            v
        }
        None => false,
    }
}

pub fn hide_cpy_dlg(s: &mut Cursive) -> bool {
    s.call_on_name("copy_stack_view", |copy_stack_view: &mut StackView| {
        copy_stack_view.move_to_back(LayerPosition::FromFront(0));
    });
    match s.call_on_name("cpy_dlg", |_: &mut Dialog| true) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(v) => {
            if v == true {
                s.screen_mut().move_to_back(LayerPosition::FromFront(0));
            }

            v
        }
        None => false,
    }
}

pub fn close_cpy_dlg(s: &mut Cursive) {
    s.call_on_name(
        //++artie rfctr
        "copy_stack_view",
        |copy_stack_view: &mut StackView| {
            copy_stack_view.move_to_front(LayerPosition::FromBack(0));
        },
    );
    match s.call_on_name("cpy_dlg", |_: &mut Dialog| true) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(v) => {
            if v == true {
                s.pop_layer();
            }
        }
        None => {}
    }
}
