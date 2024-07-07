use std::{
    fmt::Display,
    io::{Read, Write},
};

#[derive(Debug)]
pub enum Argument {
    Str(String),
    Int(i64),
}

impl Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Argument::Str(s) => write!(f, "{}", s),
            Argument::Int(i) => write!(f, "{}", i),
        }
    }
}

pub fn read_line<T: Read>(conn: &mut T) -> Result<String, String> {
    let mut buf = vec![];
    let mut tmp_buf = [0u8; 4096];
    loop {
        let sz = conn.read(&mut tmp_buf).map_err(|err| err.to_string())?;
        if sz == 0 {
            return Err("connection closed".to_string());
        }
        buf.extend_from_slice(&tmp_buf[0..sz]);
        if buf.ends_with(&[b'\n']) {
            break;
        }
    }

    Ok(String::from_utf8(buf)
        .map_err(|err| err.to_string())?
        .trim()
        .to_owned())
}

pub fn write_line<T: Write>(conn: &mut T, line: &String) -> Result<(), String> {
    conn.write_all((line.to_owned() + "\n").as_bytes())
        .map_err(|err| err.to_string())
}

pub fn parse_arguments(input: &str) -> Vec<Argument> {
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
        } else if let Ok(num) = current_arg.parse::<i64>() {
            result.push(Argument::Int(num));
        } else {
            result.push(Argument::Str(current_arg));
        }
    }

    result
}

pub fn split_command(command_line: &str) -> Option<(String, String)> {
    let sp = command_line.splitn(2, " ").collect::<Vec<&str>>();
    match sp.len() {
        0 => None,
        1 => Some((sp[0].to_string(), String::new())),
        2 => Some((sp[0].to_string(), sp[1].to_string())),
        _ => None,
    }
}
