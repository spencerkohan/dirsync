mod cli;
mod config;
mod init;
mod remote;
mod watch;

extern crate notify;

use crate::cli::CliOptions;
use crate::cli::SubCommand;
use crate::config::SessionConfig;
use crate::remote::Remote;
use clap::Parser;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use remote::receive_from_remote::watch_remote_receivable_paths;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::vec::Vec;

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

    let exclude_gitignore = config.ignore_gitignore && Path::new(".gitignore").exists();
    let exclude_file = Path::new(config.exclude_path().to_str().unwrap()).exists();

    let mut args: Vec<String> = Vec::new();
    if exclude_gitignore {
        args.push(String::from("--exclude-from=.gitignore"));
    }
    if exclude_file {
        args.push(String::from(format!(
            "--exclude-from={}",
            config.exclude_path().to_str().unwrap()
        )));
    }

    // exclude remote receive paths
    if let Some(paths) = &config.remote.receive_paths {
        for path in paths {
            args.push(String::from(format!("--exclude={}", path.path)));
        }
    }

    rsync(&config.local_root, &config.destination(), &args)
}

fn filter(event: DebouncedEvent) -> Option<DebouncedEvent> {
    match event {
        DebouncedEvent::NoticeWrite(_) => None,
        DebouncedEvent::NoticeRemove(_) => None,
        DebouncedEvent::Rescan => None,
        DebouncedEvent::Error(_, _) => None,
        _ => Some(event),
    }
}

fn start_watch_thread(
    root: String,
    flush_signal: Sender<()>,
    events: &mut Arc<Mutex<Vec<DebouncedEvent>>>,
) {
    let events = Arc::clone(events);
    thread::spawn(move || {
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
                            thread::spawn(move || {
                                sleep(Duration::from_millis(20));
                                signal.send(()).unwrap();
                            });
                        }
                        None => println!("ignoring event"),
                    };
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });
}

fn flush_events(
    config: &SessionConfig,
    remote: &mut Remote,
    events: &mut Arc<Mutex<Vec<DebouncedEvent>>>,
) {
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

    watch_remote_receivable_paths(config.clone());

    loop {
        let _ = rx.recv();
        flush_events(&config, &mut remote, &mut events);
    }
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn print_version() {
    println!("{}", version());
}

fn main() {
    let opts = CliOptions::parse();

    match &opts.subcommand {
        Some(SubCommand::Version) => {
            print_version();
            exit(0);
        }
        Some(SubCommand::Init(remote_config)) => match init::init_dirsync_dir(remote_config) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error initializing dirsync: {}", err);
                exit(1);
            }
        },
        Some(SubCommand::Clean) => {
            let config = match SessionConfig::get(opts) {
                Ok(config) => config,
                Err(config::ReadSessionConfigError::DoesNotExist) => {
                    eprintln!("Fatal: not a dirsync directory");
                    eprintln!("There is no configuration file located at .dirsync/config.json");
                    eprintln!("To initialize this as a dirsync directory, use: `dirsync init`");
                    exit(1);
                }
                Err(err) => {
                    eprintln!("Error loading configuration file: {}", err);
                    exit(1);
                }
            };
            let mut remote = remote::Remote::connect(&config);
            remote.remove_dir(config.remote.root.as_str());
        }
        Some(SubCommand::Remote { subcommand }) => {
            let config = match SessionConfig::get(opts.clone()) {
                Ok(config) => config,
                Err(config::ReadSessionConfigError::DoesNotExist) => {
                    eprintln!("Fatal: not a dirsync directory");
                    eprintln!("There is no configuration file located at .dirsync/config.json");
                    eprintln!("To initialize this as a dirsync directory, use: `dirsync init`");
                    exit(1);
                }
                Err(err) => {
                    eprintln!("Error loading configuration file: {}", err);
                    exit(1);
                }
            };
            subcommand.execute(&config);
        }
        Some(SubCommand::Watch { root, roots }) => watch::watch_paths(root, roots),
        _ => {
            let config = match SessionConfig::get(opts) {
                Ok(config) => config,
                Err(config::ReadSessionConfigError::DoesNotExist) => {
                    eprintln!("Fatal: not a dirsync directory");
                    eprintln!("There is no configuration file located at .dirsync/config.json");
                    eprintln!("To initialize this as a dirsync directory, use: `dirsync init`");
                    exit(1);
                }
                Err(err) => {
                    eprintln!("Error loading configuration file: {}", err);
                    exit(1);
                }
            };
            start_main_loop(&config);
        }
    };
}
