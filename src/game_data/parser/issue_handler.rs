use log::error;

pub trait IssueHandler {
    fn handle_issue(&mut self, issue: String);
}

#[derive(Default)]
pub struct IssueStorage {
    pub issues: Vec<String>,
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

pub struct IssueLogger;
impl IssueHandler for IssueLogger {
    #[inline]
    fn handle_issue(&mut self, issue: String) {
        error!("{issue}");
    }
}
