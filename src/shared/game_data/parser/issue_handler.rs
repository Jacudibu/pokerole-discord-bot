use log::error;
use std::fmt::Display;

pub trait IssueHandler {
    fn handle_issue(&mut self, issue: String);
}

#[derive(Default)]
pub struct IssueStorage {
    issues: Vec<String>,
}

impl IssueStorage {
    /// Use this instead of the Into<Option> blanket implementation
    pub fn into_option(self) -> Option<Self> {
        if self.issues.is_empty() {
            None
        } else {
            Some(self)
        }
    }
}

impl Clone for IssueStorage {
    // Fake clone implementation to keep issues local.
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl IssueHandler for IssueStorage {
    #[inline]
    fn handle_issue(&mut self, issue: String) {
        self.issues.push(issue);
    }
}

impl Display for IssueStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = self
            .issues
            .iter()
            .fold(String::new(), |acc, issue| acc + "- " + issue + "\n");

        write!(f, "{}", result)
    }
}

pub struct IssueLogger;
impl IssueHandler for IssueLogger {
    #[inline]
    fn handle_issue(&mut self, issue: String) {
        error!("{issue}");
    }
}
