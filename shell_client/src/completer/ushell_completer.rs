//! ushell完成器，集成了一套完成器链，经过过滤后形成适合的完成建议
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

use crate::{
    completer::{Completer, PadCommandCompleter, PathCompleter},
    tools,
};

use linefeed::{
    complete::{Completion, Suffix},
    DefaultTerminal, Prompter,
};

pub struct UshellCompleter {
    pub autocomplete_data: Mutex<Cell<Vec<(String, String)>>>,
    pub completer_chain: Vec<(fn(&str, &str) -> bool, Box<dyn Completer>)>,
}

impl linefeed::complete::Completer<DefaultTerminal> for UshellCompleter {
    fn complete(
        &self,
        word: &str,
        prompter: &Prompter<DefaultTerminal>,
        start: usize,
        end: usize,
    ) -> Option<Vec<Completion>> {
        self.completer_chain
            .iter()
            .filter(|x| x.0(&word.to_string(), &prompter.buffer().trim().to_string()))
            .find_map(|c| match c.1.complete(word, prompter, start, end) {
                Some(x) => {
                    if !x.is_empty() {
                        Some(x)
                    } else {
                        None
                    }
                }
                None => None,
            })
            .map_or_else(|| self.debug_command_complete(word), |x| Some(x))
    }
}

impl UshellCompleter {
    pub fn debug_command_complete(&self, word: &str) -> Option<Vec<Completion>> {
        for cmp in [
            tools::is_prefix,
            tools::is_prefix_nocase,
            tools::contain_nocase,
        ] {
            let ret = self
                .autocomplete_data
                .lock()
                .expect("lock autocomplete data failed")
                .get_mut()
                .iter()
                .filter(|x| cmp(&x.0, word))
                .map(|x| Completion {
                    completion: x.0.clone(),
                    display: Some(x.1.clone()),
                    suffix: Suffix::Default,
                })
                .collect::<Vec<Completion>>();
            if ret.len() != 0 {
                return Some(ret);
            }
        }
        None
    }

    pub fn set_autocomplete_data(&self, data: Vec<(String, String)>) {
        self.autocomplete_data
            .lock()
            .expect("lock autocomplete data failed")
            .set(data);
    }

    pub fn append_complete_data(&self, data: Vec<(String, String)>) {
        let mut data = data;
        data.append(
            self.autocomplete_data
                .lock()
                .expect("lock autocomplete data failed")
                .get_mut(),
        );
        self.set_autocomplete_data(data);
    }

    pub fn new() -> Arc<UshellCompleter> {
        Arc::new(UshellCompleter {
            autocomplete_data: Mutex::new(Cell::new(Vec::new())),
            completer_chain: crate::reg_completer!(PadCommandCompleter, PathCompleter),
        })
    }
}
