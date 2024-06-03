//! 支持自动完成的读取器，输入过程中使用tab可以出发自动完成
use crate::completer::UshellCompleter;
use core::time::Duration;
use linefeed::{terminal::DefaultTerminal, Interface, ReadResult};
use std::sync::{Arc, Mutex};
use std::thread::sleep;

/// 自动完成读取器
/// - interface 读取接口，见linefeed库
/// - completer 自动完成器
pub struct AutoCompleteReader {
    interface: Interface<DefaultTerminal>,
    completer: Arc<UshellCompleter>,
}

impl AutoCompleteReader {
    pub fn read(&self) -> Result<String, String> {
        sleep(Duration::from_millis(10));
        match self
            .interface
            .read_line()
            .map_err(|err| format!("read error : {}", err))?
        {
            ReadResult::Input(line) => {
                if !line.trim().is_empty() {
                    self.interface.add_history(line.clone());
                }
                return Ok(line);
            }
            ReadResult::Eof => {
                return Ok("".to_owned());
            }
            ReadResult::Signal(s) => {
                return Err(format!("recv signal {:?}", s));
            }
        }
    }
    pub fn set_debug_command_complete_data(&mut self, data: Vec<(String, String)>) {
        self.completer.set_autocomplete_data(data);
    }
    pub fn append_debug_command_complete_data(&mut self, data: Vec<(String, String)>) {
        self.completer.append_complete_data(data);
    }
    pub fn set_prompt(&mut self, p: &str) {
        self.interface.set_prompt(&p).expect("set prompt failed");
    }
}

/// 完成器的注册接口
#[macro_export]
macro_rules! reg_completer {
    ($($t:ty),*) => {
        vec![
            $(
            (
                <$t>::filter as fn(&str, &str) -> bool,
                <$t>::new(),
            ),
            )*
        ]
    };
}

impl AutoCompleteReader {
    /// 创建读取器
    pub fn new() -> Result<Arc<Mutex<Box<AutoCompleteReader>>>, String> {
        let mut ret = Box::<AutoCompleteReader>::new(AutoCompleteReader {
            interface: Interface::new("ushell-rust").expect("create interface failed"),
            completer: UshellCompleter::new(),
        });

        ret.set_prompt(">> ");
        ret.interface.set_completer(ret.completer.clone());

        Ok(Arc::new(Mutex::new(ret)))
    }
}
