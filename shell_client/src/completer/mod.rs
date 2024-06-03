//! 完成器抽象
mod pad_completer;
mod path_completer;
mod ushell_completer;

pub use pad_completer::*;
pub use path_completer::*;
pub use ushell_completer::*;

use linefeed::terminal::DefaultTerminal;
use linefeed::Completion;
use linefeed::Suffix;

use crate::tools;

/// 完成器
/// - filter 是否满足当前完成器的激活条件
/// - new 创建完成器
pub trait Completer: linefeed::complete::Completer<DefaultTerminal> {
    fn filter(w: &str, b: &str) -> bool
    where
        Self: Sized;
    fn new() -> Box<dyn Completer>
    where
        Self: Sized;
}

/// 生成自动完成条目
/// cmp_data 自动完成数据
/// word 输入
pub fn gen_autocomplete_item(
    cmp_data: &Vec<(String, String)>,
    word: &str,
) -> Option<Vec<Completion>> {
    let ops: Vec<Box<dyn Fn(&(String, String), &str) -> i32>> = vec![
        Box::new(|x: &(String, String), word: &str| -> i32 {
            if tools::is_prefix(&x.0, word) {
                0
            } else {
                -1
            }
        }),
        Box::new(|x: &(String, String), word: &str| -> i32 {
            if tools::is_prefix(&x.1, word) {
                1
            } else {
                -1
            }
        }),
        Box::new(|x: &(String, String), word: &str| -> i32 {
            if tools::is_prefix_nocase(&x.0, word) {
                0
            } else {
                -1
            }
        }),
        Box::new(|x: &(String, String), word: &str| -> i32 {
            if tools::is_prefix_nocase(&x.1, word) {
                1
            } else {
                -1
            }
        }),
        Box::new(|x: &(String, String), word: &str| -> i32 {
            if tools::contain_nocase(&x.1, word) {
                1
            } else {
                -1
            }
        }),
    ];
    for op in ops {
        let col: Vec<Completion> = cmp_data
            .iter()
            .filter_map(|x| match op(&x, &word) {
                0 => Some(Completion {
                    completion: x.0.clone(),
                    display: Some(format!("{}({})", x.0, x.1)),
                    suffix: Suffix::Default,
                }),
                1 => Some(Completion {
                    completion: x.1.clone(),
                    display: Some(format!("{}({})", x.0, x.1)),
                    suffix: Suffix::Default,
                }),
                _ => None,
            })
            .collect();
        if col.is_empty() {
            continue;
        }
        return Some(col);
    }
    return None;
}
