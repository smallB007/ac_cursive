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
) -> Cp_error {
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
            return Cp_error::CP_COULDNOT_START;
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
                return Cp_error::CP_COULDNOT_READ_STDERR;
            }
            Ok(_) => {
                if buf.len() != 0 {
                    return Cp_error::CP_EXIT_STATUS_ERROR(buf);
                }
            }
        }
    }
    Cp_error::CP_EXIT_STATUS_SUCCESS
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
