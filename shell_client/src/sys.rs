/*!
 * 系统功能封装
 */

use lazy_static::lazy_static;
use regex::Regex;
use std::process;

lazy_static! {
    static ref RE_PROCNAME: Regex = Regex::new(r"Name:\s(.*)").unwrap();
}

/// 获取进程列表
pub fn get_process_list() -> Vec<(String, String)> {
    lazy_static! {
        static ref PS_RE: Regex = Regex::new(r"(\d+)(\s+[^\s]+){2}\s+([^\s].*)").unwrap();
    }

    String::from_utf8(
        process::Command::new("ps")
            .args(["-A"])
            .output()
            .unwrap()
            .stdout,
    )
    .expect("decode failed")
    .split("\n")
    .skip(1)
    .filter_map(|x| match PS_RE.captures(x) {
        Some(sp) => Some((sp[1].to_string(), sp[3].to_string())),
        None => None,
    })
    .collect()
}
