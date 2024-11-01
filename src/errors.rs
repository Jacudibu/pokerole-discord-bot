use std::fmt;
use std::num::ParseIntError;

pub type ValidationError = CommandInvocationError;
pub type ParseError = CommandInvocationError;
pub type DatabaseError = CommandInvocationError;

#[derive(Debug)]
pub struct CommandInvocationError {
    message: String,
    pub log: bool,
}

impl CommandInvocationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            log: false,
        }
    }

    pub fn log(mut self) -> Self {
        self.log = true;
        self
    }
}

impl fmt::Display for CommandInvocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CommandInvocationError {}
impl Default for CommandInvocationError {
    fn default() -> Self {
        CommandInvocationError {
            message: String::from("Invalid input data!"),
            log: false,
        }
    }
}

#[derive(Debug)]
pub struct DataParsingError {
    message: String,
}

impl From<String> for DataParsingError {
    fn from(value: String) -> Self {
        Self { message: value }
    }
}

impl From<&str> for DataParsingError {
    fn from(value: &str) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl fmt::Display for DataParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<ParseIntError> for DataParsingError {
    fn from(value: ParseIntError) -> Self {
        DataParsingError {
            message: value.to_string(),
        }
    }
}

impl std::error::Error for DataParsingError {}
