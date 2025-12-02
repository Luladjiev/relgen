use serde::Deserialize;

#[derive(Deserialize)]
pub struct Response {
    pub total_commits: u32,
}
