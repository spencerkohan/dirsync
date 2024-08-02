use std::{
    collections::HashMap,
    io::{BufReader, Read, Write},
    thread,
};

use ssh2::Channel;
use thiserror::Error;

use crate::config::SessionConfig;

use super::Remote;

pub fn exec_remote(
    config: &SessionConfig,
    command: &str,
) -> Result<(String, i32), RemoteExecError> {
    let mut remote = Remote::connect(config);
    remote.try_exec(command)
}

pub struct RemoteCommand {
    command: String,
    pub channel: Channel,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum RemoteExecError {
    #[error("Failed to execute remote command: {0} with eror: {1}")]
    ExecError(String, String),
}

impl Remote {
    pub fn command(&self, command: &str) -> Result<RemoteCommand, RemoteExecError> {
        let path = self.root.clone();
        let path_str = path.to_str().unwrap();
        let cmd = &format!(r#"cd {} && {}"#, &path_str, &command);

        let channel = match self.session.channel_session() {
            Ok(channel) => channel,
            Err(err) => return Err(RemoteExecError::ExecError(cmd.to_string(), err.to_string())),
        };

        Ok(RemoteCommand {
            command: cmd.to_string(),
            channel,
            environment: Default::default(),
        })
    }
}

impl RemoteCommand {
    pub fn env(mut self, var: &str, val: &str) -> Result<Self, ssh2::Error> {
        self.environment.insert(var.to_string(), val.to_string());
        Ok(self)
    }

    pub fn exec(&mut self) -> Result<(), RemoteExecError> {
        let environ: String = self
            .environment
            .iter()
            .map(|(key, val)| format!("{key}={val}"))
            .collect::<Vec<String>>()
            .join(" ");

        let cmd = format!("{environ} {}", self.command);

        println!("Executing command: \n\n{cmd}\n\n");

        if let Err(err) = self.channel.request_pty("term", None, None) {
            return Err(RemoteExecError::ExecError(
                self.command.to_string(),
                err.to_string(),
            ));
        };

        if let Err(err) = self.channel.exec(&cmd) {
            return Err(RemoteExecError::ExecError(
                self.command.to_string(),
                err.to_string(),
            ));
        };
        Ok(())
    }

    pub fn wait_close(&mut self) -> Result<i32, std::io::Error> {
        self.channel.send_eof()?;
        self.channel.wait_close()?;
        let exit_status = self.channel.exit_status()?;
        Ok(exit_status)
    }

    pub fn result_string(mut self) -> Result<(String, i32), RemoteExecError> {
        self.exec()?;
        let mut s = String::new();
        if let Err(err) = self.channel.read_to_string(&mut s) {
            return Err(RemoteExecError::ExecError(
                self.command.clone(),
                err.to_string(),
            ));
        }

        let status_code = match self.wait_close() {
            Ok(code) => code,
            Err(err) => {
                return Err(RemoteExecError::ExecError(
                    self.command.clone(),
                    err.to_string(),
                ));
            }
        };

        Ok((s, status_code))
    }

    pub fn stream_to_end(mut self) -> Result<i32, RemoteExecError> {
        println!("stream_to_end: exec({})", self.command);
        self.exec()?;

        if let Err(err) = self.stream_impl() {
            return Err(RemoteExecError::ExecError(self.command, err.to_string()));
        };

        println!("stream_to_end: streamed result");

        let status_code = match self.wait_close() {
            Ok(code) => code,
            Err(err) => {
                return Err(RemoteExecError::ExecError(self.command, err.to_string()));
            }
        };
        println!("stream_to_end: finished: {status_code}");
        Ok(status_code)
    }

    fn stream_impl(&mut self) -> Result<(), std::io::Error> {
        let mut stdout = std::io::stdout();
        let mut stderr = std::io::stderr();
        let mut stdout_reader = BufReader::new(self.channel.stream(0));
        let mut stderr_reader = BufReader::new(self.channel.stderr());

        // Spawn threads to read stdout and stderr concurrently
        let stdout_handle = thread::spawn(move || -> std::io::Result<()> {
            loop {
                let mut buffer = [0; 1024];
                match stdout_reader.read(&mut buffer) {
                    Ok(0) => break, // EOF reached
                    Ok(n) => stdout.write_all(&buffer[..n])?,
                    Err(e) => return Err(e),
                }
                stdout.flush()?;
            }
            Ok(())
        });

        let stderr_handle = thread::spawn(move || -> std::io::Result<()> {
            loop {
                let mut buffer = [0; 1024];
                match stderr_reader.read(&mut buffer) {
                    Ok(0) => break, // EOF reached
                    Ok(n) => stderr.write_all(&buffer[..n])?,
                    Err(e) => return Err(e),
                }
                stderr.flush()?;
            }
            Ok(())
        });

        // Wait for both threads to finish
        stdout_handle.join().unwrap()?;
        stderr_handle.join().unwrap()?;

        Ok(())
    }
}
