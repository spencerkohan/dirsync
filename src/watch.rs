use std::{sync::mpsc::channel, time::Duration};

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    Create,
    Write,
    Chmod,
    Remove,
    Rename,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileEvent {
    Create(String),
    Write(String),
    Chmod(String),
    Remove(String),
    Rename(String, String),
}

pub fn watch_paths(root: &String, relative_paths: &Vec<String>) {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_millis(20)).unwrap();

    for path in relative_paths {
        watcher
            .watch(format!("{}/{}", root, path), RecursiveMode::Recursive)
            .unwrap();
    }

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("ehandling event: {:?}", event);
                match map(event) {
                    Some(event) => {
                        if let Ok(json) = serde_json::to_string(&event) {
                            println!("{json}");
                        }
                    }
                    None => eprintln!("ignoring event"),
                };
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }
}

fn map(event: DebouncedEvent) -> Option<FileEvent> {
    match event {
        DebouncedEvent::NoticeWrite(_) => None,
        DebouncedEvent::NoticeRemove(_) => None,
        DebouncedEvent::Rescan => None,
        DebouncedEvent::Error(_, _) => None,
        DebouncedEvent::Create(path) => Some(FileEvent::Create(path.to_string_lossy().to_string())),
        DebouncedEvent::Write(path) => Some(FileEvent::Write(path.to_string_lossy().to_string())),
        DebouncedEvent::Chmod(path) => Some(FileEvent::Chmod(path.to_string_lossy().to_string())),
        DebouncedEvent::Remove(path) => Some(FileEvent::Remove(path.to_string_lossy().to_string())),
        DebouncedEvent::Rename(from, to) => Some(FileEvent::Rename(
            from.to_string_lossy().to_string(),
            to.to_string_lossy().to_string(),
        )),
    }
}
