use crate::tui_fn::create_classic_buttons::copying_job;
use anyhow::Context;
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream, NameTypeSupport};

use std::{
    io::{self, prelude::*, BufReader},
    sync::mpsc::Sender,
};
pub fn cp_client_main(copy_jobs: Vec<copying_job>) -> anyhow::Result<()> {
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

    // Preemptively allocate a sizeable buffer for reading.
    // This size should be enough and should be easy to find for the allocator.
    let mut buffer = String::with_capacity(128);
    for copy_job in copy_jobs {
        // Create our connection. This will block until the server accepts our connection, but will fail
        // immediately if the server hasn't even started yet; somewhat similar to how happens with TCP,
        // where connecting to a port that's not bound to any server will send a "connection refused"
        // response, but that will take twice the ping, the roundtrip time, to reach the client.
        let conn = LocalSocketStream::connect(name).context("Failed to connect to server")?;
        // Wrap it into a buffered reader right away so that we could read a single line out of it.
        let mut conn = BufReader::new(conn);
        // Write our message into the stream. This will finish either when the whole message has been
        // writen or if a write operation returns an error. (`.get_mut()` is to get the writer,
        // `BufReader` doesn't implement a pass-through `Write`.)
        conn.get_mut()
            .write_all(format!("{}:{}:\n", copy_job.source, copy_job.target).as_bytes())
            .context("Socket send failed")?;
        // We now employ the buffer we allocated prior and read until EOF, which the server will
        // similarly invoke with `.shutdown()`, verifying validity of UTF-8 on the fly.
        conn.read_line(&mut buffer)
            .context("Socket receive failed")?;

        // Print out the result, getting the newline for free!
        print!("Server answered: {}", buffer);

        if buffer == "stop" {
            break;
        }
        // Clear the buffer so that the next iteration will display new data instead of messages
        // stacking on top of one another.
        buffer.clear();
    }
    /*Send final msg to the server so it can exit */
    let conn = LocalSocketStream::connect(name).context("Failed to connect to server")?;
    // Wrap it into a buffered reader right away so that we could read a single line out of it.
    let mut conn = BufReader::new(conn);
    conn.get_mut()
        .write_all(b"server_stop\n")
        .context("Socket send failed")?;
    Ok(())
}
