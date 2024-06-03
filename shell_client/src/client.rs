use crate::{autocomplete_reader::AutoCompleteReader, sys::get_process_list};
use shell_core::*;
use std::{
    os::unix::net::UnixStream,
    sync::{Arc, Mutex},
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

pub struct Client {
    cmd_channel: Option<UnixStream>,
    output_channel: Option<UnixStream>,
    copy_stdout: Option<JoinHandle<()>>,
    reader: Arc<Mutex<Box<AutoCompleteReader>>>,
}

static DEFAULT_PS1: &str = "\x1B[33m>> \x1B[0m";

impl Client {
    pub fn new() -> Client {
        Client {
            cmd_channel: None,
            output_channel: None,
            copy_stdout: None,
            reader: AutoCompleteReader::new().unwrap(),
        }
    }

    fn find_process(&self, arg: &Argument) -> Vec<(String, u64)> {
        let result: Vec<(String, u64)> = get_process_list()
            .into_iter()
            .map(|(pid, name)| (name, pid.parse().unwrap()))
            .collect();

        match arg {
            Argument::Str(name) => result
                .into_iter()
                .filter(|(name_, _)| name_.starts_with(name))
                .collect::<Vec<(String, u64)>>(),
            Argument::Int(pid) => result
                .into_iter()
                .filter(|(_, pid_)| pid.to_owned() as u64 == *pid_)
                .collect(),
        }
    }

    fn make_uds_path(pid: &u64) -> (String, String) {
        (
            format!("/tmp/rust_shell_cmd_{}", pid),
            format!("/tmp/rust_shell_output_{}", pid),
        )
    }

    fn parse_auto_complete(line: &str) -> Vec<String> {
        line.split_whitespace().map(|s| s.to_owned()).collect()
    }

    fn attach_process(&mut self, args: &Vec<Argument>) -> Result<(), String> {
        if args.len() != 1 {
            return Err(format!("argument number error"));
        }

        let pids = self.find_process(&args[0]);
        if pids.is_empty() {
            return Err(format!("process not found"));
        }
        if pids.len() != 1 {
            return Err(format!("multiple process found: \n{}", {
                pids.iter()
                    .map(|(comm, pid)| format!("{}: {}", comm, pid))
                    .collect::<Vec<String>>()
                    .join("\n")
            }));
        }

        let (cmd_path, output_path) = Self::make_uds_path(&pids[0].1);
        self.cmd_channel = Some(UnixStream::connect(&cmd_path).map_err(|err| err.to_string())?);
        self.output_channel =
            Some(UnixStream::connect(&output_path).map_err(|err| err.to_string())?);

        if let Some(c) = &self.cmd_channel {
            self.reader
                .lock()
                .map_err(|err| err.to_string())?
                .append_debug_command_complete_data(
                    Client::parse_auto_complete(&read_line(
                        &mut c.try_clone().map_err(|err| err.to_string())?,
                    )?)
                    .into_iter()
                    .map(|x| (x.clone(), x.clone()))
                    .collect(),
                )
        }

        let mut output_channel_copy = self
            .output_channel
            .as_ref()
            .ok_or("not attach to process")?
            .try_clone()
            .map_err(|err| err.to_string())?;

        self.copy_stdout = Some(spawn(move || {
            let _ = std::io::copy(&mut output_channel_copy, &mut std::io::stdout());
        }));

        self.reader
            .lock()
            .map_err(|err| err.to_string())?
            .set_prompt(format!("\x1B[32m{} >> \x1B[0m", pids[0].0).as_str());

        Ok(())
    }

    fn exit() -> Result<(), String> {
        Err("exit".to_owned())
    }

    fn run_builtin_command(&mut self, cmd: &String, args: &Vec<Argument>) -> Result<(), String> {
        match cmd.as_str() {
            "attach" => self.attach_process(&args),
            "detach" => {
                self.detach_process();
                Ok(())
            }
            "exit" => Self::exit(),
            _ => Err("custom".to_owned()),
        }
    }

    pub fn run_custom_command(&mut self, line: &String) -> Result<(), String> {
        match self.cmd_channel {
            None => Err("not attach to process".to_owned()),
            Some(ref mut cmd_channel) => write_line(cmd_channel, line),
        }
    }

    pub fn detach_process(&mut self) {
        self.cmd_channel = None;
        self.output_channel = None;
        self.copy_stdout = None;
        self.reader
            .lock()
            .expect("lock reader failed")
            .set_prompt(DEFAULT_PS1);
    }

    fn init_reader(&mut self) -> Result<(), String> {
        let mut r = self.reader.lock().map_err(|err| err.to_string())?;
        r.set_prompt(DEFAULT_PS1);
        r.set_debug_command_complete_data(vec![
            ("exit".to_owned(), "exit".to_owned()),
            ("attach".to_owned(), "attach".to_owned()),
            ("detach".to_owned(), "detach".to_owned()),
        ]);

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.init_reader()?;
        loop {
            let line: String;
            {
                line = self
                    .reader
                    .lock()
                    .expect("lock reader failed")
                    .read()
                    .expect("read failed");
            }
            if line.is_empty() {
                continue;
            }
            if let Some((cmd, args)) = split_command(line.trim()) {
                if let Err(err) = self.run_builtin_command(&cmd, &parse_arguments(&args)) {
                    match err.as_str() {
                        "exit" => break,
                        "custom" => {
                            if let Err(err) = self.run_custom_command(&line) {
                                println!("Error: {}", err);
                                self.detach_process()
                            }
                            sleep(Duration::from_millis(10));
                        }
                        _ => {
                            println!("Error: {}", err);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
