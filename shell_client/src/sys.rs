/*!
 * 系统功能封装
 */

use std::process;


/// 获取进程列表
pub fn get_process_list() -> Vec<(String, String)> {
    String::from_utf8(
        process::Command::new("ps")
            .args(["-A", "-o", "pid,comm"])
            .output()
            .unwrap()
            .stdout,
    )
    .expect("decode failed")
    .split("\n")
    .skip(1)
    .filter_map(|x| {
        let sp:Vec<&str> = x.split_whitespace().collect();
        if sp.len() == 2 {
            Some((sp[0].to_owned(),sp[1].to_owned()))
        } else {
            None
        }
    })
    .collect()
}
