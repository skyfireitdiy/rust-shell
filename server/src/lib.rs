use std::{collections::HashMap, vec};

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
        let (command, arguments) =
            split_command(command_line.trim()).ok_or("split command failed")?;
        let arguments = parse_arguments(arguments.as_str());

        let addr = self
            .func_map
            .get(&command)
            .ok_or(format!("{} not found", command))?;

        let mut argument_int64 = vec![];

        let mut str_args = vec![String::from(""); 10]; // 这个变量不能删除,需要这个vec保持对象的生命周期
        let mut index = 0;

        for a in arguments {
            match a {
                Argument::Str(s) => {
                    str_args[index] = s.clone();
                    argument_int64.push(unsafe { std::mem::transmute(&str_args[index]) });
                    index += 1;
                }
                Argument::Int(i) => argument_int64.push(i as u64),
            }
        }

        match argument_int64.len() {
            0 => Ok(create_fn_0(*addr)()),
            1 => Ok(create_fn_1(*addr)(argument_int64[0])),
            2 => Ok(create_fn_2(*addr)(argument_int64[0], argument_int64[1])),
            3 => Ok(create_fn_3(*addr)(
                argument_int64[0],
                argument_int64[1],
                argument_int64[2],
            )),
            4 => Ok(create_fn_4(*addr)(
                argument_int64[0],
                argument_int64[1],
                argument_int64[2],
                argument_int64[3],
            )),
            5 => Ok(create_fn_5(*addr)(
                argument_int64[0],
                argument_int64[1],
                argument_int64[2],
                argument_int64[3],
                argument_int64[4],
            )),
            6 => Ok(create_fn_6(*addr)(
                argument_int64[0],
                argument_int64[1],
                argument_int64[2],
                argument_int64[3],
                argument_int64[4],
                argument_int64[5],
            )),
            7 => Ok(create_fn_7(*addr)(
                argument_int64[0],
                argument_int64[1],
                argument_int64[2],
                argument_int64[3],
                argument_int64[4],
                argument_int64[5],
                argument_int64[6],
            )),
            8 => Ok(create_fn_8(*addr)(
                argument_int64[0],
                argument_int64[1],
                argument_int64[2],
                argument_int64[3],
                argument_int64[4],
                argument_int64[5],
                argument_int64[6],
                argument_int64[7],
            )),
            9 => Ok(create_fn_9(*addr)(
                argument_int64[0],
                argument_int64[1],
                argument_int64[2],
                argument_int64[3],
                argument_int64[4],
                argument_int64[5],
                argument_int64[6],
                argument_int64[7],
                argument_int64[8],
            )),
            10 => Ok(create_fn_10(*addr)(
                argument_int64[0],
                argument_int64[1],
                argument_int64[2],
                argument_int64[3],
                argument_int64[4],
                argument_int64[5],
                argument_int64[6],
                argument_int64[7],
                argument_int64[8],
                argument_int64[9],
            )),
            _ => Err("too many arguments".to_string()),
        }
    }
}

fn create_fn_0(addr: u64) -> fn() -> u64 {
    unsafe { std::mem::transmute(addr) }
}

fn create_fn_1(addr: u64) -> fn(u64) -> u64 {
    unsafe { std::mem::transmute(addr) }
}

fn create_fn_2(addr: u64) -> fn(u64, u64) -> u64 {
    unsafe { std::mem::transmute(addr) }
}

fn create_fn_3(addr: u64) -> fn(u64, u64, u64) -> u64 {
    unsafe { std::mem::transmute(addr) }
}

fn create_fn_4(addr: u64) -> fn(u64, u64, u64, u64) -> u64 {
    unsafe { std::mem::transmute(addr) }
}

fn create_fn_5(addr: u64) -> fn(u64, u64, u64, u64, u64) -> u64 {
    unsafe { std::mem::transmute(addr) }
}

fn create_fn_6(addr: u64) -> fn(u64, u64, u64, u64, u64, u64) -> u64 {
    unsafe { std::mem::transmute(addr) }
}

fn create_fn_7(addr: u64) -> fn(u64, u64, u64, u64, u64, u64, u64) -> u64 {
    unsafe { std::mem::transmute(addr) }
}

fn create_fn_8(addr: u64) -> fn(u64, u64, u64, u64, u64, u64, u64, u64) -> u64 {
    unsafe { std::mem::transmute(addr) }
}

fn create_fn_9(addr: u64) -> fn(u64, u64, u64, u64, u64, u64, u64, u64, u64) -> u64 {
    unsafe { std::mem::transmute(addr) }
}

fn create_fn_10(addr: u64) -> fn(u64, u64, u64, u64, u64, u64, u64, u64, u64, u64) -> u64 {
    unsafe { std::mem::transmute(addr) }
}

#[derive(Debug)]
enum Argument {
    Str(String),
    Int(i64),
}

impl PartialEq for Argument {
    fn eq(&self, other: &Argument) -> bool {
        match (self, other) {
            (Argument::Str(s1), Argument::Str(s2)) => s1 == s2,
            (Argument::Int(i1), Argument::Int(i2)) => i1 == i2,
            _ => false,
        }
    }
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
