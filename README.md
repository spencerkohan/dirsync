# Dirsync

Dirsync is a tool for live-updating the contents of a remote directoy to match the contents of a local directory.  Dirsync is similar to rsync, except instead of working as a one-time operation, dirsync watches the local directory and pushes changes to the remote host whenever a file is changed locally.  In fact, dirsync is built upon rsync.

This tool should be easy to understand for those who are already familiar with `rsync` and `ssh`.

## Installation

The easiest way to install dirsync is with cargo install:

```
// clone the repo:
$ git clone https://github.com/spencerkohan/dirsync && cd dirsync
// install
$ cargo install --path .
``` 

***Note:*** this crate relies on openssl to be installed on the system.  Instructions can be found [here](https://docs.rs/openssl/0.10.29/openssl/)

## Usage

### Initialization

Before dirsync can be used for a local directory, it has to be initialized.  This sets up the `.dirsync` directory which is used to manage the configuration.  This is handled by the `init` command.

For instance, if you would use the following command to synch a directory using `rsync`:

```
$ rsync -r . myUser@myRemoteHost:/path/to/sync
```

then you would use the following initialization configuration:

```
$ dirsync init -u myUser -h myRemoteHost -p /path/to/sync`
```

### Synching

Once initialization has taken place, a dirsync session can be started with the simple command:

```
$ dirsync
```

While the session is running, any changes to the local directory will be pushed to the remote specified in the configuration.

### Configuration

All configuration of `dirsync` is handled by the `.dirsync` directory, which is created by `$ dirsync init`.  This directory has the following contents:

```
.dirsync/
├── actions
│   └── onSyncDidFinish
│       └── remote
├── config.json
└── ignore
```

The elements here are:

#### config.json

This is a file which contains the configuration options for specifying the remote host, and the remote directory.  It has this format:

```
{
    "remote": {
        "root": "path/to/remote",
        "host": "hostName",
        "user": "userName",
        "port": "22",               // optional
        "identityFile": "id_rsa"    // optional
    },
    "ignoreGitignore" : true
}
```

The felds are:

- `remote.root`: the path to the directory which will be synced on the remote host.

- `remote.host`: the hostname of the remote host.

- `remote.port`: the ssh port on the remote host.  If the port is omitted, the default value is 22.

- `remote.identiyFile` is the identity file which should be used to connect to the host over ssh.  Dirsync currently only supports authentication via ssh keys.  If this value is omitted, the ssh-agent's default key will be used.

- `ignoreGitignore`: an option to specify whether paths listed in the top-level .gitignore file shoul be ignored by dirsync.  Default is true.

#### ignore file

The ignore file specifies paths which should not be synced by dirsync.  The format of the ignore file is identical to what would be passed to the `--exclude-from` option of rsync.

### Action triggers

The `.dirsync/actions` directory houses executables which are triggered by certain dirsync events.

Currently the only action supported is `onSyncDidFinish`.  This action is triggered after any sync event from the local to the remote (i.e. after a local file in the watched directory has changed, and the resulting sync has completed).

What this means is, any script located at `.dirsync/actionos/onSyncDidFinish` will be executed on the remote following any sync event.

So for example, if you were synchoronizing a rust project with your remote host, and you wanted to build the project every time a change is pushed, you could implement this `onSyncDidFinish` event:

```
#!/bin/bash
cargo build
```

This script will always be executed from the root of the synced directory.


