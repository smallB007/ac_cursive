#[cfg(unused)]
pub static GLOBAL_DATA: Lazy<Mutex<CopyJobs>> = Lazy::new(|| {
    let m = CopyJobs::new();
    Mutex::new(m)
});
#[cfg(unused)]
fn cp_path_new(
    //++artie, use execute process
    job: copy_job,
    interrupt_rx: &Crossbeam_Receiver<nix::sys::signal::Signal>,
) -> EXIT_PROCESS_STATUS {
    let mut process = match Command::new("cp")
        .arg("-f")
        .arg(job.source)
        .arg(job.target)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Err(why) => {
            return EXIT_PROCESS_STATUS::COULD_NOT_START;
        }
        Ok(process) => process,
    };

    let timeout = tick(std::time::Duration::from_secs(2));
    loop {
        select! {
            recv(interrupt_rx) -> interrupt_rx_result => {
                println!("Received interrupt notification:{:?}",interrupt_rx_result);
                let id = process.id();
                match interrupt_rx_result
                {
                    Ok(nix::sys::signal::Signal::SIGSTOP)=>{
                        nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGSTOP);
                        job.cb_sink.send(Box::new(|s|{crate::utils::cp_machinery::cp_utils::cpy_dlg_show_continue_btn(s)}));
                    },
                    Ok(nix::sys::signal::Signal::SIGCONT)=>{
                        nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGCONT);
                        job.cb_sink.send(Box::new(|s|{crate::utils::cp_machinery::cp_utils::cpy_dlg_show_pause_btn(s)}));
                    },
                    Ok(nix::sys::signal::Signal::SIGTERM)=>{
                        nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGCONT);
                        nix::sys::signal::kill(nix::unistd::Pid::from_raw(id as i32),nix::sys::signal::Signal::SIGTERM);
                        break;
                    },
                    _=>{}
                }
              },
            recv(timeout) -> _ => {
                eprintln!("Checking if we finished the long task");
                match process.try_wait() {
                    Ok(Some(status)) =>{ eprintln!("exited with: {status}");break;},
                    Ok(None) => {
                        eprintln!("status not ready yet");
                    }
                    Err(e) => {eprintln!("error attempting to wait: {e}");break;},
                }
            }
        }
        eprintln!("AFTER SELECT>>>>>>>>>>>>>>>>>>>>>>");
    }
    eprintln!("AFTER LOOP>>>>>>>>>>>>>>>>>>>>>>");

    {
        let mut buf = String::new();
        match process.stderr.unwrap().read_to_string(&mut buf) {
            Err(why) => {
                return EXIT_PROCESS_STATUS::COULD_NOT_READ_STDERR;
            }
            Ok(_) => {
                if buf.len() != 0 {
                    return EXIT_PROCESS_STATUS::EXIT_STATUS_ERROR(buf);
                }
            }
        }
    }
    EXIT_PROCESS_STATUS::EXIT_STATUS_SUCCESS
}
#[cfg(unused)]
fn transfer_copying_jobs(
    copying_jobs: Vec<copy_job>,
    jobs_sender_tx: std::sync::mpsc::Sender<Vec<copy_job>>,
    rx_client_thread_started: std::sync::mpsc::Receiver<()>,
) {
    rx_client_thread_started.recv();
    jobs_sender_tx.send(copying_jobs);
}
#[cfg(unused)]
pub fn f5_handler_interprocess(s: &mut Cursive) {
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
    let selected_items = get_active_table_selected_items(s, src_table, true);
    //eprintln!("{:?}", selected_items);
    let dest_path = get_current_path_from_dialog_name(s, String::from(dest_panel));

    let mut copying_jobs: Vec<copy_job> = Vec::new();
    for (inx, selected_item) in selected_items {
        match PathBuf::from(&selected_item).file_name() {
            Some(file_name) => {
                //std::thread::scope(|scoped| {
                let full_dest_path =
                    format!("{}/{}", &dest_path, os_string_to_lossy_string(&file_name));

                let cb_sink = s.cb_sink().clone();
                copying_jobs.push(copy_job {
                    source: selected_item.clone(),
                    target: full_dest_path.clone(),
                    cb_sink,
                    inx,
                });
            }
            None => {
                eprintln!("Couldn't copy {selected_item}");
            }
        }
    }
    show_cpy_dlg(s);
    if s.user_data::<std::sync::mpsc::Sender<Vec<copy_job>>>()
        .is_some()
    {
        let sender: &mut std::sync::mpsc::Sender<Vec<copy_job>> = s.user_data().unwrap();
        sender.send(copying_jobs);
    } else {
        let (jobs_sender_tx, jobs_receiver_rx) = std::sync::mpsc::channel();
        let (client_thread_started_tx, client_thread_started_rx) = std::sync::mpsc::channel();
        let copying_jobs_clone = copying_jobs.clone();
        let jobs_sender_clone = jobs_sender_tx.clone();
        let transfer_copying_jobs_handle = std::thread::spawn(move || {
            transfer_copying_jobs(copying_jobs_clone, jobs_sender_tx, client_thread_started_rx);
        });
        s.set_user_data(jobs_sender_clone);

        //    if show_cpy_dlg(s) {
        //        return;
        //    }
        //eprintln!("dest_path: {}", dest_path);
        let (interrupt_tx, interrupt_rx) = crossbeam::channel::unbounded();
        //std::thread::spawn(move || {
        //    crate::utils::cp_machinery::signal_handlers::await_interrupt(interrupt_tx)
        //});
        let interrupt_tx_clone_1 = interrupt_tx.clone();
        let interrupt_tx_clone_2 = interrupt_tx.clone();
        create_cp_dlg(s, interrupt_tx, interrupt_tx_clone_1, interrupt_tx_clone_2);
        let cb_sink_clone = s.cb_sink().clone();

        /*Copying in separate thread so GUI isn't blocked*/
        let cb_sink = s.cb_sink().clone();
        let cb_sink_for_client_thread = s.cb_sink().clone();
        std::thread::spawn(move || {
            use crate::utils::cp_machinery::cp_utils::update_copy_dlg_with_error;
            let (snd, rcv) = std::sync::mpsc::channel();
            let srv_thread = std::thread::spawn(move || {
                // cp_server_main(snd, cb_sink, &update_copy_dlg_with_error, interrupt_rx)
            });
            let _ = rcv.recv();
            if let Err(e) = cp_client_main(
                copying_jobs,
                &update_cpy_dlg_progress,
                &show_cpy_dlg,
                &hide_cpy_dlg,
                jobs_receiver_rx,
                client_thread_started_tx,
                cb_sink_for_client_thread,
            ) {
                eprintln!("Error during copying:{}", e);
            }

            srv_thread.join();
            match cb_sink_clone.send(Box::new(|s| {
                close_cpy_dlg(s);
            })) {
                Ok(_) => {
                    eprintln!("Sending close_cpy_dlg successfull");
                }
                Err(e) => {
                    eprintln!("Sending close_cpy_dlg NOT successfull: {}", e);
                }
            }
        });
    }
    /* std::thread::spawn(move || {
        let copying_jobs_len = copying_jobs.len();
        for (inx, copy_job) in copying_jobs.iter().enumerate() {
            let selected_item = copy_job.0.clone();
            let full_destination_path = copy_job.1.clone();
            let cb_sink = copy_job.2.clone();
            //let cb_sink_clone = cb_sink.clone(); //++artie only needed at the end
            let handle = std::thread::spawn(move || {
                copying_engine(
                    &selected_item,
                    inx as u64,
                    copying_jobs_len as u64,
                    &full_destination_path,
                    cb_sink,
                );
            });
            handle.join(); //and we make suer that we are copying in organized, well defined order
            eprintln!("Finished copying: {}", inx);
        }

        match cb_sink_clone.send(Box::new(|s| {
            close_cpy_dlg(s);
        })) {
            Ok(_) => {
                eprintln!("Sending close_cpy_dlg successfull")
            }
            Err(e) => {
                eprintln!("Sending close_cpy_dlg NOT successfull: {}", e)
            }
        }
    });
    */
}

#[cfg(unused)]
fn copying_engine(
    selected_item: &str,
    selected_item_n: u64,
    total_items: u64,
    full_dest_path: &str,
    cb_sink: CbSink,
) {
    std::thread::scope(|s| {
        let selected_item_clone = String::from(selected_item);
        let selected_item_len = match PathBuf::from(selected_item).metadata() {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                eprintln!("Couldn't get len for path: {}", selected_item);
                0
            }
        };
        let full_dest_path_clone = String::from(full_dest_path);
        let full_dest_path_clone_2 = String::from(full_dest_path);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        use std::sync::{Arc, Condvar, Mutex};
        use std::thread;

        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = Arc::clone(&pair);
        //let cb_sink_clone = cb_sink.clone();
        // Inside of our lock, spawn a new thread, and then wait for it to start.
        let _handle_cpy = s.spawn(move err_file_().unwrap();
            *started = true;

            // We notify the condvar that the value has changed.
            cvar.notify_one();
            drop(started); //++artie, unbelievable, manual mem management...
            match copy_file(&selected_item_clone, &full_dest_path_clone) {
                Ok(_) => {
                    eprintln!("Copied");
                }
                Err(e) => {
                    eprintln!("couldn't copy: {e}");
                }
            }
            tx.send(true);
        });

        // Wait for the thread to start up.
        let (lock, cvar) = &*pair;
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }
        drop(started); //++artie, unbelievable, manual mem management...

        println!("Copying thread started. Proceeding to spawn watch thread.");
        let _handle_read = s.spawn(move || loop {
            match rx.try_recv() {
                Ok(res) => {
                    if res {
                        //eprintln!("Received end of copying msg");
                        break;
                    }
                }
                Err(e) => {
                    //eprintln!("Receiving error: {}", e);
                }
            }
            let full_dest_path_clone_2_clone = full_dest_path_clone_2.clone();
            match std::fs::File::open(full_dest_path_clone_2_clone) {
                Ok(f) => {
                    let len = f.metadata().unwrap().len();
                    //eprintln!("opened, len: {len}");
                    let percent = (len as f64 / selected_item_len as f64) * 100_f64;
                    cb_sink
                        .send(Box::new(move |siv| {
                            update_cpy_dlg_progress(
                                siv,
                                selected_item_n,
                                total_items,
                                percent as u64,
                            )
                        }))
                        .unwrap();
                }
                Err(e) => {
                    eprintln!("couldn't open: {e}");
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(250));
        });
    });
}

#[cfg(unused)]
pub fn copy_file(src: &str, dest: &str) -> std::io::Result<()> {
    std::fs::copy(src, dest)?;
    Ok(())
}
#[cfg(unused)]
fn dmain() {
    let mut siv = cursive::default();

    // The main dialog will just have a textarea.
    // Its size expand automatically with the content.
    siv.add_fullscreen_layer(
        Dialog::new()
            .title("Describe your issue")
            .padding_lrtb(1, 1, 1, 0)
            .content(TextArea::new().with_name("text").min_height(30))
            .button("Ok", Cursive::quit),
    );

    // We'll add a find feature!
    siv.add_layer(Dialog::info("Hint: press Ctrl-F to find in text!"));

    siv.add_global_callback(Event::CtrlChar('f'), |s| {
        // When Ctrl-F is pressed, show the Find popup.
        // Pressing the Escape key will discard it.
        s.add_fullscreen_layer(
            OnEventView::new(
                Dialog::new()
                    .title("Find")
                    .content(
                        EditView::new()
                            .on_submit(find)
                            .with_name("edit")
                            .min_width(10),
                    )
                    .button("Ok", |s| {
                        let text = s
                            .call_on_name("edit", |view: &mut EditView| view.get_content())
                            .unwrap();
                        find(s, &text);
                    })
                    .dismiss_button("Cancel"),
            )
            .on_event(Event::Key(Key::Esc), |s| {
                s.pop_layer();
            }),
        )
    });

    siv.run();
}

fn find(siv: &mut Cursive, text: &str) {
    // First, remove the find popup
    siv.pop_layer();

    let res = siv.call_on_name("text", |v: &mut TextArea| {
        // Find the given text from the text area content
        // Possible improvement: search after the current cursor.
        if let Some(i) = v.get_content().find(text) {
            // If we found it, move the cursor
            v.set_cursor(i);
            Ok(())
        } else {
            // Otherwise, return an error so we can show a warning.
            Err(())
        }
    });

    if let Some(Err(())) = res {
        // If we didn't find anything, tell the user!
        siv.add_layer(Dialog::info(format!("`{}` not found", text)));
    }
}
