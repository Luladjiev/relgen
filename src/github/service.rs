use crate::github::client::{ClientError, GithubClient};
use futures::future::join_all;

pub struct GitHub<C: GithubClient> {
    pub client: C,
}

impl<C: GithubClient> GitHub<C> {
    pub async fn get_repos_with_changes(
        &self,
        owner: &str,
        repositories: &[String],
        base: &str,
        head: &str,
    ) -> Vec<String> {
        let requests = repositories
            .iter()
            .map(|repo| self.client.compare(owner, repo, base, head));
        let responses = join_all(requests).await;
        let mut repos_with_changes = vec![];

        for (repo, response) in repositories.iter().zip(responses) {
            match response {
                Ok(0) => println!("No changes found in {repo}"),
                Ok(_) => repos_with_changes.push(repo.clone()),
                Err(err) => eprintln!("{err}"),
            }
        }

        repos_with_changes
    }

    pub async fn create_pr(
        &self,
        owner: &str,
        repo: &str,
        base: &str,
        base_name: &Option<String>,
        head: &str,
        dry_run: bool,
    ) -> Result<Option<u64>, ClientError> {
        let pull_requests = self.client.open_prs(owner, repo, base, head).await?;

        if !pull_requests.is_empty() {
            pull_requests.iter().for_each(|item| {
                println!("{repo} already has an opened Pull request: https://github.com/{owner}/{repo}/pull/{item}");
            });
            return Ok(None);
        }

        println!("Creating Pull request for {repo}...");

        if dry_run {
            return Ok(None);
        }

        let date = chrono::Local::now().format("%Y-%m-%d").to_string();

        let name = match base_name {
            None => base,
            Some(name) => name,
        };

        let title = format!("Release to {name} {date}");

        let pr_number = self
            .client
            .create_pr(owner, repo, &title, base, head)
            .await?;

        println!(
            "Pull request created in {repo}: https://github.com/{owner}/{repo}/pull/{pr_number}"
        );

        Ok(Some(pr_number))
    }

    pub async fn request_reviews(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        reviewers: &[String],
        dry_run: bool,
    ) -> Result<(), ClientError> {
        if dry_run || reviewers.is_empty() {
            return Ok(());
        }

        self.client
            .request_reviews(owner, repo, pr_number, reviewers)
            .await?;

        println!(
            "Review requested from {} in {repo}'s Pull Request",
            reviewers.join(", ")
        );

        Ok(())
    }
}
