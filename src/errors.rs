use std::fmt;

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

impl From<String> for CommandInvocationError {
    fn from(value: String) -> Self {
        Self {
            message: value,
            log: false,
        }
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
