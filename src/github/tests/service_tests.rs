use crate::github::client::{ClientError, GithubClient};
use crate::github::service::GitHub;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, PartialEq)]
struct CreatedPr {
    repo: String,
    title: String,
    base: String,
    head: String,
}

#[derive(Debug, PartialEq)]
struct ReviewRequest {
    repo: String,
    pr: u64,
    reviewers: Vec<String>,
}

#[derive(Default)]
struct MockClient {
    /// total_commits per repo, or an error message
    compare: HashMap<&'static str, Result<u64, &'static str>>,
    /// open PR numbers per repo, or an error message; missing repo = no open PRs
    open_prs: HashMap<&'static str, Result<Vec<u64>, &'static str>>,
    /// error message returned by create_pr, if set
    create_pr_error: Option<&'static str>,
    /// error message returned by request_reviews, if set
    request_reviews_error: Option<&'static str>,
    created: Mutex<Vec<CreatedPr>>,
    reviews: Mutex<Vec<ReviewRequest>>,
}

fn error(message: &str) -> ClientError {
    ClientError {
        message: message.to_string(),
    }
}

impl GithubClient for MockClient {
    async fn compare(
        &self,
        _owner: &str,
        repo: &str,
        _base: &str,
        _head: &str,
    ) -> Result<u64, ClientError> {
        match self.compare.get(repo) {
            Some(Ok(total_commits)) => Ok(*total_commits),
            Some(Err(message)) => Err(error(message)),
            None => panic!("unexpected compare for {repo}"),
        }
    }

    async fn open_prs(
        &self,
        _owner: &str,
        repo: &str,
        _base: &str,
        _head: &str,
    ) -> Result<Vec<u64>, ClientError> {
        match self.open_prs.get(repo) {
            Some(Ok(prs)) => Ok(prs.clone()),
            Some(Err(message)) => Err(error(message)),
            None => Ok(vec![]),
        }
    }

    async fn create_pr(
        &self,
        _owner: &str,
        repo: &str,
        title: &str,
        base: &str,
        head: &str,
    ) -> Result<u64, ClientError> {
        if let Some(message) = self.create_pr_error {
            return Err(error(message));
        }

        let mut created = self.created.lock().unwrap();
        created.push(CreatedPr {
            repo: repo.to_string(),
            title: title.to_string(),
            base: base.to_string(),
            head: head.to_string(),
        });

        Ok(created.len() as u64)
    }

    async fn request_reviews(
        &self,
        _owner: &str,
        repo: &str,
        pr: u64,
        reviewers: &[String],
    ) -> Result<(), ClientError> {
        if let Some(message) = self.request_reviews_error {
            return Err(error(message));
        }

        self.reviews.lock().unwrap().push(ReviewRequest {
            repo: repo.to_string(),
            pr,
            reviewers: reviewers.to_vec(),
        });

        Ok(())
    }
}

fn repos(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

#[tokio::test]
async fn get_repos_with_changes_keeps_only_repos_with_commits() {
    let gh = GitHub {
        client: MockClient {
            compare: HashMap::from([("a", Ok(3)), ("b", Ok(0)), ("c", Ok(1))]),
            ..Default::default()
        },
    };

    let result = gh
        .get_repos_with_changes("owner", &repos(&["a", "b", "c"]), "prod", "main")
        .await;

    assert_eq!(result, ["a", "c"]);
}

#[tokio::test]
async fn get_repos_with_changes_skips_failed_compare_and_continues() {
    let gh = GitHub {
        client: MockClient {
            compare: HashMap::from([("a", Err("not found")), ("b", Ok(2))]),
            ..Default::default()
        },
    };

    let result = gh
        .get_repos_with_changes("owner", &repos(&["a", "b"]), "prod", "main")
        .await;

    assert_eq!(result, ["b"]);
}

#[tokio::test]
async fn create_pr_passes_base_and_head_in_order() {
    let gh = GitHub {
        client: MockClient::default(),
    };

    let result = gh
        .create_pr("owner", "a", "prod", &None, "main", false)
        .await;

    assert_eq!(result.unwrap(), Some(1));

    let created = gh.client.created.lock().unwrap();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].repo, "a");
    assert_eq!(created[0].base, "prod");
    assert_eq!(created[0].head, "main");
    assert!(created[0].title.starts_with("Release to prod "));
}

#[tokio::test]
async fn create_pr_uses_base_name_in_title() {
    let gh = GitHub {
        client: MockClient::default(),
    };

    gh.create_pr(
        "owner",
        "a",
        "prod",
        &Some("Production".to_string()),
        "main",
        false,
    )
    .await
    .unwrap();

    let created = gh.client.created.lock().unwrap();
    assert!(created[0].title.starts_with("Release to Production "));
}

#[tokio::test]
async fn create_pr_skips_repo_with_open_pr() {
    let gh = GitHub {
        client: MockClient {
            open_prs: HashMap::from([("a", Ok(vec![42]))]),
            ..Default::default()
        },
    };

    let result = gh
        .create_pr("owner", "a", "prod", &None, "main", false)
        .await;

    assert_eq!(result.unwrap(), None);
    assert!(gh.client.created.lock().unwrap().is_empty());
}

#[tokio::test]
async fn create_pr_does_not_create_on_dry_run() {
    let gh = GitHub {
        client: MockClient::default(),
    };

    let result = gh
        .create_pr("owner", "a", "prod", &None, "main", true)
        .await;

    assert_eq!(result.unwrap(), None);
    assert!(gh.client.created.lock().unwrap().is_empty());
}

#[tokio::test]
async fn create_pr_returns_open_prs_error() {
    let gh = GitHub {
        client: MockClient {
            open_prs: HashMap::from([("a", Err("forbidden"))]),
            ..Default::default()
        },
    };

    let result = gh
        .create_pr("owner", "a", "prod", &None, "main", false)
        .await;

    assert_eq!(result.unwrap_err().message, "forbidden");
    assert!(gh.client.created.lock().unwrap().is_empty());
}

#[tokio::test]
async fn create_pr_returns_create_error() {
    let gh = GitHub {
        client: MockClient {
            create_pr_error: Some("validation failed"),
            ..Default::default()
        },
    };

    let result = gh
        .create_pr("owner", "a", "prod", &None, "main", false)
        .await;

    assert_eq!(result.unwrap_err().message, "validation failed");
}

#[tokio::test]
async fn request_reviews_sends_reviewers() {
    let gh = GitHub {
        client: MockClient::default(),
    };

    gh.request_reviews("owner", "a", 7, &repos(&["alice", "bob"]), false)
        .await
        .unwrap();

    assert_eq!(
        *gh.client.reviews.lock().unwrap(),
        [ReviewRequest {
            repo: "a".to_string(),
            pr: 7,
            reviewers: repos(&["alice", "bob"]),
        }]
    );
}

#[tokio::test]
async fn request_reviews_skips_without_reviewers() {
    let gh = GitHub {
        client: MockClient::default(),
    };

    gh.request_reviews("owner", "a", 7, &[], false)
        .await
        .unwrap();

    assert!(gh.client.reviews.lock().unwrap().is_empty());
}

#[tokio::test]
async fn request_reviews_skips_on_dry_run() {
    let gh = GitHub {
        client: MockClient::default(),
    };

    gh.request_reviews("owner", "a", 7, &repos(&["alice"]), true)
        .await
        .unwrap();

    assert!(gh.client.reviews.lock().unwrap().is_empty());
}

#[tokio::test]
async fn request_reviews_returns_error() {
    let gh = GitHub {
        client: MockClient {
            request_reviews_error: Some("not a collaborator"),
            ..Default::default()
        },
    };

    let result = gh
        .request_reviews("owner", "a", 7, &repos(&["alice"]), false)
        .await;

    assert_eq!(result.unwrap_err().message, "not a collaborator");
}
