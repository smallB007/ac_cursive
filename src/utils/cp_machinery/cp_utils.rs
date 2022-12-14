use cursive::{
    views::{Dialog, LayerPosition, ListView, ResizedView, TextContent, TextView},
    Cursive,
};

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
