//! pad 命令的完成器，用于完成进程列表
use linefeed::{complete::Completion, prompter::Prompter, terminal::DefaultTerminal};

use crate::{completer::Completer, sys};

pub struct AttachCommandCompleter;

impl linefeed::complete::Completer<DefaultTerminal> for AttachCommandCompleter {
    fn complete(
        &self,
        word: &str,
        _prompter: &Prompter<DefaultTerminal>,
        _start: usize,
        _end: usize,
    ) -> Option<Vec<Completion>> {
        crate::completer::gen_autocomplete_item(&sys::get_process_list(), word)
    }
}

impl Completer for AttachCommandCompleter {
    fn filter(_w: &str, b: &str) -> bool {
        b.split_whitespace()
            .map(|x| x.trim().to_owned())
            .next()
            .unwrap_or("".to_owned())
            == "attach"
    }

    fn new() -> Box<dyn Completer> {
        Box::new(AttachCommandCompleter)
    }
}
