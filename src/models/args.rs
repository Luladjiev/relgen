use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long, help = "Base branch to use for the Pull Request")]
    pub base: String,
    #[arg(
        long,
        help = "Prettified name of the base branch to use for the title of the Pull Request"
    )]
    pub base_name: Option<String>,
    #[arg(long, help = "Head branch to use for the Pull Request")]
    pub head: String,
    #[arg(long, help = "Repositories owner")]
    pub owner: String,
    #[arg(
        long,
        required = true,
        help = "List of repositories to create Pull Requests to"
    )]
    pub repo: Vec<String>,
    #[arg(long, help = "List of reviewers to add to each Pull Request")]
    pub reviewer: Vec<String>,
    #[arg(long, help = "Dry run mode, no Pull Requests will be created")]
    pub dry_run: bool,
}
