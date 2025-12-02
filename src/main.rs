mod data;
mod models;

use clap::Parser;
use data::github::GitHub;
use futures::future::join_all;
use models::args::Args;
use octocrab::Octocrab;

#[tokio::main]
async fn main() {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Failed to parse arguments: {}", e);
            return;
        }
    };

    let token = match std::env::var("GITHUB_TOKEN") {
        Ok(token) => token,
        Err(e) => {
            println!("Failed to retrieve GITHUB_TOKEN: {e}");
            return;
        }
    };
    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .expect("Failed to instantiate Octocrab");

    let gh = GitHub {
        octocrab: &octocrab,
    };

    let res = gh
        .get_repos_with_changes(&args.owner, &args.repo, &args.base, &args.head)
        .await;

    match res {
        Ok(repos) => {
            let mut responses = vec![];

            for repo in repos {
                responses.push(gh.create_pr(
                    &args.owner,
                    repo,
                    &args.base,
                    &args.base_name,
                    &args.head,
                    &args.reviewer,
                    args.dry_run,
                ));
            }

            let result = join_all(responses).await;

            for res in result {
                if let Err(err) = res {
                    println!("{err}");
                }
            }
        }
        Err(e) => {
            eprintln!("App error: {}", e);
            std::process::exit(1);
        }
    }
}
