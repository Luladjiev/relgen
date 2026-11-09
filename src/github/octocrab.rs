use crate::github::client::{ClientError, GithubClient};
use octocrab::params::State;
use octocrab::Octocrab;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Response {
    pub total_commits: u64,
}

pub struct OctocrabClient {
    pub octocrab: Octocrab,
}

impl GithubClient for OctocrabClient {
    async fn compare(
        &self,
        owner: &str,
        repo: &str,
        base: &str,
        head: &str,
    ) -> Result<u64, ClientError> {
        let url = format!("/repos/{owner}/{repo}/compare/{base}...{head}");
        let response = self
            .octocrab
            .get::<Response, _, ()>(url, None)
            .await
            .map_err(|err| ClientError {
                message: format!("Failed to compare commits in {repo}: {}", describe(err)),
            })?;

        Ok(response.total_commits)
    }

    async fn open_prs(
        &self,
        owner: &str,
        repo: &str,
        base: &str,
        head: &str,
    ) -> Result<Vec<u64>, ClientError> {
        let response = self
            .octocrab
            .pulls(owner, repo)
            .list()
            .state(State::Open)
            .base(base)
            .head(format!("{owner}:{head}"))
            .send()
            .await
            .map_err(|err| ClientError {
                message: format!("Failed to list open PRs in {repo}: {}", describe(err)),
            })?;

        Ok(response.items.iter().map(|pr| pr.number).collect())
    }

    async fn create_pr(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        base: &str,
        head: &str,
    ) -> Result<u64, ClientError> {
        let response = self
            .octocrab
            .pulls(owner, repo)
            .create(title, head, base)
            .send()
            .await
            .map_err(|err| ClientError {
                message: format!("Failed to create PR in {repo}: {}", describe(err)),
            })?;

        Ok(response.number)
    }

    async fn request_reviews(
        &self,
        owner: &str,
        repo: &str,
        pr: u64,
        reviewers: &[String],
    ) -> Result<(), ClientError> {
        self.octocrab
            .pulls(owner, repo)
            .request_reviews(pr, reviewers, [])
            .await
            .map_err(|err| ClientError {
                message: format!(
                    "Failed to request reviews for PR {pr} in {repo}: {}",
                    describe(err)
                ),
            })?;

        Ok(())
    }
}

fn describe(err: octocrab::Error) -> String {
    match err {
        octocrab::Error::GitHub { source, .. } => source.to_string(),
        other => other.to_string(),
    }
}
