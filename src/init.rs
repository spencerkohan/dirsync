use std::fs;
use thiserror::Error;

use crate::cli::RemoteConfigRecord;
use crate::config::Config;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Config file error: {0}")]
    Io(std::io::Error),
    #[error("TOML error: {0}")]
    Toml(String),
}

fn create_dirsync_dirs() -> Result<(), std::io::Error> {
    fs::create_dir_all("./.dirsync/actions/onSyncDidFinish")?;
    fs::create_dir_all("./.dirsync/actions/onSessionDidStart")?;
    Ok(())
}

pub fn init_dirsync_dir(remote_options: RemoteConfigRecord) -> Result<(), InitError> {
    create_dirsync_dirs().map_err(|err| InitError::Io(err))?;
    let _ignore_file = File::create("./.dirsync/ignore").map_err(|err| InitError::Io(err))?;
    let mut config_file =
        File::create("./.dirsync/config.toml").map_err(|err| InitError::Io(err))?;
    let config = Config::new(remote_options);
    let json = toml::to_string_pretty(&config).map_err(|err| InitError::Toml(err.to_string()))?;
    config_file
        .write_all(json.as_bytes())
        .map_err(|err| InitError::Io(err))?;
    Ok(())
}
