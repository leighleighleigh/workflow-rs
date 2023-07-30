//!
//! Clear line helper (ANSI escape codes to clear the terminal line)
//!

use std::fmt;

pub struct EscapeCode { code : &'static str }

impl EscapeCode {
    pub fn new(code: &'static str) -> Self {
        EscapeCode { code }
    }
}

// Implement Display for ClearCommands
impl fmt::Display for EscapeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

// Implement AsRef<str> for EscapeCode
impl AsRef<str> for EscapeCode {
    fn as_ref(&self) -> &str {
        self.code
    }
}

// Implement AsRef<[u8]> for EscapeCode
impl AsRef<[u8]> for EscapeCode {
    fn as_ref(&self) -> &[u8] {
        self.code.as_bytes()
    }
}

// Define some common escape codes
pub const CLEAR_LINE : EscapeCode = EscapeCode { code: "\x1b[2K\r" };
pub const CLEAR_SCREEN : EscapeCode = EscapeCode { code: "\x1b[2J\x1b[1;1H" };