use clap::Parser;
use futures::future::join_all;
use octocrab::params::State;
use octocrab::{GitHubError, Octocrab};
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
struct Response {
    total_commits: u32,
}

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long, help = "Base branch to use for the Pull Request")]
    base: String,
    #[arg(long, help = "Head branch to use for the Pull Request")]
    head: String,
    #[arg(long, help = "Repositories owner")]
    owner: String,
    #[arg(
        long,
        required = true,
        help = "List of repositories to create Pull Requests to"
    )]
    repo: Vec<String>,
    #[arg(long, help = "List of reviewers to add to each Pull Request")]
    reviewer: Vec<String>,
}

#[tokio::main]
async fn main() {
    let token = std::env::var("GITHUB_TOKEN");

    match token {
        Ok(token) => run(token).await,
        Err(e) => println!("Failed to retrieve GITHUB_TOKEN: {e}"),
    }
}

async fn run(token: String) {
    let cli = Args::parse();

    let owner = cli.owner;
    let base = cli.base;
    let head = cli.head;
    let repositories = cli.repo;
    let reviewers = cli.reviewer;

    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .expect("Failed to instantiate Octocrab");

    let res = get_repos_with_changes(&octocrab, &owner, &repositories, &base, &head).await;

    match res {
        Ok(repos) => {
            let mut responses = vec![];

            for repo in repos {
                responses.push(create_pr(&octocrab, &owner, repo, &base, &head, &reviewers));
            }

            let result = join_all(responses).await;

            for res in result {
                if let Err(err) = res {
                    println!("{err}");
                }
            }
        }
        Err(e) => println!("{e}"),
    }
}

fn generate_error(message: String, e: &octocrab::Error) -> Result<(), String> {
    if let Some(s) = e.source() {
        let err = s
            .downcast_ref::<GitHubError>()
            .expect("Failed to extract source error");

        return Err(format!("{message}: {}", err.message));
    }

    Err(message)
}

async fn get_repos_with_changes(
    octocrab: &Octocrab,
    owner: &String,
    repositories: &Vec<String>,
    base: &String,
    head: &String,
) -> Result<Vec<String>, String> {
    let mut responses = vec![];

    for repo in repositories {
        let res = octocrab.get(
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
                let err = generate_error(
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

async fn create_pr(
    octocrab: &Octocrab,
    owner: &String,
    repo: String,
    base: &String,
    head: &String,
    reviewers: &[String],
) -> Result<(), String> {
    let res = octocrab
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
                println!("{repo} already has an opened Pull request");
                return Ok(());
            }

            println!("Creating Pull request for {repo}...");

            let date = chrono::Local::now().format("%Y-%m-%d").to_string();

            let res = octocrab
                .pulls(owner, &repo)
                .create(format!("Release to {base} {date}"), head, base)
                .send()
                .await;

            match res {
                Ok(pr) => request_review(octocrab, owner, &repo, reviewers, pr.number).await,
                Err(e) => generate_error(format!("Failed creating a Pull request in {repo}"), &e),
            }
        }
        Err(e) => generate_error(format!("Failed to fetch {repo}'s pull requests."), &e),
    }
}

async fn request_review(
    octocrab: &Octocrab,
    owner: &String,
    repo: &str,
    reviewers: &[String],
    pr_id: u64,
) -> Result<(), String> {
    let res = octocrab
        .pulls(owner, repo)
        .request_reviews(pr_id, reviewers, [])
        .await;

    match res {
        Ok(_res) => {
            println!("Review requested for {repo}");

            Ok(())
        }
        Err(e) => generate_error(
            format!("Failed to request review for {repo}'s Pull request with ID:{pr_id}"),
            &e,
        ),
    }
}
