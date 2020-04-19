use std::fs;
use crate::cli::RemoteConfigRecord;
use crate::config::Config;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub enum InitError {
    Io(std::io::Error),
    Serde(serde_json::error::Error)
}

fn create_dirsync_dirs() -> Result<(), std::io::Error> {
    fs::create_dir_all("./.dirsync/actions/onSyncDidFinish")?;
    fs::create_dir_all("./.dirsync/actions/onSessionDidStart")?;
    Ok(())
}

pub fn init_dirsync_dir(remote_options: RemoteConfigRecord) ->  Result<(), InitError> {
    create_dirsync_dirs().map_err(|err| InitError::Io(err))?;
    let _ignore_file = File::create("./.dirsync/ignore").map_err(|err| InitError::Io(err))?;
    let mut config_file = File::create("./.dirsync/config.json").map_err(|err| InitError::Io(err))?;
    let config = Config::new(remote_options);
    let json = serde_json::to_string_pretty(&config).map_err(|err| InitError::Serde(err))?;
    config_file.write_all(json.as_bytes()).map_err(|err| InitError::Io(err))?;
    Ok(())
}