
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::cli::RemoteConfigRecord;
// use std::error::Error;
// use ssh_config::SSHConfig;

use crate::cli::CliOptions;

fn default_as_true() -> bool {
    true
}

#[derive(Debug)]
#[derive(Deserialize, Serialize)]
pub struct Config {
    #[serde(alias = "ignoreGitignore", default = "default_as_true")]
    pub ignore_gitignore: bool,
    pub remote: RemoteConfigRecord
}

impl Config {
    pub fn new(remote: RemoteConfigRecord) -> Config {
        return Config {
            ignore_gitignore: true,
            remote: remote
        };
    }
}


impl RemoteConfigRecord {
    fn host_string(&self) -> String {
        let mut s: String = String::new();
        let host = &format!(
            "{}@{}", 
            &self.user.clone(),
            &self.host.clone());
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


impl SessionConfig {

    pub fn host_port_string(&self) -> String {
        let mut s: String = String::new();
        let host = &format!(
            "{}:22",
            &self.remote.host.clone());
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

    pub fn with_local_root(local_root: &String) -> SessionConfig {

        let mut config_path = PathBuf::new();
        config_path.push(local_root.clone());
        config_path.push(".dirsync");
        config_path.push("config.json");

        let config_string = fs::read_to_string(config_path)
        .expect("failed to read config");
        let config: Config = serde_json::from_str(&config_string)
        .expect("failed to deserialize json");

        return SessionConfig {
            local_root: local_root.clone(),
            remote: config.remote,
            ignore_gitignore: config.ignore_gitignore
        }
    }

    pub fn get(args: CliOptions) -> SessionConfig {
        let local_root = args.source.unwrap_or(".".to_string());
        return SessionConfig::with_local_root(&local_root);
    }
}