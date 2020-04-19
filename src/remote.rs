extern crate ssh2;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::PathBuf;

use ssh2::Session;
use ssh2::ExtendedData;
use crate::config::SessionConfig;


pub struct Remote {
    session: ssh2::Session,
    root: PathBuf
}

impl Remote {

    // todo: this shouold take configuration arguemnts
    pub fn connect(config: &SessionConfig) -> Remote {
        let tcp = TcpStream::connect(
            &config.host_port_string().as_str()
        ).unwrap();
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();
    
        // Try to authenticate with the first identity in the agent.
        sess.userauth_agent(&config.remote.user.clone().as_str()).unwrap();
    
        // Make sure we succeeded
        assert!(sess.authenticated());
    
        let mut root = PathBuf::new();
        root.push( &config.remote.root.clone() );

        return Remote { 
            session: sess,
            root: root
        }
    }

    fn exec(&mut self, command: &str) -> String {
        let path = self.root.clone();
        let path_str = path.to_str().unwrap();
        let cmd = &format!("cd {} && {}", &path_str, &command);

        let channel = &mut self.session.channel_session().unwrap();
        channel.exec(
            &cmd
        ).unwrap();

        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        println!("exec: {}", &cmd);
        let _ = channel.wait_close();
        return s;
    }

    fn exec_stream(&mut self, command: &str) {
        let path = self.root.clone();
        let path_str = path.to_str().unwrap();
        let cmd = &format!("cd {} && {}", &path_str, &command);

        let channel = &mut self.session.channel_session().unwrap();

        let mut channel_out = channel.stream(0);
        
        channel.handle_extended_data(ExtendedData::Merge).unwrap();
        channel.request_pty("term", None, None).unwrap();
        channel.exec(
            &cmd
        ).unwrap();

        std::io::copy(&mut channel_out, &mut std::io::stdout()).unwrap();
        let _ = channel.wait_close();
    }

    fn file_exists(&mut self, filename: &str) -> bool {
        let command = &format!("test -f {} && echo 1 || echo 0", filename);
        let s = self.exec(&command);
        return s.as_str() == "1\n";
    }

    pub fn execute_if_exists(&mut self, event: &str) {
        let mut path = self.root.clone();
        path.push(".dirsync/actions");
        path.push(event);
        path.push("remote");
        let path_str = path.to_str().unwrap();

        if !self.file_exists(&path_str) {
            println!("file does not exist: {}", &path_str);
            return
        }

        let command1 = &format!("chmod +x {}", &path_str);
        let _ = self.exec(&command1);
        
        let command2 = &format!("{}", &path_str);
        self.exec_stream(&command2);
    }

    pub fn remove_dir(&mut self, path: &str) {
        let command = &format!("rm -rf {}", path);
        let s = self.exec(&command);
        print!("clean result: {}\n", s);
    }

}