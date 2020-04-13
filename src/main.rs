mod config;
mod remote;
mod cli;
mod init;

extern crate notify;

use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::channel;
use std::time::Duration;
use crate::config::SessionConfig;
use crate::cli::SubCommand;
use crate::cli::CliOptions;
use structopt::StructOpt;
use std::io::prelude::*;


// Perform rsync from source to destination
fn rsync(config: &config::SessionConfig) {
    use std::process::Command;

    // we sync actions explicitly here, since they might be ignored otherwise
    let dirsync_dir_local = &format!("{}/.dirsync/actions", &config.local_root);
    let dirsync_dir_remote = &format!("{}", &config.destination());

    let output = &Command::new("rsync")
    .arg("-v") // verbose output
    .arg("-r")
    .arg(dirsync_dir_local)
    .arg(dirsync_dir_remote)
    .output()
    .expect("failed to execute process");

    println!("executing rsync: {} {}", &dirsync_dir_local, &dirsync_dir_remote);
    println!("status: {}", output.status);
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());

    if config.ignore_gitignore {

        let output = &Command::new("rsync")
        .arg("-v") // verbose output
        .arg("-r")
        .arg(format!("--exclude-from={}", config.exclude_path().to_str().unwrap()))
        .arg("--exclude-from=.gitignore")
        .arg(&config.local_root)
        .arg(&config.destination())
        .output()
        .expect("failed to execute process");

        println!("executing rsync: {} {}", &config.local_root, &config.destination());
        println!("status: {}", output.status);
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        assert!(output.status.success());

    } else {

        let output = &Command::new("rsync")
        .arg("-v") // verbose output
        .arg("-r")
        .arg(format!("--exclude-from={}", config.exclude_path().to_str().unwrap()))
        .arg(&config.local_root)
        .arg(&config.destination())
        .output()
        .expect("failed to execute process");

        println!("executing rsync: {} {}", &config.local_root, &config.destination());
        println!("status: {}", output.status);
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        assert!(output.status.success());

    }

}

fn filter(event: DebouncedEvent) -> Option<DebouncedEvent> {
    match event {
        DebouncedEvent::NoticeWrite(_) => None,
        DebouncedEvent::NoticeRemove(_) => None,
        DebouncedEvent::Chmod(_) => None,
        DebouncedEvent::Rescan => None,
        DebouncedEvent::Error(_, _) => None,
        _ => Some(event)
    }
}


fn start_main_loop(config: &SessionConfig) {

    println!("config: {:?}", config);

    rsync(&config);
    let mut remote = remote::Remote::connect(config);
    remote.execute_if_exists("onSyncDidFinish");

    // Create a channel to receive the events.
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_millis(20)).unwrap();
    watcher.watch(config.local_root.clone(), RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("handling event: {:?}", event);
                match filter(event) {
                    Some(_) => {
                        rsync(&config);
                        println!("Executing onSyncDidFinish action");
                        remote.execute_if_exists("onSyncDidFinish");
                    },
                    None => println!("ignoring event")
                };

            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}


fn main() {
    
    let opts = CliOptions::from_args();

    match opts.subcommand {
        Some(SubCommand::Init(remote_config)) => init::init_dirsync_dir(remote_config).unwrap(),
        _ => {
            let config = SessionConfig::get(opts);
            start_main_loop(&config);
        }
    };
}
