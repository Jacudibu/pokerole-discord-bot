use std::fmt;

pub type ValidationError = SimpleErrorWithMessage;
pub type ParseError = SimpleErrorWithMessage;

#[derive(Debug)]
pub struct SimpleErrorWithMessage {
    message: String,
}

impl SimpleErrorWithMessage {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for SimpleErrorWithMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SimpleErrorWithMessage {}
impl Default for SimpleErrorWithMessage {
    fn default() -> Self {
        SimpleErrorWithMessage {
            message: String::from("Invalid input data!"),
        }
    }
}
