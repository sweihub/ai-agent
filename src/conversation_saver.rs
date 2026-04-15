#![allow(dead_code)]

pub async fn save_conversation(
    _messages: &[Message],
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(String::new())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}
