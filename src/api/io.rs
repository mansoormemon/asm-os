use alloc::string::String;

use crate::dev;

pub struct StdIn;

impl StdIn {
    fn new() -> Self {
        Self {}
    }

    pub fn read_char(&self) -> char { dev::console::read_char() }

    pub fn read_line(&self) -> String { dev::console::read_line() }
}

pub struct StdOut;

impl StdOut {
    fn new() -> Self {
        Self {}
    }
}

pub struct StdErr;

impl StdErr {
    fn new() -> Self {
        Self {}
    }
}

pub fn stdin() -> StdIn {
    StdIn::new()
}

pub fn stdout() -> StdOut {
    StdOut::new()
}

pub fn stderr() -> StdErr {
    StdErr::new()
}
