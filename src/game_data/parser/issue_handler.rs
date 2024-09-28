use log::error;

pub trait IssueHandler {
    fn handle_issue(&mut self, issue: String);
}

#[derive(Default)]
pub struct IssueStorage {
    issues: Vec<String>,
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
