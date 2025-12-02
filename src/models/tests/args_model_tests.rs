use crate::models::args::Args;

#[cfg(test)]
pub mod tests {
    use super::Args;
    use clap::Parser;

    #[test]
    fn test_parse_args() {
        let args = Args::parse_from(&[
            "relgen",
            "--base",
            "master",
            "--head",
            "slave",
            "--owner",
            "coolOrg",
            "--repo",
            "example/repo",
            "--repo",
            "example/another-repo",
            "--reviewer",
            "JohnDoe",
        ]);

        assert_eq!(args.base, "master");
        assert_eq!(args.head, "slave");
        assert_eq!(args.owner, "coolOrg");
        assert_eq!(args.repo, ["example/repo", "example/another-repo"]);
        assert_eq!(args.reviewer, ["JohnDoe"]);
    }
}
