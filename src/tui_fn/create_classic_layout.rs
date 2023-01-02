use std::{cell::RefCell, rc::Rc};

use crate::{
    definitions::definitions::{
        LEFT_PANEL_NAME, LEFT_TABLE_VIEW_NAME, RIGHT_PANEL_NAME, RIGHT_TABLE_VIEW_NAME,
    },
    tui_fn::create_panel::{create_panel, update_table},
    utils::common_utils::get_current_path_from_dialog_name,
};
use crate::{
    tui_fn::create_classic_buttons::create_classic_buttons, utils::common_utils::init_watcher,
};
use cursive::{direction::Orientation, views::CircularFocus};
use cursive::{
    views::{
        Dialog, DummyView, HideableView, LinearLayout, NamedView, ResizedView, StackView, TextView,
    },
    CbSink,
};
pub fn create_classic_layout(left_dir: &str, right_dir: &str, cb_sink: CbSink) -> LinearLayout {
    #[cfg(manual_debug)]
    {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    let mut layout_panes = LinearLayout::new(Orientation::Horizontal);
    let cb_sink_clone = cb_sink.clone();
    let (tx_change_in_dir_detected, rx_change_in_dir_detected) = std::sync::mpsc::channel();
    let left_watcher = match init_watcher(
        LEFT_TABLE_VIEW_NAME.to_owned(),
        left_dir.to_owned(),
        tx_change_in_dir_detected,
    ) {
        Ok(watcher) => {
            std::thread::spawn(move || {
                for info in rx_change_in_dir_detected.iter() {
                    if cb_sink
                        .send(Box::new(move |s| {
                            let path = get_current_path_from_dialog_name(s, LEFT_PANEL_NAME);
                            update_table(s, &path, &info.table_view_name);
                        }))
                        .is_err()
                    {
                        eprintln!("Err: create_classic_layout::cb_sink::send");
                    }
                }
            });
            Some(Rc::new(RefCell::new(watcher)))
        }
        Err(e) => {
            eprintln!("Error: could not init watcher, reason: {}", e);
            None
        }
    };

    layout_panes.add_child(create_panel(LEFT_PANEL_NAME, left_dir, None, left_watcher));
    let (tx_change_in_dir_detected, rx_change_in_dir_detected) = std::sync::mpsc::channel();

    let right_watcher = match init_watcher(
        RIGHT_TABLE_VIEW_NAME.to_owned(),
        right_dir.to_owned(),
        tx_change_in_dir_detected,
    ) {
        Ok(watcher) => {
            std::thread::spawn(move || {
                for info in rx_change_in_dir_detected.iter() {
                    //++artie, do async on this feature
                    if cb_sink_clone
                        .send(Box::new(move |s| {
                            let path = get_current_path_from_dialog_name(s, RIGHT_PANEL_NAME);

                            update_table(s, &path, &info.table_view_name);
                        }))
                        .is_err()
                    {
                        eprintln!("Err: create_classic_layout::cb_sink::send");
                    }
                }
            });
            Some(Rc::new(RefCell::new(watcher)))
        }
        Err(e) => {
            eprintln!("Error: could not init watcher, reason: {}", e);
            None
        }
    };
    layout_panes.add_child(create_panel(
        RIGHT_PANEL_NAME,
        right_dir,
        None,
        right_watcher,
    ));

    let layout_circular_panes = CircularFocus::new(layout_panes);
    let layout_circular_panes =
        layout_circular_panes.wrap_tab().wrap_up_down(/*won't go to the function keys */);

    let buttons = create_classic_buttons();
    let layout = LinearLayout::vertical()
        .child(layout_circular_panes)
        .child(buttons);

    layout
}
