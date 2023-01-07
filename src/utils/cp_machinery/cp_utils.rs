use super::{
    cp_types::ExistingPathDilemma, create_cp_dlg::create_cp_dlg,
    create_path_exists_dlg::create_path_exists_dlg,
    no_paths_selected_dlg::show_no_paths_selected_dlg,
};
use crate::{
    cursive::view::{Nameable, Resizable},
    tui_fn::create_panel::update_dlg_title,
};
use crate::{
    definitions::definitions::*,
    tui_fn::create_table::{BasicColumn, DirView},
    utils::{
        common_utils::*,
        //cp_machinery::cp_client_main::cp_client_main,
        cp_machinery::cp_types::{copy_job, CopyJobs},
    },
};
use crate::{tui_fn::create_panel::update_table, utils::cp_machinery::copy::init_cp_sequence};
use crossbeam::channel::{
    self, after, select, tick, Receiver as Crossbeam_Receiver, Sender as Crossbeam_Sender,
};
use cursive::{
    self,
    event::Key,
    theme::{self, Theme},
    views,
    views::{
        Dialog, LayerPosition, LinearLayout, ListView, NamedView, ProgressBar, ResizedView,
        ScrollView, StackView, TextContent, TextView, ThemedView,
    },
    CbSink, Cursive, View, With,
};
use cursive_table_view::TableView;
use futures::SinkExt;
use once_cell::sync::Lazy;
use regex::Regex;
use std::os::unix::prelude::PermissionsExt;
use std::time::SystemTime;
use std::{cmp::Ordering, fs::Permissions};
use std::{collections::VecDeque, path::PathBuf, sync::mpsc::Sender};

fn deselect_copied_item(s: &mut Cursive, copied_item_inx: usize) {
    s.call_on_name(
        LEFT_TABLE_VIEW_NAME,
        |table: &mut TableView<DirView, BasicColumn>| {
            table.deselect_item(copied_item_inx);
        },
    );
}

pub fn update_cpy_dlg_progress(s: &mut Cursive, percent: u64) {
    //++artie, change name to update_progress
    //s.call_on_all_named(CPY_PROGRESSBAR_NAME, |progress_bar: &mut ProgressBar| {
    //    progress_bar.set_value(percent as usize);
    //});
    s.call_on_name(CPY_PROGRESSBAR_NAME, |progress_bar: &mut ProgressBar| {
        progress_bar.set_value(percent as usize);
    });
}

pub fn update_cpy_dlg_current_item_number_hlpr(cb_sink: CbSink, current_item_no: u64) {
    cb_sink.send(Box::new(move |s| {
        update_cpy_dlg_current_item_number(s, current_item_no);
    }));
}

pub fn update_cpy_dlg_current_item_number(s: &mut Cursive, current_item_no: u64) {
    s.call_on_name("copied_n_of_x", |text_view: &mut TextView| {
        text_view.set_content(format!("{current_item_no}",)); //++artie, must be number only, otherwise parsing will panic
    });
    //s.call_on_name(CPY_PROGRESSBAR_NAME, |progress_bar: &mut ProgressBar| {
    //    progress_bar.set_value(10 as usize);
    //});
}

pub fn update_cpy_dlg_current_item_source_target_hlpr(
    cb_sink: CbSink,
    souce: String,
    target: String,
) {
    cb_sink.send(Box::new(move |s| {
        update_cpy_dlg_current_item_source_target(s, souce, target);
    }));
}
pub fn update_cpy_dlg_current_item_source_target(s: &mut Cursive, souce: String, target: String) {
    s.call_on_name("source_path", |text_view: &mut TextView| {
        text_view.set_content(souce); //++artie, must be number only, otherwise parsing will panic
    });
    s.call_on_name("target_path", |text_view: &mut TextView| {
        text_view.set_content(target); //++artie, must be number only, otherwise parsing will panic
    });
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

pub fn cpy_dlg_show_continue_btn(s: &mut Cursive) {
    //++artie refactor to show button + lbl
    s.call_on_name(CPY_DLG_NAME, move |dlg: &mut Dialog| {
        dlg.show_button("<Continue>", "<Pause>");
    });
}

pub fn cpy_dlg_show_pause_btn(s: &mut Cursive) {
    s.call_on_name(CPY_DLG_NAME, move |dlg: &mut Dialog| {
        dlg.show_button("<Pause>", "<Continue>");
    });
}
pub fn set_dlg_visible_hlpr(cb_sink: CbSink, dlg_name: &'static str, visible: bool) {
    cb_sink.send(Box::new(move |s| {
        set_dlg_visible(s, &dlg_name, visible);
    }));
}
#[cfg(unused)]
pub fn show_dlg_hlpr(cb_sink: CbSink, dlg_name: &'static str) {
    cb_sink.send(Box::new(move |s| {
        show_dlg(s, &dlg_name);
    }));
}
pub fn set_dlg_visible(s: &mut Cursive, dlg_name: &str, visible: bool) {
    match s.call_on_name(dlg_name, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(dlg_name)
            {
                Some(inx) => {
                    if visible {
                        s.screen_mut().move_to_front(LayerPosition::FromBack(inx));
                    } else {
                        s.screen_mut().move_to_back(LayerPosition::FromBack(inx));
                    }
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
pub fn show_dlg(s: &mut Cursive, dlg_name: &str) {
    //++artie, deprecated, use set_dlg_visible
    match s.call_on_name(dlg_name, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(dlg_name)
            {
                Some(inx) => {
                    s.screen_mut().move_to_front(LayerPosition::FromBack(inx));
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
pub fn hide_dlg(s: &mut Cursive, dlg_name: &str) {
    //++artie, deprecated, use set_dlg_visible
    match s.call_on_name(dlg_name, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(dlg_name)
            {
                Some(inx) => {
                    s.screen_mut().move_to_back(LayerPosition::FromBack(inx));
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
pub fn show_cpy_dlg(s: &mut Cursive) {
    //++artie, deprecated use, show_dlg
    s.call_on_name(
        //++artie rfctr
        "copy_stack_view",
        |copy_stack_view: &mut StackView| match copy_stack_view
            .find_layer_from_name_like_human_being("copy_progress_layout")
        {
            Some(inx) => {
                copy_stack_view.move_to_back(LayerPosition::FromBack(inx));
            }
            None => {}
        },
    );
    match s.call_on_name(CPY_DLG_NAME, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(CPY_DLG_NAME)
            {
                Some(inx) => {
                    s.screen_mut().move_to_front(LayerPosition::FromBack(inx));
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
#[cfg(unused)]
pub fn hide_dlg_hlpr(cb_sink: CbSink, dlg_name: &'static str) {
    cb_sink.send(Box::new(move |s| {
        hide_dlg(s, dlg_name);
    }));
}
#[cfg(unused)]
pub fn hide_cpy_dlg(s: &mut Cursive, show_progress_on_cpy_btn: bool) {
    //++artie, deprecated, use hide_dlg
    s.call_on_name(
        //++artie rfctr
        "copy_stack_view",
        |copy_stack_view: &mut StackView| match copy_stack_view
            .find_layer_from_name_like_human_being("copy_progress_layout")
        {
            Some(inx) => {
                if show_progress_on_cpy_btn {
                    copy_stack_view.move_to_front(LayerPosition::FromBack(inx));
                } else {
                    copy_stack_view.move_to_back(LayerPosition::FromBack(inx));
                }
            }
            None => {}
        },
    );
    match s.call_on_name(CPY_DLG_NAME, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(CPY_DLG_NAME)
            {
                Some(inx) => {
                    s.screen_mut().move_to_back(LayerPosition::FromBack(inx));
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
pub fn open_cpy_dlg(
    s: &mut Cursive,
    interrupt_tx_pause: Crossbeam_Sender<nix::sys::signal::Signal>,
    interrupt_tx_continue: Crossbeam_Sender<nix::sys::signal::Signal>,
    interrupt_tx_cancel: Crossbeam_Sender<nix::sys::signal::Signal>,
) {
    let cpy_dlg = create_cp_dlg(
        interrupt_tx_pause,
        interrupt_tx_continue,
        interrupt_tx_cancel,
    );
    s.add_layer(cpy_dlg);
}
pub fn close_dlg(s: &mut Cursive, dlg_name: &str) {
    match s.call_on_name(dlg_name, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(dlg_name)
            {
                Some(inx) => {
                    s.screen_mut().remove_layer(LayerPosition::FromBack(inx));
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}
pub fn close_cpy_dlg(s: &mut Cursive) {
    //++artie, deprecated, use close_dlg
    s.call_on_name(
        //++artie rfctr
        "copy_stack_view",
        |copy_stack_view: &mut StackView| match copy_stack_view
            .find_layer_from_name_like_human_being("copy_progress_layout")
        {
            Some(inx) => {
                if inx == 1 {
                    copy_stack_view.move_to_back(LayerPosition::FromFront(0));
                }
            }
            None => {}
        },
    );
    match s.call_on_name(CPY_DLG_NAME, |_: &mut Dialog| ()) {
        /*If call on name succeeds it means that dlg with that name exists */
        Some(()) => {
            match s
                .screen_mut()
                .find_layer_from_name_like_human_being(CPY_DLG_NAME)
            {
                Some(inx) => {
                    s.screen_mut().remove_layer(LayerPosition::FromBack(inx));
                }
                None => {
                    eprintln!("Layer not found")
                }
            }
        }
        None => {}
    }
}

fn prepare_cp_jobs(s: &mut Cursive) -> Option<CopyJobs> {
    let ((src_table, _), (_, dest_panel)) = if get_active_table_name(s) == LEFT_TABLE_VIEW_NAME {
        (
            //++artie only one item neede to return
            (LEFT_TABLE_VIEW_NAME, LEFT_PANEL_NAME),
            (RIGHT_TABLE_VIEW_NAME, RIGHT_PANEL_NAME),
        )
    } else {
        (
            (RIGHT_TABLE_VIEW_NAME, RIGHT_PANEL_NAME),
            (LEFT_TABLE_VIEW_NAME, LEFT_PANEL_NAME),
        )
    };
    let mut selected_items = get_active_table_selected_items(s, src_table, true);
    if selected_items.is_empty() {
        selected_items.push(get_active_table_focused_item_with_inx(s, src_table));
    }
    let selected_items = selected_items; //++artie, seal
    let dest_path = get_current_path_from_dialog_name(s, dest_panel);

    let mut copying_jobs = CopyJobs::new();
    for (inx, selected_item) in selected_items {
        match PathBuf::from(&selected_item).file_name() {
            Some(file_name) => {
                let full_dest_path =
                    format!("{}/{}", &dest_path, os_string_to_lossy_string(&file_name));

                let cb_sink = s.cb_sink().clone();
                copying_jobs.push_back(copy_job {
                    source: selected_item.clone(),
                    target: full_dest_path.clone(),
                    cb_sink,
                    inx,
                });
            }
            None => {
                eprintln!("Couldn't copy {selected_item}");
                return None;
            }
        }
    }

    Some(copying_jobs)
}

pub fn open_cpy_dlg_hlpr(cb_sink: CbSink) -> Crossbeam_Receiver<nix::sys::signal::Signal> {
    let (interrupt_tx_cancel, interrupt_rx) = crossbeam::channel::unbounded();
    let interrupt_tx_continue = interrupt_tx_cancel.clone();
    let interrupt_tx_pause = interrupt_tx_cancel.clone();
    cb_sink.send(Box::new(move |s| {
        open_cpy_dlg(
            s,
            interrupt_tx_pause,
            interrupt_tx_continue,
            interrupt_tx_cancel,
        );
    }));

    interrupt_rx
}
pub fn close_cpy_dlg_hlpr(cb_sink: &CbSink) {
    if cb_sink
        .send(Box::new(|s| {
            s.set_user_data(());
            close_cpy_dlg(s);
        }))
        .is_err()
    {
        eprintln!("Err close_cpy_dlg_hlpr");
    }
}
pub fn show_cpy_dlg_hlpr(cb_sink: CbSink) {
    if cb_sink
        .send(Box::new(|s| {
            crate::utils::cp_machinery::cp_utils::show_cpy_dlg(s);
        }))
        .is_err()
    {
        eprintln!("Err show_cpy_dlg_hlpr");
    }
}
pub fn show_and_update_cpy_dlg_with_total_count(cb_sink: CbSink, total_count: u64) {
    let cb_sink_a = cb_sink.clone();
    let cb_sink_b = cb_sink_a.clone();
    show_cpy_dlg_hlpr(cb_sink_a);
    update_cpy_dlg_with_new_items_hlpr(cb_sink_b, total_count);
}
pub fn update_cpy_dlg_with_new_items_hlpr(cb_sink: CbSink, new_items_count: u64) {
    if cb_sink
        .send(Box::new(move |s| {
            crate::utils::cp_machinery::cp_utils::update_cpy_dlg_with_new_items(s, new_items_count);
        }))
        .is_err()
    {
        eprintln!("Err show_cpy_dlg_hlpr");
    }
}
pub fn update_cpy_dlg_with_new_items(s: &mut Cursive, total_items: u64) {
    s.call_on_name("total_items", |text_view: &mut TextView| {
        let total_so_far = match text_view.get_content().source().parse::<u64>() {
            Ok(val) => val,
            Err(_) => 0,
        };
        let new_total = total_so_far + total_items;
        text_view.set_content(format!("{new_total}",)); //++artie, must be number only, otherwise parsing will panic
    });
}

pub fn f5_handler(s: &mut Cursive) {
    let cp_jobs = match prepare_cp_jobs(s) {
        None => {
            show_no_paths_selected_dlg(s);
            return;
        }
        Some(cp_jobs) => cp_jobs,
    };

    if s.user_data::<std::sync::mpsc::Sender<CopyJobs>>().is_some() {
        let tx_cp_jobs: &mut std::sync::mpsc::Sender<CopyJobs> = s.user_data().unwrap();
        show_and_update_cpy_dlg_with_total_count(cp_jobs[0].cb_sink.clone(), cp_jobs.len() as u64);
        if tx_cp_jobs.send(cp_jobs).is_err() {
            eprintln!("Send err 1: tx_cp_jobs.send(cp_jobs)");
        }
    } else {
        let (tx_cp_jobs, rx_cp_jobs) = std::sync::mpsc::channel();
        if tx_cp_jobs.send(cp_jobs).is_err() {
            eprintln!("Send err 2: tx_cp_jobs.send(cp_jobs)");
        }
        init_cp_sequence(rx_cp_jobs, s.cb_sink().clone());
        s.set_user_data(tx_cp_jobs);
    }
}

pub fn show_path_exists_dlg_hlpr(
    cb_sink: CbSink,
    source: String,
    target: String,
    overwrite_current_tx: Sender<ExistingPathDilemma>,
) {
    cb_sink.send(Box::new(move |s| {
        show_path_exists_dlg(s, source, target, overwrite_current_tx);
    }));
}

pub fn show_path_exists_dlg(
    s: &mut Cursive,
    source: String,
    target: String,
    overwrite_current_tx: Sender<ExistingPathDilemma>,
) {
    let source_info = match file_info(&source) {
        Ok(info) => info,
        Err(e) => {
            format!("Couldn't get info for {}, reason: {}", source, e)
        }
    };
    let target_info = match file_info(&target) {
        Ok(info) => info,
        Err(e) => {
            format!("Couldn't get info for {}, reason: {}", source, e)
        }
    };

    let dlg = create_path_exists_dlg(source_info, target_info, overwrite_current_tx);
    show_error_themed_view(s, dlg);
}

pub fn show_error_themed_view<V: View>(s: &mut cursive::Cursive, dlg: V) {
    s.add_layer(views::ThemedView::new(
        ERROR_THEME.clone(),
        views::Layer::new(dlg),
    ));
}
pub fn show_info_themed_view<V: View>(s: &mut cursive::Cursive, dlg: V) {
    s.add_layer(views::ThemedView::new(
        INFO_THEME.clone(),
        views::Layer::new(dlg),
    ));
}
static ERROR_THEME: Lazy<Theme> = Lazy::new(|| {
    eprintln!("Lazy theme: ERROR_THEME");
    let mut theme = Theme::default();

    theme.palette[theme::PaletteColor::View] = theme::Color::Dark(theme::BaseColor::Red);
    theme.palette[theme::PaletteColor::Primary] = theme::Color::Light(theme::BaseColor::White);
    theme.palette[theme::PaletteColor::TitlePrimary] =
        theme::Color::Light(theme::BaseColor::Yellow);
    theme.palette[theme::PaletteColor::Highlight] = theme::Color::Dark(theme::BaseColor::Green);

    theme
});
static INFO_THEME: Lazy<Theme> = Lazy::new(|| {
    eprintln!("Lazy theme: ERROR_THEME");
    let mut theme = Theme::default();

    theme.palette[theme::PaletteColor::View] = theme::Color::Dark(theme::BaseColor::Cyan);
    theme.palette[theme::PaletteColor::Primary] = theme::Color::Light(theme::BaseColor::White);
    theme.palette[theme::PaletteColor::TitlePrimary] = theme::Color::Light(theme::BaseColor::Black);
    theme.palette[theme::PaletteColor::Highlight] = theme::Color::Dark(theme::BaseColor::Black);

    theme
});
static RESULT_THEME: Lazy<Theme> = Lazy::new(|| {
    eprintln!("Lazy theme: RESULT_THEME");
    let mut theme = Theme::default();

    theme.palette[theme::PaletteColor::View] = theme::Color::Dark(theme::BaseColor::Cyan);
    theme.palette[theme::PaletteColor::Primary] = theme::Color::Light(theme::BaseColor::White);
    theme.palette[theme::PaletteColor::TitlePrimary] = theme::Color::Light(theme::BaseColor::Black);
    theme.palette[theme::PaletteColor::Highlight] = theme::Color::Dark(theme::BaseColor::Black);

    theme
});
pub fn show_result_themed_view<V: View>(s: &mut cursive::Cursive, dlg: V) {
    s.add_layer(views::ThemedView::new(
        RESULT_THEME.clone(),
        views::Layer::new(dlg),
    ));
}
#[cfg(target_os = "linux")]
pub fn file_info_size(file: &str) -> Result<u64, std::io::Error> {
    let metadata = std::fs::metadata(file)?;
    Ok(metadata.len())
}
pub fn compare_paths_for_size(path_a: &str, path_b: &str) -> Ordering {
    let ord = (|| {
        let size_a = file_info_size(path_a).ok()?;
        let size_b = file_info_size(path_b).ok()?;
        Some(size_a.cmp(&size_b))
    })()
    .unwrap_or(Ordering::Equal);

    ord
}
#[cfg(target_os = "linux")]
pub fn file_info_modification_time(file: &str) -> Result<SystemTime, std::io::Error> {
    let metadata = std::fs::metadata(file)?;
    metadata.modified()
}
pub fn compare_paths_for_modification_time(path_a: &str, path_b: &str) -> Ordering {
    let ord = (|| {
        let modification_time_a = file_info_modification_time(path_a).ok()?;
        let modification_time_b = file_info_modification_time(path_b).ok()?;
        modification_time_a.partial_cmp(&modification_time_b)
    })()
    .unwrap_or(Ordering::Equal);

    ord
}
#[cfg(target_os = "linux")]
pub fn file_info(file: &str) -> Result<String, std::io::Error> {
    let metadata = std::fs::metadata(file)?;

    let path = format!("Path: {}", file);
    let file_type = format!("File type: {:?}", metadata.file_type());
    let accessed = match metadata.accessed() {
        Ok(val) => format!("Access time: {:>25}", pretty_print_system_time(val)),
        Err(e) => {
            eprintln!("cannot get accessed time: {}", e);
            String::from("Access time: UNKNOWN")
        }
    };
    let created = match metadata.created() {
        Ok(val) => format!("Created time: {:>24}", pretty_print_system_time(val)),
        Err(e) => {
            eprintln!("cannot get created time: {}", e);
            String::from("Created time: UNKNOWN")
        }
    };
    let modified = match metadata.modified() {
        Ok(val) => format!("Modified time: {:>23}", pretty_print_system_time(val)),
        Err(e) => {
            eprintln!("cannot get modified time: {}", e);
            String::from("Modified time: UNKNOWN")
        }
    };

    let size_in_bytes = format!("Size in bytes: {:>9}", metadata.len());

    let permissions = metadata.permissions();
    let mode = format!(
        "mode: {:>22}",
        <Permissions as PermissionsExt>::mode(&permissions)
    );

    Ok(path
        + "\n"
        + &file_type
        + "\n"
        + &accessed
        + "\n"
        + &created
        + "\n"
        + &modified
        + "\n"
        + &size_in_bytes
        + "\n"
        + &mode)
}

pub fn check_if_path_exists(target: &str) -> bool {
    std::path::Path::new(target).exists()
}

pub fn alt_f1_handler(s: &mut Cursive) {
    eprintln!("alt_f1_handler");
    display_quick_cd_hint(s);
}

fn display_quick_cd_hint(s: &mut Cursive) {
    let dialog_name = get_active_dlg_name(s); //++artie, it must return String, if returns &str will complain that siv is borrowed mutably more than once
    let current_path = get_current_path_from_dialog_name(s, &dialog_name);
    let path_with_hints = prepare_path_with_hints(s, current_path);
    update_dlg_title(s, &dialog_name, &path_with_hints);
}

fn prepare_path_with_hints(s: &mut Cursive, current_path: String) -> String {
    let path = current_path
        .split_inclusive(PATH_SEPARATOR)
        .collect::<Vec<&str>>();
    //update_dlg_title(s, &dialog_name, path);
    let mut path_with_hints = String::new();
    for (inx, p) in path.iter().enumerate() {
        let parts_combined = if inx == path.len() - 1 {
            format!("{}", p)
        } else {
            format!("[{}]{}", inx, *p)
        };
        path_with_hints.push_str(&parts_combined);
    }

    path_with_hints
}

pub static PATH_HINT_REGES: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\[\d+\])").unwrap());
pub static PATH_SEPARATOR_REGES: Lazy<Regex> = Lazy::new(|| Regex::new(PATH_SEPARATOR).unwrap());

pub fn prepare_path_without_hints(s: &mut Cursive, current_path: String) -> String {
    let mut path_without_hints = String::new();
    path_without_hints = PATH_HINT_REGES.replace_all(&current_path, "").to_string();
    path_without_hints
}

pub fn quick_cd_handler(key: char, s: &mut Cursive) {
    eprintln!("quick_cd_handler: {}", key);
    let dialog_name = get_active_dlg_name(s);
    let table_view_name = get_active_table_name(s);

    let current_path = get_current_path_from_dialog_name(s, &dialog_name);
    let parts = current_path
        .split_inclusive(PATH_SEPARATOR)
        .collect::<Vec<_>>();
    match key.to_digit(10) {
        Some(digit) => {
            if digit as usize >= parts.len() {
                let dlg = create_cd_too_long_dlg(s);
                show_info_themed_view(s, dlg);
                return;
            }
            let (left, _) = parts.split_at((digit as usize) + 1);
            let mut cd_path = String::new();
            for part in left {
                cd_path.push_str(part);
            }
            eprintln!("Path to cd to:{}", cd_path);
            update_table(s, &cd_path, &table_view_name);
            update_dlg_title(s, &dialog_name, &cd_path);
        }
        None => {
            panic!("Ehe")
        }
    }
    // let k_as_usize = key as usize - '0' as usize;

    //for i in 0..=key as u8 {
    //    cd_path.push_str(&String::from(parts[i as usize]));
    //}
    //let cd_to_path = PathBuf::from(current_path);
    //let comps = cd_to_path.components();
    //for comp in comps {
    //    eprintln!("{:?}", comp);
    //}
    //for i in '0'..key {}
}

fn create_cd_too_long_dlg(s: &mut Cursive) -> Dialog {
    Dialog::around(TextView::new(
        "Path doesn't have that many parts.\nFor hints press Alt+F1",
    ))
    .dismiss_button("OK")
    .title("Path is too short")
}
