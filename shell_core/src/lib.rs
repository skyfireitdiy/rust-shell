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

/// 实现一个函数，从提供的读取器中读取一行文本。
pub fn read_line<T: Read>(conn: &mut T) -> Result<String, String> {
    // 用于存储读取的数据的缓冲区。
    let mut buf = vec![];

    // 用于临时存储读取的数据的缓冲区。
    let mut tmp_buf = [0u8; 4096];

    loop {
        // 从读取器中读取数据到临时缓冲区。
        let sz = conn.read(&mut tmp_buf).map_err(|err| err.to_string())?;

        // 如果读取到的数据大小为 0，表示连接已关闭。
        if sz == 0 {
            return Err("connection closed".to_string());
        }

        // 将读取到的数据添加到缓冲区中。
        buf.extend_from_slice(&tmp_buf[0..sz]);

        // 如果缓冲区中包含换行符，表示一行文本已读取完成。
        if buf.ends_with(&[b'\n']) {
            break;
        }
    }

    // 将缓冲区中的数据转换为 UTF-8 字符串并返回。
    Ok(String::from_utf8(buf)
        .map_err(|err| err.to_string())?
        .trim()
        .to_owned())
}

/// 实现一个函数，将一行文本写入提供的写入器中。
pub fn write_line<T: Write>(conn: &mut T, line: &String) -> Result<(), String> {
    // 将一行文本添加到缓冲区中，并在末尾添加换行符。
    let data_to_write = line.to_owned() + "\n";

    // 将缓冲区中的数据写入到写入器中。
    conn.write_all(data_to_write.as_bytes())
        .map_err(|err| err.to_string())
}

/// 实现一个函数，将一段文本解析为一组参数。
pub fn parse_arguments(input: &str) -> Vec<Argument> {
    // 用于存储解析出的参数的向量。
    let mut result = Vec::new();

    // 用于存储当前正在解析的参数的字符串。
    let mut current_arg = String::new();

    // 用于指示是否在引号中。
    let mut in_quotes = false;

    // 用于指示是否需要转义。
    let mut escape = false;

    // 迭代输入文本中的字符
    for c in input.chars() {
        // 如果需要转义，直接将字符添加到当前参数中并跳过转义处理
        if escape {
            current_arg.push(c);
            escape = false;
        } else {
            match c {
                // 如果遇到引号，切换 in_quotes 状态
                '"' => in_quotes = !in_quotes,

                // 如果遇到反斜杠，需要转义下一个字符
                '\\' => escape = true,

                // 如果遇到逗号且不在引号中，表示一个参数结束
                ',' if !in_quotes => {
                    // 如果参数以引号开始和结束，移除引号并作为字符串处理
                    if current_arg.starts_with('"') && current_arg.ends_with('"') {
                        current_arg.remove(current_arg.len() - 1);
                        current_arg.remove(0);
                        result.push(Argument::Str(current_arg.clone()));
                    } else {
                        // 尝试将当前参数解析为整数
                        if let Ok(num) = current_arg.trim().parse::<i64>() {
                            result.push(Argument::Int(num));
                        } else {
                            // 如果无法将参数解析为整数，则作为字符串处理
                            result.push(Argument::Str(current_arg.clone()));
                        }
                    }
                    // 清空当前参数
                    current_arg.clear();
                }
                // 其他字符直接添加到当前参数中
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

    // 返回解析出的参数列表
    result
}

/// 实现一个函数，将一段命令行文本拆分为命令和参数。
pub fn split_command(command_line: &str) -> Option<(String, String)> {
    // 将命令行文本按空格拆分为一组字符串。
    let sp = command_line.splitn(2, " ").collect::<Vec<&str>>();

    match sp.len() {
        // 如果没有拆分出任何字符串，返回 None。
        0 => None,

        // 如果拆分出一个字符串，返回该字符串作为命令，参数为空字符串。
        1 => Some((sp[0].to_string(), String::new())),

        // 如果拆分出两个字符串，返回前一个字符串作为命令，后一个字符串作为参数。
        2 => Some((sp[0].to_string(), sp[1].to_string())),

        // 如果拆分出了多个字符串，返回 None。
        _ => None,
    }
}
