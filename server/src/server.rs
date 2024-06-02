use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::thread::spawn;
use std::thread::JoinHandle;

use libc::c_int;
use libc::close;
use libc::STDOUT_FILENO;
use libc::{dup, dup2};

use crate::shell::Shell;

pub struct Server {
    shell: Shell,
    uds_cmd_path: String,
    uds_output_path: String,
}

impl Server {
    pub fn new(shell_: Shell, uds_path_: String, uds_output_path_: String) -> Server {
        Server {
            shell: shell_,
            uds_cmd_path: uds_path_,
            uds_output_path: uds_output_path_,
        }
    }

    fn handle_cmd_connect(mut conn: UnixStream, shell: Shell) -> Result<(), String> {
        let mut buf = String::new();
        loop {
            conn.read_to_string(&mut buf)
                .map_err(|err| err.to_string())?;
            shell.run_command(&buf)?;
        }
    }

    fn cmd_thread(path: String, shell: Shell) -> Result<(), String> {
        while let Ok(conn) = UnixListener::bind(&path)
            .map_err(|err| format!("bind err: {:?}", err))?
            .incoming()
            .next()
            .ok_or("listen err")?
        {
            spawn({
                let conn_copy = conn
                    .try_clone()
                    .map_err(|err| format!("clone err: {:?}", err))?;
                let shell_copy = shell.clone();
                move || {
                    let _ = Server::handle_cmd_connect(conn_copy, shell_copy)
                        .map_err(|err| format!("handle cmd connect err: {:?}", err));
                }
            });
        }

        Ok(())
    }

    fn redirect_stdout_to_unix_stream(stream: UnixStream) -> c_int {
        let original_fd = unsafe { dup(STDOUT_FILENO) }; // 保存原始stdout的文件描述符
        let ret = original_fd;

        let stream_fd = stream.as_raw_fd(); // 获取UnixStream的文件描述符
        unsafe { dup2(stream_fd, STDOUT_FILENO) }; // 将stdout的文件描述符重定向到UnixStream

        ret
    }

    fn restore_stdout(old: c_int) {
        unsafe { dup2(old, STDOUT_FILENO) }; // 恢复原始stdout的文件描述符
        unsafe { close(old) }; // 关闭原始文件描述符
    }

    fn output_thread(path: String) -> Result<(), String> {
        let mut future: Option<JoinHandle<()>> = None;
        let mut old_conn: Option<UnixStream> = None;
        while let Ok(conn) = UnixListener::bind(&path)
            .map_err(|err| format!("bind err: {:?}", err))?
            .incoming()
            .next()
            .ok_or("listen err")?
        {
            if let Some(o) = old_conn.take() {
                drop(o);
                future.take().unwrap().join().unwrap();
            }

            old_conn = Some(conn.try_clone().map_err(|err| err.to_string())?);

            let mut conn_copy = conn.try_clone().map_err(|err| err.to_string())?;

            let old_stdout = Server::redirect_stdout_to_unix_stream(conn);

            future = Some(spawn(move || {
                let mut buf = String::new();
                let _ = conn_copy
                    .read_to_string(&mut buf)
                    .map_err(|err| err.to_string());
                Server::restore_stdout(old_stdout);
            }));
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        let uds_cmd_path = self.uds_cmd_path.clone();
        let uds_output_path = self.uds_output_path.clone();

        let shell_copy = self.shell.clone();

        let command_thread = spawn(move || Server::cmd_thread(uds_cmd_path, shell_copy));
        let output_thread = spawn(move || Server::output_thread(uds_output_path));

        let _ = command_thread
            .join()
            .map_err(|err| format!("run command err: {:?}", err))?;
        let _ = output_thread
            .join()
            .map_err(|err| format!("run output err: {:?}", err))?;

        Ok(())
    }
}
