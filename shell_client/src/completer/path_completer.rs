//! 路径完成器
use crate::completer::Completer;

use linefeed::{complete::Completion, DefaultTerminal, Prompter};

pub struct PathCompleter {
    pub inner: linefeed::complete::PathCompleter,
}

impl Completer for PathCompleter {
    fn filter(w: &str, b: &str) -> bool {
        w.trim() != b.trim()
    }
    fn new() -> Box<dyn Completer> {
        Box::new(PathCompleter {
            inner: linefeed::complete::PathCompleter,
        })
    }
}

impl linefeed::complete::Completer<DefaultTerminal> for PathCompleter {
    fn complete(
        &self,
        word: &str,
        prompter: &Prompter<DefaultTerminal>,
        start: usize,
        end: usize,
    ) -> Option<Vec<Completion>> {
        self.inner.complete(word, prompter, start, end)
    }
}
