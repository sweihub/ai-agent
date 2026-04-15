use reqwest::Client;

#[allow(dead_code)]
pub struct HttpClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl HttpClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
        }
    }

    pub fn create_message_request(
        &self,
        model: &str,
        messages: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::error::AgentError> {
        let body = serde_json::json!({
            "model": model,
            "max_tokens": 16384,
            "messages": messages,
        });
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_request() {
        let client = HttpClient::new("test-key".to_string());
        let messages = vec![serde_json::json!({"role": "user", "content": "Hello"})];
        let req = client.create_message_request("claude-sonnet-4-6", messages);
        assert!(req.is_ok());
    }
}
