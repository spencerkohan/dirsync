use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

use crate::remote::cli::RemoteSubcommand;

#[derive(Debug, Subcommand, Clone)]
pub enum SubCommand {
    #[command(arg_required_else_help = true, name = "init")]
    #[command(about = "Initialize dirsync for a directory")]
    Init(RemoteConfigRecord),

    #[command(name = "clean")]
    #[command(about = "Delete the contents of the remote directory")]
    Clean,

    #[command(name = "remote")]
    #[command(about = "Tools for working with the remote")]
    Remote {
        #[command(subcommand)]
        subcommand: RemoteSubcommand,
    },

    #[command(name = "watch")]
    #[command(about = "Watch a set of files, and emit any events")]
    Watch {
        #[arg(short, long)]
        root: String,

        #[arg(trailing_var_arg = true)]
        roots: Vec<String>,
    },

    #[command(name = "version")]
    #[command(about = "Print the current version")]
    Version,
}

#[derive(Debug, Parser, Clone)]
#[command(version = "0.2", author = "Spencer Kohan")]
#[command(name = "dirsync")]
#[command(
    about = "A tool for syncronizing directories between a local and remote host",
    long_about = r#"
Dirsync is a tool for syncronizing a local directory with a remote host, built on top of rsync.

While dirsync is running, it observes file system events in the local directory, and uses rsync to push any altered files to the remote host.

More information can be found here: https://github.com/spencerkohan/dirsync
"#
)]
pub struct CliOptions {
    // The locaal root directory to be synchronized
    pub source: Option<String>,
    // Initialize the .dirsync directory
    #[command(subcommand)]
    pub subcommand: Option<SubCommand>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReceivePathRec {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Args)]
pub struct RemoteConfigRecord {
    /// The remote root of the sync directory
    #[arg(short, long)]
    pub root: String,

    /// The remote host
    #[arg(short, long)]
    pub host: String,

    /// The remote user
    #[arg(short, long)]
    pub user: String,

    #[clap(skip)]
    pub receive_paths: Option<Vec<ReceivePathRec>>,
}
