use structopt::StructOpt;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[derive(StructOpt)]
#[derive(Clone)]
pub enum SubCommand {
    #[structopt(name="init")]
    Init(RemoteConfigRecord)
}

#[derive(Debug)]
#[derive(StructOpt)]
#[derive(Clone)]
#[structopt(version = "0.1", author = "Spencer Kohan")]
pub struct CliOptions {
    // The locaal root directory to be synchronized
    pub source: Option<String>,
    // Initialize the .dirsync directory
    #[structopt(subcommand)]
    pub subcommand: Option<SubCommand>
}

#[derive(Debug)]
#[derive(StructOpt)]
#[derive(Clone)]
#[derive(Deserialize, Serialize)]
pub struct RemoteConfigRecord {
    /// The remote root of the sync directory
    #[structopt(short = "r", long = "root")]
    pub root: String,
    /// The remote host
    #[structopt(short = "h", long = "host")]
    pub host: String,

    /// The remote user
    #[structopt(short = "u", long = "user")]
    pub user: String,
}
