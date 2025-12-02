use crate::models::github::Response;
use futures::future::join_all;
use octocrab::params::State;
use octocrab::{GitHubError, Octocrab, Result};
use std::error::Error;

pub struct GitHub<'a> {
    pub octocrab: &'a Octocrab,
}

impl<'a> GitHub<'a> {
    pub async fn get_repos_with_changes(
        &self,
        owner: &String,
        repositories: &Vec<String>,
        base: &String,
        head: &String,
    ) -> Result<Vec<String>, String> {
        let mut responses = vec![];

        for repo in repositories {
            let res = self.octocrab.get(
                format!("/repos/{owner}/{repo}/compare/{base}...{head}"),
                None::<&()>,
            );

            responses.push(res);
        }

        let responses: Vec<octocrab::Result<Response>> = join_all(responses).await;
        let mut repos_with_changes = vec![];

        for (index, res) in responses.iter().enumerate() {
            let repo = repositories.get(index).expect("Failed to get repo");

            match res {
                Ok(res) => {
                    if res.total_commits > 0 {
                        repos_with_changes.push(repo.clone());
                    } else {
                        println!("{repo}'s {head} and {base} branches are in sync, skipping.");
                    }
                }
                Err(e) => {
                    let err = &self.generate_error(
                        format!("Error comparing {repo}'s {head} and {base} branches"),
                        e,
                    );

                    if let Err(err) = err {
                        println!("{err}");
                    }
                }
            }
        }

        Ok(repos_with_changes)
    }

    pub async fn create_pr(
        &self,
        owner: &String,
        repo: String,
        base: &String,
        base_name: &Option<String>,
        head: &String,
        reviewers: &[String],
        dry_run: bool,
    ) -> Result<(), String> {
        let res = &self
            .octocrab
            .pulls(owner, &repo)
            .list()
            .state(State::Open)
            .head(head)
            .base(base)
            .send()
            .await;

        match res {
            Ok(pr) => {
                if !pr.items.is_empty() {
                    pr.items.iter().for_each(|item| {
                        println!("{repo} already has an opened Pull request: https://github.com/{owner}/{repo}/pull/{}", item.number);
                    });
                    return Ok(());
                }

                println!("Creating Pull request for {repo}...");

                if dry_run {
                    return Ok(());
                }

                let date = chrono::Local::now().format("%Y-%m-%d").to_string();

                let name = match base_name {
                    None => base,
                    Some(name) => name,
                };

                let res = &self
                    .octocrab
                    .pulls(owner, &repo)
                    .create(format!("Release to {name} {date}"), head, base)
                    .send()
                    .await;

                match res {
                    Ok(pr) => {
                        println!(
                            "Pull request created in {repo}: https://github.com/{owner}/{repo}/pull/{}",
                            pr.number
                        );

                        self.request_review(owner, &repo, reviewers, pr.number)
                            .await
                    }
                    Err(e) => {
                        self.generate_error(format!("Failed creating a Pull Request in {repo}"), &e)
                    }
                }
            }
            Err(e) => self.generate_error(format!("Failed to fetch {repo}'s pull requests."), &e),
        }
    }

    pub async fn request_review(
        &self,
        owner: &String,
        repo: &str,
        reviewers: &[String],
        pr_id: u64,
    ) -> Result<(), String> {
        let res = &self
            .octocrab
            .pulls(owner, repo)
            .request_reviews(pr_id, reviewers, [])
            .await;

        match res {
            Ok(_res) => {
                println!(
                    "Review requested from {} in {repo}'s Pull Request",
                    reviewers.join(", ")
                );

                Ok(())
            }
            Err(e) => self.generate_error(
                format!("Failed to request review in {repo}'s Pull Request"),
                &e,
            ),
        }
    }

    pub fn generate_error<T: Error>(&self, message: String, e: &T) -> Result<(), String> {
        if let Some(s) = e.source() {
            let err = s
                .downcast_ref::<GitHubError>()
                .expect("Failed to extract source error");

            return Err(format!("{message}: {}", err.message));
        }

        Err(message)
    }
}
