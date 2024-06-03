use shell_core::*;
use std::{collections::HashMap, panic, vec};

#[derive(Clone)]
pub struct Shell {
    func_map: HashMap<String, u64>,
}

#[macro_export]
macro_rules! reg_shell_cmd {
    ($var:expr,$({$name:expr, $func:expr}),+) => {
        $(
            $var.reg_func($name.to_string(), unsafe{ std::mem::transmute($func as u64) });
        )+
    };
}

impl Shell {
    pub fn new() -> Shell {
        Shell {
            func_map: HashMap::new(),
        }
    }

    pub fn get_reg_commands(&self) -> Vec<String> {
        self.func_map.keys().map(|k| k.to_string()).collect()
    }

    pub fn reg_func(&mut self, name: String, addr: u64) {
        self.func_map.insert(name, addr);
    }

    pub fn run_command(&self, command_line: &String) -> Result<(), String> {
        panic::catch_unwind(|| {
            let (command, arguments) =
                split_command(command_line.trim()).ok_or("split command failed")?;

            let addr = self
                .func_map
                .get(&command)
                .ok_or(format!("{} not found", command))?;

            let mut argument_int64 = vec![];

            let mut str_args = vec![String::from(""); 10]; // 这个变量不能删除,需要这个vec保持对象的生命周期
            let mut index = 0;

            for a in parse_arguments(arguments.as_str()) {
                match a {
                    Argument::Str(s) => {
                        str_args[index] = s.clone();
                        argument_int64.push(unsafe { std::mem::transmute(&str_args[index]) });
                        index += 1;
                    }
                    Argument::Int(i) => argument_int64.push(i as u64),
                }
            }

            macro_rules! call_func {
                ($func:expr) => {
                    Ok($func(*addr)())
                };
                ($func:expr,$($n:expr),*) => {
                    Ok(
                        $func(*addr)(
                            $(argument_int64[$n],)+
                        )
                    )
                };
            }

            println!(
                "\x1B[34m------------[begin to excel func {}]------------\x1B[0m",
                command
            );
            if let Ok(ret) = match argument_int64.len() {
                0 => call_func!(create_fn_0),
                1 => call_func!(create_fn_1, 0),
                2 => call_func!(create_fn_2, 0, 1),
                3 => call_func!(create_fn_3, 0, 1, 2),
                4 => call_func!(create_fn_4, 0, 1, 2, 3),
                5 => call_func!(create_fn_5, 0, 1, 2, 3, 4),
                6 => call_func!(create_fn_6, 0, 1, 2, 3, 4, 5),
                7 => call_func!(create_fn_7, 0, 1, 2, 3, 4, 5, 6),
                8 => call_func!(create_fn_8, 0, 1, 2, 3, 4, 5, 6, 7),
                9 => call_func!(create_fn_9, 0, 1, 2, 3, 4, 5, 6, 7, 8),
                10 => call_func!(create_fn_10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9),
                _ => Err("too many arguments".to_string()),
            } {
                println!(
                    "\x1B[35m------------[end to excel func {}]:{}------------\x1B[0m",
                    command, ret
                );
            }
            Ok(())
        })
        .map_err(|err| format!("run command err: {:?}", err))?
    }
}

macro_rules! def_create_fn {
    ($name:ident) => {
        fn $name(addr:u64) -> fn() -> u64 {
            unsafe { std::mem::transmute(addr) }
        }
    };
    ($name:ident, $($param:ty),*) => {
        fn $name(addr:u64) -> fn($($param,)+) -> u64 {
            unsafe { std::mem::transmute(addr) }
        }
    };
}

def_create_fn!(create_fn_0);
def_create_fn!(create_fn_1, u64);
def_create_fn!(create_fn_2, u64, u64);
def_create_fn!(create_fn_3, u64, u64, u64);
def_create_fn!(create_fn_4, u64, u64, u64, u64);
def_create_fn!(create_fn_5, u64, u64, u64, u64, u64);
def_create_fn!(create_fn_6, u64, u64, u64, u64, u64, u64);
def_create_fn!(create_fn_7, u64, u64, u64, u64, u64, u64, u64);
def_create_fn!(create_fn_8, u64, u64, u64, u64, u64, u64, u64, u64);
def_create_fn!(create_fn_9, u64, u64, u64, u64, u64, u64, u64, u64, u64);
def_create_fn!(
    create_fn_10,
    u64,
    u64,
    u64,
    u64,
    u64,
    u64,
    u64,
    u64,
    u64,
    u64
);
