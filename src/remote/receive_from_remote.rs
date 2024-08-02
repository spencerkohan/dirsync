use std::{
    io::{BufRead, BufReader},
    thread,
};

use crate::config::SessionConfig;

use super::Remote;

pub fn watch_remote_receivable_paths(config: SessionConfig) {
    let Some(paths) = &config.remote.receive_paths else {
        return;
    };
    let root = config.remote.root.clone();

    let mut remote = Remote::connect(&config);
    let paths = paths
        .into_iter()
        .map(|path| path.path.clone())
        .collect::<Vec<String>>()
        .join(" ");

    // spawn a thread to connect to the other rsync instance
    thread::spawn(move || {
        if let Err(err) = remote.install_dirsync() {
            eprintln!("Error installing dirsync at the remote: ${err}");
        }

        // launch cargo in the other directory
        let Ok(mut command) = remote.command(&format!(
            r#"cargo run --manifest-path {root}/{dirsync_client_dir}/dirsync/Cargo.toml \
            -- watch -r {root} {paths}
            "#,
            dirsync_client_dir = remote.dirsync_client_dir()
        )) else {
            eprintln!("Failed to init cargo command");
            return;
        };

        if let Err(err) = command.exec() {
            eprintln!("Error executing remote: ${err}");
        }

        let stdout = command.channel.stream(0);
        let stdout_reader = BufReader::new(stdout);

        for line in stdout_reader.lines() {
            match line {
                Ok(line) => println!("Read line from remote: {}", line),
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }

        if let Err(err) = command.wait_close() {
            eprintln!("Error finishing command: ${err}");
        }
    });
}
