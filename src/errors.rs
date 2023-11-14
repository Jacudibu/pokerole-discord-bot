use std::fmt;

pub type ValidationError = CommandInvocationError;
pub type ParseError = CommandInvocationError;

#[derive(Debug)]
pub struct CommandInvocationError {
    message: String,
}

impl CommandInvocationError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
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
        }
    }
}
