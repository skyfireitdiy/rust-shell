use rustyline::{error::ReadlineError, DefaultEditor};
use shell_core::*;
use std::{
    io::Write,
    os::unix::net::UnixStream,
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

pub struct Client {
    prefix: String,
    cmd_channel: Option<UnixStream>,
    output_channel: Option<UnixStream>,
    copy_stdout: Option<JoinHandle<()>>,
}

impl Client {
    pub fn new() -> Client {
        Client {
            prefix: ">> ".to_owned(),
            cmd_channel: None,
            output_channel: None,
            copy_stdout: None,
        }
    }

    fn find_process(&self, arg: &Argument) -> Vec<(String, u64)> {
        let output = std::process::Command::new("ps")
            .arg("-A")
            .arg("-o")
            .arg("pid,comm")
            .output()
            .map_err(|err| err.to_string())
            .expect("ps error");

        let mut result = Vec::new();
        for line in String::from_utf8(output.stdout)
            .map_err(|err| err.to_string())
            .expect("invalid utf8")
            .lines()
        {
            let sp = line.split_whitespace().collect::<Vec<&str>>();
            if sp.len() != 2 {
                continue;
            }

            if let Ok(pid) = sp[0].parse::<u64>() {
                result.push((sp[1].to_string(), pid));
            }
        }

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

    fn pad(&mut self, args: &Vec<Argument>) -> Result<(), String> {
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

        let mut output_channel_copy = self
            .output_channel
            .as_ref()
            .ok_or("not pad to process")?
            .try_clone()
            .map_err(|err| err.to_string())?;

        self.copy_stdout = Some(spawn(move || {
            let _ = std::io::copy(&mut output_channel_copy, &mut std::io::stdout());
        }));

        self.prefix = format!("{} >> ", pids[0].0);

        Ok(())
    }

    fn exit() -> Result<(), String> {
        Err("exit".to_owned())
    }

    fn run_builtin_command(&mut self, cmd: &String, args: &Vec<Argument>) -> Result<(), String> {
        match cmd.as_str() {
            "pad" => self.pad(&args),
            "exit" => Self::exit(),
            _ => Err("custom".to_owned()),
        }
    }

    pub fn run_custom_command(&mut self, line: &String) -> Result<(), String> {
        match self.cmd_channel {
            None => Err("not pad to process".to_owned()),
            Some(ref mut cmd_channel) => cmd_channel
                .write_all(line.as_bytes())
                .map_err(|err| err.to_string()),
        }
    }

    pub fn npad(&mut self) {
        self.cmd_channel = None;
        self.output_channel = None;
        self.copy_stdout = None;
        self.prefix = ">> ".to_owned();
    }

    pub fn run(&mut self) -> Result<(), String> {
        let mut editor = DefaultEditor::new().map_err(|err| err.to_string())?;

        loop {
            match editor.readline(&self.prefix) {
                Ok(line) => {
                    if line.is_empty() {
                        continue;
                    }
                    let _ = editor.add_history_entry(line.as_str());
                    if let Some((cmd, args)) = split_command(line.trim()) {
                        if let Err(err) = self.run_builtin_command(&cmd, &parse_arguments(&args)) {
                            match err.as_str() {
                                "exit" => break,
                                "custom" => {
                                    if let Err(err) = self.run_custom_command(&line) {
                                        println!("Error: {}", err);
                                        self.npad()
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
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    continue;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        Ok(())
    }
}
