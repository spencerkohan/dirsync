use thiserror::Error;

use crate::remote::Remote;
use crate::remote::RemoteExecError;

#[derive(Debug, Error)]
pub enum RemoteInstallError {
    #[error("Remote install error: {0}")]
    Error(String),
}

impl From<RemoteExecError> for RemoteInstallError {
    fn from(value: RemoteExecError) -> Self {
        Self::Error(value.to_string())
    }
}

impl From<std::io::Error> for RemoteInstallError {
    fn from(value: std::io::Error) -> Self {
        Self::Error(value.to_string())
    }
}

impl From<ssh2::Error> for RemoteInstallError {
    fn from(value: ssh2::Error) -> Self {
        Self::Error(value.to_string())
    }
}

impl Remote {
    /// This function attempts to install dirsync on the remote host.
    /// First, it's checked if dirsync exists with the same version as the localhost.
    /// If not, dirsync will attempt to download and install it from github.
    pub fn install_dirsync(&mut self) -> Result<(), RemoteInstallError> {
        let (_, status_code) = self.try_exec("which dirsync")?;

        if status_code != 0 {
            self.install_vendored_dirsync()?;
        }

        Ok(())
    }

    /// This function attempts to remove dirsync from the remote host
    pub fn remove_dirsync(&mut self) -> Result<(), RemoteInstallError> {
        let status_code = self.command(r#"rm -rf .dirsync/client"#)?.stream_to_end()?;
        if status_code != 0 {
            return Err(RemoteInstallError::Error(format!(
                "Failed to remove dirsync with status: {status_code}"
            )));
        }
        Ok(())
    }

    pub fn dirsync_client_dir(&self) -> &str {
        r#".dirsync/client"#
    }

    fn install_vendored_dirsync(&mut self) -> Result<(), RemoteInstallError> {
        let status_code = self
            .command(
                format!(
                    r#"echo "Making dir: {client_dir}" \
                && mkdir -p {client_dir} \
                && cd {client_dir} \
                && ls \
                "#,
                    client_dir = self.dirsync_client_dir()
                )
                .as_str(),
            )?
            .stream_to_end()?;

        if status_code != 0 {
            return Err(RemoteInstallError::Error(format!(
                "Failed make directory with status: {status_code}"
            )));
        }

        let status_code = self
            .command(
                format!(
                    r#"echo "cloning into dir: {client_dir}"\
                && cd {client_dir} \
                && ls \
                && git clone https://github.com/spencerkohan/dirsync
                "#,
                    client_dir = self.dirsync_client_dir()
                )
                .as_str(),
            )?
            .stream_to_end()?;

        if status_code != 0 {
            return Err(RemoteInstallError::Error(format!("Failed to clone https://github.com/spencerkohan/dirsync with status: {status_code}")));
        }

        let status_code = self
            .command(
                format!(
                    r#"
                    cd {client_dir}/dirsync \
                    && cargo build
                    "#,
                    client_dir = self.dirsync_client_dir()
                )
                .as_str(),
            )?
            .stream_to_end()?;

        if status_code != 0 {
            return Err(RemoteInstallError::Error(format!(
                "Failed to build dirsync with status: {status_code}"
            )));
        }

        Ok(())
    }
}
