use anyhow::Context;
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream, NameTypeSupport};
use std::{
    io::{self, prelude::*, BufReader},
    sync::mpsc::Sender,
};

pub fn cp_server_main(notify: Sender<()>) -> anyhow::Result<()> {
    // Define a function that checks for errors in incoming connections. We'll use this to filter
    // through connections that fail on initialization for one reason or another.
    fn handle_error(conn: io::Result<LocalSocketStream>) -> Option<LocalSocketStream> {
        match conn {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Incoming connection failed: {}", e);
                None
            }
        }
    }

    // Pick a name. There isn't a helper function for this, mostly because it's largely unnecessary:
    // in Rust, `match` is your concise, readable and expressive decision making construct.
    let name = {
        // This scoping trick allows us to nicely contain the import inside the `match`, so that if
        // any imports of variants named `Both` happen down the line, they won't collide with the
        // enum we're working with here. Maybe someone should make a macro for this.
        use NameTypeSupport::*;
        match NameTypeSupport::query() {
            OnlyPaths => "/tmp/example.sock",
            OnlyNamespaced | Both => "@example.sock",
        }
    };

    // Bind our listener.
    let listener = match LocalSocketListener::bind(name) {
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
            // One important problem that is easy to handle improperly (or not at all) is the
            // "corpse sockets" that are left when a program that uses a file-type socket name
            // terminates its socket server without deleting the file. There's no single strategy
            // for handling this kind of address-already-occupied error. Services that are supposed
            // to only exist as a single instance running on a system should check if another
            // instance is actually running, and if not, delete the socket file. In this example,
            // we leave this up to the user, but in a real application, you usually don't want to do
            // that.
            eprintln!(
                "\
Error: could not start server because the socket file is occupied. Please check if {} is in use by \
another process and try again.",
                name,
            );
            return Err(e.into());
        }
        x => x?,
    };

    println!("Server running at {}", name);
    // Stand-in for the syncronization used, if any, between the client and the server.
    let _ = notify.send(());

    // Preemptively allocate a sizeable buffer for reading at a later moment. This size should be
    // enough and should be easy to find for the allocator. Since we only have one concurrent
    // client, there's no need to reallocate the buffer repeatedly.
    let mut buffer = String::with_capacity(128);

    for conn in listener.incoming().filter_map(handle_error) {
        // Wrap the connection into a buffered reader right away
        // so that we could read a single line out of it.
        let mut conn = BufReader::new(conn);
        println!("Incoming connection!");

        // Since our client example writes first, the server should read a line and only then send a
        // response. Otherwise, because reading and writing on a connection cannot be simultaneous
        // without threads or async, we can deadlock the two processes by having both sides wait for
        // the write buffer to be emptied by the other.
        conn.read_line(&mut buffer)
            .context("Socket receive failed")?;

        // Print out the result, getting the newline for free!
        print!("Client first read answered: {}", buffer);
        // Let's add an exit condition to shut the server down gracefully.
        if buffer == "server_stop\n" {
            break;
        }
        let src_target: Vec<&str> = buffer.split(':').collect();

        match cp_path(&src_target[0], &src_target[1]) {
            Some(cp_err) => {
                conn.get_mut()
                    .write_all(format!("Copying finished with error: {}!\n", cp_err).as_bytes())
                    .context("Socket send write_all err failed")?;
            }
            None => {
                conn.get_mut()
                    .write_all(format!("Copying finished: {}!\n", &src_target[0]).as_bytes())
                    .context("Socket send write_all 2 failed")?;
            }
        }

        // Clear the buffer so that the next iteration will display new data instead of messages
        // stacking on top of one another.
        buffer.clear();
    }
    Ok(())
}

use std::io::prelude::*;
use std::process::{Command, Stdio};

static PANGRAM: &'static str = "the quick brown fox jumped over the lazy dog\n";
//use anyhow::{Context, Result};
use thiserror::Error;
#[derive(Error, Debug)]
enum Cp_error {
    #[error("Source does not exist")]
    CP_SOURCE_DOESNOT_EXIST,
    #[error("Target does not exist")]
    CP_TARGET_DOESNOT_EXIST,
    #[error("Could not start cp process")]
    CP_COULDNOT_START,
    #[error("Could not read stderr")]
    CP_COULDNOT_READ_STDERR,
    #[error("Could not read stdout")]
    CP_COULDNOT_READ_STDOUT,
}
fn cp_path(src: &str, target: &str) -> Option<Cp_error> {
    //for i in 1..=3 {
    //let source = format!("/home/artie/Desktop/Apprentice/{}.avi", i);
    //let target = "/tmp";
    // Spawn the `wc` command
    let mut process = match Command::new("cp")
        .arg("-f")
        .arg(src)
        .arg(target)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Err(why) => {
            return Some(Cp_error::CP_COULDNOT_START);
            //    panic!("couldn't spawn wc: {}", why)
        }
        Ok(process) => process,
    };

    // Write a string to the `stdin` of `wc`.
    //
    // `stdin` has type `Option<ChildStdin>`, but since we know this instance
    // must have one, we can directly `unwrap` it.
    //match process.stdin.unwrap().write_all(PANGRAM.as_bytes()) {
    //    Err(why) => panic!("couldn't write to wc stdin: {}", why),
    //    Ok(_) => println!("sent pangram to wc"),
    //}

    // Because `stdin` does not live after the above calls, it is `drop`ed,
    // and the pipe is closed.
    //
    // This is very important, otherwise `wc` wouldn't start processing the
    // input we just sent.

    process.wait();
    {
        let mut s = String::new();
        match process.stderr.unwrap().read_to_string(&mut s) {
            Err(why) => {
                return Some(Cp_error::CP_COULDNOT_READ_STDERR);
                //    panic!("couldn't read wc stdout: {}", why)
            }
            Ok(_) => print!("cp responded with:\n{}", s),
        }
    }

    // The `stdout` field also has type `Option<ChildStdout>` so must be unwrapped.
    let mut s = String::new();
    match process.stdout.unwrap().read_to_string(&mut s) {
        Err(why) => {
            return Some(Cp_error::CP_COULDNOT_READ_STDOUT);
            //    panic!("couldn't read wc stdout: {}", why)
        }
        Ok(_) => print!("cp responded with:\n{}", s),
    }
    None
    // }
}
