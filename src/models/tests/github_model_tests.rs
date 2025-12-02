use crate::models::github::Response;

#[cfg(test)]
pub mod tests {
    use super::Response;
    use serde_json::json;

    #[test]
    fn test_deserialize_response() {
        let data = json!({
            "total_commits": 42
        });

        let response: Response = serde_json::from_value(data).unwrap();
        assert_eq!(response.total_commits, 42);
    }
}
