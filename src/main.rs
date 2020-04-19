mod config;
mod remote;
mod cli;
mod init;

extern crate notify;

use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;
use crate::config::SessionConfig;
use crate::cli::SubCommand;
use crate::cli::CliOptions;
use structopt::StructOpt;
use std::path::Path;
use std::thread;
use std::sync::{Mutex, Arc};
use std::vec::Vec;
use crate::remote::Remote;
use std::thread::sleep;

// Perform rsync from source to destination
fn sync(config: &config::SessionConfig) {
    use std::process::Command;

    fn rsync(source: &str, destinatin: &str, args: &Vec<String>) {
        println!("executing rsync: {} {}", source, destinatin);

        let mut rsync = Command::new("rsync")
        .arg("-v") // verbose output
        .arg("-a") // archived: we use this to only sync files which have changed
        .arg("-r") // recursive
        .args(args)
        .arg(source)
        .arg(destinatin)
        .spawn()
        .expect("failed to execute rsync");

        let result = rsync.wait().expect("failed to wait for process");

        println!("rsync finished");
        assert!(result.success());
    }

    // we sync actions explicitly here, since they might be ignored otherwise
    let dirsync_dir_local = &format!("{}/.dirsync", &config.local_root);
    let dirsync_dir_remote = &format!("{}", &config.destination());
    rsync(dirsync_dir_local, dirsync_dir_remote, &Vec::new());

    let exclude_gitignore = config.ignore_gitignore &&  Path::new(".gitignore").exists();
    let exclude_file = Path::new(config.exclude_path().to_str().unwrap()).exists();

    let mut args: Vec<String> = Vec::new();
    if exclude_gitignore {
        args.push(
            String::from("--exclude-from=.gitignore")
        );
    }
    if exclude_file {
        args.push(
            String::from(
                format!("--exclude-from={}", config.exclude_path().to_str().unwrap())
            )
        );
    }

    rsync(&config.local_root, &config.destination(), &args)
}

fn filter(event: DebouncedEvent) -> Option<DebouncedEvent> {
    match event {
        DebouncedEvent::NoticeWrite(_) => None,
        DebouncedEvent::NoticeRemove(_) => None,
        DebouncedEvent::Rescan => None,
        DebouncedEvent::Error(_, _) => None,
        _ => Some(event)
    }
}

fn start_watch_thread(root: String, flush_signal: Sender<()>, events: &mut Arc<Mutex<Vec<DebouncedEvent>>>) {
    let events = Arc::clone(events);
    thread::spawn(move|| {

         // Create a channel to receive watcher events.
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_millis(20)).unwrap();
        watcher.watch(root, RecursiveMode::Recursive).unwrap();



        loop {
            match rx.recv() {
                Ok(event) => {
                    println!("handling event: {:?}", event);
                    match filter(event) {
                        Some(event) => {
                            let signal = flush_signal.clone();
                            let mut events_vec = events.lock().unwrap();
                            events_vec.push(event);
                            thread::spawn(move|| {
                                sleep(Duration::from_millis(20));
                                signal.send(()).unwrap();
                            });
                        },
                        None => println!("ignoring event")
                    };

                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }

    });
}

fn flush_events(config: &SessionConfig, remote: &mut Remote,  events: &mut Arc<Mutex<Vec<DebouncedEvent>>>) {
    let mut events_vec = events.lock().unwrap();
    if !events_vec.is_empty() {
        events_vec.clear();
        sync(&config);
        println!("Executing onSyncDidFinish action");
        remote.execute_if_exists("onSyncDidFinish");
    }
}

fn start_main_loop(config: &SessionConfig) {
    println!("config: {:?}", config);

    sync(&config);
    let mut remote = Remote::connect(config);
    remote.execute_if_exists("onSessionDidStart");

    let mut events: Arc<Mutex<Vec<DebouncedEvent>>> = Arc::new(Mutex::new(vec![]));

    // create a channel for flush events
    let (tx, rx) = channel();
    start_watch_thread(config.local_root.clone(), tx, &mut events);

    loop {
        let _ =  rx.recv();
        flush_events(&config, &mut remote, &mut events);
    }

}


fn main() {
    let opts = CliOptions::from_args();

    match opts.subcommand {
        Some(SubCommand::Init(remote_config)) => init::init_dirsync_dir(remote_config).unwrap(),
        Some(SubCommand::Clean) => {
            let config = SessionConfig::get(opts);
            let mut remote = remote::Remote::connect(&config);
            remote.remove_dir(config.remote.root.as_str());
        },
        _ => {
            let config = SessionConfig::get(opts);
            start_main_loop(&config);
        }
    };
}
