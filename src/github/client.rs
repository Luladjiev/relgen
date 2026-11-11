use std::fmt;

#[derive(Debug)]
pub struct ClientError {
    pub message: String,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub trait GithubClient {
    async fn compare(
        &self,
        owner: &str,
        repo: &str,
        base: &str,
        head: &str,
    ) -> Result<u64, ClientError>; // total_commits
    async fn open_prs(
        &self,
        owner: &str,
        repo: &str,
        base: &str,
        head: &str,
    ) -> Result<Vec<u64>, ClientError>;
    async fn create_pr(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        base: &str,
        head: &str,
    ) -> Result<u64, ClientError>;
    async fn request_reviews(
        &self,
        owner: &str,
        repo: &str,
        pr: u64,
        reviewers: &[String],
    ) -> Result<(), ClientError>;
}
