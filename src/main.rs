mod github;
mod models;

use crate::github::octocrab::OctocrabClient;
use clap::Parser;
use futures::future::join_all;
use github::service::GitHub;
use models::args::Args;
use octocrab::Octocrab;

#[tokio::main]
async fn main() {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(e) => {
            e.exit();
        }
    };

    let token = match std::env::var("GITHUB_TOKEN") {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Failed to retrieve GITHUB_TOKEN: {e}");
            std::process::exit(1);
        }
    };

    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .expect("Failed to instantiate Octocrab");

    let client = OctocrabClient { octocrab };
    let gh = GitHub { client };

    let repos = gh
        .get_repos_with_changes(&args.owner, &args.repo, &args.base, &args.head)
        .await;

    let (gh, args) = (&gh, &args);

    let created = join_all(repos.iter().map(|repo| async move {
        gh.create_pr(
            &args.owner,
            repo,
            &args.base,
            &args.base_name,
            &args.head,
            args.dry_run,
        )
        .await
        .map(|pr_number| pr_number.map(|n| (repo, n)))
    }))
    .await;

    let prs = created
        .into_iter()
        .filter_map(|res| res.inspect_err(|err| eprintln!("{err}")).ok().flatten());

    let reviewed = join_all(prs.map(|(repo, pr_number)| {
        gh.request_reviews(&args.owner, repo, pr_number, &args.reviewer, args.dry_run)
    }))
    .await;

    for err in reviewed.into_iter().filter_map(Result::err) {
        eprintln!("{err}");
    }
}
