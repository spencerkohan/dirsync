use clap::Subcommand;

use crate::{config::SessionConfig, remote::exec_remote};

use super::Remote;

#[derive(Debug, Subcommand, Clone)]
pub enum RemoteSubcommand {
    #[command(name = "exec")]
    #[command(about = "Execute a command on the remote")]
    Exec {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    #[command(name = "install")]
    #[command(about = "Install dirsync on the remote")]
    Install,

    #[command(name = "uninstall")]
    #[command(about = "Uninstall dirsync from the remote")]
    Uninstall,
}

impl RemoteSubcommand {
    pub fn execute(&self, config: &SessionConfig) -> i32 {
        match self {
            RemoteSubcommand::Exec { args } => {
                let command = args.join(" ");
                println!("Executing remote command: {}", command);
                let (output, code) = match exec_remote(&config, command.as_str()) {
                    Ok(result) => result,
                    Err(err) => {
                        eprintln!("{}", err);
                        return 1;
                    }
                };
                println!("[{code}]:\n{output}");
                0
            }
            RemoteSubcommand::Install => {
                if let Err(err) = Remote::connect(config).install_dirsync() {
                    panic!("{err}");
                };
                1
            }
            RemoteSubcommand::Uninstall => {
                if let Err(err) = Remote::connect(config).remove_dirsync() {
                    panic!("{err}");
                };
                1
            }
        }
    }
}
