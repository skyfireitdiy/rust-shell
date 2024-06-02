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

    pub fn reg_func(&mut self, name: String, addr: u64) {
        self.func_map.insert(name, addr);
    }

    pub fn run_command(&self, command_line: &String) -> Result<u64, String> {
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

            match argument_int64.len() {
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
            }
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

#[derive(Debug)]
enum Argument {
    Str(String),
    Int(i64),
}

fn parse_arguments(input: &str) -> Vec<Argument> {
    let mut result = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;
    let mut escape = false;

    for c in input.chars() {
        if escape {
            current_arg.push(c);
            escape = false;
        } else {
            match c {
                '"' => in_quotes = !in_quotes,
                '\\' => escape = true,
                ',' if !in_quotes => {
                    if current_arg.starts_with('"') && current_arg.ends_with('"') {
                        // 如果以引号开始和结束，移除引号并作为字符串处理
                        current_arg.remove(current_arg.len() - 1);
                        current_arg.remove(0);
                        result.push(Argument::Str(current_arg.clone()));
                    } else {
                        // 尝试将当前参数解析为整数
                        if let Ok(num) = current_arg.trim().parse::<i64>() {
                            result.push(Argument::Int(num));
                        } else {
                            result.push(Argument::Str(current_arg.clone()));
                        }
                    }
                    current_arg.clear();
                }
                _ => current_arg.push(c),
            }
        }
    }

    // 处理最后一个参数
    if !current_arg.is_empty() {
        if current_arg.starts_with('"') && current_arg.ends_with('"') {
            current_arg.remove(current_arg.len() - 1);
            current_arg.remove(0);
            result.push(Argument::Str(current_arg));
        } else {
            if let Ok(num) = current_arg.parse::<i64>() {
                result.push(Argument::Int(num));
            } else {
                result.push(Argument::Str(current_arg));
            }
        }
    }

    result
}

fn split_command(command_line: &str) -> Option<(String, String)> {
    let sp = command_line.splitn(2, " ").collect::<Vec<&str>>();
    match sp.len() {
        0 => None,
        1 => Some((sp[0].to_string(), String::new())),
        2 => Some((sp[0].to_string(), sp[1].to_string())),
        _ => None,
    }
}
