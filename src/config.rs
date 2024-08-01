use crate::cli::RemoteConfigRecord;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

use crate::cli::CliOptions;

fn default_as_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(alias = "ignoreGitignore", default = "default_as_true")]
    pub ignore_gitignore: bool,
    pub remote: RemoteConfigRecord,
}

impl Config {
    pub fn new(remote: RemoteConfigRecord) -> Config {
        return Config {
            ignore_gitignore: true,
            remote,
        };
    }
}

impl RemoteConfigRecord {
    fn host_string(&self) -> String {
        let mut s: String = String::new();
        let host = &format!("{}@{}", &self.user.clone(), &self.host.clone());
        s.push_str(host);
        return s;
    }
}

#[derive(Debug)]
pub struct SessionConfig {
    // The root directory to sync to the remote
    pub local_root: String,
    pub remote: RemoteConfigRecord,
    pub ignore_gitignore: bool,
}

#[derive(Error, Debug)]
pub enum ReadSessionConfigError {
    #[error("Does not exist")]
    DoesNotExist,
    #[error("Failed to read config file: {0}")]
    FailedToRead(String),
    #[error("Failed to deserialize config file: {0}")]
    FailedToDeserialzie(String),
}

impl SessionConfig {
    pub fn host_port_string(&self) -> String {
        let mut s: String = String::new();
        let host = &format!("{}:22", &self.remote.host.clone());
        s.push_str(host);
        return s;
    }

    pub fn exclude_path(&self) -> PathBuf {
        let mut path = PathBuf::new();
        path.push(self.local_root.clone());
        path.push(".dirsync");
        path.push("ignore");
        return path;
    }

    pub fn destination(&self) -> String {
        let mut s: String = String::new();
        s.push_str(&self.remote.host_string().as_str());
        s.push_str(":");
        s.push_str(self.remote.root.clone().as_str());
        return s;
    }

    pub fn with_local_root(local_root: &String) -> Result<SessionConfig, ReadSessionConfigError> {
        let mut config_path = PathBuf::new();
        config_path.push(local_root.clone());
        config_path.push(".dirsync");
        config_path.push("config.json");

        let config_string = match fs::read_to_string(config_path) {
            Ok(config_string) => config_string,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => return Err(ReadSessionConfigError::DoesNotExist),
                _ => {
                    return Err(ReadSessionConfigError::FailedToRead(err.to_string()));
                }
            },
        };
        let config: Config = match serde_json::from_str(&config_string) {
            Ok(config) => config,
            Err(err) => return Err(ReadSessionConfigError::FailedToDeserialzie(err.to_string())),
        };

        return Ok(SessionConfig {
            local_root: local_root.clone(),
            remote: config.remote,
            ignore_gitignore: config.ignore_gitignore,
        });
    }

    pub fn get(args: CliOptions) -> Result<SessionConfig, ReadSessionConfigError> {
        let local_root = args.source.unwrap_or(".".to_string());
        SessionConfig::with_local_root(&local_root)
    }
}
